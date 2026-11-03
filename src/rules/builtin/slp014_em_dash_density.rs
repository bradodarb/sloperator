//! SLP014 `em-dash-density` (SPEC §10, FR-4.3): em dashes (U+2014, or ` -- ` in prose). One dash is normal prose, so
//! the rule fires only when the file has more than `max_per_1000_words` of them per 1,000 prose words; then every
//! dash is reported. The safe fix turns a dash between two words, with the spaces around it, into ", ".

use std::ops::Range;

use crate::doc::{BlockText, OBJ};
use crate::rules::{
    Family, Finding, FixClass, ParamSpec, Rule, RuleCode, RuleCtx, RuleMeta, Severity,
};

const MAX_PER_1000_WORDS: &str = "max_per_1000_words";
const EM_DASH: char = '\u{2014}';
/// The ASCII stand-in; only with a space on each side, so `--flag` and `---` are not dashes.
const DOUBLE_HYPHEN: &str = " -- ";

struct EmDashDensity {
    meta: RuleMeta,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(EmDashDensity {
        meta: RuleMeta {
            code: RuleCode(14),
            name: "em-dash-density",
            family: Family::EmDash,
            severity: Severity::Warning,
            default_on: true,
            fix: FixClass::Safe,
            explain: "Flags every em dash (U+2014, or \" -- \" in prose) once the file has more than \
                      `max_per_1000_words` of them per 1,000 prose words. One dash is fine; model-written text uses \
                      them in place of commas, colons and full stops. The fix turns a dash between two words into \
                      \", \".",
            example: (
                "The cache is cold\u{2014}the build is slow.",
                "The cache is cold, the build is slow.",
            ),
            params: &[ParamSpec {
                name: MAX_PER_1000_WORDS,
                default: 3.0,
                min: 0.0,
                max: f64::MAX,
                help: "Em dashes per 1,000 prose words above which every dash is reported",
            }],
            since: "0.1.0",
        },
    })
}

impl Rule for EmDashDensity {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        let mut hits = Vec::new();
        for bt in ctx.blocks() {
            for dash in dashes(&bt.text) {
                if let Some(src) = bt.src_range(dash.clone()) {
                    let fix = comma_fix(&bt, dash);
                    hits.push((src, fix));
                }
            }
        }
        let max = ctx.param(MAX_PER_1000_WORDS);
        let density = hits.len() as f64 * 1000.0 / ctx.prose_words().max(1) as f64;
        // A NaN threshold (a param the rule does not declare) never fires.
        if density.partial_cmp(&max) != Some(std::cmp::Ordering::Greater) {
            return;
        }
        for (range, fix) in hits {
            let message = format!("em dash; {density:.1} per 1000 words is over {max}");
            out.push(ctx.finding(&self.meta, range, message, fix));
        }
    }
}

/// The text range of each dash: an em dash, or the `--` of a ` -- `.
fn dashes(text: &str) -> Vec<Range<usize>> {
    let mut out: Vec<Range<usize>> = text
        .match_indices(EM_DASH)
        .map(|(i, m)| i..i + m.len())
        .collect();
    let mut pos = 0;
    while let Some(i) = text.get(pos..).and_then(|t| t.find(DOUBLE_HYPHEN)) {
        let at = pos + i + 1;
        out.push(at..at + 2);
        // The closing space may open the next ` -- `.
        pos = at + 2;
    }
    out.sort_by_key(|r| r.start);
    out
}

/// The dash with one space on each side, when there is one, → ", ": only when a word character ends the text before
/// it and starts the text after it, so the comma joins two words. Anything else (a dash at a block edge, next to
/// punctuation or a non-prose run) has no fix.
fn comma_fix(bt: &BlockText, dash: Range<usize>) -> Option<(Range<usize>, String)> {
    let text = bt.text.as_str();
    let start = if text[..dash.start].ends_with(' ') {
        dash.start - 1
    } else {
        dash.start
    };
    let end = if text[dash.end..].starts_with(' ') {
        dash.end + 1
    } else {
        dash.end
    };
    let word = |c: char| c != OBJ && c.is_alphanumeric();
    let before = text[..start].chars().next_back()?;
    let after = text[end..].chars().next()?;
    if !word(before) || !word(after) {
        return None;
    }
    let src = bt.src_range(start..end)?;
    Some((src, ", ".to_string()))
}
