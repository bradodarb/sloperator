//! Directives (SPEC §12): which lines a file's `sloperator-*` comments suppress.
//!
//! [`extract`] reads the directive lines of a [`Document`] into the [`Suppressions`] that `rules::detect` and `edit`
//! query. A Markdown line is a directive only when one HTML span covers all of it (Gap 6), so a directive-shaped
//! line in a code fence, an indented block, inline code or front matter has no effect. A plain-text line is a
//! directive when `doc` tagged it `Protected(Directive)`.

use std::collections::BTreeSet;
use std::ops::Range;
use std::path::Path;
use std::sync::{Arc, LazyLock};

use regex::Regex;

use crate::doc::{Document, Format, ProtectReason, Span, SpanKind};
use crate::error::Error;
use crate::rules::registry::Registry;
use crate::types::RuleCode;

/// Markdown: a whole line `<!-- sloperator-<kind> [SLP### ...] -->`.
pub const MD_DIRECTIVE: &str =
    r"^\s*<!--\s*sloperator-(ignore-file|ignore|disable|enable)((?:\s+[A-Za-z0-9]+)*)\s*-->\s*$";
/// Plain text: a whole line `[sloperator-<kind> [SLP### ...]]`. `doc::plain` keeps a copy of this pattern.
pub const TXT_DIRECTIVE: &str =
    r"^\s*\[sloperator-(ignore-file|ignore|disable|enable)((?:\s+[A-Za-z0-9]+)*)\s*\]\s*$";

static MD_RE: LazyLock<Regex> = LazyLock::new(|| match Regex::new(MD_DIRECTIVE) {
    Ok(re) => re,
    Err(e) => unreachable!("MD_DIRECTIVE is a constant and must compile: {e}"),
});

static TXT_RE: LazyLock<Regex> = LazyLock::new(|| match Regex::new(TXT_DIRECTIVE) {
    Ok(re) => re,
    Err(e) => unreachable!("TXT_DIRECTIVE is a constant and must compile: {e}"),
});

/// The span reasons that can hold a Markdown directive line, and a plain-text one.
const MD_REASONS: &[ProtectReason] = &[ProtectReason::Html, ProtectReason::HtmlBlock];
const TXT_REASONS: &[ProtectReason] = &[ProtectReason::Directive];

/// The rules a directive names; `None` means every rule.
type Codes = Option<BTreeSet<RuleCode>>;

/// A `disable` waiting for its `enable`.
struct Region {
    codes: Codes,
    line: u32,
}

/// The suppressions of one file. The default suppresses nothing.
#[derive(Debug, Default)]
pub struct Suppressions {
    /// `ignore-file`.
    file: bool,
    /// 1-based line ranges; `None` means every rule.
    lines: Vec<(Range<u32>, Option<BTreeSet<RuleCode>>)>,
    /// Byte ranges of suppressed lines (FR-6.6).
    frozen: Vec<Range<usize>>,
}

impl Suppressions {
    /// Whether a finding of `code` on the 1-based `line` is suppressed. A system code (SLP900 and above) never is.
    pub fn is_suppressed(&self, line: u32, code: RuleCode) -> bool {
        !code.is_system()
            && (self.file
                || self.lines.iter().any(|(r, codes)| {
                    r.contains(&line) && codes.as_ref().is_none_or(|c| c.contains(&code))
                }))
    }

    /// The byte ranges of suppressed lines. Edits that touch one are dropped (FR-6.6).
    pub fn frozen_ranges(&self) -> &[Range<usize>] {
        &self.frozen
    }

    /// Adds a suppressed line range, for building a value by hand in tests. `codes` of `None` means every rule;
    /// `frozen` is the byte range of those lines.
    #[doc(hidden)]
    pub fn with_lines(
        mut self,
        lines: Range<u32>,
        codes: Option<BTreeSet<RuleCode>>,
        frozen: Range<usize>,
    ) -> Self {
        self.lines.push((lines, codes));
        self.frozen.push(frozen);
        self
    }

    /// Marks the whole file ignored (`ignore-file`), for the same use as [`Suppressions::with_lines`].
    #[doc(hidden)]
    pub fn with_file_ignored(mut self) -> Self {
        self.file = true;
        self
    }
}

/// Reads the directives of `doc` (`path` is only for error messages).
///
/// - `ignore` suppresses the next line that holds a non-blank prose byte; blank lines and other directives in
///   between are skipped. With no such line it does nothing.
/// - `disable` opens a region that an `enable` with the same code set closes. A `disable` still open at the end is
///   `Error::Directive` at its line (FR-6.3). An `enable` that closes nothing is `Error::Directive` too.
/// - `ignore-file` suppresses and freezes the whole document (FR-6.4).
/// - A word that is not the code of a registered rule, a system code included, is `Error::UnknownRule` naming
///   `path:line` (FR-6.5).
///
/// Each directive line is re-tagged `Protected(Directive)` in `doc.spans`.
///
/// # Errors
/// `Error::UnknownRule` or `Error::Directive`, as above.
pub fn extract(
    doc: &mut Document,
    path: &Path,
    registry: &Registry,
) -> Result<Suppressions, Error> {
    let src = Arc::clone(&doc.src);
    let (re, reasons): (&Regex, &[ProtectReason]) = match doc.format {
        Format::Markdown => (&*MD_RE, MD_REASONS),
        Format::PlainText => (&*TXT_RE, TXT_REASONS),
    };
    let mut sup = Suppressions::default();
    let mut pending: Vec<Codes> = Vec::new();
    let mut open: Vec<Region> = Vec::new();
    let mut retag: Vec<Range<usize>> = Vec::new();
    let mut at = 0;
    for (i, raw) in src.split_inclusive('\n').enumerate() {
        let line = u32::try_from(i + 1).unwrap_or(u32::MAX);
        let range = at..at + raw.len();
        at = range.end;
        let content = raw.trim_end_matches('\n').trim_end_matches('\r');
        let lead = content.len() - content.trim_start().len();
        let text = range.start + lead..range.start + lead + content.trim().len();
        let caps = if text.is_empty() || !covered_by(doc, &text, reasons) {
            None
        } else {
            re.captures(content)
        };
        let Some(caps) = caps else {
            if !pending.is_empty() && has_prose(doc, &range) {
                for codes in pending.drain(..) {
                    sup.lines.push((line..line + 1, codes));
                    sup.frozen.push(range.clone());
                }
            }
            continue;
        };
        retag.push(text);
        let words = caps.get(2).map_or("", |m| m.as_str());
        let codes = parse_codes(words, path, line, registry)?;
        match caps.get(1).map_or("", |m| m.as_str()) {
            "ignore-file" => {
                sup.file = true;
                sup.frozen.push(0..src.len());
            }
            "ignore" => pending.push(codes),
            "disable" => open.push(Region { codes, line }),
            _ => {
                let Some(p) = open.iter().rposition(|r| r.codes == codes) else {
                    return Err(Error::Directive {
                        file: path.to_path_buf(),
                        line,
                        reason: "sloperator-enable without matching sloperator-disable".into(),
                    });
                };
                let region = open.remove(p);
                if region.line + 1 < line {
                    sup.lines.push((region.line + 1..line, region.codes));
                    sup.frozen
                        .push(doc.line_range(region.line + 1).start..range.start);
                }
            }
        }
    }
    if let Some(region) = open.first() {
        return Err(Error::Directive {
            file: path.to_path_buf(),
            line: region.line,
            reason: "sloperator-disable without matching sloperator-enable".into(),
        });
    }
    for range in retag {
        retag_directive(&mut doc.spans, &range);
    }
    Ok(sup)
}

/// The codes in a directive's word list: `None` for no words. A word that names no registered rule is an error.
fn parse_codes(words: &str, path: &Path, line: u32, registry: &Registry) -> Result<Codes, Error> {
    let mut codes = BTreeSet::new();
    for word in words.split_whitespace() {
        let known = word
            .parse::<RuleCode>()
            .ok()
            .filter(|c| !c.is_system() && registry.get(*c).is_some());
        match known {
            Some(code) => {
                codes.insert(code);
            }
            None => {
                return Err(Error::UnknownRule {
                    code: word.to_string(),
                    where_: Some(format!("{}:{line}", path.display())),
                });
            }
        }
    }
    Ok((!codes.is_empty()).then_some(codes))
}

/// Whether one span with a reason in `reasons` covers all of `range`.
fn covered_by(doc: &Document, range: &Range<usize>, reasons: &[ProtectReason]) -> bool {
    let i = doc.spans.partition_point(|s| s.range.end <= range.start);
    doc.spans.get(i).is_some_and(|s| {
        s.range.start <= range.start
            && range.end <= s.range.end
            && matches!(s.kind, SpanKind::Protected(r) if reasons.contains(&r))
    })
}

/// Whether `range` holds a prose byte that is not whitespace.
fn has_prose(doc: &Document, range: &Range<usize>) -> bool {
    let first = doc.spans.partition_point(|s| s.range.end <= range.start);
    doc.spans[first..]
        .iter()
        .take_while(|s| s.range.start < range.end)
        .filter(|s| s.kind == SpanKind::Prose)
        .any(|s| {
            doc.src
                .get(s.range.start.max(range.start)..s.range.end.min(range.end))
                .is_some_and(|t| t.chars().any(|c| !c.is_whitespace()))
        })
}

/// Makes `range` a `Protected(Directive)` span, splitting the span that holds it. The span already is one when
/// `doc::plain` tagged it.
fn retag_directive(spans: &mut Vec<Span>, range: &Range<usize>) {
    let i = spans.partition_point(|s| s.range.end <= range.start);
    let Some(span) = spans.get(i).cloned() else {
        return;
    };
    let directive = SpanKind::Protected(ProtectReason::Directive);
    if span.kind == directive || span.range.start > range.start || span.range.end < range.end {
        return;
    }
    let mut pieces = Vec::with_capacity(3);
    if span.range.start < range.start {
        pieces.push(Span {
            range: span.range.start..range.start,
            ..span.clone()
        });
    }
    pieces.push(Span {
        range: range.clone(),
        kind: directive,
        block: span.block,
    });
    if range.end < span.range.end {
        pieces.push(Span {
            range: range.end..span.range.end,
            ..span.clone()
        });
    }
    spans.splice(i..=i, pieces);
}
