//! Contextual transformations, applied in table order within a sentence.
use super::{Context, ContextRule, Tag::*};

pub(super) static RULES: &[ContextRule] = &[
    ContextRule {
        from: Verb,
        to: Noun,
        when: Context::PrevTag(Det),
    },
    ContextRule {
        from: Noun,
        to: Verb,
        when: Context::PrevTag(Modal),
    },
    ContextRule {
        from: Noun,
        to: Verb,
        when: Context::PrevWord("to"),
    },
    ContextRule {
        from: Adj,
        to: Adv,
        when: Context::NextTag(Adj),
    },
    ContextRule {
        from: Noun,
        to: Verb,
        when: Context::NextWord("it"),
    },
];
