//! Write (SPEC §19, FR-2.6, NFR-5): commits `--fix` results. Files are replaced atomically and keep their
//! permissions; stdin's fixed text goes to stdout.
//!
//! Never uses `engine` or `rules`: it consumes the pipeline's `FileResult` only.

use std::io::{self, Write};
use std::path::Path;

use crate::Error;
use crate::input::Origin;
use crate::pipeline::FileResult;

/// The file-system seam (architecture §11): tests inject a failing implementation.
pub trait Fs {
    /// Replaces `path` with `bytes` so that a failure at any point leaves the original byte-identical.
    ///
    /// # Errors
    /// Any I/O error; the original file is untouched.
    fn write_atomic(&self, path: &Path, bytes: &[u8]) -> io::Result<()>;
}

/// The real file system: a temp file in the same directory, synced, given the original's permissions, then
/// renamed over the original.
pub struct RealFs;

impl Fs for RealFs {
    fn write_atomic(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        let dir = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let perms = std::fs::metadata(path)?.permissions();
        let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
        tmp.write_all(bytes)?;
        tmp.as_file().sync_all()?;
        // NamedTempFile is created 0600; without this --fix would tighten every file it touches.
        std::fs::set_permissions(tmp.path(), perms)?;
        tmp.persist(path).map_err(|e| e.error)?;
        Ok(())
    }
}

/// Commits the results in order and returns the number of files written. Called only when the run produced no
/// exit-2 error (DQ-10). A file is written only when `fixed` is set, i.e. its text changed. With stdin, the fixed
/// text (or the original, if unchanged) is printed to `stdout` (FR-2.6).
///
/// # Errors
/// `Error::Io` naming the path whose write failed; earlier files stay fully replaced, later ones untouched (NFR-5).
/// A failed write to `stdout` is `Error::Io` naming the stdin display path.
pub fn commit(results: &[FileResult], fs: &dyn Fs, stdout: &mut dyn Write) -> Result<usize, Error> {
    let mut written = 0;
    for r in results {
        match &r.origin {
            Origin::File(path) => {
                if let Some(fixed) = &r.fixed {
                    fs.write_atomic(path, fixed.as_bytes())
                        .map_err(|source| Error::Io {
                            path: path.clone(),
                            source,
                        })?;
                    written += 1;
                }
            }
            Origin::Stdin { as_path } => {
                let text = r.fixed.as_deref().unwrap_or(&r.original);
                stdout
                    .write_all(text.as_bytes())
                    .and_then(|()| stdout.flush())
                    .map_err(|source| Error::Io {
                        path: as_path.clone(),
                        source,
                    })?;
            }
        }
    }
    Ok(written)
}
