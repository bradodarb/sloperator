//! SLP008 `hedging-preamble` (SPEC §10): a sentence that opens with a filler preamble. The safe fix deletes the
//! preamble and capitalises the next letter.

use crate::rules::helpers::{PatternRule, delete_lead_in, starts_sentence};
use crate::rules::{Family, Finding, FixClass, Rule, RuleCode, RuleCtx, RuleMeta, Severity};

/// Each pattern takes the whitespace after the phrase, so the match is the whole lead-in to delete.
const PATTERNS: &[&str] = &[
    r"(?i)\bit(?:'s|’s| is) important to note that\s+",
    r"(?i)\bit is worth noting that\s+",
    r"(?i)\bit should be noted that\s+",
];

struct HedgingPreamble {
    meta: RuleMeta,
    patterns: PatternRule,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(HedgingPreamble {
        meta: RuleMeta {
            code: RuleCode(8),
            name: "hedging-preamble",
            family: Family::Hedging,
            severity: Severity::Warning,
            default_on: true,
            fix: FixClass::Safe,
            explain: "Flags a sentence that opens with \"It's important to note that\", \"It is worth noting that\" \
                      or \"It should be noted that\". The preamble adds no claim; the fix deletes it and \
                      capitalises the next letter.",
            example: (
                "It is worth noting that the cache is cold.",
                "The cache is cold.",
            ),
            params: &[],
            since: "0.1.0",
        },
        patterns: PatternRule::new(PATTERNS),
    })
}

impl Rule for HedgingPreamble {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        for bt in ctx.blocks() {
            for (_, lead) in self.patterns.find_text(&bt.text) {
                if !starts_sentence(&bt.text, lead.start) {
                    continue;
                }
                let phrase_end = lead.start + bt.text[lead.clone()].trim_end().len();
                let Some(range) = bt.src_range(lead.start..phrase_end) else {
                    continue;
                };
                let message = format!("`{}` is a hedging preamble", &ctx.doc.src[range.clone()]);
                let fix = delete_lead_in(&bt, lead);
                out.push(ctx.finding(&self.meta, range, message, fix));
            }
        }
    }
}
