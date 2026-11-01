//! SLP009 `stacked-hedge` (SPEC §10): a modal followed by an adverb that says the same thing. The safe fix keeps
//! the modal.

use crate::rules::helpers::WordList;
use crate::rules::{Family, Finding, FixClass, Rule, RuleCode, RuleCtx, RuleMeta, Severity};

static ENTRIES: &[(&str, Option<&str>)] = &[
    ("may potentially", Some("may")),
    ("could possibly", Some("could")),
    ("might perhaps", Some("might")),
];

struct StackedHedge {
    meta: RuleMeta,
    list: WordList,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(StackedHedge {
        meta: RuleMeta {
            code: RuleCode(9),
            name: "stacked-hedge",
            family: Family::Hedging,
            severity: Severity::Suggestion,
            default_on: true,
            fix: FixClass::Safe,
            explain: "Flags two hedges stacked on one verb: may potentially, could possibly, might perhaps. The \
                      modal already hedges, so the fix keeps the first word.",
            example: ("It may potentially fail.", "It may fail."),
            params: &[],
            since: "0.1.0",
        },
        list: WordList::new(ENTRIES),
    })
}

impl Rule for StackedHedge {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        for bt in ctx.blocks() {
            for (range, replacement) in self.list.find(&bt) {
                let message = format!("`{}` stacks two hedges", &ctx.doc.src[range.clone()]);
                let fix = replacement.map(|r| (range.clone(), r));
                out.push(ctx.finding(&self.meta, range, message, fix));
            }
        }
    }
}
