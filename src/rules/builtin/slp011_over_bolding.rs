//! SLP011 `over-bolding` (SPEC §10): more than `max_per_100_words` strong spans per 100 prose words in one block;
//! then every strong span in that block is reported. Detection only: there is no fix that keeps the author's
//! emphasis, so the model does not fix it either (PRD Deviation Notes: D-4).

use std::collections::BTreeMap;
use std::ops::Range;

use pulldown_cmark::{Event, Tag, TagEnd};
use unicode_segmentation::UnicodeSegmentation;

use crate::doc::{BlockId, Document};
use crate::rules::helpers::{first_prose, markdown_events};
use crate::rules::{
    Family, Finding, FixClass, ParamSpec, Rule, RuleCode, RuleCtx, RuleMeta, Severity,
};

const MAX_PER_100_WORDS: &str = "max_per_100_words";

struct OverBolding {
    meta: RuleMeta,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(OverBolding {
        meta: RuleMeta {
            code: RuleCode(11),
            name: "over-bolding",
            family: Family::Markup,
            severity: Severity::Suggestion,
            default_on: true,
            fix: FixClass::None,
            explain: "Flags every strong span in a block that has more than `max_per_100_words` of them per 100 \
                      prose words. Model-written text bolds phrases that need no emphasis, so none stands out. Only \
                      the author knows which emphasis to keep, so there is no automatic fix.",
            example: (
                "The **cache** is **cold**, so the **build** is **slow**.",
                "The cache is cold, so the build is **slow**.",
            ),
            params: &[ParamSpec {
                name: MAX_PER_100_WORDS,
                default: 2.0,
                help: "Strong spans per 100 prose words in one block above which each is reported",
            }],
            since: "0.1.0",
        },
    })
}

impl Rule for OverBolding {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        let max = ctx.param(MAX_PER_100_WORDS);
        for (block, spans) in strong_by_block(ctx.doc) {
            let words = ctx.doc.block_text(block).text.unicode_words().count();
            if words == 0 {
                continue;
            }
            let density = spans.len() as f64 * 100.0 / words as f64;
            // A NaN threshold (a param the rule does not declare) never fires.
            if density.partial_cmp(&max) != Some(std::cmp::Ordering::Greater) {
                continue;
            }
            for strong in spans {
                let Some(range) = first_prose(ctx.doc, &strong) else {
                    continue;
                };
                let message = format!(
                    "`{}` is bold; {density:.1} strong spans per 100 words is over {max}",
                    &ctx.doc.src[strong]
                );
                out.push(ctx.finding(&self.meta, range, message, None));
            }
        }
    }
}

/// The outermost strong spans of the document, grouped by the block that holds them. A span in no block (inside a
/// construct painted whole) is skipped.
fn strong_by_block(doc: &Document) -> BTreeMap<BlockId, Vec<Range<usize>>> {
    let mut out: BTreeMap<BlockId, Vec<Range<usize>>> = BTreeMap::new();
    let mut depth = 0u32;
    for (event, range) in markdown_events(doc) {
        match event {
            Event::Start(Tag::Strong) => {
                depth += 1;
                if depth > 1 {
                    continue;
                }
                let block = doc
                    .blocks
                    .iter()
                    .find(|b| b.range.start <= range.start && range.end <= b.range.end);
                if let Some(b) = block {
                    out.entry(b.id).or_default().push(range);
                }
            }
            Event::End(TagEnd::Strong) => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    out
}
