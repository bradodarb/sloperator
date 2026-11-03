//! Reading metrics from integer counts over paragraph blocks (classical-nlp SPEC §8, ADR-04, D-5, D-6).

use std::collections::BTreeSet;

use crate::doc::{BlockKind, Document};

use super::segment::Segments;
use super::syllable::syllables;

/// Below either floor every derived value is unavailable (D-5, provisional). Rule params may raise them, never
/// lower them (ADR-09).
pub const DEFAULT_MIN_SENTENCES: u64 = 5;
pub const DEFAULT_MIN_WORDS: u64 = 100;

/// Minimum sample sizes required for all derived values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Floors {
    pub sentences: u64,
    pub words: u64,
}

impl Default for Floors {
    fn default() -> Self {
        Self {
            sentences: DEFAULT_MIN_SENTENCES,
            words: DEFAULT_MIN_WORDS,
        }
    }
}

/// Sufficient statistics (ADR-04). `merge` is integer addition plus set union, so overall = merge of files, never
/// a mean of per-file values (FR-2.8). `types` holds case-folded words and is not serialised.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub words: u64,
    pub sentences: u64,
    pub syllables: u64,
    /// Σ (words in sentence)², which gives the population SD exactly with `words` and `sentences`.
    pub sentence_len_sq: u64,
    pub types: BTreeSet<Box<str>>,
}

/// The reported values, rounded to two decimals; `None` (JSON `null`) when the counts are below the floors.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metrics {
    pub words: u64,
    pub sentences: u64,
    pub avg_sentence_length: Option<f64>,
    pub grade_level: Option<f64>,
    pub sentence_length_sd: Option<f64>,
    pub sentence_length_cv: Option<f64>,
    pub type_token_ratio: Option<f64>,
}

impl Counts {
    pub fn merge(&mut self, other: &Counts) {
        self.words += other.words;
        self.sentences += other.sentences;
        self.syllables += other.syllables;
        self.sentence_len_sq += other.sentence_len_sq;
        self.types.extend(other.types.iter().cloned());
    }

    /// FR-2.2-2.5 and FR-2.7: every value from the integers, rounded once at the end (FR-2.9).
    pub fn derive(&self, floors: &Floors) -> Metrics {
        let mut m = Metrics {
            words: self.words,
            sentences: self.sentences,
            avg_sentence_length: None,
            grade_level: None,
            sentence_length_sd: None,
            sentence_length_cv: None,
            type_token_ratio: None,
        };
        if self.sentences < floors.sentences
            || self.words < floors.words
            || self.sentences == 0
            || self.words == 0
        {
            return m;
        }
        let (w, s) = (self.words as f64, self.sentences as f64);
        let mean = w / s;
        // Population variance as an exact integer numerator: (n·Σx² − (Σx)²) / n².
        let n = u128::from(self.sentences);
        let spread = (n * u128::from(self.sentence_len_sq))
            .saturating_sub(u128::from(self.words) * u128::from(self.words));
        let sd = (spread as f64 / (n * n) as f64).sqrt();
        m.avg_sentence_length = Some(round2(mean));
        m.grade_level = Some(round2(
            0.39 * mean + 11.8 * (self.syllables as f64 / w) - 15.59,
        ));
        m.sentence_length_sd = Some(round2(sd));
        m.sentence_length_cv = Some(round2(sd / mean));
        m.type_token_ratio = Some(round2(self.types.len() as f64 / w));
        m
    }
}

/// The score's rounding rule (`src/score/mod.rs:54-56`), kept private here because `text` may not use `score`.
fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// The counts of paragraph sentences only (D-5, FR-2.10). A sentence with no prose word is skipped.
pub fn counts(doc: &Document, seg: &Segments) -> Counts {
    let mut c = Counts::default();
    for s in &seg.sentences {
        let paragraph = doc
            .blocks
            .get(s.block.0 as usize)
            .is_some_and(|b| b.kind == BlockKind::Paragraph);
        if !paragraph || s.words.is_empty() {
            continue;
        }
        let n = u64::from(s.words.end - s.words.start);
        c.sentences += 1;
        c.words += n;
        c.sentence_len_sq += n * n;
        for w in &seg.words[s.words.start as usize..s.words.end as usize] {
            let text = &doc.src[w.range.clone()];
            c.syllables += u64::from(syllables(text));
            let lower = text.to_lowercase();
            if !c.types.contains(lower.as_str()) {
                c.types.insert(lower.into_boxed_str());
            }
        }
    }
    c
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::super::segment::{scans, segment};
    use super::*;
    use crate::doc::{self, Format, ProtectSettings};

    fn sample() -> Counts {
        Counts {
            words: 100,
            sentences: 5,
            syllables: 150,
            sentence_len_sq: 2200,
            types: (0..52)
                .map(|i| format!("word{i}").into_boxed_str())
                .collect(),
        }
    }

    fn assert_unavailable(m: Metrics) {
        assert_eq!(m.avg_sentence_length, None);
        assert_eq!(m.grade_level, None);
        assert_eq!(m.sentence_length_sd, None);
        assert_eq!(m.sentence_length_cv, None);
        assert_eq!(m.type_token_ratio, None);
    }

    #[test]
    fn hand_computed_values() {
        let m = sample().derive(&Floors::default());
        assert_eq!(m.words, 100);
        assert_eq!(m.sentences, 5);
        assert_eq!(m.avg_sentence_length, Some(20.0));
        assert_eq!(m.grade_level, Some(9.91));
        assert_eq!(m.sentence_length_sd, Some(6.32));
        assert_eq!(m.sentence_length_cv, Some(0.32));
        assert_eq!(m.type_token_ratio, Some(0.52));
    }

    #[test]
    fn below_floor_is_unavailable() {
        let c = Counts {
            sentences: 4,
            ..sample()
        };
        let m = c.derive(&Floors::default());
        assert_unavailable(m);
        assert_eq!(
            serde_json::to_value(m).unwrap(),
            serde_json::json!({
                "words": 100,
                "sentences": 4,
                "avgSentenceLength": null,
                "gradeLevel": null,
                "sentenceLengthSd": null,
                "sentenceLengthCv": null,
                "typeTokenRatio": null
            })
        );
        assert_unavailable(
            Counts {
                words: 99,
                ..sample()
            }
            .derive(&Floors::default()),
        );
        assert_unavailable(Counts::default().derive(&Floors {
            sentences: 0,
            words: 0,
        }));
    }

    #[test]
    fn configured_floor_is_honoured() {
        let c = sample();
        assert_unavailable(c.derive(&Floors {
            sentences: 10,
            words: 100,
        }));
        assert_unavailable(c.derive(&Floors {
            sentences: 5,
            words: 101,
        }));
        assert_eq!(
            c.derive(&Floors {
                sentences: 5,
                words: 100
            })
            .avg_sentence_length,
            Some(20.0)
        );
        assert_eq!(
            Floors::default(),
            Floors {
                sentences: 5,
                words: 100
            }
        );
    }

    #[test]
    fn merge_unions_types() {
        let mut c = sample();
        c.merge(&c.clone());
        assert_eq!(c.words, 200);
        assert_eq!(c.sentences, 10);
        assert_eq!(c.syllables, 300);
        assert_eq!(c.sentence_len_sq, 4400);
        assert_eq!(c.types.len(), 52);
        let m = c.derive(&Floors::default());
        assert_eq!(m.type_token_ratio, Some(0.26));
        assert_eq!(m.avg_sentence_length, Some(20.0));
        assert_eq!(m.sentence_length_sd, Some(6.32));
        c.merge(&Counts::default());
        let mut empty = Counts::default();
        empty.merge(&c);
        assert_eq!(empty, c);
    }

    #[test]
    fn paragraphs_only() {
        let doc = doc::parse(
            "# Heading words.\n\n- Listed words.\n\n| Table | Header |\n| --- | --- |\n| Cell | Words |\n\nCat cat. Dog!\n\n`ignored`\n".into(),
            Format::Markdown,
            &ProtectSettings::default(),
        );
        assert!(
            doc.blocks
                .iter()
                .any(|b| matches!(b.kind, BlockKind::Heading(_)))
        );
        assert!(
            doc.blocks
                .iter()
                .any(|b| matches!(b.kind, BlockKind::ListItem { .. }))
        );
        assert!(
            doc.blocks
                .iter()
                .any(|b| matches!(b.kind, BlockKind::TableCell { .. }))
        );
        let seg = segment(&doc);
        let before = scans();
        let c = counts(&doc, &seg);
        assert_eq!(scans(), before);
        assert_eq!(
            c,
            Counts {
                words: 3,
                sentences: 2,
                syllables: 3,
                sentence_len_sq: 5,
                types: ["cat".into(), "dog".into()].into(),
            }
        );
    }
}
