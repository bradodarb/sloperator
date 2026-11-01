//! `doc` span map (SPEC §23.3): what is prose, what is protected, and the cover-every-byte invariant.
#![allow(clippy::unwrap_used, clippy::expect_used)] // test helpers may unwrap (architecture §8)

mod common;

use std::sync::Arc;

use common::fixture;
use sloperator::doc::{
    self, BlockKind, Document, Format, Newline, ProtectReason, ProtectSettings, SpanKind,
};

const FIXTURES: &[&str] = &[
    "code",
    "links",
    "front_matter",
    "table",
    "nested_lists",
    "crlf",
    "math",
    "html",
    "refdefs",
    "mixed",
];

fn parse_with(src: &str, format: Format, protect: &ProtectSettings) -> Document {
    doc::parse(Arc::from(src), format, protect)
}

fn md(src: &str) -> Document {
    parse_with(src, Format::Markdown, &ProtectSettings::default())
}

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture(&format!("protect/{name}.md"))).unwrap()
}

/// The reason of the protected span that holds `needle`, which must lie inside exactly one span.
fn reason_of(d: &Document, needle: &str) -> Option<ProtectReason> {
    match kind_of(d, needle) {
        SpanKind::Protected(r) => Some(r),
        SpanKind::Prose => None,
    }
}

fn kind_of(d: &Document, needle: &str) -> SpanKind {
    let at = d
        .src
        .find(needle)
        .unwrap_or_else(|| panic!("`{needle}` not in source"));
    let span = d
        .spans
        .iter()
        .find(|s| s.range.start <= at && at < s.range.end)
        .unwrap();
    assert!(
        span.range.end >= at + needle.len(),
        "`{needle}` crosses a span boundary: {span:?}"
    );
    span.kind
}

fn prose(d: &Document) -> String {
    d.spans
        .iter()
        .filter(|s| s.kind == SpanKind::Prose)
        .map(|s| &d.src[s.range.clone()])
        .collect()
}

fn is_prose(d: &Document, needle: &str) -> bool {
    kind_of(d, needle) == SpanKind::Prose
}

#[test]
fn code_blocks_and_inline_code_protected() {
    let d = md(&read_fixture("code"));
    assert_eq!(
        reason_of(&d, "`cargo test`"),
        Some(ProtectReason::InlineCode)
    );
    assert_eq!(
        reason_of(&d, "fn main() {\n    println!(\"hello\");\n}\n"),
        Some(ProtectReason::CodeBlock)
    );
    assert_eq!(reason_of(&d, "```rust"), Some(ProtectReason::CodeBlock));
    assert_eq!(
        reason_of(&d, "indented code block"),
        Some(ProtectReason::CodeBlock)
    );
    assert!(is_prose(&d, "Run "));
    assert!(is_prose(&d, "Plain text after the code."));
}

#[test]
fn link_destination_protected_link_text_prose() {
    let d = md(&read_fixture("links"));
    assert!(is_prose(&d, "project guide"));
    assert_eq!(
        reason_of(&d, "](https://example.com/guide \"The guide\")"),
        Some(ProtectReason::LinkDest)
    );
    assert!(is_prose(&d, "the reference"));
    assert_eq!(reason_of(&d, "][ref]"), Some(ProtectReason::LinkDest));
    assert!(is_prose(&d, "Read the "));
    assert!(is_prose(&d, " for details."));
}

#[test]
fn autolink_bare_url_image_refdef_protected() {
    let d = md(&read_fixture("links"));
    assert_eq!(
        reason_of(&d, "<https://example.org/page>"),
        Some(ProtectReason::Autolink)
    );
    assert_eq!(
        reason_of(&d, "<dev@example.org>"),
        Some(ProtectReason::Autolink)
    );
    // The sentence's trailing dot is not part of the URL.
    assert_eq!(
        reason_of(&d, "https://example.net/path/to/page"),
        Some(ProtectReason::BareUrl)
    );
    assert!(is_prose(&d, ". And "));
    assert_eq!(
        reason_of(&d, "www.example.com/x"),
        Some(ProtectReason::BareUrl)
    );
    assert_eq!(
        reason_of(&d, "![alt text here](img/logo.png)"),
        Some(ProtectReason::Image)
    );
    assert_eq!(
        reason_of(&d, "[ref]: https://example.com/ref \"Ref title\""),
        Some(ProtectReason::RefDef)
    );
    assert_eq!(
        reason_of(&d, "[shortcut]: https://example.com/shortcut"),
        Some(ProtectReason::RefDef)
    );
    let r = md(&read_fixture("refdefs"));
    assert_eq!(
        reason_of(&r, "[two]: <https://example.com/two> 'Second'"),
        Some(ProtectReason::RefDef)
    );
}

#[test]
fn front_matter_html_math_protected() {
    let d = md(&read_fixture("front_matter"));
    assert_eq!(
        reason_of(&d, "title: A document"),
        Some(ProtectReason::FrontMatter)
    );
    assert!(is_prose(&d, "Heading after front matter"));

    let h = md(&read_fixture("html"));
    assert_eq!(
        reason_of(&h, "Raw html block text."),
        Some(ProtectReason::HtmlBlock)
    );
    assert_eq!(
        reason_of(&h, "<span class=\"x\">"),
        Some(ProtectReason::Html)
    );
    assert!(is_prose(&h, "inline html"));
    assert_eq!(
        reason_of(&h, "<!-- sloperator-disable SLP001 -->"),
        Some(ProtectReason::HtmlBlock)
    );
    // `html = false` only lets rules see HTML text: it stays protected here (DQ-8).
    let off = ProtectSettings {
        html: false,
        ..ProtectSettings::default()
    };
    let h = parse_with(&read_fixture("html"), Format::Markdown, &off);
    assert_eq!(
        reason_of(&h, "Raw html block text."),
        Some(ProtectReason::HtmlBlock)
    );

    let m = md(&read_fixture("math"));
    assert_eq!(reason_of(&m, "$E = mc^2$"), Some(ProtectReason::Math));
    assert_eq!(
        reason_of(&m, "\\int_0^1 x \\, dx = \\frac{1}{2}"),
        Some(ProtectReason::Math)
    );
    assert!(is_prose(&m, "Prose resumes here."));

    // `front_matter = false` and `math = false` turn those parses off: the text is plain markdown again.
    let off = ProtectSettings {
        math: false,
        ..ProtectSettings::default()
    };
    let m = parse_with("Cost is $a$ here.\n", Format::Markdown, &off);
    assert!(is_prose(&m, "$a$"));
}

#[test]
fn heading_list_cell_quote_text_is_prose_markup_protected() {
    let d = md("# Title text\n\n- First item\n  - Nested item\n\n> Quoted words\n");
    assert!(is_prose(&d, "Title text"));
    assert_eq!(reason_of(&d, "# "), Some(ProtectReason::Markup));
    assert!(is_prose(&d, "First item"));
    assert!(is_prose(&d, "Nested item"));
    assert_eq!(reason_of(&d, "- "), Some(ProtectReason::Markup));
    assert!(is_prose(&d, "Quoted words"));
    assert_eq!(reason_of(&d, "> "), Some(ProtectReason::Markup));
    let kinds: Vec<_> = d.blocks.iter().map(|b| (b.kind, b.quote_depth)).collect();
    assert_eq!(
        kinds,
        vec![
            (BlockKind::Heading(1), 0),
            (
                BlockKind::ListItem {
                    depth: 1,
                    ordered: false
                },
                0
            ),
            (
                BlockKind::ListItem {
                    depth: 2,
                    ordered: false
                },
                0
            ),
            (BlockKind::Paragraph, 1),
        ]
    );

    let t = md(&read_fixture("table"));
    assert!(is_prose(&t, "Alpha"));
    assert!(is_prose(&t, "The second letter"));
    assert_eq!(reason_of(&t, "|------"), Some(ProtectReason::Markup));
    let cells: Vec<_> = t
        .blocks
        .iter()
        .filter_map(|b| match b.kind {
            BlockKind::TableCell { row, col } => Some((row, col, &t.src[b.range.clone()])),
            _ => None,
        })
        .collect();
    assert_eq!(
        cells,
        vec![
            (0, 0, "Name"),
            (0, 1, "Description"),
            (1, 0, "Alpha"),
            (1, 1, "The first letter"),
            (2, 0, "Beta"),
            (2, 1, "The second letter"),
        ]
    );

    let n = md(&read_fixture("nested_lists"));
    let items: Vec<_> = n
        .blocks
        .iter()
        .map(|b| (n.src[b.range.clone()].to_string(), b.kind))
        .collect();
    let item = |depth, ordered| BlockKind::ListItem { depth, ordered };
    assert_eq!(
        items,
        vec![
            ("First item".to_string(), item(1, false)),
            ("Nested item one".to_string(), item(2, false)),
            ("Nested item two".to_string(), item(2, false)),
            ("Second item".to_string(), item(1, false)),
            ("Ordered child".to_string(), item(2, true)),
            ("Another ordered child".to_string(), item(2, true)),
            ("Top ordered".to_string(), item(1, true)),
            ("Next ordered".to_string(), item(1, true)),
        ]
    );

    // A multi-line quote: the `> ` after the first line is a span of its own, counted once per block.
    let q = md("> one two\n> three four\n> five six\n");
    let strings = q.protected_strings();
    assert_eq!(
        strings.iter().filter(|s| **s == "> ").count(),
        2,
        "{strings:?}"
    );
    let rewrapped = md("> one two three\n> four five six\n");
    assert_eq!(
        q.protected_strings().len(),
        rewrapped.protected_strings().len()
    );
}

#[test]
fn user_patterns_and_terms_split_prose() {
    let settings = ProtectSettings::new(
        vec![r"[A-Z]{3}-\d+".to_string()],
        vec!["ACME-42".to_string()],
        true,
        true,
        true,
    )
    .unwrap();
    let d = parse_with(&read_fixture("mixed"), Format::Markdown, &settings);
    assert_eq!(reason_of(&d, "ACME-42"), Some(ProtectReason::UserTerm));
    assert_eq!(reason_of(&d, "ABC-123"), Some(ProtectReason::UserPattern));
    assert!(is_prose(&d, "Final paragraph, using the term "));
    assert!(is_prose(&d, " and ticket "));
    // `ACME-42` also matches the pattern; the term and the pattern cover the same bytes once.
    let hits: Vec<_> = d
        .spans
        .iter()
        .filter(|s| {
            matches!(
                s.kind,
                SpanKind::Protected(ProtectReason::UserTerm | ProtectReason::UserPattern)
            )
        })
        .collect();
    assert_eq!(hits.len(), 2);

    let defaults = md(&read_fixture("mixed"));
    assert!(is_prose(&defaults, "ACME-42"));

    // A bare URL inside a block of plain text and a term in plain text.
    let p = parse_with("See ACME-42 now.\n", Format::PlainText, &settings);
    assert_eq!(reason_of(&p, "ACME-42"), Some(ProtectReason::UserTerm));

    // The block text keeps one OBJ per protected run, and maps back to source bytes.
    let id = d.blocks.last().unwrap().id;
    let bt = d.block_text(id);
    assert_eq!(
        bt.text,
        format!(
            "Final paragraph, using the term {} and ticket {}.",
            doc::OBJ,
            doc::OBJ
        )
    );
    let at = bt.text.find("term").unwrap();
    assert_eq!(bt.to_src(at), d.src.find("term"));
    let r = bt.src_range(at..at + 4).unwrap();
    assert_eq!(&d.src[r], "term");
    let obj = bt.text.find(doc::OBJ).unwrap();
    assert_eq!(bt.to_src(obj), None);
    assert_eq!(bt.src_range(obj - 1..obj + doc::OBJ.len_utf8()), None);
}

#[test]
fn spans_cover_every_byte_without_overlap() {
    let mut texts: Vec<(String, Format)> = FIXTURES
        .iter()
        .map(|n| (read_fixture(n), Format::Markdown))
        .collect();
    texts.push((
        "Plain text.\n\n[sloperator-disable SLP001]\nMore https://x.org/a.\n".into(),
        Format::PlainText,
    ));
    texts.push(("".into(), Format::Markdown));
    texts.push(("no trailing newline".into(), Format::Markdown));
    for (text, format) in &texts {
        let d = parse_with(text, *format, &ProtectSettings::default());
        let mut at = 0;
        for s in &d.spans {
            assert_eq!(s.range.start, at, "gap or overlap before {s:?} in {text:?}");
            assert!(s.range.end > s.range.start, "empty span {s:?}");
            assert!(text.is_char_boundary(s.range.start) && text.is_char_boundary(s.range.end));
            at = s.range.end;
        }
        assert_eq!(at, text.len(), "spans stop short in {text:?}");
        for w in d.blocks.windows(2) {
            assert!(w[0].range.start <= w[1].range.start);
        }
        for (i, b) in d.blocks.iter().enumerate() {
            assert_eq!(b.id.0 as usize, i);
        }
    }
    // Prose words and positions use the same map.
    let d = md("Two words **bold** `code`.\n");
    assert_eq!(d.prose_words(), 3);
    assert_eq!(d.line_col(0), (1, 1));
    let d = md("a\nbé c\n");
    assert_eq!(d.line_col(d.src.find('c').unwrap()), (2, 4));
    assert_eq!(d.line_range(2), 2..8);
    assert!(d.is_protected(&(0..d.src.len())) || prose(&d) == d.src.as_ref());
}

#[test]
fn plain_text_paragraphs_and_directive_lines() {
    let src = "First paragraph line one\nline two.\n\n[sloperator-disable SLP001]\nSecond paragraph, see https://example.com/x.\n\n[sloperator-ignore-file]\n";
    let d = parse_with(src, Format::PlainText, &ProtectSettings::default());
    assert_eq!(d.format, Format::PlainText);
    let paras: Vec<_> = d
        .blocks
        .iter()
        .map(|b| (b.kind, &src[b.range.clone()]))
        .collect();
    assert_eq!(
        paras,
        vec![
            (BlockKind::Paragraph, "First paragraph line one\nline two."),
            (
                BlockKind::Paragraph,
                "Second paragraph, see https://example.com/x."
            ),
        ]
    );
    assert_eq!(
        reason_of(&d, "[sloperator-disable SLP001]"),
        Some(ProtectReason::Directive)
    );
    assert_eq!(
        reason_of(&d, "[sloperator-ignore-file]"),
        Some(ProtectReason::Directive)
    );
    assert_eq!(
        reason_of(&d, "https://example.com/x"),
        Some(ProtectReason::BareUrl)
    );
    assert!(is_prose(&d, "First paragraph line one\nline two."));
    // A line that merely mentions a directive is prose.
    let d = parse_with(
        "Write [sloperator-disable] on its own line.\n",
        Format::PlainText,
        &ProtectSettings::default(),
    );
    assert!(is_prose(&d, "Write [sloperator-disable] on its own line."));
    assert_eq!(doc::skeleton(&d).len(), 1);
}

#[test]
fn crlf_detected() {
    let d = md(&read_fixture("crlf"));
    assert_eq!(d.newline, Newline::Crlf);
    assert_eq!(md("one\ntwo\r\n").newline, Newline::Lf);
    assert_eq!(md("no newline").newline, Newline::Lf);
    let p = parse_with("a\r\nb\r\n", Format::PlainText, &ProtectSettings::default());
    assert_eq!(p.newline, Newline::Crlf);
    // The line ending is not part of a plain-text paragraph's range.
    assert_eq!(&p.src[p.blocks[0].range.clone()], "a\r\nb");
}
