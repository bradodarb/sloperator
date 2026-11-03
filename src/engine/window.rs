//! Window selection (SPEC §16, FR-9.2, FR-9.7, Gap 7): which source bytes the model rewrites, and what it is shown.
//!
//! A window is one prose block, or one sentence of it when `model.window = "sentence"`, that overlaps a finding the
//! model may fix. A window that cannot be sent safely is a [`WindowPlan::Skip`], which the pipeline reports as SLP902.
//! Nothing is ever skipped silently. Each planned rewrite retains one code per overlapping non-system finding,
//! including duplicates, in finding order; the same overlap filter computes its original weight.

use std::ops::Range;

use super::mask;
use crate::config::{Settings, WindowMode};
use crate::doc::{BlockId, Document, Newline};
use crate::rules::registry::Registry;
use crate::rules::{Finding, FixClass, RuleCode, Severity};
use crate::text::Prose;

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
    /// One code per overlapping non-system finding, in finding order; duplicates are retained.
    pub codes: Vec<RuleCode>,
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
    prose: &Prose,
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
                    WindowMode::Sentence => prose.segments().sentence_span(doc, b, &f.range),
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
    let overlapping = || {
        findings.iter().filter(|f| {
            !f.code.is_system() && f.range.start < range.end && range.start < f.range.end
        })
    };
    let weight_before = overlapping()
        .map(|f| crate::score::weight(f.severity))
        .sum();
    let codes = overlapping().map(|f| f.code).collect();
    WindowPlan::Rewrite(Window {
        block,
        range,
        masked,
        slots,
        wrap,
        weight_before,
        codes,
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
    let content0 =
        (first_line + doc.continuation_prefix_len(first_line, range.end)).max(block_start);
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::config::{CliOverrides, Resolver};
    use crate::doc::{self, Format, ProtectSettings};
    use crate::text::segment::scans;

    #[test]
    fn planning_does_not_rescan() {
        let doc = doc::parse(
            "Delve and unlock. Leverage and elevate.".into(),
            Format::Markdown,
            &ProtectSettings::default(),
        );
        assert_eq!(doc.blocks.len(), 1);
        let before = scans();
        let prose = Prose::new(&doc);
        assert_eq!(scans() - before, 1);
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("no-config-here/doc.md");
        let mut settings = (*Resolver::new(CliOverrides::default(), None)
            .unwrap()
            .for_file(&path)
            .unwrap())
        .clone();
        settings.model.window = WindowMode::Sentence;
        let findings: Vec<_> = ["Delve", "unlock", "Leverage", "elevate"]
            .into_iter()
            .map(|word| {
                let start = doc.src.find(word).unwrap();
                Finding {
                    code: RuleCode(2),
                    range: start..start + word.len(),
                    message: "test".into(),
                    severity: Severity::Warning,
                    fix: None,
                }
            })
            .collect();
        let before = scans();
        let plans = windows(&doc, &prose, &findings, &[], &settings);
        assert_eq!(scans(), before);
        let ranges: Vec<_> = plans
            .into_iter()
            .map(|plan| match plan {
                WindowPlan::Rewrite(w) => w.range,
                WindowPlan::Skip { reason, .. } => panic!("unexpected skip: {reason}"),
            })
            .collect();
        assert_eq!(ranges, vec![0..17, 18..39]);
    }
}
