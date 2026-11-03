//! SLP021 lexical-diversity: a provisional paragraph metric, default-off with no fix.

use crate::rules::helpers::{
    MIN_SENTENCES_PARAM, MIN_WORDS_PARAM, document_metrics, metric_anchor,
};
use crate::rules::{
    Family, Finding, FixClass, ParamSpec, Rule, RuleCode, RuleCtx, RuleMeta, Severity,
};

const MIN_TTR: &str = "min_ttr";

struct LexicalDiversity {
    meta: RuleMeta,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(LexicalDiversity {
        meta: RuleMeta {
            code: RuleCode(21),
            name: "lexical-diversity",
            family: Family::Readability,
            severity: Severity::Suggestion,
            default_on: false,
            fix: FixClass::None,
            explain: "Reports a document with a low ratio of distinct lowercased words to total paragraph words. Type/token ratio falls with text length; compare similar lengths. Detection only: no rewrite is attached and no model window is selected. The threshold is provisional until calibrated against human prose.",
            example: (
                "A paragraph exceeding the provisional metric threshold.",
                "Varied sentences with plain, precise words.",
            ),
            params: &[
                ParamSpec {
                    name: MIN_TTR,
                    default: 0.4,
                    min: 0.0,
                    max: 1.0,
                    help: "min_ttr threshold (provisional)",
                },
                MIN_SENTENCES_PARAM,
                MIN_WORDS_PARAM,
            ],
            since: "0.2.0",
        },
    })
}

impl Rule for LexicalDiversity {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        let Some(ttr) = document_metrics(ctx).and_then(|m| m.type_token_ratio) else {
            return;
        };
        let min = ctx.param(MIN_TTR);
        if ttr >= min {
            return;
        }
        let Some(range) = metric_anchor(ctx.doc) else {
            return;
        };
        let message = format!("type/token ratio {ttr:.2} is below {min} (provisional threshold)");
        out.push(ctx.finding(&self.meta, range, message, None));
    }
}
