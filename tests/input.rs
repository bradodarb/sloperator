//! `input` (SPEC §14, §23.10): input forms, stdin, the pinned walker filters (Gap 5), errors and stable order.
#![allow(clippy::unwrap_used, clippy::expect_used)] // test helpers may unwrap (architecture §8)

use std::fs;
use std::path::{Path, PathBuf};

use sloperator::Error;
use sloperator::config::{CliOverrides, Resolver};
use sloperator::input::{self, Format, Origin, SourceRef};

/// A temp project root shielded from any `.gitignore` above the system temp dir (the walker reads parent ignore files
/// even outside a git repo, and a `*` there would hide everything). The shield's `!*` is consulted only after the
/// project's own `.gitignore` files, which are deeper, so each test's rules still decide.
struct Sandbox {
    _tmp: tempfile::TempDir,
    root: PathBuf,
}

fn sandbox() -> Sandbox {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join(".gitignore"), "!*\n").unwrap();
    let root = tmp.path().join("project");
    fs::create_dir(&root).unwrap();
    Sandbox { _tmp: tmp, root }
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// Runs `resolve` in `cwd` with no `--config`; returns the refs and what went to the notes writer.
fn resolve_in(
    cwd: &Path,
    args: &[&str],
    stdin_filename: Option<&str>,
) -> Result<(Vec<SourceRef>, String), Error> {
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let mut resolver = Resolver::new(CliOverrides::default(), None).unwrap();
    let mut notes = Vec::new();
    let refs = input::resolve(
        &args,
        stdin_filename.map(Path::new),
        cwd,
        &mut resolver,
        &mut notes,
    )?;
    Ok((refs, String::from_utf8(notes).unwrap()))
}

/// The refs as `cwd`-relative strings (`-` or the stdin filename for stdin, prefixed `stdin:`).
fn names(cwd: &Path, refs: &[SourceRef]) -> Vec<String> {
    refs.iter()
        .map(|r| match &r.origin {
            Origin::File(p) => p
                .strip_prefix(cwd)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/"),
            Origin::Stdin { as_path } => format!("stdin:{}", as_path.display()),
        })
        .collect()
}

fn ok(cwd: &Path, args: &[&str]) -> Vec<String> {
    let (refs, _) = resolve_in(cwd, args, None).unwrap();
    names(cwd, &refs)
}

#[test]
fn files_dirs_globs_and_stdin() {
    let sb = sandbox();
    let root = sb.root.as_path();
    write(&root.join("a.md"), "a\n");
    write(&root.join("docs/b.md"), "b\n");
    write(&root.join("docs/c.txt"), "c\n");
    write(&root.join("docs/d.rs"), "fn main() {}\n");
    write(&root.join("docs/sub/e.markdown"), "e\n");
    write(&root.join("notes/f.md"), "f\n");
    write(&root.join("notes/deep/g.md"), "g\n");

    let (refs, notes) = resolve_in(root, &["a.md", "docs", "notes/*.md", "-"], None).unwrap();
    assert_eq!(
        names(root, &refs),
        vec![
            "a.md",
            "docs/b.md",
            "docs/c.txt",
            "docs/sub/e.markdown",
            "notes/f.md",
            "stdin:-"
        ],
        "a file, a walked dir (include filters .rs out), a glob (`*` stays in one dir) and stdin"
    );
    assert_eq!(notes, "");
    let format_of = |n: &str| refs[names(root, &refs).iter().position(|x| x == n).unwrap()].format;
    assert_eq!(format_of("docs/c.txt"), Format::PlainText);
    assert_eq!(format_of("docs/b.md"), Format::Markdown);
    assert_eq!(format_of("docs/sub/e.markdown"), Format::Markdown);

    assert_eq!(
        ok(root, &["notes/**/*.md"]),
        vec!["notes/deep/g.md", "notes/f.md"],
        "`**` crosses directories"
    );

    let src = input::read(&refs[0], None).unwrap();
    assert_eq!(&*src.text, "a\n");
    let stdin = input::read(refs.last().unwrap(), Some("from stdin\n")).unwrap();
    assert_eq!(&*stdin.text, "from stdin\n");
}

#[test]
fn no_args_checks_cwd() {
    let sb = sandbox();
    let root = sb.root.as_path();
    write(&root.join("a.md"), "a\n");
    write(&root.join("sub/b.txt"), "b\n");
    write(&root.join("sub/c.json"), "{}\n");

    assert_eq!(ok(root, &[]), vec!["a.md", "sub/b.txt"]);
    assert_eq!(ok(root, &[]), ok(root, &["."]));
}

#[test]
fn stdin_filename_selects_format_and_config() {
    let sb = sandbox();
    let root = sb.root.as_path();
    write(&root.join("sub/sloperator.toml"), "select = [\"SLP001\"]\n");

    let (refs, _) = resolve_in(root, &["-"], Some("sub/notes.txt")).unwrap();
    assert_eq!(refs.len(), 1);
    assert_eq!(
        refs[0].origin,
        Origin::Stdin {
            as_path: PathBuf::from("sub/notes.txt")
        }
    );
    assert_eq!(
        refs[0].format,
        Format::PlainText,
        "the stdin filename sets the format"
    );
    assert_eq!(
        refs[0].settings.source,
        Some(root.join("sub/sloperator.toml").canonicalize().unwrap()),
        "the stdin filename sets the config lookup path"
    );
    assert_eq!(
        refs[0]
            .settings
            .rules
            .keys()
            .map(|c| c.0)
            .collect::<Vec<_>>(),
        vec![1]
    );

    let (md, _) = resolve_in(root, &["-"], Some("sub/notes.md")).unwrap();
    assert_eq!(md[0].format, Format::Markdown);
}

#[test]
fn stdin_defaults_markdown() {
    let sb = sandbox();
    let root = sb.root.as_path();
    write(&root.join("sloperator.toml"), "select = [\"SLP002\"]\n");

    let (refs, _) = resolve_in(root, &["-", "-"], None).unwrap();
    assert_eq!(refs.len(), 1, "a repeated `-` is one source");
    assert_eq!(
        refs[0].origin,
        Origin::Stdin {
            as_path: PathBuf::from("-")
        }
    );
    assert_eq!(refs[0].format, Format::Markdown);
    assert_eq!(input::format_for(Path::new("-")), Format::Markdown);
    assert_eq!(
        refs[0].settings.source,
        Some(root.join("sloperator.toml").canonicalize().unwrap()),
        "without a stdin filename the config is resolved for cwd"
    );
}

#[test]
fn gitignore_and_exclude_skipped_respect_gitignore_false() {
    let sb = sandbox();
    let root = sb.root.as_path();
    write(&root.join(".gitignore"), "ignored.md\nbuild/\n");
    write(
        &root.join("sloperator.toml"),
        "exclude = [\"vendor/**\", \"CHANGELOG.md\"]\n",
    );
    write(&root.join("kept.md"), "k\n");
    write(&root.join("ignored.md"), "i\n");
    write(&root.join("build/out.md"), "o\n");
    write(&root.join("vendor/v.md"), "v\n");
    write(&root.join("CHANGELOG.md"), "c\n");

    assert_eq!(ok(root, &[]), vec!["kept.md"]);

    // An explicit file that .gitignore or `exclude` matches is skipped with a note (DQ-13).
    let (refs, notes) = resolve_in(root, &["ignored.md", "vendor/v.md", "kept.md"], None).unwrap();
    assert_eq!(names(root, &refs), vec!["kept.md"]);
    assert_eq!(
        notes,
        "sloperator: skipped (excluded): ignored.md\nsloperator: skipped (excluded): vendor/v.md\n"
    );

    write(
        &root.join("sloperator.toml"),
        "exclude = [\"vendor/**\", \"CHANGELOG.md\"]\nrespect_gitignore = false\n",
    );
    assert_eq!(
        ok(root, &[]),
        vec!["build/out.md", "ignored.md", "kept.md"],
        "with respect_gitignore = false only `exclude` skips files"
    );
    let (refs, notes) = resolve_in(root, &["ignored.md"], None).unwrap();
    assert_eq!(names(root, &refs), vec!["ignored.md"]);
    assert_eq!(notes, "");
}

#[test]
fn hidden_dir_walked_dot_ignore_and_global_ignore_not_read() {
    let home = tempfile::tempdir().unwrap();
    // A global excludes file in every place git looks: core.excludesFile and $XDG_CONFIG_HOME/git/ignore.
    write(&home.path().join("global-ignore"), "*.md\n");
    write(
        &home.path().join(".gitconfig"),
        &format!(
            "[core]\n\texcludesFile = {}\n",
            home.path().join("global-ignore").display()
        ),
    );
    write(&home.path().join(".config/git/ignore"), "*.md\n");
    // SAFETY: nextest runs each test in its own process, so no other thread reads the environment.
    unsafe {
        std::env::set_var("HOME", home.path());
        std::env::set_var("XDG_CONFIG_HOME", home.path().join(".config"));
    }

    let sb = sandbox();
    let root = sb.root.as_path();
    write(&root.join(".github/x.md"), "x\n");
    write(&root.join(".ignore"), "*.md\n");
    write(&root.join("doc.md"), "d\n");
    write(&root.join(".git/HEAD"), "ref: refs/heads/main\n");
    write(&root.join(".git/notes.md"), "never walked\n");

    assert_eq!(
        ok(root, &[]),
        vec![".github/x.md", "doc.md"],
        "dot-dirs are walked, .git is not, and neither .ignore nor the global excludes file hides anything"
    );
}

#[test]
fn gitignore_applies_outside_git_repo() {
    let sb = sandbox();
    let root = sb.root.as_path();
    assert!(!root.join(".git").exists());
    write(&root.join(".gitignore"), "skip.md\n");
    write(&root.join("sub/.gitignore"), "local.md\n");
    write(&root.join("keep.md"), "k\n");
    write(&root.join("skip.md"), "s\n");
    write(&root.join("sub/skip.md"), "s\n");
    write(&root.join("sub/local.md"), "l\n");
    write(&root.join("sub/other.md"), "o\n");

    assert_eq!(ok(root, &[]), vec!["keep.md", "sub/other.md"]);
    assert_eq!(
        ok(root, &["sub"]),
        vec!["sub/other.md"],
        "a parent directory's .gitignore applies to a walked subdirectory"
    );
}

#[test]
fn missing_path_exits_two() {
    let sb = sandbox();
    let root = sb.root.as_path();
    write(&root.join("a.md"), "a\n");

    match resolve_in(root, &["a.md", "nope.md"], None) {
        Err(Error::Io { path, source }) => {
            assert_eq!(path, PathBuf::from("nope.md"));
            assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
        }
        other => panic!("expected Error::Io, got {other:?}"),
    }
    match resolve_in(root, &["*.rst"], None) {
        Err(Error::Io { path, source }) => {
            assert_eq!(path, PathBuf::from("*.rst"));
            assert!(source.to_string().contains("glob matched no files"));
        }
        other => panic!("expected Error::Io for an empty glob, got {other:?}"),
    }
    assert!(
        matches!(
            resolve_in(root, &["missing/*.md"], None),
            Err(Error::Io { .. })
        ),
        "a glob whose literal prefix does not exist matches nothing"
    );
}

#[test]
fn non_utf8_exits_two_unmodified() {
    let sb = sandbox();
    let root = sb.root.as_path();
    let bytes: &[u8] = b"good text \xff\xfe more\n";
    let path = root.join("bad.md");
    fs::write(&path, bytes).unwrap();

    let (refs, _) = resolve_in(root, &["bad.md"], None).unwrap();
    match input::read(&refs[0], None) {
        Err(e @ Error::NotUtf8 { .. }) => {
            let Error::NotUtf8 { path: p, offset } = &e else {
                unreachable!()
            };
            assert_eq!(p, &path);
            assert_eq!(*offset, 10, "the offset is the first invalid byte");
            assert!(e.to_string().contains("byte 10"));
        }
        other => panic!("expected Error::NotUtf8, got {other:?}"),
    }
    assert_eq!(
        fs::read(&path).unwrap(),
        bytes,
        "the file is left unchanged"
    );
}

#[test]
fn order_independent_of_walk_order() {
    let sb = sandbox();
    let root = sb.root.as_path();
    // Created out of order, so directory listing order differs from sorted order on most filesystems.
    for name in ["z.md", "m/b.md", "a.md", "m/a.md", "b/z.txt", "B.md"] {
        write(&root.join(name), "x\n");
    }

    let want = ok(root, &[]);
    let mut sorted = want.clone();
    sorted.sort_by(|a, b| Path::new(a).cmp(Path::new(b)));
    assert_eq!(want, sorted, "output is sorted by path");
    assert_eq!(want.len(), 6);

    assert_eq!(ok(root, &["z.md", "m", "a.md", "b", "B.md"]), want);
    assert_eq!(ok(root, &["B.md", "b", "a.md", "m", "z.md"]), want);
    assert_eq!(
        ok(root, &["m/a.md", ".", "m", "*.md"]),
        want,
        "overlapping arguments are de-duplicated"
    );
}
