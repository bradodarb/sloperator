//! SLP015 `ai-disclaimer` (SPEC §10): "As of my last (knowledge) update" and "As an AI language model". Detection
//! only; the model engine rewrites.

use crate::rules::helpers::PatternRule;
use crate::rules::{Family, Finding, FixClass, Rule, RuleCode, RuleCtx, RuleMeta, Severity};

const PATTERNS: &[&str] = &[
    r"(?i)\bas\s+of\s+my\s+last\s+(?:knowledge\s+)?update\b",
    r"(?i)\bas\s+an\s+ai\s+(?:language\s+)?model\b",
];

struct AiDisclaimer {
    meta: RuleMeta,
    patterns: PatternRule,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(AiDisclaimer {
        meta: RuleMeta {
            code: RuleCode(15),
            name: "ai-disclaimer",
            family: Family::ChatbotResidue,
            severity: Severity::Error,
            default_on: true,
            fix: FixClass::Model,
            explain: "Flags \"As of my last (knowledge) update\" and \"As an AI language model\". The speaker is a \
                      model talking about itself, which does not belong in a document. Whether the sentence \
                      survives depends on its claim, so the model engine rewrites it.",
            example: (
                "As of my last knowledge update, the API was stable.",
                "The API was stable in the last release.",
            ),
            params: &[],
            since: "0.1.0",
        },
        patterns: PatternRule::new(PATTERNS),
    })
}

impl Rule for AiDisclaimer {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        for bt in ctx.blocks() {
            for (_, range) in self.patterns.find(&bt) {
                let message = format!("`{}` is an AI disclaimer", &ctx.doc.src[range.clone()]);
                out.push(ctx.finding(&self.meta, range, message, None));
            }
        }
    }
}
