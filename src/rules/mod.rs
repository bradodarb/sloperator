//! Rule core (SPEC §9, ADR-06): the `Rule` trait, finding and metadata types, the registry and `detect`.
//!
//! A rule sees prose only (FR-4.4): `detect` drops every finding whose range overlaps a protected span, and every
//! finding on a line a directive suppresses. System codes (SLP900 and above) are not rules and are never suppressed.

use std::collections::BTreeMap;
use std::ops::Range;

use crate::doc::{BlockText, Document};

use crate::text::Prose;
use crate::types::ProposalOrigin;
pub use crate::types::{Proposal, RuleCode}; // re-exported so callers keep writing `rules::RuleCode` (Gap 3)

pub mod builtin;
pub mod helpers;
pub mod registry;

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    clap::ValueEnum,
)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Suggestion,
    Warning,
    Error,
}

/// What a rule's fix may be (FR-4.7): only `Safe` rules attach [`Finding::fix`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FixClass {
    Safe,
    Model,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Family {
    AiVocabulary,
    NegativeParallelism,
    RuleOfThree,
    CopulaAvoidance,
    SignificancePuffery,
    Hedging,
    Markup,
    ChatbotResidue,
    EmDash,
    Readability,
    Wordiness,
    Redundancy,
}

/// One tunable threshold (e.g. SLP014 `max_per_1000_words`); config validates names and types against these (FR-7.6).
#[derive(Clone, Copy, Debug)]
pub struct ParamSpec {
    pub name: &'static str,
    pub default: f64,
    /// Inclusive minimum accepted by config.
    pub min: f64,
    /// Inclusive maximum; `f64::MAX` denotes an unbounded upper limit.
    pub max: f64,
    pub help: &'static str,
}

pub struct RuleMeta {
    pub code: RuleCode,
    pub name: &'static str,
    pub family: Family,
    pub severity: Severity,
    pub default_on: bool,
    pub fix: FixClass,
    pub explain: &'static str,
    /// (flagged, fixed) text for `explain`.
    pub example: (&'static str, &'static str),
    pub params: &'static [ParamSpec],
    /// The crate version that added the rule (FR-4.5: a rule newer than the first release starts off).
    pub since: &'static str,
}

/// What a rule sees: prose only (FR-4.4), its resolved params, the prose word count (FR-4.3).
pub struct RuleCtx<'a> {
    pub doc: &'a Document,
    pub prose: &'a Prose,
    pub params: &'a BTreeMap<&'static str, f64>,
    pub severity: Severity,
}

impl RuleCtx<'_> {
    /// The prose of every block, in document order.
    pub fn blocks(&self) -> impl Iterator<Item = BlockText> + '_ {
        self.doc.blocks.iter().map(|b| self.doc.block_text(b.id))
    }

    pub fn prose_words(&self) -> usize {
        self.doc.prose_words()
    }

    /// A resolved param by name; NaN when the rule declares no such param (a NaN threshold never fires).
    pub fn param(&self, name: &str) -> f64 {
        self.params.get(name).copied().unwrap_or(f64::NAN)
    }

    /// A finding of `meta`'s rule at the context's severity. `fix` is a (source range, replacement) pair; it is kept
    /// only when the rule's fix class is `Safe` (FR-4.7), so a model or detection-only rule never carries one.
    pub fn finding(
        &self,
        meta: &RuleMeta,
        range: Range<usize>,
        message: String,
        fix: Option<(Range<usize>, String)>,
    ) -> Finding {
        let fix = fix
            .filter(|_| meta.fix == FixClass::Safe)
            .map(|(range, replacement)| Proposal {
                range,
                replacement,
                origin: ProposalOrigin::Rule(meta.code),
            });
        Finding {
            code: meta.code,
            range,
            message,
            severity: self.severity,
            fix,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Finding {
    pub code: RuleCode,
    pub range: Range<usize>,
    pub message: String,
    pub severity: Severity,
    pub fix: Option<Proposal>,
}

pub trait Rule: Send + Sync {
    fn meta(&self) -> &RuleMeta;
    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>);
}

/// The rule-set version (architecture §6): the crate version at which the default list last changed.
pub const RULE_SET_VERSION: &str = "0.2.0";

/// A rule with its resolved severity and params for one file (built by `config::Settings::enabled`, SPEC §13).
pub struct Enabled<'a> {
    pub rule: &'a dyn Rule,
    pub severity: Severity,
    pub params: BTreeMap<&'static str, f64>,
}

/// Runs every enabled rule, drops findings that overlap a protected span (FR-4.4) or sit on a suppressed line, and
/// sorts by (start, code): source order (FR-3.6). A finding's line is the line of its start byte. System codes are
/// never suppressed (`Suppressions::is_suppressed`).
pub fn detect(
    doc: &Document,
    prose: &Prose,
    sup: &crate::directives::Suppressions,
    rules: &[Enabled<'_>],
) -> Vec<Finding> {
    let mut out = Vec::new();
    for enabled in rules {
        let ctx = RuleCtx {
            doc,
            prose,
            params: &enabled.params,
            severity: enabled.severity,
        };
        enabled.rule.check(&ctx, &mut out);
    }
    out.retain(|f| {
        !doc.is_protected(&f.range) && !sup.is_suppressed(doc.line_col(f.range.start).0, f.code)
    });
    out.sort_by_key(|f| (f.range.start, f.code));
    out
}
