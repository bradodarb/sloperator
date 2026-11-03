//! Window selection (SPEC §16, FR-9.2, FR-9.7, Gap 7): which source bytes the model rewrites, and what it is shown.
//!
//! A window is one prose block, or one sentence of it when `model.window = "sentence"`, that overlaps a finding the
//! model may fix. A window that cannot be sent safely is a [`WindowPlan::Skip`], which the pipeline reports as SLP902.
//! Nothing is ever skipped silently.

use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

use super::mask;
use crate::config::{Settings, WindowMode};
use crate::doc::{BlockId, Document, Newline};
use crate::rules::registry::Registry;
use crate::rules::{Finding, FixClass, RuleCode, Severity};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Window {
    pub block: BlockId,
    /// The source bytes the rewrite replaces.
    pub range: Range<usize>,
    /// Prose with ⟦n⟧ placeholders; soft breaks (and the next line's container prefix) folded to single spaces.
    pub masked: String,
    /// `slots[n-1]` is the source range of placeholder ⟦n⟧.
    pub slots: Vec<Range<usize>>,
    /// Some when the window spans more than one source line (Gap 7).
    pub wrap: Option<Wrap>,
    /// Weighted findings overlapping `range` (FR-9.5), system codes excluded.
    pub weight_before: u32,
}

/// Line structure of a multi-line window (Gap 7). `prefix` is the container continuation text that every line after
/// the first begins with: the bytes from the line start to the first prose byte, e.g. `"> "` or `"   "` (list-item
/// indentation). All continuation lines carry identical prefix bytes, or the window is a `Skip`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wrap {
    /// The widest source line of the window in chars, measured without its prefix.
    pub width: usize,
    pub prefix: String,
    pub newline: Newline,
    /// Chars on the window's first line before the window starts (non-zero only for a sentence window).
    pub indent: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WindowPlan {
    Rewrite(Window),
    /// Reported as SLP902 (see [`WindowPlan::skipped_finding`]).
    Skip {
        range: Range<usize>,
        reason: String,
    },
}

impl WindowPlan {
    /// The SLP902 `window-skipped` warning for a `Skip` (FR-9.7); `None` for a `Rewrite`.
    pub fn skipped_finding(&self) -> Option<Finding> {
        match self {
            WindowPlan::Rewrite(_) => None,
            WindowPlan::Skip { range, reason } => Some(Finding {
                code: RuleCode::WINDOW_SKIPPED,
                range: range.clone(),
                message: format!("model window skipped: {reason}"),
                severity: Severity::Warning,
                fix: None,
            }),
        }
    }
}

/// One window per block (or per sentence when `model.window = "sentence"`) that overlaps a finding with severity
/// ≥ `min_severity` the model may fix: a `Model`-class finding, or a `Safe`-class finding that carries no fix (the
/// rules engine had nothing to apply). A `FixClass::None` finding never selects a window. Windows come back in
/// source order, one per distinct range.
pub fn windows(
    doc: &Document,
    findings: &[Finding],
    frozen: &[Range<usize>],
    s: &Settings,
) -> Vec<WindowPlan> {
    let mut picked: Vec<(Option<BlockId>, Range<usize>)> = Vec::new();
    for f in findings.iter().filter(|f| selects(f, s.min_severity)) {
        let entry = match block_of(doc, &f.range) {
            Some(b) => {
                let range = match s.model.window {
                    WindowMode::Paragraph => doc.blocks[b.0 as usize].range.clone(),
                    WindowMode::Sentence => sentence_range(doc, b, &f.range),
                };
                (Some(b), range)
            }
            None => (None, f.range.clone()),
        };
        if !picked.contains(&entry) {
            picked.push(entry);
        }
    }
    picked.sort_by_key(|(_, r)| (r.start, r.end));
    picked
        .into_iter()
        .map(|(block, range)| match block {
            Some(b) => plan(doc, b, range, findings, frozen, s),
            None => WindowPlan::Skip {
                range,
                reason: "the finding lies outside every prose block".into(),
            },
        })
        .collect()
}

fn selects(f: &Finding, min: Severity) -> bool {
    if f.severity < min || f.code.is_system() {
        return false;
    }
    match Registry::builtin().get(f.code).map(|r| r.meta().fix) {
        Some(FixClass::Model) => true,
        Some(FixClass::Safe) => f.fix.is_none(),
        Some(FixClass::None) | None => false,
    }
}

fn block_of(doc: &Document, r: &Range<usize>) -> Option<BlockId> {
    let end = r.end.max(r.start + 1);
    doc.blocks
        .iter()
        .find(|b| b.range.start < end && r.start < b.range.end)
        .map(|b| b.id)
}

fn plan(
    doc: &Document,
    block: BlockId,
    range: Range<usize>,
    findings: &[Finding],
    frozen: &[Range<usize>],
    s: &Settings,
) -> WindowPlan {
    let skip = |reason: String| WindowPlan::Skip {
        range: range.clone(),
        reason,
    };
    let src = &doc.src[range.clone()];
    if frozen
        .iter()
        .any(|f| f.start < range.end && range.start < f.end)
    {
        return skip("the window overlaps a suppressed region".into());
    }
    if src.contains([mask::OPEN, mask::CLOSE]) {
        return skip(format!(
            "the source already contains a placeholder glyph ({} or {})",
            mask::OPEN,
            mask::CLOSE
        ));
    }
    let line_starts: Vec<usize> = src
        .match_indices('\n')
        .map(|(i, _)| range.start + i + 1)
        .filter(|&ls| ls < range.end)
        .collect();
    let prefixes: Vec<&str> = line_starts
        .iter()
        .map(|&ls| &doc.src[ls..ls + doc.continuation_prefix_len(ls, range.end)])
        .collect();
    if prefixes.windows(2).any(|p| p[0] != p[1]) {
        return skip(format!(
            "inconsistent container prefix ({})",
            prefixes
                .iter()
                .map(|p| format!("{p:?}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let (masked, slots) = mask::mask(doc, block, range.clone());
    if slots.iter().any(|r| doc.src[r.clone()].contains('\n')) {
        return skip("a hard line break or a protected run spans lines".into());
    }
    let chars = masked.chars().count();
    if chars > s.model.max_window_chars {
        return skip(format!(
            "the window is {chars} chars, over model.max_window_chars = {}",
            s.model.max_window_chars
        ));
    }
    let wrap = (!line_starts.is_empty()).then(|| {
        let (width, indent) = measure(doc, block, &range, &line_starts);
        Wrap {
            width,
            prefix: prefixes.first().map(|p| p.to_string()).unwrap_or_default(),
            newline: doc.newline,
            indent,
        }
    });
    let weight_before = findings
        .iter()
        .filter(|f| !f.code.is_system() && f.range.start < range.end && range.start < f.range.end)
        .map(|f| crate::score::weight(f.severity))
        .sum();
    WindowPlan::Rewrite(Window {
        block,
        range,
        masked,
        slots,
        wrap,
        weight_before,
    })
}

/// The widest line the window touches (in chars, prefix and trailing whitespace excluded), and how many chars of
/// its first line come before the window.
fn measure(
    doc: &Document,
    block: BlockId,
    range: &Range<usize>,
    line_starts: &[usize],
) -> (usize, usize) {
    let src = &*doc.src;
    let block_start = doc.blocks[block.0 as usize].range.start;
    let first_line = src[..range.start].rfind('\n').map_or(0, |i| i + 1);
    let content0 = (first_line + doc.continuation_prefix_len(first_line, range.end)).max(block_start);
    let indent = src[content0.min(range.start)..range.start].chars().count();
    let line_width = |from: usize| {
        let end = src[from..].find('\n').map_or(src.len(), |i| from + i);
        src[from..end].trim_end().chars().count()
    };
    let width = std::iter::once(line_width(content0.min(range.start)))
        .chain(
            line_starts
                .iter()
                .map(|&ls| line_width(ls + doc.continuation_prefix_len(ls, range.end))),
        )
        .max()
        .unwrap_or(0);
    (width.max(1), indent)
}

/// The sentences of `block` that `r` touches, as one source range with surrounding whitespace trimmed.
///
/// Sentences are found on a copy of the block in which line breaks and continuation prefixes are blanked (UAX #29
/// breaks after every line feed otherwise). A boundary inside a protected run is not a boundary.
fn sentence_range(doc: &Document, block: BlockId, r: &Range<usize>) -> Range<usize> {
    let b = doc.blocks[block.0 as usize].range.clone();
    let mut scan = doc.src.as_bytes()[b.clone()].to_vec();
    for byte in scan.iter_mut().filter(|b| **b == b'\n' || **b == b'\r') {
        *byte = b' ';
    }
    for (i, _) in doc.src[b.clone()].match_indices('\n') {
        let ls = b.start + i + 1;
        if ls < b.end {
            let p = doc.continuation_prefix_len(ls, b.end);
            scan[ls - b.start..ls - b.start + p].fill(b' ');
        }
    }
    // Only ASCII bytes were replaced with ASCII bytes, so the copy is still UTF-8.
    let Ok(scan) = String::from_utf8(scan) else {
        return b;
    };
    let mut starts: Vec<usize> = scan
        .split_sentence_bound_indices()
        .map(|(i, _)| i)
        .filter(|&i| i == 0 || !doc.is_protected(&(b.start + i..b.start + i)))
        .collect();
    starts.push(scan.len());
    let (lo, hi) = (
        r.start.saturating_sub(b.start),
        r.end.max(r.start + 1).saturating_sub(b.start),
    );
    let first = starts.partition_point(|&s| s <= lo).saturating_sub(1);
    let last = starts.partition_point(|&s| s < hi).max(first + 1);
    let (s, e) = (starts[first], starts[last.min(starts.len() - 1)]);
    let piece = &scan[s..e];
    let lead = piece.len() - piece.trim_start().len();
    let trail = piece.len() - piece.trim_end().len();
    if lead + trail >= piece.len() {
        return b;
    }
    b.start + s + lead..b.start + e - trail
}
