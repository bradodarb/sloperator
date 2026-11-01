//! The edit gate (SPEC §8, ADR-05). Every edit that reaches the source goes through [`edit`], and only this module
//! can build a [`Verified`], so an unverified byte is never written.
//!
//! A rejection is a finding, not an error: `pipeline` turns each [`Rejection`] into SLP901 and the original text
//! stays. Verification covers numbers, URLs, terms and block structure, not names or claim polarity.

use std::ops::Range;
use std::sync::Arc;

use crate::doc::{self, Document, ProtectReason, ProtectSettings, SpanKind};
use crate::types::Proposal;

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
/// protected bytes.
pub fn edit(
    doc: &Document,
    p: Proposal,
    frozen: &[Range<usize>],
    protect: &ProtectSettings,
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
