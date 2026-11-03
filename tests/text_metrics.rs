//! Pipeline metrics for original and final text (FR-3.4).
#![allow(clippy::unwrap_used, clippy::expect_used)]

use sloperator::config::{CliOverrides, Resolver};
use sloperator::engine::DEFAULT_SEED;
use sloperator::input::{Format, Origin, Source};
use sloperator::pipeline::{self, Engines};
use std::sync::Arc;

#[test]
fn fix_reports_before_and_after() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("doc.md");
    let settings = Resolver::new(CliOverrides::default(), None)
        .unwrap()
        .for_file(&path)
        .unwrap();
    let source = Source {
        origin: Origin::File(path),
        format: Format::Markdown,
        text: Arc::from("We utilize the cache in order to look things up.\n".repeat(5)),
        settings,
    };
    let a = pipeline::analyse(source.clone()).unwrap();
    let before = a.prose.counts(&a.doc).clone();
    let r = pipeline::fix(
        a,
        Engines {
            rules: true,
            model: false,
        },
        None,
        DEFAULT_SEED,
    )
    .unwrap();
    let fixed = r.fixed.as_deref().expect("safe fixes change text");
    assert_eq!(fixed, "We use the cache to look things up.\n".repeat(5));
    let after = pipeline::analyse(Source {
        text: Arc::from(fixed),
        ..source
    })
    .unwrap();
    assert_eq!(r.metrics_before, Some(before));
    assert_eq!(r.metrics, *after.prose.counts(&after.doc));
    assert_eq!(r.metrics_before.as_ref().unwrap().words, 50);
    assert_eq!(r.metrics.words, 40);
    assert_eq!(r.floors, after.source.settings.metrics);
}

fn metrics_cli(dir: &std::path::Path, format: &str) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_sloperator"))
        .current_dir(dir)
        .args(["check", "doc.md", "--output-format", format, "--exit-zero"])
        .output()
        .unwrap()
}

#[test]
fn json_identical_across_runs() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("sloperator.toml"),
        "respect_gitignore = false\n",
    )
    .unwrap();
    std::fs::write(tmp.path().join("doc.md"),
        "The cat watched the dog and the bird beside the old tree near the quiet house all morning long. ".repeat(6)).unwrap();
    let a = metrics_cli(tmp.path(), "json");
    let b = metrics_cli(tmp.path(), "json");
    assert!(a.status.success(), "{}", String::from_utf8_lossy(&a.stderr));
    assert!(b.status.success(), "{}", String::from_utf8_lossy(&b.stderr));
    assert_eq!(a.stdout, b.stdout);
    let v: serde_json::Value = serde_json::from_slice(&a.stdout).unwrap();
    assert_eq!(v["files"][0]["metrics"]["sentences"], 6, "{v}");
    assert!(v["files"][0]["metrics"]["avgSentenceLength"].is_number());
}

#[test]
fn summary_uses_run_root_floors() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir(tmp.path().join("sub")).unwrap();
    std::fs::write(
        tmp.path().join("sloperator.toml"),
        "respect_gitignore = false\n[metrics]\nmin_sentences = 10\n",
    )
    .unwrap();
    std::fs::write(
        tmp.path().join("sub/sloperator.toml"),
        "respect_gitignore = false\n[metrics]\nmin_sentences = 5\n",
    )
    .unwrap();
    std::fs::write(tmp.path().join("sub/doc.md"),
        "The cat watched the dog and the bird beside the old tree near the quiet house all morning long. ".repeat(6)).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_sloperator"))
        .current_dir(tmp.path())
        .args([
            "check",
            "sub/doc.md",
            "--output-format",
            "json",
            "--exit-zero",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["summary"]["metrics"]["sentences"], 6, "{v}");
    assert!(v["files"][0]["metrics"]["avgSentenceLength"].is_number());
    assert!(v["summary"]["metrics"]["avgSentenceLength"].is_null());
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_sloperator"))
        .current_dir(tmp.path())
        .args(["check", "sub/doc.md", "--exit-zero"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(
        String::from_utf8(out.stdout)
            .unwrap()
            .contains("needs 10 sentences and 100 words")
    );
}
