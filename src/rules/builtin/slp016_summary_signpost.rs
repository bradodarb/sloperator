//! SLP016 `summary-signpost` (SPEC §10): "In conclusion,", "In summary,", "Overall," or "Ultimately," opening the
//! last sentence of a block. The safe fix deletes it and capitalises the next letter.

use crate::rules::helpers::{PatternRule, delete_lead_in, starts_sentence};
use crate::rules::{Family, Finding, FixClass, Rule, RuleCode, RuleCtx, RuleMeta, Severity};

/// The signpost with its comma and the whitespace after it, so the match is the whole lead-in to delete.
const PATTERNS: &[&str] = &[r"(?i)\b(?:in conclusion|in summary|overall|ultimately),\s+"];

/// A sentence end followed by more text: after it, the sentence is not the block's last.
const SENTENCE_BREAK: &[&str] = &[r#"[.!?]['")\]]*\s+\S"#];

struct SummarySignpost {
    meta: RuleMeta,
    patterns: PatternRule,
    breaks: PatternRule,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(SummarySignpost {
        meta: RuleMeta {
            code: RuleCode(16),
            name: "summary-signpost",
            family: Family::SignificancePuffery,
            severity: Severity::Suggestion,
            default_on: true,
            fix: FixClass::Safe,
            explain: "Flags \"In conclusion,\", \"In summary,\", \"Overall,\" and \"Ultimately,\" at the start of a \
                      block's last sentence. The signpost announces a summary the reader can see; the fix deletes it \
                      and capitalises the next letter.",
            example: (
                "In conclusion, the flat layout wins.",
                "The flat layout wins.",
            ),
            params: &[],
            since: "0.1.0",
        },
        patterns: PatternRule::new(PATTERNS),
        breaks: PatternRule::new(SENTENCE_BREAK),
    })
}

impl Rule for SummarySignpost {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        for bt in ctx.blocks() {
            for (_, lead) in self.patterns.find_text(&bt.text) {
                let is_last = bt
                    .text
                    .get(lead.end..)
                    .is_some_and(|rest| self.breaks.find_text(rest).is_empty());
                if !is_last || !starts_sentence(&bt.text, lead.start) {
                    continue;
                }
                // The finding covers the signpost word(s) and the comma, not the whitespace after.
                let phrase_end = lead.start + bt.text[lead.clone()].trim_end().len();
                let Some(range) = bt.src_range(lead.start..phrase_end) else {
                    continue;
                };
                let message = format!("`{}` announces a summary", &ctx.doc.src[range.clone()]);
                let fix = delete_lead_in(&bt, lead);
                out.push(ctx.finding(&self.meta, range, message, fix));
            }
        }
    }
}
