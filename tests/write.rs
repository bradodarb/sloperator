//! Write (SPEC §19, §23.15): unchanged files are skipped, writes are atomic and keep permissions, a failed write
//! leaves no partial file, and stdin's fixed text goes to stdout.
#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may unwrap (architecture §8)

mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use common::FailingFs;
use sloperator::Error;
use sloperator::input::Origin;
use sloperator::pipeline::FileResult;
use sloperator::score::Score;
use sloperator::write::{self, RealFs};

const ZERO: Score = Score {
    value: 0.0,
    prose_words: 0,
    weighted: 0,
};

fn result(origin: Origin, original: &str, fixed: Option<&str>) -> FileResult {
    FileResult {
        origin,
        original: Arc::from(original),
        findings: Vec::new(),
        score: ZERO,
        score_before: Some(ZERO),
        fixed: fixed.map(str::to_string),
    }
}

fn file(dir: &Path, name: &str, text: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, text).unwrap();
    path
}

#[test]
fn unchanged_file_not_written() {
    let tmp = tempfile::tempdir().unwrap();
    let path = file(tmp.path(), "a.md", "Plain text.\n");
    let fs_ = FailingFs::new(u32::MAX);
    let mut out = Vec::new();

    let written = write::commit(
        &[result(Origin::File(path.clone()), "Plain text.\n", None)],
        &fs_,
        &mut out,
    )
    .unwrap();

    assert_eq!(written, 0);
    assert_eq!(fs_.writes.get(), 0, "an unchanged file must not reach Fs");
    assert!(out.is_empty(), "files never print to stdout");
    assert_eq!(fs::read_to_string(&path).unwrap(), "Plain text.\n");
}

#[cfg(unix)]
#[test]
fn atomic_write_keeps_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempfile::tempdir().unwrap();
    let path = file(tmp.path(), "a.md", "Before.\n");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    let mut out = Vec::new();

    let written = write::commit(
        &[result(
            Origin::File(path.clone()),
            "Before.\n",
            Some("After.\n"),
        )],
        &RealFs,
        &mut out,
    )
    .unwrap();

    assert_eq!(written, 1);
    assert_eq!(fs::read_to_string(&path).unwrap(), "After.\n");
    let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(
        mode, 0o644,
        "the temp file's 0600 must not replace the original mode"
    );
    let leftovers: Vec<_> = fs::read_dir(tmp.path()).unwrap().collect();
    assert_eq!(leftovers.len(), 1, "no temp file is left behind");
}

#[test]
fn failure_on_second_file_leaves_files_byte_identical() {
    let tmp = tempfile::tempdir().unwrap();
    let a = file(tmp.path(), "a.md", "First before.\n");
    let b = file(tmp.path(), "b.md", "Second before.\n");
    let fs_ = FailingFs::new(2);
    let mut out = Vec::new();

    let err = write::commit(
        &[
            result(
                Origin::File(a.clone()),
                "First before.\n",
                Some("First after.\n"),
            ),
            result(
                Origin::File(b.clone()),
                "Second before.\n",
                Some("Second after.\n"),
            ),
        ],
        &fs_,
        &mut out,
    )
    .unwrap_err();

    match err {
        Error::Io { path, .. } => {
            assert_eq!(path, b, "the error names the file whose write failed")
        }
        other => panic!("expected Error::Io, got {other:?}"),
    }
    let first = fs::read_to_string(&a).unwrap();
    assert!(
        first == "First before.\n" || first == "First after.\n",
        "the first file is either untouched or fully replaced, never partial: {first:?}"
    );
    assert_eq!(fs::read_to_string(&b).unwrap(), "Second before.\n");
    let names: Vec<_> = fs::read_dir(tmp.path()).unwrap().collect();
    assert_eq!(names.len(), 2, "no temp file is left behind");
}

#[test]
fn stdin_fix_prints_fixed_text() {
    let fs_ = FailingFs::new(1);
    let stdin = Origin::Stdin {
        as_path: PathBuf::from("-"),
    };

    let mut out = Vec::new();
    let written = write::commit(
        &[result(stdin.clone(), "Before.\n", Some("After.\n"))],
        &fs_,
        &mut out,
    )
    .unwrap();
    assert_eq!(written, 0, "stdin is never written to disk");
    assert_eq!(fs_.writes.get(), 0);
    assert_eq!(String::from_utf8(out).unwrap(), "After.\n");

    let mut out = Vec::new();
    write::commit(&[result(stdin, "Unchanged.\n", None)], &fs_, &mut out).unwrap();
    assert_eq!(
        String::from_utf8(out).unwrap(),
        "Unchanged.\n",
        "unchanged stdin is echoed as is"
    );
}
