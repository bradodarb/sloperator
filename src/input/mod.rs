//! Input resolution (SPEC §14): files, directories, globs and `-` become a sorted, de-duplicated list of sources,
//! each with the settings that apply to it.
//!
//! Every walker filter is set explicitly (Gap 5): hidden directories are walked, `.ignore` files and the user's global
//! gitignore are never read, and `.gitignore` applies outside a git repo too. File paths are joined onto `cwd`, so
//! [`read`] never depends on the process working directory.

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use globset::{Glob, GlobBuilder, GlobMatcher, GlobSet, GlobSetBuilder};

use crate::Error;
use crate::config::{CONFIG_FILE_NAME, Resolver, Settings};
pub use crate::doc::Format;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Origin {
    File(PathBuf),
    /// `as_path` = `--stdin-filename`, else `-`.
    Stdin {
        as_path: PathBuf,
    },
}

#[derive(Clone, Debug)]
pub struct SourceRef {
    pub origin: Origin,
    pub format: Format,
    pub settings: Arc<Settings>,
}

#[derive(Clone, Debug)]
pub struct Source {
    pub origin: Origin,
    pub format: Format,
    pub text: Arc<str>,
    pub settings: Arc<Settings>,
}

/// The stdin argument and the stdin display name when `--stdin-filename` is absent.
const STDIN: &str = "-";

/// Files, directories (walked), globs and `-` become sorted, de-duplicated refs. No args means the current directory.
///
/// A path that does not exist, or a glob that matches nothing, is `Error::Io`. An explicit file that `exclude` or
/// `.gitignore` matches is skipped with `sloperator: skipped (excluded): PATH` on `notes` (DQ-13).
pub fn resolve(
    args: &[String],
    stdin_filename: Option<&Path>,
    cwd: &Path,
    resolver: &mut Resolver,
    notes: &mut dyn std::io::Write,
) -> Result<Vec<SourceRef>, Error> {
    let default_args = [".".to_string()];
    let args = if args.is_empty() {
        &default_args[..]
    } else {
        args
    };
    let mut cx = Cx {
        cwd,
        cwd_canon: cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf()),
        resolver,
        notes,
        sets: HashMap::new(),
        out: Vec::new(),
    };
    for arg in args {
        if arg == STDIN {
            cx.stdin(stdin_filename)?;
        } else if is_glob(arg) {
            cx.glob(arg)?;
        } else {
            cx.path(arg)?;
        }
    }
    let mut out = cx.out;
    out.sort_by(|a, b| a.origin.cmp(&b.origin));
    out.dedup_by(|a, b| a.origin == b.origin);
    Ok(out)
}

/// Reads a file, or takes the stdin text cli already read. Non-UTF-8 input returns Error::NotUtf8, whose offset is
/// Utf8Error::valid_up_to() (FR-8.8). Nothing is ever written here, so a rejected file is left unchanged.
pub fn read(r: &SourceRef, stdin_text: Option<&str>) -> Result<Source, Error> {
    let text: Arc<str> = match &r.origin {
        Origin::File(path) => {
            let bytes = std::fs::read(path).map_err(|source| Error::Io {
                path: path.clone(),
                source,
            })?;
            String::from_utf8(bytes)
                .map_err(|e| Error::NotUtf8 {
                    path: path.clone(),
                    offset: e.utf8_error().valid_up_to(),
                })?
                .into()
        }
        Origin::Stdin { .. } => stdin_text
            .ok_or_else(|| Error::Usage("stdin source read without the stdin text".to_string()))?
            .into(),
    };
    Ok(Source {
        origin: r.origin.clone(),
        format: r.format,
        text,
        settings: r.settings.clone(),
    })
}

/// `.txt` → PlainText; everything else (`.md`, `.markdown`, `-`) → Markdown.
pub fn format_for(path: &Path) -> Format {
    match path.extension().and_then(|e| e.to_str()) {
        Some(e) if e.eq_ignore_ascii_case("txt") => Format::PlainText,
        _ => Format::Markdown,
    }
}

/// `*`, `?` or `[` makes an argument a glob.
fn is_glob(arg: &str) -> bool {
    arg.contains(['*', '?', '['])
}

/// `cwd` joined with `p`, with `.` components dropped so equal files compare equal.
fn join(cwd: &Path, p: &Path) -> PathBuf {
    cwd.join(p)
        .components()
        .filter(|c| !matches!(c, Component::CurDir))
        .collect()
}

fn walk_error(root: &Path, e: ignore::Error) -> Error {
    Error::Io {
        path: root.to_path_buf(),
        source: e
            .into_io_error()
            .unwrap_or_else(|| std::io::Error::other("walk failed")),
    }
}

/// The walker with every filter pinned (SPEC §14, Gap 5). `keep` narrows the entries below the root.
fn walker(
    root: &Path,
    respect_gitignore: bool,
    keep: impl Fn(&ignore::DirEntry) -> bool + Send + Sync + 'static,
) -> ignore::WalkBuilder {
    let g = respect_gitignore;
    let mut b = ignore::WalkBuilder::new(root);
    b.standard_filters(false) // start from nothing; each filter below is a stated choice
        .hidden(false) // `.github/*.md` and other dot-dirs are checked; `.git/` is skipped by filter_entry
        .ignore(false) // `.ignore` files are not read: FR-2.4 names only .gitignore and `exclude`
        .git_ignore(g) // .gitignore, in the walked dirs ...
        .parents(g) // ... and in their parents
        .git_exclude(g) // .git/info/exclude is part of git's ignore set
        .git_global(false) // a user's global gitignore never silently hides a project file
        .require_git(false) // .gitignore applies outside a git repo too
        .follow_links(false)
        .filter_entry(move |e| e.file_name() != ".git" && (e.depth() == 0 || keep(e)))
        .sort_by_file_path(|a, b| a.cmp(b));
    b
}

struct Cx<'a> {
    cwd: &'a Path,
    /// `cwd` canonicalized: config `source` paths are canonical, so glob-relative paths are computed canonically.
    cwd_canon: PathBuf,
    resolver: &'a mut Resolver,
    notes: &'a mut dyn std::io::Write,
    /// Compiled `include`/`exclude` lists (validated by config, so they compile).
    sets: HashMap<Vec<String>, Arc<GlobSet>>,
    out: Vec<SourceRef>,
}

impl Cx<'_> {
    fn stdin(&mut self, stdin_filename: Option<&Path>) -> Result<(), Error> {
        let as_path = stdin_filename.map_or_else(|| PathBuf::from(STDIN), Path::to_path_buf);
        // `-` has no directory of its own, so the config lookup lands on `cwd` (FR-2.3).
        let settings = self.resolver.for_file(&join(self.cwd, &as_path))?;
        self.out.push(SourceRef {
            format: format_for(&as_path),
            origin: Origin::Stdin { as_path },
            settings,
        });
        Ok(())
    }

    fn path(&mut self, arg: &str) -> Result<(), Error> {
        let path = join(self.cwd, Path::new(arg));
        let meta = std::fs::metadata(&path).map_err(|source| Error::Io {
            path: PathBuf::from(arg),
            source,
        })?;
        if meta.is_dir() {
            return self.walk_dir(&path, None);
        }
        let settings = self.resolver.for_file(&path)?;
        if self.excluded(&path, &settings)? || self.gitignored(&path, &settings)? {
            writeln!(self.notes, "sloperator: skipped (excluded): {arg}").map_err(|source| {
                Error::Io {
                    path: PathBuf::from("<stderr>"),
                    source,
                }
            })?;
            return Ok(());
        }
        self.push(path, settings);
        Ok(())
    }

    /// Walks from the glob's literal prefix and keeps the files the whole pattern matches. The glob is the user's
    /// filter, so `include` does not apply; `exclude` and `.gitignore` still do.
    fn glob(&mut self, arg: &str) -> Result<(), Error> {
        let pattern = Path::new(arg);
        let prefix: PathBuf = pattern
            .components()
            .take_while(|c| !is_glob(&c.as_os_str().to_string_lossy()))
            .collect();
        // The cwd part is escaped: a `[` in a directory name is literal, not a class.
        let full = if pattern.is_absolute() {
            arg.to_string()
        } else {
            let rest: Vec<_> = pattern
                .components()
                .filter(|c| !matches!(c, Component::CurDir))
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect();
            format!(
                "{}/{}",
                globset::escape(&self.cwd.to_string_lossy()),
                rest.join("/")
            )
        };
        let matcher = GlobBuilder::new(&full)
            .literal_separator(true)
            .build()
            .map_err(|e| Error::Usage(format!("invalid glob `{arg}`: {e}")))?
            .compile_matcher();
        let root = join(self.cwd, &prefix);
        let before = self.out.len();
        if root.is_dir() {
            self.walk_dir(&root, Some(&matcher))?;
        }
        if self.out.len() == before {
            return Err(Error::Io {
                path: PathBuf::from(arg),
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "glob matched no files"),
            });
        }
        Ok(())
    }

    /// Walks `dir`. Without a `glob`, a file is kept when its settings' `include` matches and `exclude` does not; with
    /// one, the glob replaces `include`. `respect_gitignore` comes from the settings of the walked directory.
    fn walk_dir(&mut self, dir: &Path, glob: Option<&GlobMatcher>) -> Result<(), Error> {
        let root_settings = self.resolver.for_file(&dir.join(CONFIG_FILE_NAME))?;
        for entry in walker(dir, root_settings.respect_gitignore, |_| true).build() {
            let entry = entry.map_err(|e| walk_error(dir, e))?;
            if !entry.file_type().is_some_and(|t| t.is_file()) {
                continue;
            }
            let path = entry.into_path();
            let settings = self.resolver.for_file(&path)?;
            let wanted = match glob {
                Some(m) => m.is_match(&path),
                None => self
                    .set(&settings.include)?
                    .is_match(self.rel(&path, &settings)),
            };
            if wanted && !self.excluded(&path, &settings)? {
                self.push(path, settings);
            }
        }
        Ok(())
    }

    /// `exclude` matches the path, or a directory above it, relative to the settings' base.
    fn excluded(&mut self, path: &Path, settings: &Settings) -> Result<bool, Error> {
        let exclude = self.set(&settings.exclude)?;
        let rel = self.rel(path, settings);
        Ok(rel
            .ancestors()
            .filter(|p| !p.as_os_str().is_empty())
            .any(|p| exclude.is_match(p)))
    }

    /// An explicit file is a walk root, which the walker never filters, so its gitignore status is asked of a
    /// one-level walk of its directory that keeps only that name: the same rules as a directory walk.
    fn gitignored(&mut self, path: &Path, settings: &Settings) -> Result<bool, Error> {
        if !settings.respect_gitignore {
            return Ok(false);
        }
        let (Some(dir), Some(name)) = (path.parent(), path.file_name()) else {
            return Ok(false);
        };
        let name = name.to_os_string();
        let walk = walker(dir, true, move |e| e.file_name() == name)
            .max_depth(Some(1))
            .build();
        for entry in walk {
            let entry = entry.map_err(|e| walk_error(dir, e))?;
            if entry.depth() == 1 {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// The compiled `include` or `exclude` list. Config has validated each glob, so an error here is loud, not skipped.
    fn set(&mut self, globs: &[String]) -> Result<Arc<GlobSet>, Error> {
        if let Some(s) = self.sets.get(globs) {
            return Ok(s.clone());
        }
        let invalid = |e: globset::Error| Error::Usage(format!("invalid glob: {e}"));
        let mut b = GlobSetBuilder::new();
        for g in globs {
            b.add(Glob::new(g).map_err(invalid)?);
        }
        let set = Arc::new(b.build().map_err(invalid)?);
        self.sets.insert(globs.to_vec(), set.clone());
        Ok(set)
    }

    /// `path` relative to the directory `include`/`exclude` are written against: the config file's directory, else
    /// `cwd`. Both sides are canonical; a path outside that directory is returned whole.
    fn rel(&self, path: &Path, settings: &Settings) -> PathBuf {
        let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let base = settings
            .source
            .as_deref()
            .and_then(Path::parent)
            .unwrap_or(&self.cwd_canon);
        path.strip_prefix(base)
            .map(Path::to_path_buf)
            .unwrap_or(path)
    }

    fn push(&mut self, path: PathBuf, settings: Arc<Settings>) {
        self.out.push(SourceRef {
            format: format_for(&path),
            origin: Origin::File(path),
            settings,
        });
    }
}
