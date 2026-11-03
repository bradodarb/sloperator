//! SLP019 sentence-length-variation: a provisional paragraph metric, default-off with no fix.

use crate::rules::helpers::{
    MIN_SENTENCES_PARAM, MIN_WORDS_PARAM, document_metrics, metric_anchor,
};
use crate::rules::{
    Family, Finding, FixClass, ParamSpec, Rule, RuleCode, RuleCtx, RuleMeta, Severity,
};

const MIN_CV: &str = "min_cv";

struct SentenceLengthVariation {
    meta: RuleMeta,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(SentenceLengthVariation {
        meta: RuleMeta {
            code: RuleCode(19),
            name: "sentence-length-variation",
            family: Family::Readability,
            severity: Severity::Suggestion,
            default_on: false,
            fix: FixClass::None,
            explain: "Reports low sentence-length variation, measured by the population coefficient of variation over paragraph sentences. This is not perplexity-based burstiness. Detection only: no rewrite is attached and no model window is selected. The threshold is provisional until calibrated against human prose.",
            example: (
                "A paragraph exceeding the provisional metric threshold.",
                "Varied sentences with plain, precise words.",
            ),
            params: &[
                ParamSpec {
                    name: MIN_CV,
                    default: 0.35,
                    min: 0.0,
                    max: 5.0,
                    help: "min_cv threshold (provisional)",
                },
                MIN_SENTENCES_PARAM,
                MIN_WORDS_PARAM,
            ],
            since: "0.2.0",
        },
    })
}

impl Rule for SentenceLengthVariation {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        let Some(cv) = document_metrics(ctx).and_then(|m| m.sentence_length_cv) else {
            return;
        };
        let min = ctx.param(MIN_CV);
        if cv >= min {
            return;
        }
        let Some(range) = metric_anchor(ctx.doc) else {
            return;
        };
        let message = format!(
            "sentence-length variation is low: cv {cv:.2} is below {min} (provisional threshold)"
        );
        out.push(ctx.finding(&self.meta, range, message, None));
    }
}
