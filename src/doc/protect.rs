//! Bare URLs, user patterns and user terms: carved out of prose spans as protected runs (SPEC §6, FR-8.3).

use std::cmp::Reverse;
use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;

use super::{ProtectReason, ProtectSettings, Span, SpanKind};

/// Bare URLs; trailing `.,;:!?)` trimmed after matching.
pub const BARE_URL: &str = r"(?:https?://|www\.)[^\s<>\x{FFFC}]+";

static BARE_URL_RE: LazyLock<Regex> = LazyLock::new(|| match Regex::new(BARE_URL) {
    Ok(re) => re,
    Err(e) => unreachable!("BARE_URL is a constant and must compile: {e}"),
});

/// Splits every prose span around the bare URLs, user patterns and user terms in it. Matching runs inside one
/// prose span, so a match never crosses code, a link or markup. Protected spans pass through unchanged.
///
/// Patterns are compiled here from `protect.patterns`. Config validates them first (an invalid one is
/// `Error::Config`), so a pattern that does not compile is skipped, not reported, at this layer.
pub(super) fn split(spans: Vec<Span>, src: &str, protect: &ProtectSettings) -> Vec<Span> {
    let patterns: Vec<Regex> = protect
        .patterns
        .iter()
        .filter_map(|p| Regex::new(p).ok())
        .collect();
    let terms: Vec<&str> = protect
        .terms
        .iter()
        .map(String::as_str)
        .filter(|t| !t.is_empty())
        .collect();

    let mut out = Vec::with_capacity(spans.len());
    for span in spans {
        if span.kind != SpanKind::Prose {
            out.push(span);
            continue;
        }
        let text = &src[span.range.clone()];
        let runs = matches(text, &patterns, &terms);
        let mut at = 0;
        for (run, reason) in runs {
            if run.start > at {
                out.push(piece(&span, at..run.start, SpanKind::Prose));
            }
            out.push(piece(&span, run.clone(), SpanKind::Protected(reason)));
            at = run.end;
        }
        if at < text.len() {
            out.push(piece(&span, at..text.len(), SpanKind::Prose));
        }
    }
    out
}

/// A piece of `span`; `rel` is relative to the span's start.
fn piece(span: &Span, rel: Range<usize>, kind: SpanKind) -> Span {
    Span {
        range: (span.range.start + rel.start)..(span.range.start + rel.end),
        kind,
        block: span.block,
    }
}

/// Non-overlapping protected runs in `text`, ascending. Of two overlapping matches the earlier start wins, then the
/// longer one.
fn matches(text: &str, patterns: &[Regex], terms: &[&str]) -> Vec<(Range<usize>, ProtectReason)> {
    let mut all: Vec<(Range<usize>, ProtectReason)> = Vec::new();
    for m in BARE_URL_RE.find_iter(text) {
        let trimmed = m
            .as_str()
            .trim_end_matches(['.', ',', ';', ':', '!', '?', ')']);
        if !trimmed.is_empty() {
            all.push((m.start()..m.start() + trimmed.len(), ProtectReason::BareUrl));
        }
    }
    for re in patterns {
        for m in re.find_iter(text).filter(|m| !m.is_empty()) {
            all.push((m.range(), ProtectReason::UserPattern));
        }
    }
    for term in terms {
        for (i, _) in text.match_indices(term) {
            all.push((i..i + term.len(), ProtectReason::UserTerm));
        }
    }
    all.sort_by_key(|(r, _)| (r.start, Reverse(r.end)));
    let mut kept: Vec<(Range<usize>, ProtectReason)> = Vec::with_capacity(all.len());
    for (range, reason) in all {
        if kept.last().is_none_or(|(last, _)| range.start >= last.end) {
            kept.push((range, reason));
        }
    }
    kept
}
