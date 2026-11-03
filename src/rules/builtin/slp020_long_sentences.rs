//! SLP020 long-sentences: a provisional paragraph metric, default-off with no fix.

use crate::rules::helpers::{
    MIN_SENTENCES_PARAM, MIN_WORDS_PARAM, document_metrics, metric_anchor,
};
use crate::rules::{
    Family, Finding, FixClass, ParamSpec, Rule, RuleCode, RuleCtx, RuleMeta, Severity,
};

const MAX_AVG_WORDS: &str = "max_avg_words";

struct LongSentences {
    meta: RuleMeta,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(LongSentences {
        meta: RuleMeta {
            code: RuleCode(20),
            name: "long-sentences",
            family: Family::Readability,
            severity: Severity::Suggestion,
            default_on: false,
            fix: FixClass::None,
            explain: "Reports a document whose paragraph sentences exceed an average word-count threshold. Detection only: no rewrite is attached and no model window is selected. The threshold is provisional until calibrated against human prose.",
            example: (
                "A paragraph exceeding the provisional metric threshold.",
                "Varied sentences with plain, precise words.",
            ),
            params: &[
                ParamSpec {
                    name: MAX_AVG_WORDS,
                    default: 25.0,
                    min: 1.0,
                    max: 1000.0,
                    help: "max_avg_words threshold (provisional)",
                },
                MIN_SENTENCES_PARAM,
                MIN_WORDS_PARAM,
            ],
            since: "0.2.0",
        },
    })
}

impl Rule for LongSentences {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        let Some(avg) = document_metrics(ctx).and_then(|m| m.avg_sentence_length) else {
            return;
        };
        let max = ctx.param(MAX_AVG_WORDS);
        if avg <= max {
            return;
        }
        let Some(range) = metric_anchor(ctx.doc) else {
            return;
        };
        let message = format!(
            "average sentence length {avg:.2} words is above {max} (provisional threshold)"
        );
        out.push(ctx.finding(&self.meta, range, message, None));
    }
}
