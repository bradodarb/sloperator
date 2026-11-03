//! SLP004 `not-just-but-also` (SPEC §29): join certain single-word pairs; other matches fall through to the model.

use std::ops::Range;

use crate::doc::BlockText;
use crate::rules::helpers::PatternRule;
use crate::rules::{Family, Finding, FixClass, Rule, RuleCode, RuleCtx, RuleMeta, Severity};
use crate::text::Tag;

/// Up to 80 characters of X inside one sentence, then "but". "not only" needs "but also"; "not just" takes a bare
/// "but" too.
const PATTERNS: &[&str] = &[
    r"(?i)\bnot just\b[^.!?\n]{1,80}?,?\s+but\b(?:\s+also\b)?",
    r"(?i)\bnot only\b[^.!?\n]{1,80}?,?\s+but\s+also\b",
];

const TEMPLATE: &str = r"(?i)\bnot (?:just|only) (\w+),? but(?: also)? (\w+)\b";
const CONTENT: &[Tag] = &[
    Tag::Adj,
    Tag::Adv,
    Tag::Noun,
    Tag::Verb,
    Tag::VerbPast,
    Tag::VerbPart,
    Tag::Verb3s,
];

struct NotJustButAlso {
    meta: RuleMeta,
    patterns: PatternRule,
    template: regex::Regex,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(NotJustButAlso {
        meta: RuleMeta {
            code: RuleCode(4),
            name: "not-just-but-also",
            family: Family::NegativeParallelism,
            severity: Severity::Warning,
            default_on: true,
            fix: FixClass::Safe,
            explain: "Flags \"not just X, but (also) Y\" and \"not only X but also Y\". The frame pads a plain \
                      addition with a contrast. When X and Y are single words of the same part of speech that \
                      the tagger is certain of, Y ends the clause and all of it is one sentence, the fix is \
                      \"X and Y\"; otherwise no fix is attached and the model engine joins them.",
            example: (
                "The tool is not just fast, but also simple.",
                "The tool is fast and simple.",
            ),
            params: &[],
            since: "0.1.0",
        },
        patterns: PatternRule::new(PATTERNS),
        template: match regex::Regex::new(TEMPLATE) {
            Ok(r) => r,
            Err(e) => panic!("rule pattern does not compile: {e}"),
        },
    })
}

impl NotJustButAlso {
    /// `X and Y`, or `None` when any guard fails (FR-8.1, FR-8.3).
    fn template_fix(&self, ctx: &RuleCtx<'_>, bt: &BlockText, at: usize) -> Option<(Range<usize>, String)> {
        let c = self
            .template
            .captures_at(&bt.text, at)
            .filter(|c| c.get(0).is_some_and(|g| g.start() == at))?;
        let (whole, x, y) = (c.get(0)?, c.get(1)?, c.get(2)?);
        if !bt.text[whole.end()..]
            .chars()
            .next()
            .is_none_or(|ch| ".,;:!?".contains(ch))
        {
            return None;
        }
        let range = bt.src_range(whole.range())?;
        let (xs, ys) = (bt.src_range(x.range())?, bt.src_range(y.range())?);
        let seg = ctx.prose.segments();
        let (xi, yi) = (seg.word_at(xs.start)?, seg.word_at(ys.start)?);
        if seg.words[xi].range != xs
            || seg.words[yi].range != ys
            || seg.sentence_of(xi)? != seg.sentence_of(yi)?
        {
            return None;
        }
        let tags = ctx.prose.tags(ctx.doc);
        let (tx, ty) = (tags[xi], tags[yi]);
        if !(tx.certain && ty.certain && tx.tag == ty.tag && CONTENT.contains(&tx.tag)) {
            return None;
        }
        let first = if bt.text[at..].starts_with('N') {
            capitalise(x.as_str())
        } else {
            x.as_str().to_string()
        };
        Some((range, format!("{first} and {}", y.as_str())))
    }
}

fn capitalise(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

impl Rule for NotJustButAlso {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        for bt in ctx.blocks() {
            for (_, m) in self.patterns.find_text(&bt.text) {
                let Some(range) = bt.src_range(m.clone()) else {
                    continue;
                };
                let message = format!(
                    "`{}` is a negative parallelism",
                    &ctx.doc.src[range.clone()]
                );
                let fix = self.template_fix(ctx, &bt, m.start);
                out.push(ctx.finding(&self.meta, range, message, fix));
            }
        }
    }
}
