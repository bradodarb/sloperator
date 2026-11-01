//! SLP001 `ai-vocabulary-replaceable` (SPEC §10): padded verbs and phrases with a plain, same-meaning replacement.
//!
//! Every verb is listed per inflection with its own replacement, never as a lemma (safe-fix rule, Gap 2). Bare
//! "leverage" is a noun as often as a verb, so it is reported with no fix and left to the model.

use crate::rules::helpers::WordList;
use crate::rules::{Family, Finding, FixClass, Rule, RuleCode, RuleCtx, RuleMeta, Severity};

static ENTRIES: &[(&str, Option<&str>)] = &[
    ("delve into", Some("look at")),
    ("delves into", Some("looks at")),
    ("delved into", Some("looked at")),
    ("delving into", Some("looking at")),
    ("utilize", Some("use")),
    ("utilizes", Some("uses")),
    ("utilized", Some("used")),
    ("utilizing", Some("using")),
    ("in order to", Some("to")),
    ("facilitate", Some("help")),
    ("facilitates", Some("helps")),
    ("facilitated", Some("helped")),
    ("facilitating", Some("helping")),
    ("leverages", Some("uses")),
    ("leveraged", Some("used")),
    ("leveraging", Some("using")),
    ("leverage", None),
];

struct AiVocabularyReplaceable {
    meta: RuleMeta,
    list: WordList,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(AiVocabularyReplaceable {
        meta: RuleMeta {
            code: RuleCode(1),
            name: "ai-vocabulary-replaceable",
            family: Family::AiVocabulary,
            severity: Severity::Warning,
            default_on: true,
            fix: FixClass::Safe,
            explain: "Flags verbs and phrases that pad plain wording: delve into, utilize, in order to, facilitate and \
                      leverage. Each inflection with a safe replacement is fixed; a bare `leverage` is reported with \
                      no fix because it may be a noun.",
            example: (
                "We utilize the cache in order to look things up.",
                "We use the cache to look things up.",
            ),
            params: &[],
            since: "0.1.0",
        },
        list: WordList::new(ENTRIES),
    })
}

impl Rule for AiVocabularyReplaceable {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        for bt in ctx.blocks() {
            for (range, replacement) in self.list.find(&bt) {
                let message = match &replacement {
                    Some(r) => format!(
                        "`{}` is padded wording; use `{r}`",
                        &ctx.doc.src[range.clone()]
                    ),
                    None => format!(
                        "`{}` is padded wording; plainer wording is needed",
                        &ctx.doc.src[range.clone()]
                    ),
                };
                let fix = replacement.map(|r| (range.clone(), r));
                out.push(ctx.finding(&self.meta, range, message, fix));
            }
        }
    }
}
