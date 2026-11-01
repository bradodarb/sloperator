//! Eval corpus guard (SPEC §1, §22): the twenty spike paragraphs that the eval harness reuses must all be present, and
//! the harness's latency budget must match the value recorded in `eval/README.md`.
//! Std only, so it runs in the unit tier (`--no-default-features`).
#![allow(clippy::unwrap_used, clippy::expect_used)] // test helpers may unwrap (architecture §8)

use std::fs;
use std::path::PathBuf;

const COUNT: usize = 20;

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("eval/paragraphs")
}

#[test]
fn twenty_paragraphs_present_without_placeholder_glyphs() {
    let dir = corpus_dir();
    for n in 1..=COUNT {
        let name = format!("p{n:02}.md");
        let text = fs::read_to_string(dir.join(&name))
            .unwrap_or_else(|e| panic!("{name} missing or unreadable: {e}"));
        assert!(!text.trim().is_empty(), "{name} is empty");
        assert!(
            !text.contains('\u{27E6}') && !text.contains('\u{27E7}'),
            "{name} contains a placeholder glyph (⟦ or ⟧); the engine reserves them (ADR-04)"
        );
    }
    let mut names: Vec<String> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    let expected: Vec<String> = (1..=COUNT).map(|n| format!("p{n:02}.md")).collect();
    assert_eq!(
        names, expected,
        "eval/paragraphs must hold exactly p01.md..p20.md"
    );
}

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The number that follows `key` in the crate file `rel`; panics naming the file if it is missing.
fn number_after(rel: &str, key: &str) -> f64 {
    let path = manifest().join(rel);
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let at = text
        .find(key)
        .unwrap_or_else(|| panic!("no `{key}` in {}", path.display()))
        + key.len();
    let end = text[at..]
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap();
    text[at..at + end]
        .parse()
        .unwrap_or_else(|e| panic!("`{key}` in {}: {e}", path.display()))
}

/// The budget's record lives inside the crate, so this holds in an extracted copy too (SPEC §5, NFR-2). Reads the
/// constant from the source because `tests/eval.rs` builds only with the `model` feature.
#[test]
fn budget_matches_eval_readme() {
    let constant = number_after("tests/eval.rs", "const MODEL_WINDOW_BUDGET_S: f64 = ");
    let recorded = number_after("eval/README.md", "MODEL_WINDOW_BUDGET_S = ");
    assert!(constant > 0.0, "MODEL_WINDOW_BUDGET_S is unset (0.0)");
    assert!(
        (constant - recorded).abs() < f64::EPSILON,
        "MODEL_WINDOW_BUDGET_S in tests/eval.rs ({constant}) differs from eval/README.md ({recorded})"
    );
}
