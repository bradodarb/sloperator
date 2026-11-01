//! Model registry, cache and `pull` (SPEC §15, ADR-08).
//!
//! The runtime only ever receives a file path from here, so resolving a model makes no network request (FR-9.1).
//! Only `pull` touches the network.

use std::path::{Path, PathBuf};

use crate::Error;
use crate::config::ModelSettings;

pub mod pull;
pub mod registry;

pub use registry::{DEFAULT_MODEL, ModelSpec, PROFILES, PromptProfile, REGISTRY};

/// A model resolved to a file on disk, with the profile it is prompted with.
#[derive(Debug)]
pub struct ResolvedModel {
    pub name: String,
    pub path: PathBuf,
    pub profile: &'static PromptProfile,
}

/// `model.cache_dir`, else `dirs::cache_dir()/sloperator/models`.
pub fn cache_dir(s: &ModelSettings) -> Result<PathBuf, Error> {
    if let Some(dir) = &s.cache_dir {
        return Ok(dir.clone());
    }
    dirs::cache_dir()
        .map(|d| d.join("sloperator").join("models"))
        .ok_or_else(|| {
            Error::Usage(
                "no user cache directory on this platform; set `model.cache_dir` in sloperator.toml".into(),
            )
        })
}

/// `CACHE/<repo with "/" replaced by "__">/<file>`.
pub fn cached_path(dir: &Path, spec: &ModelSpec) -> PathBuf {
    dir.join(spec.repo.replace('/', "__")).join(spec.file)
}

/// `FILE.sha256`: the sidecar `pull` writes after a verified download.
pub fn sidecar_path(file: &Path) -> PathBuf {
    let mut s = file.as_os_str().to_os_string();
    s.push(".sha256");
    PathBuf::from(s)
}

/// The cached file is present and its sidecar records `spec.sha256` (`model pull` then has nothing to do).
pub fn up_to_date(dir: &Path, spec: &ModelSpec) -> bool {
    let path = cached_path(dir, spec);
    path.is_file()
        && std::fs::read_to_string(sidecar_path(&path))
            .is_ok_and(|s| s.split_whitespace().next() == Some(spec.sha256))
}

/// `model.path` (with its profile), or the registry entry's cached file. Absent → `Error::ModelMissing` (FR-9.6).
pub fn resolve(s: &ModelSettings) -> Result<ResolvedModel, Error> {
    if let Some(path) = &s.path {
        let profile = match s.profile.as_deref() {
            Some(name) => profile(name).ok_or_else(|| Error::ModelLoad {
                path: path.clone(),
                reason: format!(
                    "unknown prompt profile `{name}` (known: {})",
                    PROFILES
                        .iter()
                        .map(|p| p.name)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            })?,
            None => {
                return Err(Error::Usage("`model.path` requires `model.profile`".into()));
            }
        };
        if !path.is_file() {
            return Err(Error::ModelMissing {
                name: s.name.clone(),
                path: path.clone(),
            });
        }
        return Ok(ResolvedModel {
            name: s.name.clone(),
            path: path.clone(),
            profile,
        });
    }
    let spec = spec(&s.name).ok_or_else(|| {
        Error::Usage(format!(
            "unknown model `{}` (known: {})",
            s.name,
            REGISTRY
                .iter()
                .map(|m| m.name)
                .collect::<Vec<_>>()
                .join(", ")
        ))
    })?;
    let path = cached_path(&cache_dir(s)?, spec);
    if !path.is_file() {
        return Err(Error::ModelMissing {
            name: spec.name.to_string(),
            path,
        });
    }
    Ok(ResolvedModel {
        name: spec.name.to_string(),
        path,
        profile: spec.profile,
    })
}

/// The registry entry named `name`.
pub fn spec(name: &str) -> Option<&'static ModelSpec> {
    REGISTRY.iter().find(|m| m.name == name)
}

/// The prompt profile named `name`.
pub fn profile(name: &str) -> Option<&'static PromptProfile> {
    PROFILES.iter().copied().find(|p| p.name == name)
}
