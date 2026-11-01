//! Score (SPEC §11, FR-5.1-5.3): weighted findings normalised per 1,000 prose words.
//!
//! Pure: it reads findings and word counts only. It never uses `engine`, `write`, `report` or `input`.

use crate::rules::{Finding, Severity};

/// A score: `value` is the formula result rounded to two decimals, `weighted` the weight sum behind it.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Score {
    pub value: f64,
    pub prose_words: usize,
    pub weighted: u32,
}

/// The weight of one finding of this severity.
pub const fn weight(s: Severity) -> u32 {
    match s {
        Severity::Suggestion => 1,
        Severity::Warning => 3,
        Severity::Error => 9,
    }
}

/// `value = round2(1000 × Σ weight(severity) / max(prose_words, 1))`. System findings (SLP9xx) are excluded (DQ-4).
pub fn score(findings: &[Finding], prose_words: usize) -> Score {
    let weighted: u32 = findings
        .iter()
        .filter(|f| !f.code.is_system())
        .map(|f| weight(f.severity))
        .sum();
    Score {
        value: per_thousand(weighted, prose_words),
        prose_words,
        weighted,
    }
}

/// Word-weighted: `Σ weighted / Σ words` over all files, not a mean of the per-file scores.
pub fn overall(files: &[Score]) -> Score {
    let words: usize = files.iter().map(|s| s.prose_words).sum();
    let weighted: u32 = files.iter().map(|s| s.weighted).sum();
    Score {
        value: per_thousand(weighted, words),
        prose_words: words,
        weighted,
    }
}

fn per_thousand(weighted: u32, prose_words: usize) -> f64 {
    round2(1000.0 * f64::from(weighted) / prose_words.max(1) as f64)
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
