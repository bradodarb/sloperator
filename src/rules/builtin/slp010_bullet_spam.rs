//! SLP010 `bullet-spam` (SPEC §10): a list of at least `min_items` items in which at least 80% open with a bold
//! lead-in or have no more than `max_words` words. Detection only: fixing it changes list structure, which verify
//! rejects, so the model does not fix it either (PRD Deviation Notes: D-4).

use std::ops::Range;

use pulldown_cmark::{Event, Tag, TagEnd};
use unicode_segmentation::UnicodeSegmentation;

use crate::doc::{BlockKind, Document};
use crate::rules::helpers::{first_prose, markdown_events};
use crate::rules::{
    Family, Finding, FixClass, ParamSpec, Rule, RuleCode, RuleCtx, RuleMeta, Severity,
};

const MIN_ITEMS: &str = "min_items";
const MAX_WORDS: &str = "max_words";
/// The share of spam-shaped items, in percent, at which a list is reported.
const SPAM_PERCENT: usize = 80;

struct BulletSpam {
    meta: RuleMeta,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(BulletSpam {
        meta: RuleMeta {
            code: RuleCode(10),
            name: "bullet-spam",
            family: Family::Markup,
            severity: Severity::Suggestion,
            default_on: true,
            fix: FixClass::None,
            explain: "Flags a list of at least `min_items` items in which at least 80% of the items open with a \
                      bold lead-in or have no more than `max_words` words. Model-written text breaks reasoning into \
                      fragments. Fixing it means turning the list into prose, which changes the document's \
                      structure, so there is no automatic fix.",
            example: (
                "- **Fast:** quick\n- **Simple:** easy\n- **Safe:** sound\n- **Small:** tiny\n- **Cheap:** free",
                "It is fast, simple and safe, and it is small and free.",
            ),
            params: &[
                ParamSpec {
                    name: MIN_ITEMS,
                    default: 5.0,
                    min: 0.0,
                    max: f64::MAX,
                    help: "Fewest items a list needs before it can be reported",
                },
                ParamSpec {
                    name: MAX_WORDS,
                    default: 6.0,
                    min: 0.0,
                    max: f64::MAX,
                    help: "Most words an item may have and still count as a fragment",
                },
            ],
            since: "0.1.0",
        },
    })
}

/// One list item: its source range, and whether its first inline is a strong span.
struct Item {
    range: Range<usize>,
    bold_lead: bool,
}

struct List {
    range: Range<usize>,
    depth: u8,
    items: Vec<Item>,
}

impl Rule for BulletSpam {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        let min_items = ctx.param(MIN_ITEMS);
        let max_words = ctx.param(MAX_WORDS);
        for list in lists(ctx.doc) {
            let items = list.items.len();
            // A NaN threshold (a param the rule does not declare) never fires.
            if (items as f64)
                .partial_cmp(&min_items)
                .is_none_or(|o| o.is_lt())
            {
                continue;
            }
            let spam = list
                .items
                .iter()
                .filter(|item| {
                    let words = item_words(ctx.doc, list.depth, &item.range) as f64;
                    item.bold_lead || words <= max_words
                })
                .count();
            if spam * 100 < items * SPAM_PERCENT {
                continue;
            }
            let Some(range) = first_prose(ctx.doc, &list.range) else {
                continue;
            };
            let message = format!(
                "a list of {items} items, {spam} of them fragments or bold lead-ins, is bullet spam"
            );
            out.push(ctx.finding(&self.meta, range, message, None));
        }
    }
}

/// Every list in the document with its items, inner lists before the list that holds them.
fn lists(doc: &Document) -> Vec<List> {
    let mut done = Vec::new();
    let mut open: Vec<List> = Vec::new();
    // Set at an item's start until its first inline event: whether that event opens a strong span.
    let mut leading = false;
    for (event, range) in markdown_events(doc) {
        match event {
            Event::Start(Tag::List(_)) => {
                let depth = u8::try_from(open.len() + 1).unwrap_or(u8::MAX);
                open.push(List {
                    range,
                    depth,
                    items: Vec::new(),
                });
                leading = false;
            }
            Event::End(TagEnd::List(_)) => {
                if let Some(list) = open.pop() {
                    done.push(list);
                }
            }
            Event::Start(Tag::Item) => {
                if let Some(list) = open.last_mut() {
                    list.items.push(Item {
                        range,
                        bold_lead: false,
                    });
                }
                leading = true;
            }
            Event::Start(Tag::Paragraph) | Event::TaskListMarker(_) if leading => {}
            Event::Start(Tag::Strong) if leading => {
                if let Some(item) = open.last_mut().and_then(|l| l.items.last_mut()) {
                    item.bold_lead = true;
                }
                leading = false;
            }
            _ => leading = false,
        }
    }
    done
}

/// The word count of an item's own text: the first list-item block of the list's depth inside the item. An item
/// with no text has none.
fn item_words(doc: &Document, depth: u8, item: &Range<usize>) -> usize {
    doc.blocks
        .iter()
        .find(|b| {
            item.contains(&b.range.start)
                && matches!(b.kind, BlockKind::ListItem { depth: d, .. } if d == depth)
        })
        .map_or(0, |b| doc.block_text(b.id).text.unicode_words().count())
}
