//! SLP023: detection-only paragraph sentence redundancy with provisional thresholds.
use crate::doc::BlockKind;
use crate::rules::helpers::first_prose;
use crate::rules::{
    Family, Finding, FixClass, ParamSpec, Rule, RuleCode, RuleCtx, RuleMeta, Severity,
};
use crate::text::similarity::{self, Bag};

const MIN_OVERLAP: &str = "min_overlap";
const MIN_CONTENT_WORDS: &str = "min_content_words";
const MIN_SENTENCES: &str = "min_sentences";

struct RedundantSentence {
    meta: RuleMeta,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(RedundantSentence {
        meta: RuleMeta {
            code: RuleCode(23),
            name: "redundant-sentence",
            family: Family::Redundancy,
            severity: Severity::Suggestion,
            default_on: false,
            fix: FixClass::None,
            since: "0.2.0",
            explain: "Reports later sentences with overlapping stemmed content words within a paragraph. Thresholds are provisional. Detection only: deleting a sentence cannot be verified, so no fix or model window is attached.",
            example: (
                "The cache stores compiled build artifacts. The cache stores compiled build artifacts.",
                "The cache stores compiled build artifacts.",
            ),
            params: &[
                ParamSpec {
                    name: MIN_OVERLAP,
                    default: 0.6,
                    min: 0.1,
                    max: 1.0,
                    help: "Minimum weighted Jaccard overlap (provisional)",
                },
                ParamSpec {
                    name: MIN_CONTENT_WORDS,
                    default: 4.0,
                    min: 1.0,
                    max: 100.0,
                    help: "Minimum content words per compared sentence",
                },
                ParamSpec {
                    name: MIN_SENTENCES,
                    default: 3.0,
                    min: 2.0,
                    max: f64::MAX,
                    help: "Minimum paragraph sentences in the document",
                },
            ],
        },
    })
}

impl Rule for RedundantSentence {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }
    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        let seg = ctx.prose.segments();
        if (ctx.prose.counts(ctx.doc).sentences as f64) < ctx.param(MIN_SENTENCES) {
            return;
        }
        let (min_overlap, min_words) = (ctx.param(MIN_OVERLAP), ctx.param(MIN_CONTENT_WORDS));
        // Sentences are in block order, so each paragraph's sentences are one contiguous run (FR-9.5).
        for run in seg.sentences.chunk_by(|a, b| a.block == b.block) {
            let paragraph = ctx
                .doc
                .blocks
                .get(run[0].block.0 as usize)
                .is_some_and(|b| b.kind == BlockKind::Paragraph);
            if !paragraph {
                continue;
            }
            let bags: Vec<Bag> = run
                .iter()
                .map(|s| similarity::bag(ctx.doc, seg, s))
                .collect();
            for j in 1..run.len() {
                if f64::from(bags[j].size()) < min_words {
                    continue;
                }
                let earlier = (0..j).find(|&i| {
                    f64::from(bags[i].size()) >= min_words
                        && similarity::jaccard(&bags[i], &bags[j]) >= min_overlap
                });
                let Some(i) = earlier else { continue };
                let Some(range) = first_prose(ctx.doc, &run[j].range) else {
                    continue;
                };
                let (line, _) = ctx.doc.line_col(run[i].range.start);
                let message = format!(
                    "this sentence repeats {:.0}% of the content words of the sentence on line {line}",
                    100.0 * similarity::jaccard(&bags[i], &bags[j])
                );
                out.push(ctx.finding(&self.meta, range, message, None));
            }
        }
    }
}
