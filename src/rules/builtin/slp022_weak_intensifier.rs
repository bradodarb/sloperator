//! SLP022 weak-intensifier (classical-nlp SPEC §21): detection only because the verifier does not check strength.

use crate::rules::helpers::WordList;
use crate::rules::{Family, Finding, FixClass, Rule, RuleCode, RuleCtx, RuleMeta, Severity};

static ENTRIES: &[(&str, Option<&str>)] = &[
    ("absolutely", None),
    ("actually", None),
    ("basically", None),
    ("extremely", None),
    ("highly", None),
    ("incredibly", None),
    ("literally", None),
    ("quite", None),
    ("really", None),
    ("totally", None),
    ("truly", None),
    ("very", None),
];

struct WeakIntensifier {
    meta: RuleMeta,
    list: WordList,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(WeakIntensifier {
        meta: RuleMeta {
            code: RuleCode(22),
            name: "weak-intensifier",
            family: Family::Wordiness,
            severity: Severity::Suggestion,
            default_on: false,
            fix: FixClass::None,
            explain: "Flags weak intensifiers that add emphasis rather than meaning. Detection only: deleting an \
                      intensifier changes strength, and the verifier does not check strength. No automatic fix \
                      is attached and no model window is selected.",
            example: ("The result is very clear.", "The result is clear."),
            params: &[],
            since: "0.2.0",
        },
        list: WordList::new(ENTRIES),
    })
}

impl Rule for WeakIntensifier {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        for bt in ctx.blocks() {
            for (range, _) in self.list.find(&bt) {
                let word = &ctx.doc.src[range.clone()];
                let message = format!("`{word}` adds emphasis, not meaning; consider deleting it");
                out.push(ctx.finding(&self.meta, range, message, None));
            }
        }
    }
}
