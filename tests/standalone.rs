//! Standalone scans (SPEC §5): the repository must hold no reference to the project it was developed in, nor to that
//! project's build-contract paths.
//!
//! Every needle is assembled at runtime from fragments, so no needle appears literally in this file: the scan covers
//! this file too, with no exemption, and a literal list would match itself on every run. The scan is
//! case-insensitive, so no identifier, comment or string in this file may contain a needle in any case: the function
//! is `host_markers` and the test is `no_host_references`.

#![allow(clippy::unwrap_used, clippy::expect_used)] // test helpers may unwrap (architecture §8)

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The needles, lower-cased: the development host's name, its build-contract paths and verbs, and its build tool.
fn host_markers() -> Vec<String> {
    let host = ["ae", "gis"].concat();
    let fragments: &[&[&str]] = &[
        &[&host],
        &["modules", "/", "sloperator"],
        &["directives", "/", "sloperator"],
        &["docs", "/", "sloperator"],
        &["make", " -", "C"],
        &["UN", "IT="],
        &["module", ".mk"],
        &["MODULE", "_NAMESPACES"],
        &["units", ".sh"],
        &["grad", "lew"],
        &["GRAD", "LE_"],
    ];
    fragments
        .iter()
        .map(|f| f.concat().to_lowercase())
        .collect()
}

/// Top-level entries that are not part of the repository's content: VCS metadata and build output.
const NOT_SCANNED: &[&str] = &[".git", "target"];

/// Every file under `root` except the top-level `NOT_SCANNED` entries, as paths relative to `root`.
fn crate_files(root: &Path) -> Vec<PathBuf> {
    fn walk(root: &Path, rel: &Path, out: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(root.join(rel)).unwrap() {
            let entry = entry.unwrap();
            let rel_path = rel.join(entry.file_name());
            if rel.as_os_str().is_empty() && NOT_SCANNED.iter().any(|n| entry.file_name() == *n) {
                continue;
            }
            if entry.file_type().unwrap().is_dir() {
                walk(root, &rel_path, out);
            } else {
                out.push(rel_path);
            }
        }
    }
    let mut out = Vec::new();
    walk(root, Path::new(""), &mut out);
    out.sort();
    out
}

/// The scan itself, so the planted-reference test exercises the same code as `no_host_references`.
/// Returns (path relative to `root`, first marker found), matching case-insensitively; non-UTF-8 files are skipped.
fn find_markers(root: &Path) -> Vec<(PathBuf, String)> {
    let markers = host_markers();
    let mut found = Vec::new();
    for rel in crate_files(root) {
        let Ok(text) = fs::read_to_string(root.join(&rel)) else {
            continue;
        };
        let text = text.to_lowercase();
        if let Some(marker) = markers.iter().find(|m| text.contains(m.as_str())) {
            found.push((rel, marker.clone()));
        }
    }
    found
}

fn crate_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn no_host_references() {
    let found = find_markers(crate_root());
    assert!(found.is_empty(), "host references found: {found:?}");
}

#[test]
fn planted_reference_is_caught() {
    let tmp = tempfile::TempDir::new().unwrap();
    let files = ["src/lib.rs", "tests/standalone.rs"];
    for file in files {
        let dest = tmp.path().join(file);
        fs::create_dir_all(dest.parent().unwrap()).unwrap();
        fs::copy(crate_root().join(file), &dest).unwrap();
    }
    // The unmodified copies report nothing: the fragment form cannot hide a real reference.
    assert_eq!(find_markers(tmp.path()), vec![]);

    let word = ["ae", "gis"].concat();
    let lib = tmp.path().join("src/lib.rs");
    let mut text = fs::read_to_string(&lib).unwrap();
    text.push_str(&format!("\n// planted reference to {word}\n"));
    fs::write(&lib, text).unwrap();

    let found = find_markers(tmp.path());
    assert_eq!(
        found.len(),
        1,
        "expected exactly the planted file: {found:?}"
    );
    assert_eq!(found[0].0, PathBuf::from("src/lib.rs"));
}

#[test]
fn no_external_path_deps() {
    let manifest = fs::read_to_string(crate_root().join("Cargo.toml")).unwrap();
    let doc: toml::Table = toml::from_str(&manifest).unwrap();
    const DEP_TABLES: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];

    let mut violations = Vec::new();
    let mut check = |scope: &str, tables: &toml::Table| {
        for dep_table in DEP_TABLES {
            let Some(deps) = tables.get(*dep_table).and_then(|v| v.as_table()) else {
                continue;
            };
            for (name, spec) in deps {
                if spec.as_table().is_some_and(|t| t.contains_key("path")) {
                    violations.push(format!("{scope}{dep_table}.{name}"));
                }
            }
        }
    };
    check("", &doc);
    if let Some(targets) = doc.get("target").and_then(|v| v.as_table()) {
        for (cfg, table) in targets {
            if let Some(table) = table.as_table() {
                check(&format!("target.{cfg}."), table);
            }
        }
    }
    assert!(violations.is_empty(), "path dependencies: {violations:?}");
}

/// Copies `src` to `dest`, leaving out the top-level `NOT_SCANNED` entries.
fn copy_crate(src: &Path, dest: &Path) {
    for rel in crate_files(src) {
        let to = dest.join(&rel);
        fs::create_dir_all(to.parent().unwrap()).unwrap();
        fs::copy(src.join(&rel), &to).unwrap();
    }
}

#[test]
#[ignore = "integration tier"]
fn builds_when_extracted() {
    let tmp = tempfile::TempDir::new().unwrap();
    copy_crate(crate_root(), tmp.path());
    let out = Command::new(env!("CARGO"))
        .args(["test", "--no-default-features", "--locked", "--offline"])
        .current_dir(tmp.path())
        .env_remove("CARGO_TARGET_DIR")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "extracted copy failed ({}):\n{}\n{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
