//! Exit-code contract (SPEC §23.16): 0 clean, 1 on a reported finding, 2 on any error or usage error, plus
//! `--exit-zero`, `--min-severity`, the engine and mode flag conflicts and the `rules` command.

#![allow(clippy::unwrap_used, clippy::expect_used)] // test helpers may unwrap (architecture §8)
mod common;

use common::{fixture, run_cli};
use sloperator::cli::ExitCode;
use sloperator::rules::registry::Registry;
use std::path::PathBuf;

/// Copies fixture `rel`, keeping its file name, into a fresh [`common::cli_tempdir`] and returns the dir (keep it
/// alive) and the copy's path. A global ignore rule that matches the fixture's own location (for example
/// `$TMPDIR/.gitignore` holding `*` when the crate is extracted under `$TMPDIR`) cannot then skip the input.
fn fixture_copy(rel: &str) -> (tempfile::TempDir, PathBuf) {
    let src = fixture(rel);
    let tmp = common::cli_tempdir();
    let dst = tmp.path().join(src.file_name().expect("fixture file name"));
    std::fs::copy(&src, &dst).expect("copy fixture");
    (tmp, dst)
}

#[test]
fn help_and_version_exit_zero() {
    let (code, out, _) = run_cli(&["--help"], "");
    assert_eq!(code, ExitCode::Clean);
    assert!(out.contains("Usage"), "help text missing: {out}");
    let (code, out, _) = run_cli(&["--version"], "");
    assert_eq!(code, ExitCode::Clean);
    assert!(out.contains("sloperator"), "version text missing: {out}");
}

#[test]
fn exit_zero_clean() {
    let (_tmp, clean) = fixture_copy("clean.md");
    let (code, _, err) = run_cli(&["check", &clean.to_string_lossy()], "");
    assert_eq!(code, ExitCode::Clean, "stderr: {err}");
    assert!(err.is_empty(), "stderr: {err}");
}

#[test]
fn exit_two_unknown_flag_names_it() {
    let (code, _, err) = run_cli(&["check", "--bogus-flag"], "");
    assert_eq!(code, ExitCode::Failure);
    assert!(err.contains("--bogus-flag"), "stderr: {err}");
}

#[test]
fn exit_two_missing_path_names_it() {
    let (code, _, err) = run_cli(&["check", "missing.md"], "");
    assert_eq!(code, ExitCode::Failure);
    assert!(err.contains("missing.md"), "stderr: {err}");
}

/// FR-2.7, D-8: a warning finding exits 1 and is printed as `path:line:col: CODE [severity] message`.
#[test]
fn exit_one_on_warning() {
    let (_tmp, path) = fixture_copy("cli/warning.md");
    let path = path.to_string_lossy();
    let (code, out, err) = run_cli(&["check", "--output-format", "concise", &path], "");
    assert_eq!(code, ExitCode::Findings, "stdout: {out} stderr: {err}");
    assert!(err.is_empty(), "stderr: {err}");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 1, "stdout: {out}");
    assert!(
        lines[0].starts_with(&format!("{path}:1:1: SLP003 [warning] ")),
        "stdout: {out}"
    );
}

/// FR-2.7, D-8: suggestions sit below the default minimum severity (warning), so they are neither reported nor
/// counted toward exit 1.
#[test]
fn exit_zero_on_suggestion_only() {
    let (_tmp, path) = fixture_copy("cli/suggestion.md");
    let (code, out, err) = run_cli(
        &[
            "check",
            "--output-format",
            "concise",
            &path.to_string_lossy(),
        ],
        "",
    );
    assert_eq!(code, ExitCode::Clean, "stdout: {out} stderr: {err}");
    assert!(err.is_empty(), "stderr: {err}");
    assert!(out.is_empty(), "stdout: {out}");
}

/// A directive error (an unknown rule code) exits 2 and names the file and line (FR-6.5).
#[test]
fn exit_two_on_directive_error() {
    let (_tmp, path) = fixture_copy("cli/bad_directive.md");
    let path = path.to_string_lossy();
    let (code, _, err) = run_cli(&["check", &path], "");
    assert_eq!(code, ExitCode::Failure, "stderr: {err}");
    assert!(err.contains("bad_directive.md"), "stderr: {err}");
}

/// FR-2.7: an invalid config file exits 2 and names the file and the offending key.
#[test]
fn exit_two_on_bad_config() {
    let config = fixture("config/errors/unknown_key.toml");
    let clean = fixture("clean.md");
    let (code, _, err) = run_cli(
        &[
            "check",
            "--config",
            &config.to_string_lossy(),
            &clean.to_string_lossy(),
        ],
        "",
    );
    assert_eq!(code, ExitCode::Failure, "stderr: {err}");
    assert!(err.contains("unknown_key.toml"), "stderr: {err}");
    assert!(err.contains("model.windwo"), "stderr: {err}");
}

/// FR-2.15: an unknown code in a rule flag exits 2 and names the code.
#[test]
fn exit_two_unknown_rule_flag() {
    let clean = fixture("clean.md");
    for flag in ["--select", "--extend-select", "--ignore"] {
        let (code, _, err) = run_cli(&["check", flag, "SLP999", &clean.to_string_lossy()], "");
        assert_eq!(code, ExitCode::Failure, "{flag}: stderr: {err}");
        assert!(err.contains("SLP999"), "{flag}: stderr: {err}");
    }
}

/// FR-2.8: `--exit-zero` turns exit 1 into 0, but an error still exits 2.
#[test]
fn exit_zero_flag_masks_findings_not_errors() {
    let (_tmp_warning, warning) = fixture_copy("cli/warning.md");
    let warning = warning.to_string_lossy();
    let (code, out, err) = run_cli(&["check", &warning], "");
    assert_eq!(code, ExitCode::Findings, "stdout: {out} stderr: {err}");
    let (code, out, err) = run_cli(&["check", "--exit-zero", &warning], "");
    assert_eq!(code, ExitCode::Clean, "stdout: {out} stderr: {err}");
    assert!(out.contains("SLP003"), "findings are still reported: {out}");

    let (_tmp_bad, bad) = fixture_copy("cli/bad_directive.md");
    let (code, _, err) = run_cli(
        &["check", "--exit-zero", &warning, &bad.to_string_lossy()],
        "",
    );
    assert_eq!(code, ExitCode::Failure, "stderr: {err}");
    let (code, _, err) = run_cli(&["check", "--exit-zero", "missing.md"], "");
    assert_eq!(code, ExitCode::Failure, "stderr: {err}");
}

/// FR-2.9: `--min-severity` decides both what is reported and what counts toward exit 1.
#[test]
fn min_severity_filters_report_and_exit() {
    let (_tmp_suggestion, suggestion) = fixture_copy("cli/suggestion.md");
    let suggestion = suggestion.to_string_lossy();
    let (code, out, err) = run_cli(
        &[
            "check",
            "--output-format",
            "concise",
            "--min-severity",
            "suggestion",
            &suggestion,
        ],
        "",
    );
    assert_eq!(code, ExitCode::Findings, "stdout: {out} stderr: {err}");
    assert!(out.contains(" SLP002 [suggestion] "), "stdout: {out}");

    let (_tmp_warning, warning) = fixture_copy("cli/warning.md");
    let (code, out, err) = run_cli(
        &[
            "check",
            "--output-format",
            "concise",
            "--min-severity",
            "error",
            &warning.to_string_lossy(),
        ],
        "",
    );
    assert_eq!(code, ExitCode::Clean, "stdout: {out} stderr: {err}");
    assert!(
        out.is_empty(),
        "a warning is below --min-severity error: {out}"
    );

    let (code, _, err) = run_cli(&["check", "--min-severity", "loud", &suggestion], "");
    assert_eq!(code, ExitCode::Failure, "stderr: {err}");
    assert!(err.contains("loud"), "stderr: {err}");
}

/// FR-2.10: the two engine flags together are a usage error.
#[test]
fn rules_only_and_model_only_together_exit_two() {
    let clean = fixture("clean.md");
    let (code, _, err) = run_cli(
        &[
            "check",
            "--rules-only",
            "--model-only",
            &clean.to_string_lossy(),
        ],
        "",
    );
    assert_eq!(code, ExitCode::Failure, "stderr: {err}");
    assert!(
        err.contains("--rules-only") && err.contains("--model-only"),
        "stderr: {err}"
    );
}

/// FR-2.12: `rules` lists every rule with its default state and fix class; `rules CODE` explains one with its
/// example; an unknown code exits 2 naming it.
#[test]
fn rules_list_and_explain() {
    let reg = Registry::builtin();
    let (code, out, err) = run_cli(&["rules"], "");
    assert_eq!(code, ExitCode::Clean, "stderr: {err}");
    assert!(out.contains(reg.version), "rule-set version missing: {out}");
    for r in reg.all() {
        let m = r.meta();
        let line = out
            .lines()
            .find(|l| l.starts_with(&m.code.to_string()))
            .unwrap_or_else(|| panic!("{} missing: {out}", m.code));
        assert!(line.contains(m.name), "{line}");
        assert!(
            line.contains(if m.default_on { " on " } else { " off " }),
            "{line}"
        );
    }

    let slp001 = reg
        .all()
        .find(|r| r.meta().code.to_string() == "SLP001")
        .unwrap()
        .meta();
    let (code, out, err) = run_cli(&["rules", "SLP001"], "");
    assert_eq!(code, ExitCode::Clean, "stderr: {err}");
    assert!(out.contains(slp001.name), "{out}");
    assert!(out.contains(slp001.explain), "{out}");
    assert!(
        out.contains(slp001.example.0) && out.contains(slp001.example.1),
        "{out}"
    );

    let (code, _, err) = run_cli(&["rules", "SLP999"], "");
    assert_eq!(code, ExitCode::Failure, "stderr: {err}");
    assert!(err.contains("SLP999"), "stderr: {err}");
}

/// DQ-11: `--fix` with `--diff` is a usage error.
#[test]
fn fix_and_diff_together_exit_two() {
    let clean = fixture("clean.md");
    let (code, _, err) = run_cli(&["check", "--fix", "--diff", &clean.to_string_lossy()], "");
    assert_eq!(code, ExitCode::Failure, "stderr: {err}");
    assert!(
        err.contains("--fix") && err.contains("--diff"),
        "stderr: {err}"
    );
}

/// DQ-4 (deferred item 4), FR-8.7, FR-2.8, FR-2.9: SLP901 is a warning that passes `--min-severity error`, makes the
/// run exit 1, and under `--exit-zero` the run exits 0 with SLP901 still printed.
#[test]
fn slp901_survives_min_severity_and_exit_zero() {
    let tmp = common::cli_tempdir();
    common::plant_model_config(tmp.path());
    let doc = tmp.path().join("doc.md");
    let text = "As an AI language model, I count 3 apples.\n";
    std::fs::write(&doc, text).unwrap();
    let path = doc.to_string_lossy();
    let base = [
        "check",
        "--fix",
        "--min-severity",
        "error",
        "--output-format",
        "concise",
    ];
    let slp901 = |out: &str| {
        out.lines().any(|l| {
            l.starts_with(&format!(
                "{path}:1:1: SLP901 [warning] rewrite rejected (numbers)"
            ))
        })
    };

    let factory = Box::new(common::FakeFactory::returning("I count apples."));
    let (code, out, err) = common::run_cli_with(&[&base[..], &[&path]].concat(), "", factory);
    assert_eq!(code, ExitCode::Findings, "stdout: {out} stderr: {err}");
    assert!(slp901(&out), "stdout: {out}");

    let factory = Box::new(common::FakeFactory::returning("I count apples."));
    let (code, out, err) =
        common::run_cli_with(&[&base[..], &["--exit-zero", &path]].concat(), "", factory);
    assert_eq!(code, ExitCode::Clean, "stdout: {out} stderr: {err}");
    assert!(slp901(&out), "stdout: {out}");
    assert_eq!(
        std::fs::read_to_string(&doc).unwrap(),
        text,
        "the original is kept"
    );
}
