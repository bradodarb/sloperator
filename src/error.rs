use std::path::PathBuf;

/// Every failure that exits 2 (NFR-4). Rejected edits are findings (SLP901/SLP902), never errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{path}: not valid UTF-8 (byte {offset}); file left unchanged")]
    NotUtf8 { path: PathBuf, offset: usize },
    #[error("{file}: config key `{key}`: {reason}")]
    Config {
        file: PathBuf,
        key: String,
        reason: String,
    },
    #[error("unknown rule code `{code}`{}", where_.as_ref().map(|w| format!(" in {w}")).unwrap_or_default())]
    UnknownRule {
        code: String,
        where_: Option<String>,
    },
    #[error("{file}:{line}: {reason}")]
    Directive {
        file: PathBuf,
        line: u32,
        reason: String,
    },
    #[error("usage: {0}")]
    Usage(String),
    #[error(
        "model `{name}` is not in the cache ({path}); run `sloperator model pull {name}` or pass --rules-only"
    )]
    ModelMissing { name: String, path: PathBuf },
    #[error(
        "this binary was built without model support; rebuild with the `model` feature or pass --rules-only"
    )]
    ModelUnsupported,
    #[error("loading model {path}: {reason}")]
    ModelLoad { path: PathBuf, reason: String },
    #[error("model runtime: {0}")]
    Runtime(String),
    #[error("download {url}: {reason}")]
    Download { url: String, reason: String },
}
