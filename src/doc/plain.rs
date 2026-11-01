//! Plain-text parse (SPEC §6, D-10): each run of non-blank lines is one `Paragraph` block, and every byte is prose
//! except bare URLs, user patterns, user terms and directive lines.
//!
//! A directive line (FR-6.7) is `Protected(Directive)` and belongs to no block: it ends the paragraph before it, so a
//! directive written directly above a paragraph does not become part of that paragraph.

use std::sync::{Arc, LazyLock};

use regex::Regex;

use super::{
    Block, BlockId, BlockKind, Canvas, Document, Format, ProtectReason, ProtectSettings, SpanKind,
    protect,
};

/// A whole line `[sloperator-<kind> [SLP### ...]]`. The same pattern as `directives::TXT_DIRECTIVE` (SPEC §12):
/// `doc` may not depend on `directives` (SPEC §5), and `directives` tests keep the two in step.
const TXT_DIRECTIVE: &str =
    r"^\s*\[sloperator-(ignore-file|ignore|disable|enable)((?:\s+[A-Za-z0-9]+)*)\s*\]\s*$";

static DIRECTIVE_RE: LazyLock<Regex> = LazyLock::new(|| match Regex::new(TXT_DIRECTIVE) {
    Ok(re) => re,
    Err(e) => unreachable!("TXT_DIRECTIVE is a constant and must compile: {e}"),
});

pub(super) fn parse(src: Arc<str>, protect: &ProtectSettings) -> Document {
    let mut canvas = Canvas::new(src.len(), SpanKind::Prose);
    let mut blocks: Vec<Block> = Vec::new();
    // Start and end (line ending excluded) of the paragraph being collected.
    let mut para: Option<(usize, usize)> = None;
    let mut at = 0;
    for line in src.split_inclusive('\n') {
        let start = at;
        at += line.len();
        let content = line.strip_suffix('\n').unwrap_or(line);
        let content = content.strip_suffix('\r').unwrap_or(content);
        let end = start + content.len();
        if content.trim().is_empty() {
            flush(&mut para, &mut blocks, &mut canvas);
        } else if DIRECTIVE_RE.is_match(content) {
            flush(&mut para, &mut blocks, &mut canvas);
            canvas.paint(
                start..end,
                SpanKind::Protected(ProtectReason::Directive),
                None,
            );
        } else {
            para = Some((para.map_or(start, |(s, _)| s), end));
        }
    }
    flush(&mut para, &mut blocks, &mut canvas);

    let spans = protect::split(canvas.into_spans(src.as_bytes()), &src, protect);
    Document::new(src, Format::PlainText, spans, blocks)
}

fn flush(para: &mut Option<(usize, usize)>, blocks: &mut Vec<Block>, canvas: &mut Canvas) {
    if let Some((start, end)) = para.take() {
        let id = BlockId(u32::try_from(blocks.len()).unwrap_or(u32::MAX));
        canvas.paint(start..end, SpanKind::Prose, Some(id));
        blocks.push(Block {
            id,
            kind: BlockKind::Paragraph,
            quote_depth: 0,
            range: start..end,
        });
    }
}
