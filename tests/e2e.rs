//! End to end (SPEC §23.17): the built binary, run through `assert_cmd` in a temp directory. Protected bytes survive
//! `--fix` (NFR-8), `--diff` and plain `check` write nothing (FR-2.5, FR-2.1), JSON is byte-stable (NFR-6), and a
//! `--fix` that needs the model exits 2 before any write (FR-9.6). This build has no runtime, so the model is
//! always unavailable.
#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may unwrap (architecture §8)

mod common;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use assert_cmd::Command;
use common::fixture;
use sloperator::doc::{self, Format, ProtectSettings};

/// Prose with safe fixes (SLP001, SLP013), so `--fix --rules-only` has something to change.
const SLOP: &str = "Certainly! We utilize the cache in order to look things up.\n";

/// A temp directory with its own `sloperator.toml`, so a `.gitignore` above the system temp dir cannot hide its files.
fn workdir() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(
        tmp.path().join("sloperator.toml"),
        "respect_gitignore = false\n",
    )
    .unwrap();
    tmp
}

fn bin(dir: &Path) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_sloperator"));
    c.current_dir(dir).env("NO_COLOR", "1");
    c
}

/// `mixed.md` (code, links, math, a table, front matter, a reference definition) with a fixable paragraph appended.
fn mixed_with_slop(dir: &Path) -> PathBuf {
    let path = dir.join("mixed.md");
    let text = fs::read_to_string(fixture("protect/mixed.md")).unwrap();
    fs::write(&path, format!("{text}\n{SLOP}")).unwrap();
    path
}

/// Every file under `dir` with its bytes.
fn snapshot(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.insert(p.clone(), fs::read(&p).unwrap());
            }
        }
    }
    out
}

fn protected(text: &str) -> Vec<String> {
    let d = doc::parse(
        Arc::from(text),
        Format::Markdown,
        &ProtectSettings::default(),
    );
    let mut s: Vec<String> = d
        .protected_strings()
        .into_iter()
        .map(str::to_string)
        .collect();
    s.sort();
    s
}

#[test]
fn mixed_fixture_protected_bytes_unchanged_after_rules_fix() {
    let tmp = workdir();
    let path = mixed_with_slop(tmp.path());
    let before = fs::read_to_string(&path).unwrap();

    let out = bin(tmp.path())
        .args(["check", "--fix", "--rules-only", "mixed.md"])
        .output()
        .unwrap();
    assert_ne!(
        out.status.code(),
        Some(2),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let after = fs::read_to_string(&path).unwrap();
    assert_ne!(after, before, "the safe fixes were not written");
    assert!(
        after.contains("We use the cache to look things up."),
        "{after}"
    );
    assert!(!protected(&before).is_empty());
    assert_eq!(
        protected(&after),
        protected(&before),
        "protected bytes changed:\n{after}"
    );
}

#[test]
fn diff_writes_nothing() {
    let tmp = workdir();
    mixed_with_slop(tmp.path());
    let before = snapshot(tmp.path());

    let out = bin(tmp.path())
        .args(["check", "--diff", "--rules-only", "mixed.md"])
        .output()
        .unwrap();
    assert_ne!(
        out.status.code(),
        Some(2),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("--- a/") && stdout.contains("+++ b/") && stdout.contains("@@"),
        "{stdout}"
    );
    assert!(stdout.contains("-Certainly! We utilize"), "{stdout}");
    // The report moves to stderr because stdout carries the diff (DQ-11).
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("score"),
        "report missing from stderr: {stderr}"
    );
    assert_eq!(
        snapshot(tmp.path()),
        before,
        "--diff wrote to the directory"
    );
}

#[test]
fn check_modifies_nothing() {
    let tmp = workdir();
    mixed_with_slop(tmp.path());
    let before = snapshot(tmp.path());

    let out = bin(tmp.path())
        .args(["check", "mixed.md"])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(1),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(snapshot(tmp.path()), before, "check wrote to the directory");
}

#[test]
fn json_identical_across_runs_and_shuffled_dirs() {
    let tmp = workdir();
    let root = tmp.path();
    let files: [(&str, &str); 4] = [
        ("a.md", SLOP),
        ("sub/b.md", "It's not a bug, it's a feature.\n"),
        (
            "sub/deeper/c.md",
            "The garden is a rich tapestry of colour.\n",
        ),
        ("z.txt", SLOP),
    ];
    let create = |order: &[usize]| {
        for &i in order {
            let p = root.join(files[i].0);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(&p, files[i].1).unwrap();
        }
    };
    let json = || {
        let out = bin(root)
            .args(["check", "--output-format", "json", "."])
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(1),
            "stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        out.stdout
    };

    create(&[0, 1, 2, 3]);
    let first = json();
    assert!(!first.is_empty());
    assert_eq!(json(), first, "two runs differ");

    // Same tree, written in the opposite order, so directory entries come back in a different order.
    for e in fs::read_dir(root).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            fs::remove_dir_all(&p).unwrap();
        } else if p.extension().is_some_and(|x| x != "toml") {
            fs::remove_file(&p).unwrap();
        }
    }
    create(&[3, 2, 1, 0]);
    assert_eq!(json(), first, "creation order changed the output");
}

#[test]
fn fix_without_model_exits_two() {
    let tmp = workdir();
    mixed_with_slop(tmp.path());
    let before = snapshot(tmp.path());

    let out = bin(tmp.path())
        .args(["check", "--fix", "mixed.md"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "stderr: {stderr}");
    assert!(
        stderr.contains("--rules-only"),
        "the error names the way out: {stderr}"
    );
    assert_eq!(
        snapshot(tmp.path()),
        before,
        "a file was written before the model check"
    );
}
