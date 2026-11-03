//! SLP003 `not-x-but-y` (SPEC §10): "it's not X, it's Y" and "this isn't X — it's Y", in one sentence or two
//! adjacent sentences. Short plain denials with the same subject in one sentence receive a safe template fix;
//! additive frames and unproven contexts remain for the model.

use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

use crate::doc::{BlockText, OBJ};
use crate::rules::helpers::{PatternRule, delete_lead_in, starts_sentence};
use crate::rules::{
    Family, Finding, FixClass, ParamSpec, Rule, RuleCode, RuleCtx, RuleMeta, Severity,
};

/// A negated "it/this/that is", up to 80 characters of X, a clause break (comma, semicolon, colon, dash or a sentence
/// end), then "it/this/that is" again, with an optional "but". The match is the whole construction.
const PATTERNS: &[&str] = &[
    r"(?i)\b(?:it|this|that)(?:['’]s not| is not| isn['’]t)\b[^.!?\n]{1,80}?(?:[,;:—–]|\s--?\s|[.!?])\s*(?:but\s+)?(?:it|this|that)(?:['’]s| is)\b",
];

struct NotXButY {
    meta: RuleMeta,
    patterns: PatternRule,
    template: regex::Regex,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(NotXButY {
        meta: RuleMeta {
            code: RuleCode(3),
            name: "not-x-but-y",
            family: Family::NegativeParallelism,
            severity: Severity::Warning,
            default_on: true,
            fix: FixClass::Safe,
            explain: "Flags \"it's not X, it's Y\" and \"this isn't X — it's Y\", in one sentence or in two adjacent \
                      sentences. The construction denies a claim nobody made before it states the real one. The \
                      template deletes a short denied clause in one sentence when both subjects match and X \
                      is a plain denial, not an additive frame. X and Y must each have at most max_words words; \
                      otherwise no fix is attached and the model engine states Y directly.",
            example: ("It's not a bug, it's a feature.", "It's a feature."),
            params: &[ParamSpec {
                name: MAX_WORDS,
                default: 3.0,
                min: 1.0,
                max: 10.0,
                help: "Maximum words in each template clause",
            }],
            since: "0.1.0",
        },
        patterns: PatternRule::new(PATTERNS),
        template: match regex::Regex::new(TEMPLATE) {
            Ok(r) => r,
            Err(e) => panic!("rule pattern does not compile: {e}"),
        },
    })
}

impl Rule for NotXButY {
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

const MAX_WORDS: &str = "max_words";

/// Focus words that make "not X" additive rather than a denial: "not only cheap" and "not just cheap" assert
/// cheapness. "even" is in the list because "not even X" is scalar (it can deny less than X), so deleting it is not
/// provably neutral. Sorted; checked against the first word of X, case-folded (Gap 3).
const ADDITIVE: &[&str] = &["even", "just", "merely", "only", "simply", "solely"];

/// Whether X is a plain denial: its first word is not an [`ADDITIVE`] focus word.
fn denies(x: &str) -> bool {
    let first = x
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_lowercase();
    ADDITIVE.binary_search(&first.as_str()).is_err()
}

/// FR-8.2's shape inside one sentence: subject + negated copula, X, a clause break that is not a sentence end, an
/// optional "but", the same subject + copula. Captures 1, 2, 3 = first subject, X, second subject.
const TEMPLATE: &str = r"(?i)\b(it|this|that)(?:['’]s not| is not| isn['’]t)\s+([^.!?\n,;:—–]{1,80}?)\s*(?:[,;:—–]|\s--?\s)\s*(?:but\s+)?(it|this|that)(?:['’]s| is)\b";

impl NotXButY {
    /// Deletes "it's not X, " (capitalising the next letter at a sentence start), or `None` when any guard fails.
    fn template_fix(
        &self,
        ctx: &RuleCtx<'_>,
        bt: &BlockText,
        at: usize,
    ) -> Option<(Range<usize>, String)> {
        let c = self
            .template
            .captures_at(&bt.text, at)
            .filter(|c| c.get(0).is_some_and(|g| g.start() == at))?;
        let (whole, s1, x, s2) = (c.get(0)?, c.get(1)?, c.get(2)?, c.get(3)?);
        if !s1.as_str().eq_ignore_ascii_case(s2.as_str()) {
            return None; // "This isn't X — it's Y" changes the subject
        }
        if !denies(x.as_str()) {
            return None; // "It's not only cheap, it's quick" adds a claim; deleting the clause would lose it (Gap 3)
        }
        let max = ctx.param(MAX_WORDS);
        let x_words = x.as_str().unicode_words().count();
        // A capital, a digit or a protected run in X would be lost; the verifier would reject it, so decline here.
        if x_words == 0
            || x_words as f64 > max
            || x.as_str()
                .chars()
                .any(|ch| ch.is_uppercase() || ch.is_ascii_digit() || ch == OBJ)
        {
            return None;
        }
        let rest = &bt.text[whole.end()..];
        let y = &rest[..rest
            .find(['.', '!', '?', ',', ';', ':', '\n'])
            .unwrap_or(rest.len())];
        let y_words = y.unicode_words().count();
        if y_words == 0 || y_words as f64 > max || y.contains(OBJ) {
            return None;
        }
        let seg = ctx.prose.segments();
        let first = seg.word_at(bt.src_range(s1.range())?.start)?;
        let second = seg.word_at(bt.src_range(s2.range())?.start)?;
        if seg.sentence_of(first)? != seg.sentence_of(second)? {
            return None;
        }
        let lead = at..s2.start();
        if starts_sentence(&bt.text, at) {
            delete_lead_in(bt, lead)
        } else {
            bt.src_range(lead).map(|r| (r, String::new()))
        }
    }
}
