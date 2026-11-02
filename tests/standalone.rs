//! Standalone build checks (SPEC §5): dependencies must be self-contained, and an extracted
//! copy of the crate must build and test offline. The extraction check runs in the integration tier.

#![allow(clippy::unwrap_used, clippy::expect_used)] // test helpers may unwrap (architecture §8)

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Top-level VCS metadata and build output excluded from the standalone build copy.
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

fn crate_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
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
