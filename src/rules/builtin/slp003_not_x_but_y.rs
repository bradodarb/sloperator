//! SLP003 `not-x-but-y` (SPEC §10): "it's not X, it's Y" and "this isn't X — it's Y", in one sentence or two
//! adjacent sentences. Detection only; the model engine rewrites.

use crate::rules::helpers::PatternRule;
use crate::rules::{Family, Finding, FixClass, Rule, RuleCode, RuleCtx, RuleMeta, Severity};

/// A negated "it/this/that is", up to 80 characters of X, a clause break (comma, semicolon, colon, dash or a sentence
/// end), then "it/this/that is" again, with an optional "but". The match is the whole construction.
const PATTERNS: &[&str] = &[
    r"(?i)\b(?:it|this|that)(?:['’]s not| is not| isn['’]t)\b[^.!?\n]{1,80}?(?:[,;:—–]|\s--?\s|[.!?])\s*(?:but\s+)?(?:it|this|that)(?:['’]s| is)\b",
];

struct NotXButY {
    meta: RuleMeta,
    patterns: PatternRule,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(NotXButY {
        meta: RuleMeta {
            code: RuleCode(3),
            name: "not-x-but-y",
            family: Family::NegativeParallelism,
            severity: Severity::Warning,
            default_on: true,
            fix: FixClass::Model,
            explain: "Flags \"it's not X, it's Y\" and \"this isn't X — it's Y\", in one sentence or in two adjacent \
                      sentences. The construction denies a claim nobody made before it states the real one. No \
                      rewrite is safe, so the model engine states Y directly.",
            example: ("It's not a bug, it's a feature.", "It is a feature."),
            params: &[],
            since: "0.1.0",
        },
        patterns: PatternRule::new(PATTERNS),
    })
}

impl Rule for NotXButY {
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
