//! Placeholder masking and the output contract (SPEC §16, ADR-04 Option 1, FR-9.3).
//!
//! The model never sees protected bytes: [`mask`] replaces each protected run with `⟦n⟧`, and [`propose`] checks the
//! model's output, restores the runs from the source and re-wraps a multi-line window with its container prefix.

use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;

use super::window::{Window, Wrap};
use crate::doc::{BlockId, Document, Newline, ProtectReason, SpanKind};
use crate::edit::{Proposal, ProposalOrigin};
use crate::model::PromptProfile;
use crate::verify::{Check, Rejection};

pub const OPEN: char = '\u{27E6}'; // ⟦
pub const CLOSE: char = '\u{27E7}'; // ⟧

static PREAMBLE: LazyLock<Regex> = LazyLock::new(|| {
    match Regex::new(r"(?i)^(here(?:'s| is)|sure|certainly|rewritten|revised)\b[^\n]*:\s") {
        Ok(re) => re,
        Err(e) => unreachable!("PREAMBLE is a constant and must compile: {e}"),
    }
});

static BLANK_LINE: LazyLock<Regex> = LazyLock::new(|| match Regex::new(r"\n[ \t]*\r?\n") {
    Ok(re) => re,
    Err(e) => unreachable!("BLANK_LINE is a constant and must compile: {e}"),
});

/// Opening and closing quote pairs an output may not be wrapped in.
const QUOTES: &[(char, char)] = &[
    ('"', '"'),
    ('\'', '\''),
    ('\u{201C}', '\u{201D}'),
    ('\u{2018}', '\u{2019}'),
    ('\u{00AB}', '\u{00BB}'),
];

/// Each protected run inside the window becomes ⟦n⟧, numbered from 1 in source order (DQ-7). Protected runs are
/// inline code, link destinations, inline HTML, math, bare URLs, user patterns and terms, and emphasis or hard-break
/// markup; adjacent protected spans form one run.
///
/// A soft break becomes one space, together with the container prefix (`> `, list indentation) of the line after
/// it and any trailing-whitespace markup before it: those bytes are line structure, never sent to the model, and
/// `propose` rebuilds them when it re-wraps.
pub fn mask(doc: &Document, block: BlockId, range: Range<usize>) -> (String, Vec<Range<usize>>) {
    // A window never reaches past its block.
    let range = match doc.blocks.get(block.0 as usize) {
        Some(b) => range.start.max(b.range.start)..range.end.min(b.range.end),
        None => range,
    };
    let src = &*doc.src;
    let mut out = String::new();
    let mut slots: Vec<Range<usize>> = Vec::new();
    let mut run: Option<Range<usize>> = None;
    let flush =
        |run: &mut Option<Range<usize>>, out: &mut String, slots: &mut Vec<Range<usize>>| {
            if let Some(r) = run.take() {
                slots.push(r);
                out.push(OPEN);
                out.push_str(&slots.len().to_string());
                out.push(CLOSE);
            }
        };
    let first = doc.spans.partition_point(|s| s.range.end <= range.start);
    for span in &doc.spans[first..] {
        if span.range.start >= range.end {
            break;
        }
        let mut r = span.range.start.max(range.start)..span.range.end.min(range.end);
        match span.kind {
            SpanKind::Prose => {
                flush(&mut run, &mut out, &mut slots);
                out.push_str(&src[r].replace("\r\n", " ").replace('\n', " "));
            }
            SpanKind::Protected(reason) => {
                if r.start > range.start && src.as_bytes()[r.start - 1] == b'\n' {
                    r.start += doc.continuation_prefix_len(r.start, range.end);
                }
                let text = &src[r.clone()];
                let breaks_after =
                    src[r.end..].starts_with('\n') || src[r.end..].starts_with("\r\n");
                let line_ws = reason == ProtectReason::Markup
                    && breaks_after
                    && text.bytes().all(|b| b == b' ' || b == b'\t');
                if r.is_empty() || line_ws {
                    continue;
                }
                match run.as_mut() {
                    Some(open) if open.end == r.start => open.end = r.end,
                    _ => {
                        flush(&mut run, &mut out, &mut slots);
                        run = Some(r);
                    }
                }
            }
        }
    }
    flush(&mut run, &mut out, &mut slots);
    (out, slots)
}

/// The output contract (FR-9.3), then unmasking (ADR-04). Applied in order:
///  1. If `profile.strip_empty_think`: remove one leading `<think>` + whitespace + `</think>`.
///  2. Trim. Reject with `Check::OutputContract` if the result is empty; still contains a think tag; opens with a
///     preamble (`Here is …:`); is wrapped in quotes (unless the window itself is) or a ``` fence; contains a blank
///     line; or is longer than 2 × the masked window.
///  3. Each ⟦n⟧ for n in 1..=slots.len() must appear exactly once, and no other ⟦…⟧ may appear. Otherwise reject
///     with `Check::Placeholders`.
///  4. Replace each ⟦n⟧ with the source bytes of `slots[n-1]`. Line breaks in the output fold to spaces; if `wrap`
///     is Some, the prose is re-wrapped greedily at that width, every line after the first carrying the prefix.
pub fn propose(
    doc: &Document,
    w: &Window,
    raw: &str,
    profile: &PromptProfile,
) -> Result<Proposal, Rejection> {
    let reject = |failed, detail: String| Rejection {
        range: w.range.clone(),
        failed,
        detail,
    };
    let text = if profile.strip_empty_think {
        strip_empty_think(raw)
    } else {
        raw
    };
    let text = text.trim();
    contract(text, &w.masked).map_err(|d| reject(Check::OutputContract, d))?;
    placeholders(text, w.slots.len()).map_err(|d| reject(Check::Placeholders, d))?;
    let restore = |word: &str| unmask(doc, w, word);
    let replacement = match &w.wrap {
        None => restore(&text.split_whitespace().collect::<Vec<_>>().join(" ")),
        Some(wrap) => rewrap(text.split_whitespace().map(restore), wrap),
    };
    Ok(Proposal {
        range: w.range.clone(),
        replacement,
        origin: ProposalOrigin::Model,
    })
}

fn strip_empty_think(raw: &str) -> &str {
    raw.trim_start()
        .strip_prefix("<think>")
        .map(str::trim_start)
        .and_then(|rest| rest.strip_prefix("</think>"))
        .unwrap_or(raw)
}

fn contract(text: &str, masked: &str) -> Result<(), String> {
    if text.is_empty() {
        return Err("the output is empty".into());
    }
    if text.contains("<think>") || text.contains("</think>") {
        return Err("the output contains a think block".into());
    }
    if PREAMBLE.is_match(text) {
        return Err("the output opens with a preamble".into());
    }
    if text.starts_with("```") {
        return Err("the output is wrapped in a code fence".into());
    }
    let input = masked.trim();
    let wrapped = |t: &str, (open, close): (char, char)| {
        t.chars().count() >= 2 && t.starts_with(open) && t.ends_with(close)
    };
    if let Some(q) = QUOTES
        .iter()
        .find(|q| wrapped(text, **q) && !wrapped(input, **q))
    {
        return Err(format!("the output is wrapped in quotes ({}…{})", q.0, q.1));
    }
    if BLANK_LINE.is_match(text) {
        return Err("the output contains a blank line".into());
    }
    let (n, m) = (text.chars().count(), masked.chars().count());
    if n > 2 * m {
        return Err(format!(
            "the output is {n} chars, over twice the window ({m})"
        ));
    }
    Ok(())
}

/// Every ⟦n⟧ for n in 1..=slots exactly once, and nothing else between the glyphs.
fn placeholders(text: &str, slots: usize) -> Result<(), String> {
    let mut seen = vec![0u32; slots];
    let mut rest = text;
    while let Some(i) = rest.find([OPEN, CLOSE]) {
        let after = &rest[i..];
        if !after.starts_with(OPEN) {
            return Err(format!("a stray {CLOSE} in the output"));
        }
        let body = &after[OPEN.len_utf8()..];
        let Some(j) = body
            .find([OPEN, CLOSE])
            .filter(|&j| body[j..].starts_with(CLOSE))
        else {
            return Err(format!("an unclosed {OPEN} in the output"));
        };
        let label = &body[..j];
        match label.parse::<usize>() {
            Ok(n) if (1..=slots).contains(&n) => seen[n - 1] += 1,
            _ => return Err(format!("unknown placeholder {OPEN}{label}{CLOSE}")),
        }
        rest = &body[j + CLOSE.len_utf8()..];
    }
    let list = |want: fn(u32) -> bool| {
        seen.iter()
            .enumerate()
            .filter(|(_, c)| want(**c))
            .map(|(i, _)| format!("{OPEN}{}{CLOSE}", i + 1))
            .collect::<Vec<_>>()
    };
    let (dropped, duplicated) = (list(|c| c == 0), list(|c| c > 1));
    match (dropped.is_empty(), duplicated.is_empty()) {
        (true, true) => Ok(()),
        (false, true) => Err(format!("placeholder dropped: {}", dropped.join(" "))),
        (true, false) => Err(format!("placeholder duplicated: {}", duplicated.join(" "))),
        (false, false) => Err(format!(
            "placeholder dropped: {}; duplicated: {}",
            dropped.join(" "),
            duplicated.join(" ")
        )),
    }
}

/// `text` with each ⟦n⟧ replaced by its source bytes. `placeholders` has already checked every label.
fn unmask(doc: &Document, w: &Window, text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find(OPEN) {
        out.push_str(&rest[..i]);
        let body = &rest[i + OPEN.len_utf8()..];
        let j = body.find(CLOSE).unwrap_or(body.len());
        let slot = body[..j]
            .parse::<usize>()
            .ok()
            .and_then(|n| n.checked_sub(1))
            .and_then(|n| w.slots.get(n));
        match slot {
            Some(r) => out.push_str(&doc.src[r.clone()]),
            None => out.push_str(&rest[i..i + OPEN.len_utf8() + j]),
        }
        rest = body.get(j + CLOSE.len_utf8()..).unwrap_or("");
    }
    out.push_str(rest);
    out
}

/// Greedy re-wrap (Gap 7). Words are whole after unmasking, so a protected run is never split. A word that would
/// start a line and read as a block marker (`-`, `+`, `*`, `#`, `>`, `1.`, `1)`) stays on the line before.
fn rewrap(words: impl Iterator<Item = String>, wrap: &Wrap) -> String {
    let newline = match wrap.newline {
        Newline::Lf => "\n",
        Newline::Crlf => "\r\n",
    };
    let mut out = String::new();
    let mut col = wrap.indent;
    let mut line_empty = true;
    for word in words {
        let n = word.chars().count();
        if !line_empty && col + 1 + n > wrap.width && !is_block_marker(&word) {
            out.push_str(newline);
            out.push_str(&wrap.prefix);
            col = 0;
            line_empty = true;
        }
        if !line_empty {
            out.push(' ');
            col += 1;
        }
        out.push_str(&word);
        col += n;
        line_empty = false;
    }
    out
}

fn is_block_marker(word: &str) -> bool {
    if matches!(word, "-" | "+" | "*" | "=" | "|") || word.starts_with('#') || word.starts_with('>')
    {
        return true;
    }
    let digits = word.bytes().take_while(u8::is_ascii_digit).count();
    digits > 0 && matches!(&word[digits..], "." | ")")
}
