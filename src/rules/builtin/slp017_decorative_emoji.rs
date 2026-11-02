//! SLP017 `decorative-emoji` (SPEC §10): pictographic emoji in prose. One visible emoji (a grapheme cluster, so a ZWJ
//! sequence, a skin-tone modifier, a flag or a keycap counts once) is one finding. The safe fix deletes the emoji and
//! exactly one adjacent space.
//!
//! What counts as an emoji: a grapheme whose base is in the supplementary-plane `Extended_Pictographic` ranges, or a
//! BMP pictograph with default emoji presentation (✅, ⏳, ⭐, …), or any other BMP pictograph followed by U+FE0F.
//! So ©, ®, ™, ↔ and ☺ written as plain symbols are not flagged; `©\u{FE0F}` is. A digit, `#` or `*` is an emoji
//! only as a keycap (followed by U+20E3).
//!
//! The fix, over a cluster of emoji separated only by spaces (`🎉 🎉`), from the source characters around it:
//! - a space before and a space after: delete the cluster and the space after (`so 🎉 far` → `so far`);
//! - a space before and the end of the line or closing punctuation after: delete the space before
//!   (`so far 💪.` → `so far.`, `this! 🙌` → `this!`);
//! - the start of a line, or a word or closing punctuation before, and a space after: delete the cluster and the
//!   space after (`🚀 Just` → `Just`, `- 🎉 Done` → `- Done`);
//! - a word or closing punctuation before and the end of the line or closing punctuation after: delete the cluster
//!   (`done🎉.` → `done.`).
//!
//! Anything else (an emoji between two letters, after opening markup such as `**`, next to code or a link) has no
//! fix. An emoji that is the only prose on its line (alone on a line, the whole body of a list item or a heading) is
//! flagged with no fix: deleting it would leave an empty bullet or heading.

use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

use crate::doc::{BlockText, OBJ};
use crate::rules::{Family, Finding, FixClass, Rule, RuleCode, RuleCtx, RuleMeta, Severity};

/// Supplementary-plane `Extended_Pictographic` ranges (emoji-data.txt), plus the regional indicators that pair into
/// flags. Every one of these is an emoji however it is presented.
const SUPPLEMENTARY: &[(u32, u32)] = &[
    (0x1F000, 0x1F0FF),
    (0x1F10D, 0x1F10F),
    (0x1F12F, 0x1F12F),
    (0x1F16C, 0x1F171),
    (0x1F17E, 0x1F17F),
    (0x1F18E, 0x1F18E),
    (0x1F191, 0x1F19A),
    (0x1F1AD, 0x1F1FF),
    (0x1F201, 0x1F20F),
    (0x1F21A, 0x1F21A),
    (0x1F22F, 0x1F22F),
    (0x1F232, 0x1F23A),
    (0x1F23C, 0x1F23F),
    (0x1F249, 0x1F3FA),
    (0x1F400, 0x1F53D),
    (0x1F546, 0x1F64F),
    (0x1F680, 0x1F6FF),
    (0x1F774, 0x1F77F),
    (0x1F7D5, 0x1F7FF),
    (0x1F80C, 0x1F80F),
    (0x1F848, 0x1F84F),
    (0x1F85A, 0x1F85F),
    (0x1F888, 0x1F88F),
    (0x1F8AE, 0x1F8FF),
    (0x1F90C, 0x1F93A),
    (0x1F93C, 0x1F945),
    (0x1F947, 0x1FAFF),
    (0x1FC00, 0x1FFFD),
];

/// BMP pictographs with `Emoji_Presentation=Yes`: drawn as emoji even with no U+FE0F.
const BMP_EMOJI: &[(u32, u32)] = &[
    (0x231A, 0x231B),
    (0x23E9, 0x23EC),
    (0x23F0, 0x23F0),
    (0x23F3, 0x23F3),
    (0x25FD, 0x25FE),
    (0x2614, 0x2615),
    (0x2648, 0x2653),
    (0x267F, 0x267F),
    (0x2693, 0x2693),
    (0x26A1, 0x26A1),
    (0x26AA, 0x26AB),
    (0x26BD, 0x26BE),
    (0x26C4, 0x26C5),
    (0x26CE, 0x26CE),
    (0x26D4, 0x26D4),
    (0x26EA, 0x26EA),
    (0x26F2, 0x26F3),
    (0x26F5, 0x26F5),
    (0x26FA, 0x26FA),
    (0x26FD, 0x26FD),
    (0x2705, 0x2705),
    (0x270A, 0x270B),
    (0x2728, 0x2728),
    (0x274C, 0x274C),
    (0x274E, 0x274E),
    (0x2753, 0x2755),
    (0x2757, 0x2757),
    (0x2795, 0x2797),
    (0x27B0, 0x27B0),
    (0x27BF, 0x27BF),
    (0x2B1B, 0x2B1C),
    (0x2B50, 0x2B50),
    (0x2B55, 0x2B55),
];

/// The other BMP `Extended_Pictographic` code points (text presentation by default): ©, ®, ‼, ™, arrows, the
/// Miscellaneous Symbols and Dingbats blocks. An emoji only when U+FE0F follows.
const BMP_TEXT_DEFAULT: &[(u32, u32)] = &[
    (0x00A9, 0x00A9),
    (0x00AE, 0x00AE),
    (0x203C, 0x203C),
    (0x2049, 0x2049),
    (0x2122, 0x2122),
    (0x2139, 0x2139),
    (0x2194, 0x2199),
    (0x21A9, 0x21AA),
    (0x2328, 0x2328),
    (0x2388, 0x2388),
    (0x23CF, 0x23CF),
    (0x23E9, 0x23F3),
    (0x23F8, 0x23FA),
    (0x24C2, 0x24C2),
    (0x25AA, 0x25AB),
    (0x25B6, 0x25B6),
    (0x25C0, 0x25C0),
    (0x25FB, 0x25FE),
    (0x2600, 0x27BF),
    (0x2934, 0x2935),
    (0x2B05, 0x2B07),
    (0x3030, 0x3030),
    (0x303D, 0x303D),
    (0x3297, 0x3297),
    (0x3299, 0x3299),
];

const VS16: char = '\u{FE0F}';
const KEYCAP: char = '\u{20E3}';

struct DecorativeEmoji {
    meta: RuleMeta,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(DecorativeEmoji {
        meta: RuleMeta {
            code: RuleCode(17),
            name: "decorative-emoji",
            family: Family::Markup,
            severity: Severity::Warning,
            default_on: true,
            fix: FixClass::Safe,
            explain: "Flags pictographic emoji in prose (🚀, ✅, 👍🏽, 👨‍💻, 🇺🇸, 1️⃣), one finding per visible emoji. They \
                      decorate rather than inform, and model-written status updates scatter them through every \
                      paragraph. Plain symbols such as ©, ® and ™ are not flagged unless written with the emoji \
                      selector U+FE0F. The fix deletes the emoji and one adjacent space; an emoji that is the only \
                      prose on its line (a bare list item or heading) is flagged with no fix.",
            example: ("🚀 The build passed 🎉.", "The build passed."),
            params: &[],
            since: "0.1.0",
        },
    })
}

impl Rule for DecorativeEmoji {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        let src: &str = &ctx.doc.src;
        for bt in ctx.blocks() {
            for cluster in clusters(&bt.text) {
                let fix = cluster_fix(&bt, src, &cluster);
                for (i, emoji) in cluster.into_iter().enumerate() {
                    let Some(range) = bt.src_range(emoji) else {
                        continue;
                    };
                    // The cluster's one edit rides on its first finding, so the edits never overlap.
                    let fix = if i == 0 { fix.clone() } else { None };
                    out.push(ctx.finding(
                        &self.meta,
                        range,
                        "decorative emoji; remove it".to_string(),
                        fix,
                    ));
                }
            }
        }
    }
}

fn in_table(c: char, table: &[(u32, u32)]) -> bool {
    let c = c as u32;
    table.iter().any(|&(lo, hi)| (lo..=hi).contains(&c))
}

/// Whether one grapheme cluster is a pictographic emoji (see the module docs).
pub(crate) fn is_emoji(grapheme: &str) -> bool {
    let Some(base) = grapheme.chars().next() else {
        return false;
    };
    if matches!(base, '0'..='9' | '#' | '*') {
        return grapheme.contains(KEYCAP);
    }
    in_table(base, SUPPLEMENTARY)
        || in_table(base, BMP_EMOJI)
        || (in_table(base, BMP_TEXT_DEFAULT) && grapheme.contains(VS16))
}

/// The emoji of a block's text as clusters: runs of emoji separated only by spaces or tabs, each emoji a text range.
fn clusters(text: &str) -> Vec<Vec<Range<usize>>> {
    let mut out: Vec<Vec<Range<usize>>> = Vec::new();
    // Whether only spaces or tabs have come since the last emoji, so the next one joins its cluster.
    let mut joinable = false;
    for (at, g) in text.grapheme_indices(true) {
        if is_emoji(g) {
            let range = at..at + g.len();
            match out.last_mut() {
                Some(last) if joinable => last.push(range),
                _ => out.push(vec![range]),
            }
            joinable = true;
        } else if !(g == " " || g == "\t") {
            joinable = false;
        }
    }
    out
}

/// Punctuation that closes a phrase: the cluster may sit right before it once its space is gone, or right after it.
fn closes(c: char) -> bool {
    matches!(
        c,
        '.' | ','
            | ';'
            | ':'
            | '!'
            | '?'
            | ')'
            | ']'
            | '"'
            | '\''
            | '\u{201D}'
            | '\u{2019}'
            | '\u{2026}'
    )
}

/// The safe fix for one cluster: its source range grown by one adjacent space, → "". `None` when the cluster is the
/// only prose on its line or its neighbours are not ones the module docs list.
fn cluster_fix(
    bt: &BlockText,
    src: &str,
    cluster: &[Range<usize>],
) -> Option<(Range<usize>, String)> {
    let (first, last) = (cluster.first()?, cluster.last()?);
    if alone_on_line(&bt.text, first.start..last.end) {
        return None;
    }
    let range = bt.src_range(first.start..last.end)?;
    let before = src[..range.start].chars().next_back();
    let after = src[range.end..].chars().next();
    let space = |c: Option<char>| matches!(c, Some(' ' | '\t'));
    let line_end = |c: Option<char>| matches!(c, None | Some('\n' | '\r'));
    let word_or_close =
        |c: Option<char>| c.is_some_and(|c| c != OBJ && (c.is_alphanumeric() || closes(c)));
    let edit = if space(before) && space(after) {
        range.start..range.end + 1
    } else if space(before) && (line_end(after) || after.is_some_and(closes)) {
        range.start - 1..range.end
    } else if (line_end(before) || word_or_close(before)) && space(after) {
        range.start..range.end + 1
    } else if word_or_close(before) && (line_end(after) || after.is_some_and(closes)) {
        range
    } else {
        return None;
    };
    Some((edit, String::new()))
}

/// Whether the line of the block text holding `cluster` has no letter or digit outside it.
fn alone_on_line(text: &str, cluster: Range<usize>) -> bool {
    let start = text[..cluster.start].rfind('\n').map_or(0, |i| i + 1);
    let end = text[cluster.end..]
        .find('\n')
        .map_or(text.len(), |i| cluster.end + i);
    !text[start..cluster.start]
        .chars()
        .chain(text[cluster.end..end].chars())
        .any(char::is_alphanumeric)
}

#[cfg(test)]
mod tests {
    use super::is_emoji;

    #[test]
    fn pictographs_and_sequences_are_emoji() {
        for e in [
            "🚀",
            "✅",
            "⏳",
            "👍🏽",
            "👨\u{200D}💻",
            "🇺🇸",
            "1\u{FE0F}\u{20E3}",
            "#\u{FE0F}\u{20E3}",
            "©\u{FE0F}",
            "❤\u{FE0F}",
        ] {
            assert!(is_emoji(e), "{e:?}");
        }
    }

    #[test]
    fn plain_symbols_are_not_emoji() {
        for s in [
            "©", "®", "™", "↔", "☺", "❤", "1", "#", "*", "a", "\u{2014}", "\u{FFFC}",
        ] {
            assert!(!is_emoji(s), "{s:?}");
        }
    }
}
