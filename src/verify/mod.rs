//! The edit gate (SPEC §8, ADR-05). Every edit that reaches the source goes through [`edit`], and only this module
//! can build a [`Verified`], so an unverified byte is never written.
//!
//! A rejection is a finding, not an error: `pipeline` turns each [`Rejection`] into SLP901 and the original text
//! stays. Verification covers numbers, URLs, configured terms, placeholders, block structure, negations, modal
//! verbs and capitalised words as a proxy for names. It does not detect every change of meaning: a reworded claim
//! with the same negations, modals and capitalised words passes.

use std::ops::Range;
use std::sync::Arc;

use crate::doc::{self, Document, ProtectReason, ProtectSettings, SpanKind};
use crate::text::segment;
use crate::types::{Proposal, ProposalOrigin, RuleCode};

mod meaning;
mod tokens;

pub use tokens::{Tokens, tokens};

/// A value only this module can construct: the tuple field and `new` are private (ADR-05).
///
/// ```compile_fail
/// let _ = sloperator::verify::Verified(1);
/// ```
#[derive(Debug)]
pub struct Verified<T>(T);

impl<T> Verified<T> {
    fn new(t: T) -> Self {
        Verified(t)
    }

    pub fn get(&self) -> &T {
        &self.0
    }
}

/// The check that rejected an edit. Serialised camelCase.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Check {
    Numbers,
    Urls,
    Terms,
    Placeholders,
    /// The edit touches protected or suppressed bytes, or changes the protected bytes of the document.
    Protected,
    BlockType,
    NewBlockMarker,
    OutputContract,
    NoImprovement,
    /// The whole-document check ([`document`]).
    Document,
    /// The edit drops negations beyond the originating rule or window's allowance.
    Negation,
    /// The edit changes capitalized non-sentence-initial words (a proxy for names).
    ProperNouns,
    /// The edit changes the multiset of modal verbs.
    Modals,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rejection {
    pub range: Range<usize>,
    pub failed: Check,
    pub detail: String,
}

/// Per-edit verification (FR-8.5, FR-8.6). `frozen` is the suppressed ranges of the directives (FR-6.6).
///
/// The edit is rejected when it touches a protected or frozen range, loses, adds or changes a number, URL or
/// configured term, or when the document parsed after the edit has a different block structure or different
/// protected bytes. Meaning checks also reject dropped negations beyond the permitted allowance, changed
/// capitalized non-initial words and changed modals. `resolves` lists a model window's finding codes (duplicates
/// retained); rule-origin edits instead use their own code and skip common capitalized words.
pub fn edit(
    doc: &Document,
    p: Proposal,
    frozen: &[Range<usize>],
    protect: &ProtectSettings,
    resolves: &[RuleCode],
) -> Result<Verified<Proposal>, Rejection> {
    let r = p.range.clone();
    let reject = |failed, detail: String| Rejection {
        range: r.clone(),
        failed,
        detail,
    };
    let Some(original) = doc.src.get(r.clone()) else {
        return Err(reject(
            Check::Protected,
            "edit range is outside the document or not on a character boundary".into(),
        ));
    };
    if frozen.iter().any(|f| f.start < r.end && r.start < f.end) || doc.is_protected(&r) {
        return Err(reject(
            Check::Protected,
            "edit touches a protected or suppressed region".into(),
        ));
    }
    let before = tokens(original, protect);
    let after = tokens(&p.replacement, protect);
    if before.numbers != after.numbers {
        return Err(reject(
            Check::Numbers,
            tokens::diff(&before.numbers, &after.numbers),
        ));
    }
    if before.urls != after.urls {
        return Err(reject(Check::Urls, tokens::diff(&before.urls, &after.urls)));
    }
    if before.terms != after.terms {
        return Err(reject(
            Check::Terms,
            tokens::diff(&before.terms, &after.terms),
        ));
    }
    let initial = starts_at_sentence(doc, r.start);
    let skip_common = matches!(p.origin, ProposalOrigin::Rule(_));
    let (mb, ma) = (
        meaning::meaning(original, initial, skip_common),
        meaning::meaning(&p.replacement, initial, skip_common),
    );
    let allowance = meaning::negation_allowance(p.origin, resolves);
    if ma.negations.saturating_add(allowance) < mb.negations {
        return Err(reject(
            Check::Negation,
            format!(
                "negations {} -> {} (allowance {allowance})",
                mb.negations, ma.negations
            ),
        ));
    }
    if mb.proper != ma.proper {
        return Err(reject(
            Check::ProperNouns,
            tokens::diff(&mb.proper, &ma.proper),
        ));
    }
    if mb.modals != ma.modals {
        return Err(reject(Check::Modals, tokens::diff(&mb.modals, &ma.modals)));
    }
    let candidate: Arc<str> = format!(
        "{}{}{}",
        &doc.src[..r.start],
        p.replacement,
        &doc.src[r.end..]
    )
    .into();
    let next = doc::parse(candidate, doc.format, protect);
    let (a, b) = (doc::skeleton(doc), doc::skeleton(&next));
    if a.len() != b.len() {
        return Err(reject(
            Check::NewBlockMarker,
            format!("{} blocks became {}", a.len(), b.len()),
        ));
    }
    if a != b {
        return Err(reject(
            Check::BlockType,
            "a block changed type or level".into(),
        ));
    }
    // `directives::extract` splits each directive out of the span that held it, which a bare parse never does, so a
    // document with directives is compared through a bare parse of its own text (same bytes, same split).
    let has_directive = doc
        .spans
        .iter()
        .any(|s| s.kind == SpanKind::Protected(ProtectReason::Directive));
    let bare;
    let base = if has_directive {
        bare = doc::parse(Arc::clone(&doc.src), doc.format, protect);
        &bare
    } else {
        doc
    };
    if base.protected_strings() != next.protected_strings() {
        return Err(reject(Check::Protected, "protected bytes changed".into()));
    }
    Ok(Verified::new(p))
}

/// FR-9.5 as resolved in DQ-3: a rewrite is accepted only with strictly fewer weighted findings in its window.
/// Equal and higher counts are rejected.
pub fn improvement(before: u32, after: u32, range: Range<usize>) -> Result<(), Rejection> {
    if after < before {
        Ok(())
    } else {
        Err(Rejection {
            range,
            failed: Check::NoImprovement,
            detail: format!("weighted findings {before} -> {after}"),
        })
    }
}

/// Whole-file defence (architecture §7.2): the block skeleton and the protected strings are equal before and
/// after. The rejection covers the whole of `before`.
pub fn document(before: &Document, after: &Document) -> Result<(), Rejection> {
    let reject = |detail: String| Rejection {
        range: 0..before.src.len(),
        failed: Check::Document,
        detail,
    };
    let (a, b) = (doc::skeleton(before), doc::skeleton(after));
    if a != b {
        return Err(reject(format!(
            "block structure changed ({} blocks became {})",
            a.len(),
            b.len()
        )));
    }
    if before.protected_strings() != after.protected_strings() {
        return Err(reject("protected bytes changed".into()));
    }
    Ok(())
}

/// Whether the edit begins at a sentence boundary in its block's prose coordinates.
fn starts_at_sentence(doc: &Document, at: usize) -> bool {
    let Some(block) = doc
        .blocks
        .iter()
        .find(|b| b.range.start <= at && at < b.range.end)
    else {
        return false;
    };
    let bt = doc.block_text(block.id);
    bt.to_text(at)
        .is_some_and(|t| segment::starts_sentence(&bt.text, t))
}
