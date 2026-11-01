//! Leaf value types shared by `rules`, `edit`, `verify` and `directives` (SPEC §9, Gap 3).
//!
//! This module uses no other module of the crate, like `doc`, so every row of the layering table may depend on it.

use std::fmt;
use std::ops::Range;

/// A rule code, displayed and parsed as `SLP###`. Codes of 900 and above are system codes, not rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuleCode(pub u16);

impl RuleCode {
    /// A model rewrite the verifier rejected.
    pub const REWRITE_REJECTED: RuleCode = RuleCode(901);
    /// A window the model path skipped.
    pub const WINDOW_SKIPPED: RuleCode = RuleCode(902);

    /// System codes report what the tool did, and count as unknown in directives and config.
    pub fn is_system(self) -> bool {
        self.0 >= 900
    }
}

impl fmt::Display for RuleCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SLP{:03}", self.0)
    }
}

/// Serialized as its display form (`SLP014`), so a map keyed by code reads like the config file (SPEC §13).
impl serde::Serialize for RuleCode {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl std::str::FromStr for RuleCode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        s.strip_prefix("SLP")
            .filter(|d| d.len() == 3 && d.bytes().all(|b| b.is_ascii_digit()))
            .and_then(|d| d.parse().ok())
            .map(RuleCode)
            .ok_or_else(|| format!("`{s}` is not a rule code (SLP###)"))
    }
}

/// Who proposed an edit. The derived order puts every rule before the model, then orders rules by code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProposalOrigin {
    Rule(RuleCode),
    Model,
}

/// One edit: replace the source bytes in `range` with `replacement`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proposal {
    pub range: Range<usize>,
    pub replacement: String,
    pub origin: ProposalOrigin,
}
