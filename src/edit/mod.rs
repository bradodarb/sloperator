//! Proposals, plan, apply and diff (SPEC §7, ADR-03). Edits are source-byte ranges: every byte outside a verified
//! edit is copied unchanged, so a no-op plan round-trips the file byte for byte.
//!
//! `edit` does not use `rules`; it gets `RuleCode` through `types`.

use std::ops::Range;
use std::path::Path;

use crate::doc::Newline;
use crate::verify::Verified;

pub use crate::types::{Proposal, ProposalOrigin};

/// Conflict-free and sorted by start. Overlaps are dropped deterministically: earlier start wins, then the shorter
/// range, then the lower origin (rules by code, before the model).
#[derive(Debug)]
pub struct Plan {
    pub kept: Vec<Proposal>,
    pub dropped: Vec<Proposal>,
}

/// Sorts `props`, drops every proposal that touches a `frozen` range (FR-6.6) and every proposal that overlaps an
/// earlier kept one. The result does not depend on the input order, except between proposals that are identical in
/// range, origin and replacement text, which are interchangeable.
pub fn plan(mut props: Vec<Proposal>, frozen: &[Range<usize>]) -> Plan {
    props.sort_by(|a, b| {
        (a.range.start, a.range.end, a.origin, &a.replacement).cmp(&(
            b.range.start,
            b.range.end,
            b.origin,
            &b.replacement,
        ))
    });
    let mut kept: Vec<Proposal> = Vec::new();
    let mut dropped = Vec::new();
    for p in props {
        let hits_frozen = frozen
            .iter()
            .any(|f| f.start < p.range.end && p.range.start < f.end);
        let overlaps = kept.last().is_some_and(|k| p.range.start < k.range.end);
        if hits_frozen || overlaps {
            dropped.push(p);
        } else {
            kept.push(p);
        }
    }
    Plan { kept, dropped }
}

/// Splices verified edits (ascending, non-overlapping, as `Plan::kept` yields them) into the original bytes. Every
/// other byte is copied (ADR-03). Replacement newlines are normalised to the document's style, so CRLF files stay
/// CRLF (FR-8.4). The final newline of a file lies in a protected run, so no edit covers it.
///
/// # Panics
/// In debug builds, when the edits are not ascending and non-overlapping.
pub fn apply(src: &str, edits: &[Verified<Proposal>], nl: Newline) -> String {
    let mut out = String::with_capacity(src.len());
    let mut at = 0;
    for e in edits {
        let p = e.get();
        debug_assert!(p.range.start >= at, "edits must be ascending and disjoint");
        out.push_str(&src[at..p.range.start]);
        match nl {
            Newline::Lf => out.push_str(&p.replacement),
            Newline::Crlf => {
                out.push_str(&p.replacement.replace("\r\n", "\n").replace('\n', "\r\n"))
            }
        }
        at = p.range.end;
    }
    out.push_str(&src[at..]);
    out
}

/// `--diff` output: a unified diff with `---`/`+++` headers and `@@` hunks (FR-2.5).
pub fn unified_diff(path: &Path, before: &str, after: &str) -> String {
    let name = path.display().to_string();
    similar::TextDiff::from_lines(before, after)
        .unified_diff()
        .context_radius(3)
        .header(&format!("a/{name}"), &format!("b/{name}"))
        .to_string()
}
