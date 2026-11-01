//! SLP012 `chatbot-closer` (SPEC §10): a closing line left over from a chat reply. The safe fix deletes the whole
//! sentence and one adjoining space.

use std::ops::Range;

use crate::doc::BlockText;
use crate::rules::helpers::{PatternRule, starts_sentence};
use crate::rules::{Family, Finding, FixClass, Rule, RuleCode, RuleCtx, RuleMeta, Severity};

/// Each pattern is a whole sentence: the phrase, the rest of the sentence, its terminator and the whitespace after it
/// (one character, or the end of the block). A `.`, `!` or `?` inside a word or number (`v2.0`) does not end it.
const PATTERNS: &[&str] = &[
    r"(?i)\bI hope (?:this|that|it) helps(?: you| out)?[.!?]+(?:\s|$)",
    r"(?i)\blet me know if you have any (?:other |more |further |additional )?questions(?:[^.!?\n]|[.!?][^\s.!?])*[.!?]+(?:\s|$)",
    r"(?i)\bfeel free to (?:[^.!?\n]|[.!?][^\s.!?])*[.!?]+(?:\s|$)",
];

struct ChatbotCloser {
    meta: RuleMeta,
    patterns: PatternRule,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(ChatbotCloser {
        meta: RuleMeta {
            code: RuleCode(12),
            name: "chatbot-closer",
            family: Family::ChatbotResidue,
            severity: Severity::Error,
            default_on: true,
            fix: FixClass::Safe,
            explain: "Flags a whole sentence that closes a chat reply: \"I hope this helps!\", \"Let me know if you \
                      have any (other) questions.\" and \"Feel free to ...\". It is addressed to a chat partner, not \
                      to the reader of the document; the fix deletes the sentence and one adjoining space.",
            example: (
                "The cache is cold. I hope this helps!",
                "The cache is cold.",
            ),
            params: &[],
            since: "0.1.0",
        },
        patterns: PatternRule::new(PATTERNS),
    })
}

impl Rule for ChatbotCloser {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        for bt in ctx.blocks() {
            for (_, m) in self.patterns.find_text(&bt.text) {
                if !starts_sentence(&bt.text, m.start) {
                    continue;
                }
                // The match holds one whitespace character after the terminator; the finding stops at the terminator.
                let sentence = m.start..m.start + bt.text[m.clone()].trim_end().len();
                let Some(range) = bt.src_range(sentence.clone()) else {
                    continue;
                };
                let message = format!("`{}` is a chatbot closer", &ctx.doc.src[range.clone()]);
                let fix = delete_sentence(&bt, sentence);
                out.push(ctx.finding(&self.meta, range, message, fix));
            }
        }
    }
}

/// The source range to delete and its (empty) replacement: the sentence and the space after it, or, when it ends the
/// line or block, the space before it. A sentence alone in its block or on its own line has no adjoining space, so
/// there is no fix: deleting it would leave an empty block.
fn delete_sentence(bt: &BlockText, sentence: Range<usize>) -> Option<(Range<usize>, String)> {
    let after = bt.text.get(sentence.end..)?;
    if after.starts_with([' ', '\t']) {
        let src = bt.src_range(sentence.start..sentence.end + 1)?;
        return Some((src, String::new()));
    }
    let at_end = after.is_empty() || after.starts_with(['\n', '\r']);
    if at_end && bt.text.get(..sentence.start)?.ends_with([' ', '\t']) {
        let src = bt.src_range(sentence.start - 1..sentence.end)?;
        return Some((src, String::new()));
    }
    None
}
