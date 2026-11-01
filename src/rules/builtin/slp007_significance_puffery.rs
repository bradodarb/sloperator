//! SLP007 `significance-puffery` (SPEC §10): phrases that claim importance instead of showing it. Detection only; the
//! model engine rewrites.

use crate::rules::helpers::PatternRule;
use crate::rules::{Family, Finding, FixClass, Rule, RuleCode, RuleCtx, RuleMeta, Severity};

const PATTERNS: &[&str] = &[
    r"(?i)\b(?:play|plays|played|playing)\s+an?\s+(?:crucial|pivotal|vital)\s+role\b",
    r"(?i)\bunderscor(?:e|es|ed|ing)\s+the\s+importance\b",
    r"(?i)\ba\s+testament\s+to\b",
    r"(?i)\bmark(?:s|ed|ing)?\s+a\s+significant\s+milestone\b",
];

struct SignificancePuffery {
    meta: RuleMeta,
    patterns: PatternRule,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(SignificancePuffery {
        meta: RuleMeta {
            code: RuleCode(7),
            name: "significance-puffery",
            family: Family::SignificancePuffery,
            severity: Severity::Warning,
            default_on: true,
            fix: FixClass::Model,
            explain: "Flags \"plays a crucial/pivotal/vital role\", \"underscores the importance\", \"a testament to\" \
                      and \"marks a significant milestone\". Each says a thing matters without saying why. No \
                      rewrite is safe, so the model engine replaces the claim with the fact.",
            example: (
                "The cache plays a crucial role in the build.",
                "The cache cuts the build from ten minutes to two.",
            ),
            params: &[],
            since: "0.1.0",
        },
        patterns: PatternRule::new(PATTERNS),
    })
}

impl Rule for SignificancePuffery {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        for bt in ctx.blocks() {
            for (_, range) in self.patterns.find(&bt) {
                let message = format!(
                    "`{}` claims significance without showing it",
                    &ctx.doc.src[range.clone()]
                );
                out.push(ctx.finding(&self.meta, range, message, None));
            }
        }
    }
}
