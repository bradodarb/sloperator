//! Layering scans (SPEC §5, architecture §5): text scans over `src/` that keep the module rules honest.
//! Each test holds its table as a `const` slice, so adding a module means adding a row.
#![allow(clippy::unwrap_used, clippy::expect_used)] // test helpers may unwrap (architecture §8)

use std::fs;
use std::path::Path;

// Rows derived from architecture §5.
const FORBIDDEN_USES: &[(&str, &[&str])] = &[
    ("src/doc/", &["crate::"]),
    ("src/types.rs", &["crate::"]), // leaf: RuleCode, Proposal, ProposalOrigin (Gap 3)
    (
        "src/rules/",
        &[
            "crate::engine",
            "crate::verify",
            "crate::edit",
            "crate::write",
            "crate::report",
        ],
    ),
    (
        "src/score/",
        &[
            "crate::engine",
            "crate::write",
            "crate::report",
            "crate::input",
        ],
    ),
    (
        "src/edit/",
        &[
            "crate::engine",
            "crate::rules",
            "crate::input",
            "crate::write",
        ],
    ),
    (
        "src/verify/",
        &["crate::engine", "crate::write", "crate::report"],
    ),
    (
        "src/config/",
        &[
            "crate::pipeline",
            "crate::engine",
            "crate::report",
            "crate::write",
        ],
    ),
    (
        "src/input/",
        &["crate::rules", "crate::engine", "crate::write"],
    ),
    ("src/directives/", &["crate::engine", "crate::edit"]),
    (
        "src/model/",
        &["crate::pipeline", "crate::rules", "crate::engine"],
    ),
    (
        "src/engine/",
        &["crate::write", "crate::report", "crate::cli"],
    ),
    (
        "src/pipeline/",
        &[
            "crate::cli",
            "crate::report",
            "crate::write",
            "crate::engine::runtime",
        ],
    ),
    (
        "src/report/",
        &["crate::engine", "crate::write", "crate::config::Resolver"],
    ),
    ("src/write/", &["crate::engine", "crate::rules"]),
    (
        "src/cli/",
        &["crate::engine::runtime", "llama_cpp_2", "mistralrs"],
    ),
    // ADR-10: a thin adapter over pipeline/report/rules and the engine port. Text in, text out: no `write`.
    (
        "src/mcp/",
        &[
            "crate::write",
            "crate::cli",
            "crate::engine::runtime",
            "llama_cpp_2",
            "mistralrs",
        ],
    ),
];
const PURE_DIRS: &[&str] = &[
    "src/doc/",
    "src/types.rs",
    "src/rules/",
    "src/score/",
    "src/edit/",
    "src/verify/",
];
const IO_PATHS: &[&str] = &["std::fs", "std::net", "std::process", "std::env"];

/// Every file under `src/` as (path relative to the crate root with `/` separators, contents), sorted by path.
fn src_files() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut out = Vec::new();
    collect(root, "src", &mut out);
    out.sort();
    out
}

fn collect(root: &Path, rel: &str, out: &mut Vec<(String, String)>) {
    let entries = fs::read_dir(root.join(rel)).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let rel_path = format!("{rel}/{}", entry.file_name().to_string_lossy());
        if entry.file_type().unwrap().is_dir() {
            collect(root, &rel_path, out);
        } else {
            let text = fs::read_to_string(root.join(&rel_path)).unwrap();
            out.push((rel_path, text));
        }
    }
}

/// The files a row's path names: a directory prefix (`src/doc/`) or one file (`src/types.rs`).
fn in_row<'a>(
    files: &'a [(String, String)],
    row: &'a str,
) -> impl Iterator<Item = &'a (String, String)> {
    files.iter().filter(move |(path, _)| path.starts_with(row))
}

#[test]
fn forbidden_uses() {
    let files = src_files();
    let mut violations = Vec::new();
    for (row, forbidden) in FORBIDDEN_USES {
        let mut matched = 0;
        for (path, text) in in_row(&files, row) {
            matched += 1;
            // `doc` may use only `crate::error`; that one exception is coded in.
            let text = if *row == "src/doc/" {
                text.replace("crate::error", "")
            } else {
                text.clone()
            };
            for f in *forbidden {
                if text.contains(f) {
                    violations.push(format!("{path} uses {f}"));
                }
            }
        }
        assert!(matched > 0, "row {row} matches no file under src/");
    }
    assert!(
        violations.is_empty(),
        "forbidden uses:\n{}",
        violations.join("\n")
    );
}

#[test]
fn pure_modules_have_no_io() {
    let files = src_files();
    let mut violations = Vec::new();
    for row in PURE_DIRS {
        let mut matched = 0;
        for (path, text) in in_row(&files, row) {
            matched += 1;
            for io in IO_PATHS {
                if text.contains(io) {
                    violations.push(format!("{path} uses {io}"));
                }
            }
        }
        assert!(matched > 0, "pure row {row} matches no file under src/");
    }
    assert!(
        violations.is_empty(),
        "I/O in pure modules:\n{}",
        violations.join("\n")
    );
}

/// Asserts that `needle` appears in no `src/` file except `allowed`.
fn only_in(needles: &[&str], allowed: &str) {
    let violations: Vec<String> = src_files()
        .into_iter()
        .filter(|(path, _)| path != allowed)
        .flat_map(|(path, text)| {
            needles
                .iter()
                .filter(|n| text.contains(**n))
                .map(|n| format!("{path} mentions {n}"))
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(
        violations.is_empty(),
        "only {allowed} may mention these:\n{}",
        violations.join("\n")
    );
}

#[test]
fn runtime_crate_only_in_adapter() {
    only_in(&["llama_cpp_2", "mistralrs"], "src/engine/runtime.rs");
}

#[test]
fn network_only_in_pull() {
    only_in(&["ureq"], "src/model/pull.rs");
}

#[test]
fn mcp_sdk_only_in_server() {
    only_in(&["rmcp"], "src/mcp/mod.rs");
}

#[test]
fn single_exit_point() {
    only_in(&["std::process::exit"], "src/main.rs");
}
