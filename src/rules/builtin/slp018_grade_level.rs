//! SLP018 grade-level: paragraphs above a provisional Flesch–Kincaid threshold.
//! Default-off, Suggestion, no fix (D-4); the threshold is provisional until calibrated (FR-4.5).

use crate::rules::helpers::{
    MIN_SENTENCES_PARAM, MIN_WORDS_PARAM, document_metrics, metric_anchor,
};
use crate::rules::{
    Family, Finding, FixClass, ParamSpec, Rule, RuleCode, RuleCtx, RuleMeta, Severity,
};

const MAX_GRADE: &str = "max_grade";

struct GradeLevel {
    meta: RuleMeta,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(GradeLevel {
        meta: RuleMeta {
            code: RuleCode(18),
            name: "grade-level",
            family: Family::Readability,
            severity: Severity::Suggestion,
            default_on: false,
            fix: FixClass::None,
            explain: "Reports a document whose paragraphs read above a Flesch-Kincaid grade threshold, computed over \
                      paragraph sentences with an approximate syllable count. It needs at least 5 sentences and 100 \
                      words. The threshold is provisional until calibrated against human prose. Detection only: \
                      no rewrite is attached and no model window is selected.",
            example: (
                "A paragraph of long sentences built from polysyllabic, abstract vocabulary throughout.",
                "Short sentences. Plain words.",
            ),
            params: &[
                ParamSpec {
                    name: MAX_GRADE,
                    default: 12.0,
                    min: 0.0,
                    max: 30.0,
                    help: "Grade level above which the document is reported (provisional)",
                },
                MIN_SENTENCES_PARAM,
                MIN_WORDS_PARAM,
            ],
            since: "0.2.0",
        },
    })
}

impl Rule for GradeLevel {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        let Some(grade) = document_metrics(ctx).and_then(|m| m.grade_level) else {
            return;
        };
        let max = ctx.param(MAX_GRADE);
        if grade <= max {
            return;
        }
        let Some(range) = metric_anchor(ctx.doc) else {
            return;
        };
        let message = format!("grade level {grade:.2} is above {max} (provisional threshold)");
        out.push(ctx.finding(&self.meta, range, message, None));
    }
}
