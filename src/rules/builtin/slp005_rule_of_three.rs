//! SLP005 `rule-of-three` (SPEC §10): triads "A, B(,) and C" of items of one to three words. A triad is normal prose,
//! so the rule fires only when the file has more than `max_per_1000_words` of them per 1,000 prose words; then every
//! triad is reported. Detection only; the model engine rewrites.

use crate::rules::helpers::PatternRule;
use crate::rules::{
    Family, Finding, FixClass, ParamSpec, Rule, RuleCode, RuleCtx, RuleMeta, Severity,
};

const MAX_PER_1000_WORDS: &str = "max_per_1000_words";

/// Three items of one to three words each: two joined by a comma, the last by "and" (with an optional serial comma).
/// An item takes no punctuation but an inner apostrophe or hyphen, so a triad never crosses a sentence or a
/// non-prose run.
const PATTERNS: &[&str] = &[
    r"(?i)\b[\p{L}\p{N}'’-]+(?: [\p{L}\p{N}'’-]+){0,2}, [\p{L}\p{N}'’-]+(?: [\p{L}\p{N}'’-]+){0,2},? and [\p{L}\p{N}'’-]+(?: [\p{L}\p{N}'’-]+){0,2}\b",
];

struct RuleOfThree {
    meta: RuleMeta,
    patterns: PatternRule,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(RuleOfThree {
        meta: RuleMeta {
            code: RuleCode(5),
            name: "rule-of-three",
            family: Family::RuleOfThree,
            severity: Severity::Suggestion,
            default_on: true,
            fix: FixClass::Model,
            explain: "Flags triads \"A, B and C\" of items of one to three words once the file has more than \
                      `max_per_1000_words` of them per 1,000 prose words. Model-written prose reaches for three \
                      items whether or not there are three. No rewrite is safe, so the model engine decides which \
                      items stay.",
            example: (
                "It is fast, simple, and reliable.",
                "It is fast and simple.",
            ),
            params: &[ParamSpec {
                name: MAX_PER_1000_WORDS,
                default: 4.0,
                min: 0.0,
                max: f64::MAX,
                help: "Triads per 1,000 prose words above which every triad is reported",
            }],
            since: "0.1.0",
        },
        patterns: PatternRule::new(PATTERNS),
    })
}

impl Rule for RuleOfThree {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        let mut hits = Vec::new();
        for bt in ctx.blocks() {
            hits.extend(self.patterns.find(&bt).into_iter().map(|(_, range)| range));
        }
        let max = ctx.param(MAX_PER_1000_WORDS);
        let density = hits.len() as f64 * 1000.0 / ctx.prose_words().max(1) as f64;
        // A NaN threshold (a param the rule does not declare) never fires.
        if density.partial_cmp(&max) != Some(std::cmp::Ordering::Greater) {
            return;
        }
        for range in hits {
            let message = format!(
                "`{}` is a triad; {density:.1} per 1000 words is over {max}",
                &ctx.doc.src[range.clone()]
            );
            out.push(ctx.finding(&self.meta, range, message, None));
        }
    }
}
