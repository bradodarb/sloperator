//! The shape of `sloperator.toml` on disk (SPEC §13) and the `extend` chain (FR-7.1).
//!
//! serde with `deny_unknown_fields` is the parser. When it rejects a file, [`locate`] walks the same document as a
//! `toml::Table` to name the dotted key that failed (FR-7.6), because a serde error carries the reason but not the path.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::Error;

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFile {
    pub extend: Option<PathBuf>,
    pub select: Option<Vec<String>>,
    pub extend_select: Option<Vec<String>>,
    pub ignore: Option<Vec<String>>,
    pub min_severity: Option<crate::rules::Severity>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub respect_gitignore: Option<bool>,
    /// Validated against the registry and `RuleMeta::params` by `config::Resolver`.
    pub rules: Option<BTreeMap<String, toml::Table>>,
    pub per_file: Option<Vec<PerFile>>,
    pub protect: Option<ProtectFile>,
    pub model: Option<ModelFile>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerFile {
    pub glob: String,
    pub select: Option<Vec<String>>,
    pub extend_select: Option<Vec<String>>,
    pub ignore: Option<Vec<String>>,
    pub min_severity: Option<crate::rules::Severity>,
    pub rules: Option<BTreeMap<String, toml::Table>>,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtectFile {
    pub patterns: Option<Vec<String>>,
    pub terms: Option<Vec<String>>,
    pub front_matter: Option<bool>,
    pub math: Option<bool>,
    pub html: Option<bool>,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelFile {
    pub name: Option<String>,
    pub path: Option<PathBuf>,
    pub profile: Option<String>,
    pub cache_dir: Option<PathBuf>,
    pub window: Option<WindowMode>,
    pub max_window_chars: Option<usize>,
    pub max_tokens: Option<u32>,
    pub seed: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowMode {
    Paragraph,
    Sentence,
}

/// One parsed file of a chain: its canonical path, its directory and its contents.
#[derive(Debug)]
pub struct Loaded {
    pub path: PathBuf,
    pub dir: PathBuf,
    pub file: ConfigFile,
}

/// Reads and parses one file. `path` must be canonical: it is the name every error reports.
pub fn load(path: &Path) -> Result<ConfigFile, Error> {
    let src = std::fs::read_to_string(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse(&src, path)
}

/// Parses `src` as the contents of `path`. Every failure is `Error::Config` naming `path` and the dotted key.
pub fn parse(src: &str, path: &Path) -> Result<ConfigFile, Error> {
    match toml::from_str::<ConfigFile>(src) {
        Ok(f) => Ok(f),
        Err(serde_err) => {
            let table = match toml::from_str::<toml::Table>(src) {
                Ok(t) => t,
                Err(e) => return Err(config_err(path, "(syntax)", e.message())),
            };
            Err(locate(&table, path)
                .unwrap_or_else(|| config_err(path, "(document)", serde_err.message())))
        }
    }
}

/// The file and every file its `extend` names, root first (FR-7.1). A cycle is `Error::Config` on `extend`,
/// naming every file in the loop.
pub fn load_chain(path: &Path) -> Result<Vec<Loaded>, Error> {
    let mut chain: Vec<Loaded> = Vec::new();
    let mut next = Some(canonical(path)?);
    while let Some(p) = next.take() {
        if let Some(i) = chain.iter().position(|l| l.path == p) {
            let mut names: Vec<String> = chain[i..]
                .iter()
                .map(|l| l.path.display().to_string())
                .collect();
            names.push(p.display().to_string());
            let last = chain.last().map(|l| l.path.clone()).unwrap_or(p);
            return Err(config_err(
                &last,
                "extend",
                &format!("extend cycle: {}", names.join(" -> ")),
            ));
        }
        let file = load(&p)?;
        let dir = p.parent().map(Path::to_path_buf).unwrap_or_default();
        if let Some(ext) = &file.extend {
            let target = dir.join(ext);
            next = Some(canonical(&target).map_err(|_| {
                config_err(
                    &p,
                    "extend",
                    &format!("`{}` does not exist", target.display()),
                )
            })?);
        }
        chain.push(Loaded { path: p, dir, file });
    }
    chain.reverse();
    Ok(chain)
}

fn canonical(path: &Path) -> Result<PathBuf, Error> {
    path.canonicalize().map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })
}

pub(super) fn config_err(file: &Path, key: &str, reason: &str) -> Error {
    Error::Config {
        file: file.to_path_buf(),
        key: key.to_string(),
        reason: reason.to_string(),
    }
}

/// What a key holds, for naming the key serde rejected.
#[derive(Clone, Copy)]
enum Kind {
    Leaf(fn(&toml::Value) -> Result<(), String>),
    Table(&'static [(&'static str, Kind)]),
    Tables(&'static [(&'static str, Kind)], &'static str), // array of tables, with its required key
    RuleTables,
}

fn is<T: DeserializeOwned>(v: &toml::Value) -> Result<(), String> {
    v.clone()
        .try_into::<T>()
        .map(|_| ())
        .map_err(|e| e.message().to_string())
}

const STRINGS: Kind = Kind::Leaf(is::<Vec<String>>);
const SEVERITY: Kind = Kind::Leaf(is::<crate::rules::Severity>);
const STRING: Kind = Kind::Leaf(is::<String>);
const BOOL: Kind = Kind::Leaf(is::<bool>);

const PER_FILE: &[(&str, Kind)] = &[
    ("glob", STRING),
    ("select", STRINGS),
    ("extend_select", STRINGS),
    ("ignore", STRINGS),
    ("min_severity", SEVERITY),
    ("rules", Kind::RuleTables),
];

const PROTECT: &[(&str, Kind)] = &[
    ("patterns", STRINGS),
    ("terms", STRINGS),
    ("front_matter", BOOL),
    ("math", BOOL),
    ("html", BOOL),
];

const MODEL: &[(&str, Kind)] = &[
    ("name", STRING),
    ("path", STRING),
    ("profile", STRING),
    ("cache_dir", STRING),
    ("window", Kind::Leaf(is::<WindowMode>)),
    ("max_window_chars", Kind::Leaf(is::<usize>)),
    ("max_tokens", Kind::Leaf(is::<u32>)),
    ("seed", Kind::Leaf(is::<u32>)),
];

const TOP: &[(&str, Kind)] = &[
    ("extend", STRING),
    ("select", STRINGS),
    ("extend_select", STRINGS),
    ("ignore", STRINGS),
    ("min_severity", SEVERITY),
    ("include", STRINGS),
    ("exclude", STRINGS),
    ("respect_gitignore", BOOL),
    ("rules", Kind::RuleTables),
    ("per_file", Kind::Tables(PER_FILE, "glob")),
    ("protect", Kind::Table(PROTECT)),
    ("model", Kind::Table(MODEL)),
];

/// The first unknown key, wrong type or missing required key in `table`, as `Error::Config` with a dotted key.
fn locate(table: &toml::Table, path: &Path) -> Option<Error> {
    walk(table, TOP, "", path)
}

fn walk(table: &toml::Table, schema: &[(&str, Kind)], prefix: &str, path: &Path) -> Option<Error> {
    for (key, value) in table {
        let dotted = format!("{prefix}{key}");
        let Some((_, kind)) = schema.iter().find(|(k, _)| k == key) else {
            return Some(config_err(path, &dotted, "unknown key"));
        };
        if let Some(e) = check(value, *kind, &dotted, path) {
            return Some(e);
        }
    }
    None
}

fn check(value: &toml::Value, kind: Kind, dotted: &str, path: &Path) -> Option<Error> {
    match kind {
        Kind::Leaf(f) => f(value).err().map(|r| config_err(path, dotted, &r)),
        Kind::Table(schema) => match value.as_table() {
            Some(t) => walk(t, schema, &format!("{dotted}."), path),
            None => Some(config_err(path, dotted, "expected a table")),
        },
        Kind::Tables(schema, required) => {
            let Some(items) = value.as_array() else {
                return Some(config_err(path, dotted, "expected an array of tables"));
            };
            items.iter().enumerate().find_map(|(i, item)| {
                let at = format!("{dotted}[{i}]");
                match item.as_table() {
                    None => Some(config_err(path, &at, "expected a table")),
                    Some(t) if !t.contains_key(required) => Some(config_err(
                        path,
                        &format!("{at}.{required}"),
                        "missing required key",
                    )),
                    Some(t) => walk(t, schema, &format!("{at}."), path),
                }
            })
        }
        Kind::RuleTables => {
            let Some(t) = value.as_table() else {
                return Some(config_err(path, dotted, "expected a table"));
            };
            t.iter().find_map(|(code, v)| {
                (!v.is_table())
                    .then(|| config_err(path, &format!("{dotted}.{code}"), "expected a table"))
            })
        }
    }
}
