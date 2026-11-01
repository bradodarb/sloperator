//! Markdown parse (SPEC §6, ADR-03): a pulldown-cmark event walk that paints every source byte as prose or
//! protected, then cuts the result into spans.
//!
//! Every byte starts as `Protected(Markup)`. Text inside an open block turns it into prose, and the constructs
//! SPEC §6 lists (code, link destinations, images, HTML, front matter, math, reference definitions) are painted over.
//! A block's range runs from its first to its last inline byte, so list markers, heading hashes and table pipes stay
//! outside every block while emphasis markers and continuation prefixes (`> `) inside a block belong to it.

use std::ops::Range;
use std::sync::Arc;

use pulldown_cmark::{Event, LinkType, Options, Parser, Tag, TagEnd};

use super::{
    Block, BlockId, BlockKind, Canvas, Document, Format, ProtectReason, ProtectSettings, SpanKind,
    protect,
};

pub(super) fn options(protect: &ProtectSettings) -> Options {
    let mut o = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS;
    if protect.front_matter {
        o |= Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
            | Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS;
    }
    if protect.math {
        o |= Options::ENABLE_MATH;
    }
    o
}

pub(super) fn parse(src: Arc<str>, protect: &ProtectSettings) -> Document {
    let mut walk = Walk::new(src.len());
    {
        let mut iter = Parser::new_ext(&src, options(protect)).into_offset_iter();
        for (event, range) in iter.by_ref() {
            walk.event(event, range);
        }
        for (_, def) in iter.reference_definitions().iter() {
            walk.canvas.paint(
                def.span.clone(),
                SpanKind::Protected(ProtectReason::RefDef),
                None,
            );
        }
    }
    let (canvas, blocks) = walk.finish();
    let spans = protect::split(canvas.into_spans(src.as_bytes()), &src, protect);
    Document::new(src, Format::Markdown, spans, blocks)
}

/// A block between its start and end event. `range` grows with every inline event inside it.
struct Open {
    kind: BlockKind,
    quote_depth: u8,
    range: Option<Range<usize>>,
    is_item: bool,
    /// An item whose first paragraph has taken the item's kind; the item itself then makes no block.
    consumed: bool,
}

/// A link between its start and end event. `last_end` is where the link text ends, so the rest is the destination.
struct OpenLink {
    range: Range<usize>,
    last_end: Option<usize>,
    skips: bool,
}

struct Walk {
    canvas: Canvas,
    open: Vec<Open>,
    links: Vec<OpenLink>,
    lists: Vec<bool>,
    quote: u8,
    /// Nonzero inside a construct painted whole (code block, HTML block, front matter, image, autolink).
    skip: u32,
    table_row: u32,
    table_col: u32,
    done: Vec<(BlockKind, u8, Range<usize>)>,
}

fn protected(reason: ProtectReason) -> SpanKind {
    SpanKind::Protected(reason)
}

impl Walk {
    fn new(len: usize) -> Walk {
        Walk {
            canvas: Canvas::new(len, protected(ProtectReason::Markup)),
            open: Vec::new(),
            links: Vec::new(),
            lists: Vec::new(),
            quote: 0,
            skip: 0,
            table_row: 0,
            table_col: 0,
            done: Vec::new(),
        }
    }

    fn event(&mut self, event: Event<'_>, range: Range<usize>) {
        match event {
            Event::Start(tag) => self.start(tag, range),
            Event::End(end) => self.end(end, range),
            _ if self.skip > 0 => {}
            Event::Text(_) | Event::SoftBreak => {
                if self.note(&range) {
                    self.canvas.paint(range, SpanKind::Prose, None);
                }
            }
            Event::Code(_) => self.inline(range, ProtectReason::InlineCode),
            Event::InlineMath(_) | Event::DisplayMath(_) => self.inline(range, ProtectReason::Math),
            Event::InlineHtml(_) => self.inline(range, ProtectReason::Html),
            Event::Html(_) => self.inline(range, ProtectReason::HtmlBlock),
            Event::FootnoteReference(_) | Event::HardBreak => {
                self.note(&range);
            }
            Event::Rule | Event::TaskListMarker(_) => {}
        }
    }

    fn inline(&mut self, range: Range<usize>, reason: ProtectReason) {
        self.note(&range);
        self.canvas.paint(range, protected(reason), None);
    }

    /// Grows the innermost open block and link to cover `range`. False when no block is open.
    fn note(&mut self, range: &Range<usize>) -> bool {
        if let Some(link) = self.links.last_mut() {
            link.last_end = Some(link.last_end.map_or(range.end, |e| e.max(range.end)));
        }
        let Some(open) = self.open.last_mut() else {
            return false;
        };
        open.range = Some(match &open.range {
            Some(r) => r.start.min(range.start)..r.end.max(range.end),
            None => range.clone(),
        });
        true
    }

    fn push(&mut self, kind: BlockKind, is_item: bool) {
        self.open.push(Open {
            kind,
            quote_depth: self.quote,
            range: None,
            is_item,
            consumed: false,
        });
    }

    fn pop(&mut self) {
        if let Some(open) = self.open.pop()
            && let Some(range) = open.range
        {
            self.done.push((open.kind, open.quote_depth, range));
        }
    }

    fn start(&mut self, tag: Tag<'_>, range: Range<usize>) {
        if self.skip > 0 {
            match tag {
                Tag::Image { .. } => self.skip += 1,
                Tag::Link { .. } => self.links.push(OpenLink {
                    range,
                    last_end: None,
                    skips: false,
                }),
                _ => {}
            }
            return;
        }
        match tag {
            Tag::Paragraph => {
                let kind = match self.open.last_mut() {
                    Some(item) if item.is_item && !item.consumed && item.range.is_none() => {
                        item.consumed = true;
                        item.kind
                    }
                    _ => BlockKind::Paragraph,
                };
                self.push(kind, false);
            }
            Tag::Heading { level, .. } => self.push(BlockKind::Heading(level as u8), false),
            Tag::Item => {
                let depth = u8::try_from(self.lists.len()).unwrap_or(u8::MAX);
                let ordered = self.lists.last().copied().unwrap_or(false);
                self.push(BlockKind::ListItem { depth, ordered }, true);
            }
            Tag::TableCell => {
                let kind = BlockKind::TableCell {
                    row: self.table_row,
                    col: self.table_col,
                };
                self.push(kind, false);
            }
            Tag::BlockQuote(_) => self.quote = self.quote.saturating_add(1),
            Tag::List(first) => self.lists.push(first.is_some()),
            Tag::TableHead => {
                self.table_row = 0;
                self.table_col = 0;
            }
            Tag::TableRow => self.table_col = 0,
            Tag::CodeBlock(_) => self.whole(range, ProtectReason::CodeBlock),
            Tag::HtmlBlock => self.whole(range, ProtectReason::HtmlBlock),
            Tag::MetadataBlock(_) => self.whole(range, ProtectReason::FrontMatter),
            Tag::Image { .. } => {
                self.note(&range);
                self.whole(range, ProtectReason::Image);
            }
            Tag::Link { link_type, .. } => {
                self.note(&range);
                let skips = matches!(link_type, LinkType::Autolink | LinkType::Email);
                if skips {
                    self.whole(range.clone(), ProtectReason::Autolink);
                }
                self.links.push(OpenLink {
                    range,
                    last_end: None,
                    skips,
                });
            }
            Tag::Emphasis
            | Tag::Strong
            | Tag::Strikethrough
            | Tag::Superscript
            | Tag::Subscript => {
                self.note(&range);
            }
            Tag::Table(_)
            | Tag::FootnoteDefinition(_)
            | Tag::DefinitionList
            | Tag::DefinitionListTitle
            | Tag::DefinitionListDefinition => {}
        }
    }

    /// Paints `range` as one protected run and ignores the events inside it.
    fn whole(&mut self, range: Range<usize>, reason: ProtectReason) {
        self.canvas.paint(range, protected(reason), None);
        self.skip += 1;
    }

    fn end(&mut self, end: TagEnd, _range: Range<usize>) {
        if self.skip > 0 {
            match end {
                TagEnd::Image
                | TagEnd::CodeBlock
                | TagEnd::HtmlBlock
                | TagEnd::MetadataBlock(_) => {
                    self.skip -= 1;
                }
                TagEnd::Link => {
                    let skips = self.links.pop().is_some_and(|l| l.skips);
                    self.skip -= u32::from(skips);
                }
                _ => {}
            }
            return;
        }
        match end {
            TagEnd::Paragraph | TagEnd::Heading(_) | TagEnd::Item => self.pop(),
            TagEnd::TableCell => {
                self.pop();
                self.table_col += 1;
            }
            TagEnd::TableHead => self.table_row = 1,
            TagEnd::TableRow => self.table_row += 1,
            TagEnd::BlockQuote(_) => self.quote = self.quote.saturating_sub(1),
            TagEnd::List(_) => {
                self.lists.pop();
            }
            TagEnd::Link => {
                if let Some(link) = self.links.pop() {
                    let text_end = link.last_end.unwrap_or(link.range.start + 1);
                    if text_end < link.range.end {
                        self.canvas.paint(
                            text_end..link.range.end,
                            protected(ProtectReason::LinkDest),
                            None,
                        );
                    }
                }
            }
            _ => {}
        }
    }

    /// The painted canvas and the blocks in source order, with each block's bytes tagged.
    fn finish(mut self) -> (Canvas, Vec<Block>) {
        self.done.sort_by_key(|(_, _, r)| r.start);
        let mut blocks = Vec::with_capacity(self.done.len());
        for (i, (kind, quote_depth, range)) in self.done.into_iter().enumerate() {
            let id = BlockId(u32::try_from(i).unwrap_or(u32::MAX));
            self.canvas.tag_all(range.clone(), id);
            blocks.push(Block {
                id,
                kind,
                quote_depth,
                range,
            });
        }
        (self.canvas, blocks)
    }
}
