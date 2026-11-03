//! Score (SPEC §11): the reference formula, word-weighted overall, determinism and the reference limits text.
#![allow(clippy::unwrap_used, clippy::expect_used)] // test helpers may unwrap (architecture §8)

use sloperator::rules::{Finding, RuleCode, Severity};
use sloperator::score::{Score, overall, score, weight};

fn finding(code: u16, severity: Severity) -> Finding {
    Finding {
        code: RuleCode(code),
        range: 0..1,
        message: String::new(),
        severity,
        fix: None,
    }
}

fn reference() -> String {
    std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/REFERENCE.md")).unwrap()
}

/// The text with every run of whitespace collapsed to one space, so a hard-wrapped sentence still matches.
fn flat(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn formula_matches_reference() {
    // 2 suggestions + 1 warning + 1 error = 2×1 + 1×3 + 1×9 = 14, over 300 prose words.
    let findings = [
        finding(1, Severity::Suggestion),
        finding(2, Severity::Suggestion),
        finding(3, Severity::Warning),
        finding(4, Severity::Error),
    ];
    let by_hand = (1000.0_f64 * 14.0 / 300.0 * 100.0).round() / 100.0;
    let s = score(&findings, 300);
    assert_eq!(s.weighted, 14);
    assert_eq!(s.prose_words, 300);
    assert_eq!(s.value, by_hand);
    assert_eq!(s.value, 46.67);

    // docs/REFERENCE.md prints this formula and these weights (FR-5.1).
    let reference = flat(&reference());
    assert!(reference.contains("1000 × Σ weight(severity) / max(prose_words, 1)"));
    assert_eq!(weight(Severity::Suggestion), 1);
    assert_eq!(weight(Severity::Warning), 3);
    assert_eq!(weight(Severity::Error), 9);
    assert!(reference.contains("suggestion 1, warning 3, error 9"));

    // No prose words: the divisor is 1, never zero.
    assert_eq!(score(&findings, 0).value, 14000.0);
    assert_eq!(score(&[], 0).value, 0.0);

    // System findings (SLP901, SLP902) do not change the score (DQ-4).
    let mut with_system = findings.to_vec();
    with_system.push(finding(901, Severity::Warning));
    with_system.push(finding(902, Severity::Warning));
    assert_eq!(score(&with_system, 300), s);
}

#[test]
fn overall_is_word_weighted() {
    let small = score(&[finding(1, Severity::Error)], 10); // 900.00
    let large = score(&[finding(1, Severity::Suggestion)], 990); // 1.01
    let all = overall(&[small, large]);
    assert_eq!(all.prose_words, 1000);
    assert_eq!(all.weighted, 10);
    assert_eq!(all.value, 10.0);
    // Not the mean of the per-file values.
    assert_ne!(all.value, (small.value + large.value) / 2.0);
    assert_eq!(
        overall(&[]),
        Score {
            value: 0.0,
            prose_words: 0,
            weighted: 0
        }
    );
}

#[test]
fn identical_input_identical_score() {
    let findings = [
        finding(7, Severity::Warning),
        finding(3, Severity::Suggestion),
    ];
    let a = score(&findings, 123);
    let b = score(&findings, 123);
    assert_eq!(a, b);
    assert_eq!(a.value.to_bits(), b.value.to_bits());
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        r#"{"value":32.52,"proseWords":123,"weighted":4}"#
    );
}

#[test]
fn reference_states_limits() {
    let reference = flat(&reference());
    assert!(reference.contains(
        "The score measures style tells, not text quality or provenance. \
         sloperator does not make text undetectable and makes no such claim."
    ));
    // PRD Risk 1.
    assert!(reference.contains(
        "Verification covers numbers, URLs, configured terms, placeholders, block structure, negations, modal verbs and \
         capitalised words as a proxy for names. It does not detect every change of meaning: a reworded claim with the same \
         negations, modals and capitalised words passes."
    ));
}
