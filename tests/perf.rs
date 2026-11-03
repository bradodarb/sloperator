//! NFR-3 benchmark (SPEC §23.18): `check --rules-only` on 100 KB of Markdown finishes in under 1 s of wall time and
//! never reads the model cache. Runs only in the `ci-perf` release pass (`make test`, `RUN_PERF` in the Makefile, §3).
#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may unwrap (architecture §8)

mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use assert_cmd::Command;
use common::fixture;

/// NFR-3: the input size and the wall-time budget.
const INPUT_BYTES: usize = 100 * 1024;
const BUDGET: Duration = Duration::from_secs(1);

/// Restores the mode of a directory on drop, so `TempDir` can remove it even when the test fails.
#[cfg(unix)]
struct RestoreMode(PathBuf);

#[cfg(unix)]
impl Drop for RestoreMode {
    fn drop(&mut self) {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&self.0, fs::Permissions::from_mode(0o755));
    }
}

/// Every Markdown file under `fixtures/protect` and the `positive.md`/`negative.md` of every rule under
/// `fixtures/rules`, minus those carrying inline directives, sorted so the generated input is the same on every run.
fn corpus() -> Vec<String> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for e in fs::read_dir(fixture("protect")).unwrap() {
        let p = e.unwrap().path();
        if p.extension().is_some_and(|x| x == "md") {
            paths.push(p);
        }
    }
    for e in fs::read_dir(fixture("rules")).unwrap() {
        let dir = e.unwrap().path();
        for name in ["positive.md", "negative.md"] {
            let p = dir.join(name);
            if p.is_file() {
                paths.push(p);
            }
        }
    }
    paths.sort();
    // A fixture with inline directives is balanced only on its own; repeated or cut off it is a config error (exit 2).
    let parts: Vec<String> = paths
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .filter(|text| !text.contains("sloperator-"))
        .collect();
    assert!(
        !parts.is_empty(),
        "no fixtures found to build the perf input"
    );
    parts
}

/// At least `INPUT_BYTES` of Markdown, made by repeating the fixture corpus.
fn markdown_input() -> String {
    let parts = corpus();
    let mut out = String::with_capacity(INPUT_BYTES + 4096);
    'fill: loop {
        for part in &parts {
            out.push_str(part);
            out.push_str("\n\n");
            if out.len() >= INPUT_BYTES {
                break 'fill;
            }
        }
    }
    out
}

/// A TOML basic string holding `path`.
fn toml_path(path: &Path) -> String {
    let s = path.to_str().unwrap();
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(unix)]
#[test]
fn rules_only_100kb_under_1s() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().join("model-cache");
    fs::create_dir(&cache).unwrap();
    fs::set_permissions(&cache, fs::Permissions::from_mode(0o000)).unwrap();
    let _restore = RestoreMode(cache.clone());

    // A privileged user (uid 0) ignores mode 000, so the test could not prove the cache is unread: skip loudly.
    if fs::read_dir(&cache).is_ok() {
        eprintln!(
            "SKIPPED rules_only_100kb_under_1s: mode 000 on {} is not enforced for this user (running as root?), \
             so an unread cache cannot be proven",
            cache.display()
        );
        return;
    }

    fs::write(
        tmp.path().join("sloperator.toml"),
        format!(
            "respect_gitignore = false\nextend_select = [\"SLP018\", \"SLP019\", \"SLP020\", \"SLP021\", \"SLP022\"]\n\n[model]\ncache_dir = {}\n",
            toml_path(&cache)
        ),
    )
    .unwrap();
    let input = markdown_input();
    assert!(input.len() >= INPUT_BYTES, "input is {} bytes", input.len());
    fs::write(tmp.path().join("big.md"), &input).unwrap();

    let start = Instant::now();
    let out = Command::new(env!("CARGO_BIN_EXE_sloperator"))
        .current_dir(tmp.path())
        .env("NO_COLOR", "1")
        .args(["check", "--rules-only", "big.md"])
        .output()
        .unwrap();
    let elapsed = start.elapsed();

    let code = out.status.code();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        matches!(code, Some(0 | 1)),
        "check --rules-only must exit by findings (0 or 1), got {code:?}; stderr:\n{stderr}"
    );
    assert!(
        elapsed < BUDGET,
        "NFR-3: check --rules-only on {} bytes took {elapsed:?} (budget {BUDGET:?})",
        input.len()
    );
}
