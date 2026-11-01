//! SLP004 `not-just-but-also` (SPEC §10): "not just X, but (also) Y" and "not only X but also Y". Detection only; the
//! model engine rewrites.

use crate::rules::helpers::PatternRule;
use crate::rules::{Family, Finding, FixClass, Rule, RuleCode, RuleCtx, RuleMeta, Severity};

/// Up to 80 characters of X inside one sentence, then "but". "not only" needs "but also"; "not just" takes a bare
/// "but" too.
const PATTERNS: &[&str] = &[
    r"(?i)\bnot just\b[^.!?\n]{1,80}?,?\s+but\b(?:\s+also\b)?",
    r"(?i)\bnot only\b[^.!?\n]{1,80}?,?\s+but\s+also\b",
];

struct NotJustButAlso {
    meta: RuleMeta,
    patterns: PatternRule,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(NotJustButAlso {
        meta: RuleMeta {
            code: RuleCode(4),
            name: "not-just-but-also",
            family: Family::NegativeParallelism,
            severity: Severity::Warning,
            default_on: true,
            fix: FixClass::Model,
            explain: "Flags \"not just X, but (also) Y\" and \"not only X but also Y\". The frame pads a plain \
                      addition with a contrast. No rewrite is safe, so the model engine joins X and Y.",
            example: (
                "The tool is not just fast, but also simple.",
                "The tool is fast and simple.",
            ),
            params: &[],
            since: "0.1.0",
        },
        patterns: PatternRule::new(PATTERNS),
    })
}

impl Rule for NotJustButAlso {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        for bt in ctx.blocks() {
            for (_, range) in self.patterns.find(&bt) {
                let message = format!(
                    "`{}` is a negative parallelism",
                    &ctx.doc.src[range.clone()]
                );
                out.push(ctx.finding(&self.meta, range, message, None));
            }
        }
    }
}
