//! SLP002 `ai-vocabulary` (SPEC §10): words that read as model-written. Detection only; the model engine rewrites.

use crate::rules::helpers::WordList;
use crate::rules::{Family, Finding, FixClass, Rule, RuleCode, RuleCtx, RuleMeta, Severity};

static ENTRIES: &[(&str, Option<&str>)] = &[
    ("tapestry", None),
    ("testament", None),
    ("multifaceted", None),
    ("pivotal", None),
    ("realm", None),
    ("seamless", None),
    ("robust", None),
];

struct AiVocabulary {
    meta: RuleMeta,
    list: WordList,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(AiVocabulary {
        meta: RuleMeta {
            code: RuleCode(2),
            name: "ai-vocabulary",
            family: Family::AiVocabulary,
            severity: Severity::Suggestion,
            default_on: true,
            fix: FixClass::Model,
            explain: "Flags words that read as model-written: tapestry, testament, multifaceted, pivotal, realm, \
                      seamless and robust. No word has a safe replacement, so the model engine rewrites the \
                      sentence.",
            example: (
                "A rich tapestry of seamless tools.",
                "A varied set of tools that work together.",
            ),
            params: &[],
            since: "0.1.0",
        },
        list: WordList::new(ENTRIES),
    })
}

impl Rule for AiVocabulary {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        for bt in ctx.blocks() {
            for (range, _) in self.list.find(&bt) {
                let message = format!("`{}` reads as model-written", &ctx.doc.src[range.clone()]);
                out.push(ctx.finding(&self.meta, range, message, None));
            }
        }
    }
}
