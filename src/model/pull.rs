//! `model pull` (SPEC §15, ADR-08): the only network code (tests/layering.rs::network_only_in_pull).
//!
//! The file is streamed into `FILE.part` and hashed as it arrives. Only a matching sha256 renames it to `FILE` and
//! writes the `FILE.sha256` sidecar, so a partial or corrupt download never sits where `resolve` looks.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::{ModelSpec, cached_path, sidecar_path};
use crate::Error;

/// Where the CLI downloads from. Tests pass a local server instead (§23.11).
pub const HF_BASE: &str = "https://huggingface.co";

const CHUNK: usize = 1 << 20;

/// GET `{base}/{repo}/resolve/main/{file}`, streamed and hashed into `FILE.part`; then rename to `FILE` and write
/// `FILE.sha256`. An HTTP error, a short or oversized body, or a sha256 mismatch removes `.part` and returns
/// `Error::Download` naming the URL. Progress lines go to `progress`; a failed progress write is ignored.
pub fn pull(
    spec: &ModelSpec,
    dir: &Path,
    base: &str,
    progress: &mut dyn Write,
) -> Result<PathBuf, Error> {
    let url = format!(
        "{}/{}/resolve/main/{}",
        base.trim_end_matches('/'),
        spec.repo,
        spec.file
    );
    let path = cached_path(dir, spec);
    let parent = path.parent().unwrap_or(dir);
    fs::create_dir_all(parent).map_err(|source| Error::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    let mut part = path.clone().into_os_string();
    part.push(".part");
    let part = PathBuf::from(part);

    let result = download(spec, &url, &part, progress);
    let digest = match result {
        Ok(d) => d,
        Err(e) => {
            let _ = fs::remove_file(&part); // best effort: the download error is the one worth reporting
            return Err(e);
        }
    };
    if digest != spec.sha256 {
        let _ = fs::remove_file(&part);
        return Err(Error::Download {
            url,
            reason: format!("sha256 mismatch: expected {}, got {digest}", spec.sha256),
        });
    }
    fs::rename(&part, &path).map_err(|source| {
        let _ = fs::remove_file(&part);
        Error::Io {
            path: path.clone(),
            source,
        }
    })?;
    let sidecar = sidecar_path(&path);
    fs::write(&sidecar, format!("{digest}  {}\n", spec.file)).map_err(|source| Error::Io {
        path: sidecar.clone(),
        source,
    })?;
    Ok(path)
}

/// Streams `url` into `part`; returns the lowercase hex sha256 of the bytes written.
fn download(
    spec: &ModelSpec,
    url: &str,
    part: &Path,
    progress: &mut dyn Write,
) -> Result<String, Error> {
    let fail = |reason: String| Error::Download {
        url: url.to_string(),
        reason,
    };
    let io = |source: std::io::Error| Error::Io {
        path: part.to_path_buf(),
        source,
    };
    let mut resp = ureq::get(url).call().map_err(|e| fail(e.to_string()))?; // ureq 3.5.0
    let mut reader = resp.body_mut().with_config().limit(spec.bytes + 1).reader();
    let mut file = File::create(part).map_err(io)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; CHUNK];
    let mut done: u64 = 0;
    let mut shown: u64 = 0;
    loop {
        let n = reader.read(&mut buf).map_err(|e| fail(e.to_string()))?;
        if n == 0 {
            break;
        }
        let chunk = buf.get(..n).unwrap_or_default();
        file.write_all(chunk).map_err(io)?;
        hasher.update(chunk);
        done += n as u64;
        let pct = (done.saturating_mul(100) / spec.bytes.max(1)).min(100);
        if pct > shown {
            shown = pct;
            let _ = writeln!(
                progress,
                "{}: {pct}% ({done} of {} bytes)",
                spec.name, spec.bytes
            );
        }
    }
    file.sync_all().map_err(io)?;
    if done != spec.bytes {
        return Err(fail(format!("expected {} bytes, got {done}", spec.bytes)));
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}
