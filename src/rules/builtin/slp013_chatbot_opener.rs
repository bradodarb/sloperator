//! SLP013 `chatbot-opener` (SPEC §10): "Certainly!", "Great question!", "Absolutely!" or "Sure!" opening a block. The
//! safe fix deletes it and the whitespace after it, and capitalises the next letter.

use crate::doc::OBJ;
use crate::rules::helpers::{PatternRule, delete_lead_in};
use crate::rules::{Family, Finding, FixClass, Rule, RuleCode, RuleCtx, RuleMeta, Severity};

/// The opener with the whitespace after it, so the match is the whole lead-in to delete.
const PATTERNS: &[&str] = &[r"(?i)\b(?:certainly|great question|absolutely|sure)!\s*"];

struct ChatbotOpener {
    meta: RuleMeta,
    patterns: PatternRule,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(ChatbotOpener {
        meta: RuleMeta {
            code: RuleCode(13),
            name: "chatbot-opener",
            family: Family::ChatbotResidue,
            severity: Severity::Error,
            default_on: true,
            fix: FixClass::Safe,
            explain: "Flags \"Certainly!\", \"Great question!\", \"Absolutely!\" and \"Sure!\" at the start of a block. \
                      It answers a chat prompt, not the reader; the fix deletes it and capitalises the next \
                      letter.",
            example: ("Certainly! the cache is cold.", "The cache is cold."),
            params: &[],
            since: "0.1.0",
        },
        patterns: PatternRule::new(PATTERNS),
    })
}

/// Whether only block markup (`# `, `- `, `> `, each one [`OBJ`] with no space after it) or nothing comes before
/// `at`. An [`OBJ`] followed by a space is inline code or a link, so the phrase after it is mid-block.
fn starts_block(text: &str, at: usize) -> bool {
    let Some(before) = text.get(..at) else {
        return false;
    };
    before.chars().all(|c| c == OBJ || c.is_whitespace())
        && (!before.contains(OBJ) || !before.ends_with(char::is_whitespace))
}

impl Rule for ChatbotOpener {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        for bt in ctx.blocks() {
            for (_, lead) in self.patterns.find_text(&bt.text) {
                if !starts_block(&bt.text, lead.start) {
                    continue;
                }
                // The finding covers the opener and its "!", not the whitespace after.
                let phrase_end = lead.start + bt.text[lead.clone()].trim_end().len();
                let Some(range) = bt.src_range(lead.start..phrase_end) else {
                    continue;
                };
                let message = format!("`{}` is a chatbot opener", &ctx.doc.src[range.clone()]);
                let fix = delete_lead_in(&bt, lead);
                out.push(ctx.finding(&self.meta, range, message, fix));
            }
        }
    }
}
