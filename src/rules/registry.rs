//! The registry of built-in rules (SPEC §9). System codes SLP9xx are not rules and are never in it.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use super::{RULE_SET_VERSION, Rule, RuleCode};

pub struct Registry {
    rules: Vec<Box<dyn Rule>>,
    pub version: &'static str,
}

impl Registry {
    /// The built-in rules, built once.
    pub fn builtin() -> &'static Registry {
        static R: OnceLock<Registry> = OnceLock::new();
        R.get_or_init(|| Registry {
            rules: super::builtin::all(),
            version: RULE_SET_VERSION,
        })
    }

    /// A registry of the given rules, for tests that need codes the built-in catalogue does not have yet.
    #[doc(hidden)]
    pub fn from_rules(rules: Vec<Box<dyn Rule>>) -> Registry {
        Registry {
            rules,
            version: RULE_SET_VERSION,
        }
    }

    pub fn all(&self) -> impl Iterator<Item = &dyn Rule> {
        self.rules.iter().map(|r| r.as_ref())
    }

    pub fn get(&self, code: RuleCode) -> Option<&dyn Rule> {
        self.all().find(|r| r.meta().code == code)
    }

    /// The codes of the rules that are on by default (FR-4.5).
    pub fn default_set(&self) -> BTreeSet<RuleCode> {
        self.all()
            .filter(|r| r.meta().default_on)
            .map(|r| r.meta().code)
            .collect()
    }
}
