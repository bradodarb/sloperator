//! Parse and span map (SPEC §6, ADR-03). `doc` is a pure function of text and settings.
//!
//! Apart from `crate::error` it uses no other crate module (SPEC §5), so the input types it needs, [`Format`] and
//! [`ProtectSettings`], are defined here. Every byte of the source belongs to exactly one [`Span`], which is either
//! prose or protected. Engines may only propose edits inside prose spans, so "engines see only prose" (FR-8.1) is
//! decided here and enforced by `verify`.
//!
//! [`Document::continuation_prefix_len`] identifies container markup for masking and windowing.
//! [`BlockText::to_src`] and [`BlockText::to_text`] map prose byte offsets in either direction; protected runs
//! have no offset mapping.

use std::collections::HashSet;
use std::ops::Range;
use std::sync::Arc;

use unicode_segmentation::UnicodeSegmentation;

mod markdown;
mod plain;
mod protect;

pub use protect::BARE_URL;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    Markdown,
    PlainText,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Newline {
    Lf,
    Crlf,
}

/// The index of a block in [`Document::blocks`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockKind {
    Paragraph,
    /// Level 1..=6.
    Heading(u8),
    /// `depth` is 1 for an item of a top-level list, 2 for an item of a list nested in it, and so on.
    ListItem {
        depth: u8,
        ordered: bool,
    },
    /// `row` 0 is the header row. Both are 0-based.
    TableCell {
        row: u32,
        col: u32,
    },
}

/// A run of prose text a rule or rewrite works on. `range` is the block's own bytes: a list item stops where a
/// nested block starts, so ranges of nested blocks do not overlap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub id: BlockId,
    pub kind: BlockKind,
    pub quote_depth: u8,
    pub range: Range<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtectReason {
    CodeBlock,
    InlineCode,
    LinkDest,
    LinkTitle,
    Image,
    Autolink,
    BareUrl,
    RefDef,
    HtmlBlock,
    Html,
    FrontMatter,
    Math,
    Markup,
    UserPattern,
    UserTerm,
    Directive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpanKind {
    Prose,
    Protected(ProtectReason),
}

/// `block` is the block the bytes belong to, when they lie inside one (a block's markup and inline protected runs
/// count as inside it). Bytes between blocks have none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub range: Range<usize>,
    pub kind: SpanKind,
    pub block: Option<BlockId>,
}

/// `[protect]` in `sloperator.toml` (FR-7.4). Defaults: patterns and terms empty; `front_matter`, `math` and `html`
/// all true.
///
/// `parse` compiles `patterns` itself, because a `RegexSet` reports no match ranges. `compiled` is the set config
/// built to validate them (an invalid regex is `Error::Config`, via [`ProtectSettings::new`]). A pattern that does
/// not compile is skipped here, never guessed.
#[derive(Clone, Debug, serde::Serialize)]
pub struct ProtectSettings {
    pub patterns: Vec<String>,
    pub terms: Vec<String>,
    pub front_matter: bool,
    pub math: bool,
    pub html: bool,
    #[serde(skip)]
    pub compiled: Option<Arc<regex::RegexSet>>,
}

impl Default for ProtectSettings {
    fn default() -> Self {
        ProtectSettings {
            patterns: Vec::new(),
            terms: Vec::new(),
            front_matter: true,
            math: true,
            html: true,
            compiled: None,
        }
    }
}

impl ProtectSettings {
    /// Settings with `patterns` compiled. An invalid pattern is the returned error.
    pub fn new(
        patterns: Vec<String>,
        terms: Vec<String>,
        front_matter: bool,
        math: bool,
        html: bool,
    ) -> Result<Self, regex::Error> {
        let compiled = if patterns.is_empty() {
            None
        } else {
            Some(Arc::new(regex::RegexSet::new(&patterns)?))
        };
        Ok(ProtectSettings {
            patterns,
            terms,
            front_matter,
            math,
            html,
            compiled,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockSig {
    pub kind: BlockKind,
    pub quote_depth: u8,
}

#[derive(Clone, Debug)]
pub struct Document {
    pub src: Arc<str>,
    pub format: Format,
    /// Ordered, non-overlapping, covering `0..src.len()`.
    pub spans: Vec<Span>,
    /// Ordered by `range.start`.
    pub blocks: Vec<Block>,
    pub newline: Newline,
    line_starts: Vec<usize>,
}

/// The placeholder for a non-prose run inside [`BlockText::text`].
pub const OBJ: char = '\u{FFFC}';

impl Document {
    fn new(src: Arc<str>, format: Format, spans: Vec<Span>, blocks: Vec<Block>) -> Document {
        let newline = match src.find('\n') {
            Some(i) if i > 0 && src.as_bytes()[i - 1] == b'\r' => Newline::Crlf,
            _ => Newline::Lf,
        };
        let mut line_starts = vec![0];
        line_starts.extend(src.match_indices('\n').map(|(i, _)| i + 1));
        Document {
            src,
            format,
            spans,
            blocks,
            newline,
            line_starts,
        }
    }

    /// The prose of one block. Each non-prose run inside it becomes one [`OBJ`], so patterns never match across code
    /// or links. An unknown id gives an empty text.
    pub fn block_text(&self, id: BlockId) -> BlockText {
        let mut out = BlockText {
            block: id,
            text: String::new(),
            map: Vec::new(),
        };
        let Some(block) = self.blocks.get(id.0 as usize) else {
            return out;
        };
        let first = self
            .spans
            .partition_point(|s| s.range.end <= block.range.start);
        // End of the last non-prose span pushed as an OBJ, so a contiguous run yields one OBJ.
        let mut obj_end: Option<usize> = None;
        for span in &self.spans[first..] {
            if span.range.start >= block.range.end {
                break;
            }
            if span.block != Some(id) {
                continue;
            }
            match span.kind {
                SpanKind::Prose => {
                    out.map
                        .push((out.text.len(), span.range.start, span.range.len()));
                    out.text.push_str(&self.src[span.range.clone()]);
                    obj_end = None;
                }
                SpanKind::Protected(_) => {
                    if obj_end != Some(span.range.start) {
                        out.text.push(OBJ);
                    }
                    obj_end = Some(span.range.end);
                }
            }
        }
        out
    }

    /// The number of `unicode_words()` over the prose spans.
    pub fn prose_words(&self) -> usize {
        self.spans
            .iter()
            .filter(|s| s.kind == SpanKind::Prose)
            .map(|s| self.src[s.range.clone()].unicode_words().count())
            .sum()
    }

    /// 1-based line and 1-based column counted in chars. An offset past the end is clamped to the end.
    pub fn line_col(&self, offset: usize) -> (u32, u32) {
        let offset = offset.min(self.src.len());
        let line = self.line_starts.partition_point(|&s| s <= offset);
        let start = self.line_starts[line - 1];
        let chars = self.src.as_bytes()[start..offset]
            .iter()
            .filter(|b| (**b & 0xC0) != 0x80)
            .count();
        (as_u32(line), as_u32(chars + 1))
    }

    /// The bytes of a 1-based line, its line ending included. A line that does not exist is empty at the end.
    pub fn line_range(&self, line: u32) -> Range<usize> {
        let end = self.src.len();
        let Some(i) = (line as usize).checked_sub(1) else {
            return end..end;
        };
        match self.line_starts.get(i) {
            Some(&start) => start..self.line_starts.get(i + 1).copied().unwrap_or(end),
            None => end..end,
        }
    }

    /// Whether `range` overlaps any protected span. An empty range overlaps one only strictly inside it.
    pub fn is_protected(&self, range: &Range<usize>) -> bool {
        let first = self.spans.partition_point(|s| s.range.end <= range.start);
        self.spans[first..]
            .iter()
            .take_while(|s| s.range.start < range.end)
            .any(|s| matches!(s.kind, SpanKind::Protected(_)))
    }

    /// The container prefix at line start `ls`: the leading spaces, tabs and `>` that are block markup (not prose),
    /// up to `end`.
    pub fn continuation_prefix_len(&self, ls: usize, end: usize) -> usize {
        let bytes = self.src.as_bytes();
        let mut i = ls;
        while i < end.min(bytes.len())
            && matches!(bytes[i], b' ' | b'\t' | b'>')
            && self.is_protected(&(i..i + 1))
        {
            i += 1;
        }
        i - ls
    }

    /// The protected bytes as a sorted multiset (SPEC §16). A block's continuation-prefix spans (`> `, list
    /// indentation: markup at the start of a line after the block's first) count once per block and distinct text,
    /// not once per line, so a rewrite that re-wraps the block keeps the multiset.
    pub fn protected_strings(&self) -> Vec<&str> {
        let mut out = Vec::new();
        let mut prefixes: HashSet<(BlockId, &str)> = HashSet::new();
        for span in &self.spans {
            let SpanKind::Protected(reason) = span.kind else {
                continue;
            };
            let text = &self.src[span.range.clone()];
            match span.block {
                Some(b) if reason == ProtectReason::Markup && self.is_continuation(span, b) => {
                    if prefixes.insert((b, text)) {
                        out.push(text);
                    }
                }
                _ => out.push(text),
            }
        }
        out.sort_unstable();
        out
    }

    fn is_continuation(&self, span: &Span, block: BlockId) -> bool {
        let start = span.range.start;
        self.blocks
            .get(block.0 as usize)
            .is_some_and(|b| start > b.range.start)
            && start > 0
            && self.src.as_bytes()[start - 1] == b'\n'
    }
}

fn as_u32(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// The prose of one block (see [`Document::block_text`]). `map` holds `(text_off, src_off, len)` per prose segment.
#[derive(Clone, Debug)]
pub struct BlockText {
    pub block: BlockId,
    pub text: String,
    map: Vec<(usize, usize, usize)>,
}

impl BlockText {
    /// The source byte of a text byte inside a prose segment; `None` for an [`OBJ`] byte or past the end.
    pub fn to_src(&self, text_off: usize) -> Option<usize> {
        self.map
            .iter()
            .find(|&&(t, _, len)| t <= text_off && text_off < t + len)
            .map(|&(t, s, _)| s + (text_off - t))
    }

    /// The text byte of a source byte inside a prose segment; `None` for protected bytes or past the end.
    pub fn to_text(&self, src_off: usize) -> Option<usize> {
        self.map
            .iter()
            .find(|&&(_, s, len)| s <= src_off && src_off < s + len)
            .map(|&(t, s, _)| t + (src_off - s))
    }

    /// The source bytes of a text range; `None` if the range covers an [`OBJ`] or lies outside the text. An empty
    /// range maps to an empty range at that point when it touches a prose segment.
    pub fn src_range(&self, r: Range<usize>) -> Option<Range<usize>> {
        if r.start > r.end || r.end > self.text.len() {
            return None;
        }
        if r.start == r.end {
            return self
                .map
                .iter()
                .find(|&&(t, _, len)| t <= r.start && r.start <= t + len)
                .map(|&(t, s, _)| (s + (r.start - t))..(s + (r.start - t)));
        }
        let mut covered = r.start;
        let mut start = None;
        let mut end = None;
        for &(t, s, len) in &self.map {
            if t + len <= r.start {
                continue;
            }
            if t >= r.end {
                break;
            }
            if t > covered {
                return None;
            }
            let (lo, hi) = (r.start.max(t), r.end.min(t + len));
            start.get_or_insert(s + (lo - t));
            end = Some(s + (hi - t));
            covered = hi;
        }
        if covered < r.end {
            return None;
        }
        Some(start?..end?)
    }
}

pub fn parse(src: Arc<str>, format: Format, protect: &ProtectSettings) -> Document {
    match format {
        Format::Markdown => markdown::parse(src, protect),
        Format::PlainText => plain::parse(src, protect),
    }
}

/// The block structure of a document: kinds and quote depths, in order. An edit must not change it (FR-8.6).
pub fn skeleton(doc: &Document) -> Vec<BlockSig> {
    doc.blocks
        .iter()
        .map(|b| BlockSig {
            kind: b.kind,
            quote_depth: b.quote_depth,
        })
        .collect()
}

/// One kind and one block per source byte, painted in event order and then cut into runs. This is what makes the
/// span map cover every byte exactly once: a later paint replaces an earlier one, and a byte never painted keeps
/// the fill.
struct Canvas {
    kinds: Vec<SpanKind>,
    blocks: Vec<u32>,
}

const NO_BLOCK: u32 = u32::MAX;

impl Canvas {
    fn new(len: usize, fill: SpanKind) -> Canvas {
        Canvas {
            kinds: vec![fill; len],
            blocks: vec![NO_BLOCK; len],
        }
    }

    fn paint(&mut self, range: Range<usize>, kind: SpanKind, block: Option<BlockId>) {
        let end = range.end.min(self.kinds.len());
        let start = range.start.min(end);
        self.kinds[start..end].fill(kind);
        self.blocks[start..end].fill(block.map_or(NO_BLOCK, |b| b.0));
    }

    /// Gives `block` to every byte in `range` that belongs to no block yet.
    fn tag_all(&mut self, range: Range<usize>, block: BlockId) {
        let end = range.end.min(self.kinds.len());
        let start = range.start.min(end);
        for slot in &mut self.blocks[start..end] {
            if *slot == NO_BLOCK {
                *slot = block.0;
            }
        }
    }

    /// Runs of equal kind and block. Markup inside a block is also cut at each line start, so a continuation prefix
    /// (`> `) is a span of its own.
    fn into_spans(self, src: &[u8]) -> Vec<Span> {
        let mut spans: Vec<Span> = Vec::new();
        for i in 0..self.kinds.len() {
            let (kind, block) = (self.kinds[i], self.blocks[i]);
            let cut = kind == SpanKind::Protected(ProtectReason::Markup)
                && block != NO_BLOCK
                && src[i - usize::from(i > 0)] == b'\n'
                && i > 0;
            if let Some(last) = spans.last_mut()
                && !cut
                && last.kind == kind
                && last.block.map_or(NO_BLOCK, |b| b.0) == block
            {
                last.range.end = i + 1;
                continue;
            }
            spans.push(Span {
                range: i..i + 1,
                kind,
                block: (block != NO_BLOCK).then_some(BlockId(block)),
            });
        }
        spans
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_text_inverts_to_src() {
        let doc = parse(
            Arc::from("Café before `code` and after."),
            Format::Markdown,
            &ProtectSettings::default(),
        );
        let text = doc.block_text(doc.blocks[0].id);
        for t in 0..text.text.len() {
            if let Some(s) = text.to_src(t) {
                assert_eq!(text.to_text(s), Some(t));
            }
        }
        let code = doc.src.find("`code`").unwrap();
        for s in code..code + "`code`".len() {
            assert_eq!(text.to_text(s), None);
        }
        assert_eq!(text.to_text(doc.src.len()), None);
        assert_eq!(text.to_text(usize::MAX), None);
    }

    #[test]
    fn continuation_prefix_len_skips_quote_markup() {
        let doc = parse(
            Arc::from("> First line\n> next line\n"),
            Format::Markdown,
            &ProtectSettings::default(),
        );
        let ls = doc.src.find("\n").unwrap() + 1;
        assert_eq!(doc.continuation_prefix_len(ls, doc.src.len()), 2);
        assert_eq!(doc.continuation_prefix_len(ls, ls + 1), 1);

        let plain = parse(
            Arc::from("First line\nnext line\n"),
            Format::PlainText,
            &ProtectSettings::default(),
        );
        let ls = plain.src.find("\n").unwrap() + 1;
        assert_eq!(plain.continuation_prefix_len(ls, plain.src.len()), 0);
    }
}
