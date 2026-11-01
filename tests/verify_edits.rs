//! `verify`: token, structure and improvement checks (SPEC §23.5).
#![allow(clippy::unwrap_used, clippy::expect_used)] // test helpers may unwrap (architecture §8)

use std::sync::Arc;

use sloperator::doc::{self, Document, Format, ProtectSettings};
use sloperator::types::{Proposal, ProposalOrigin};
use sloperator::verify::{self, Check, Rejection};

fn parse(src: &str, protect: &ProtectSettings) -> Document {
    doc::parse(Arc::from(src), Format::Markdown, protect)
}

/// Replaces the first `needle` in `src` with `replacement` and verifies the edit.
fn try_edit(
    src: &str,
    needle: &str,
    replacement: &str,
    frozen: &[std::ops::Range<usize>],
    protect: &ProtectSettings,
) -> Result<(), Rejection> {
    let d = parse(src, protect);
    let start = src.find(needle).unwrap();
    let p = Proposal {
        range: start..start + needle.len(),
        replacement: replacement.to_owned(),
        origin: ProposalOrigin::Model,
    };
    verify::edit(&d, p, frozen, protect).map(|_| ())
}

fn failed(r: Result<(), Rejection>) -> Check {
    r.unwrap_err().failed
}

fn plain() -> ProtectSettings {
    ProtectSettings::default()
}

#[test]
fn rejects_lost_number() {
    let r = try_edit(
        "We shipped 42 builds today.\n",
        "We shipped 42 builds today.",
        "We shipped builds today.",
        &[],
        &plain(),
    );
    assert_eq!(failed(r), Check::Numbers);
}

#[test]
fn rejects_added_number() {
    let r = try_edit(
        "We shipped builds today.\n",
        "We shipped builds today.",
        "We shipped 7 builds today.",
        &[],
        &plain(),
    );
    let e = r.unwrap_err();
    assert_eq!(e.failed, Check::Numbers);
    assert_eq!(e.detail, "added 7 ×1");
}

#[test]
fn rejects_changed_percentage() {
    let r = try_edit(
        "Usage rose 42% last year.\n",
        "Usage rose 42% last year.",
        "Usage rose 42 last year.",
        &[],
        &plain(),
    );
    assert_eq!(failed(r), Check::Numbers);
    let r = try_edit(
        "Usage rose 42% last year.\n",
        "Usage rose 42% last year.",
        "Usage rose 43% last year.",
        &[],
        &plain(),
    );
    assert_eq!(failed(r), Check::Numbers);
}

#[test]
fn rejects_changed_digit_date() {
    let r = try_edit(
        "It launched on 2024-01-05 for everyone.\n",
        "It launched on 2024-01-05 for everyone.",
        "It launched on 2024-01-06 for everyone.",
        &[],
        &plain(),
    );
    assert_eq!(failed(r), Check::Numbers);
}

#[test]
fn rejects_changed_url() {
    let r = try_edit(
        "See https://example.com/a for details.\n",
        "See https://example.com/a for details.",
        "See https://example.com/b for details.",
        &[],
        &plain(),
    );
    // A bare URL is a protected span, so the edit is rejected at the protected check or the URL check.
    assert!(matches!(failed(r), Check::Protected | Check::Urls));
    // The URL check itself, outside the span map: the tokens differ.
    let p = plain();
    let a = verify::tokens("See https://example.com/a now.", &p);
    let b = verify::tokens("See https://example.com/b now.", &p);
    assert_ne!(a.urls, b.urls);
    assert_eq!(a.urls.get("https://example.com/a"), Some(&1));
    assert!(a.numbers.is_empty(), "URL digits are not numbers");
}

#[test]
fn rejects_lost_term() {
    let protect = ProtectSettings::new(vec![], vec!["Acme".into()], true, true, true).unwrap();
    // The term is a protected span, so an edit that spans it is rejected; an edit that merely adds one is a
    // terms failure.
    let r = try_edit(
        "The tool is fast.\n",
        "The tool is fast.",
        "The Acme tool is fast.",
        &[],
        &protect,
    );
    assert_eq!(failed(r), Check::Terms);
    let r = try_edit(
        "The Acme tool is fast.\n",
        "The Acme tool is fast.",
        "The tool is fast.",
        &[],
        &protect,
    );
    assert!(matches!(failed(r), Check::Protected | Check::Terms));
    let a = verify::tokens("The Acme tool, Acme again.", &protect);
    assert_eq!(a.terms.get("Acme"), Some(&2));
}

#[test]
fn rejects_new_heading_marker() {
    let r = try_edit(
        "Intro text here.\n\nSecond paragraph.\n",
        "Intro text here.",
        "Intro text here.\n\n# New heading",
        &[],
        &plain(),
    );
    assert_eq!(failed(r), Check::NewBlockMarker);
    let r = try_edit(
        "Intro text here.\n\nSecond paragraph.\n",
        "Intro text here.",
        "# Intro text here.",
        &[],
        &plain(),
    );
    assert_eq!(failed(r), Check::BlockType);
}

#[test]
fn rejects_list_marker() {
    let r = try_edit(
        "Intro text here.\n\nSecond paragraph.\n",
        "Intro text here.",
        "Intro text here.\n\n- one\n- two",
        &[],
        &plain(),
    );
    assert_eq!(failed(r), Check::NewBlockMarker);
    let r = try_edit(
        "Intro text here.\n\nSecond paragraph.\n",
        "Intro text here.",
        "- Intro text here.",
        &[],
        &plain(),
    );
    assert_eq!(failed(r), Check::BlockType);
}

#[test]
fn rejects_fence() {
    // A fenced block is protected bytes, not a block, so the protected-bytes check catches it.
    let r = try_edit(
        "Intro text here.\n\nSecond paragraph.\n",
        "Intro text here.",
        "Intro text here.\n\n```\ncode\n```",
        &[],
        &plain(),
    );
    assert_eq!(failed(r), Check::Protected);
}

#[test]
fn rejects_edit_into_protected_or_frozen() {
    let src = "Run `cargo build` now.\n\nSecond paragraph here.\n";
    // Into inline code.
    let r = try_edit(src, "`cargo build`", "`make`", &[], &plain());
    assert_eq!(failed(r), Check::Protected);
    // Into a frozen range of plain prose.
    let start = src.find("Second paragraph here.").unwrap();
    let range = start..start + 6;
    let frozen = std::slice::from_ref(&range);
    let r = try_edit(
        src,
        "Second paragraph here.",
        "Another paragraph here.",
        frozen,
        &plain(),
    );
    assert_eq!(failed(r), Check::Protected);
    // The same edit with nothing frozen passes.
    try_edit(
        src,
        "Second paragraph here.",
        "Another paragraph here.",
        &[],
        &plain(),
    )
    .unwrap();
    // A range outside the document is rejected, not a panic.
    let d = parse(src, &plain());
    let p = Proposal {
        range: 0..src.len() + 5,
        replacement: String::new(),
        origin: ProposalOrigin::Model,
    };
    assert_eq!(
        verify::edit(&d, p, &[], &plain()).unwrap_err().failed,
        Check::Protected
    );
}

#[test]
fn accepts_reordered_placeholders_restored() {
    let src = "It costs 5 dollars and 10 cents, see 2024-01-05.\n";
    let needle = "It costs 5 dollars and 10 cents, see 2024-01-05.";
    let d = parse(src, &plain());
    let p = Proposal {
        range: 0..needle.len(),
        replacement: "See 2024-01-05: 10 cents and 5 dollars.".into(),
        origin: ProposalOrigin::Model,
    };
    let v = verify::edit(&d, p, &[], &plain()).unwrap();
    assert_eq!(
        v.get().replacement,
        "See 2024-01-05: 10 cents and 5 dollars."
    );
}

#[test]
fn improvement_requires_strictly_fewer() {
    assert!(verify::improvement(3, 2, 0..10).is_ok());
    assert!(verify::improvement(3, 0, 0..10).is_ok());
    // The equal-score fixture (DQ-3).
    let eq = verify::improvement(3, 3, 4..9).unwrap_err();
    assert_eq!(eq.failed, Check::NoImprovement);
    assert_eq!(eq.range, 4..9);
    assert_eq!(eq.detail, "weighted findings 3 -> 3");
    let worse = verify::improvement(3, 4, 0..10).unwrap_err();
    assert_eq!(worse.failed, Check::NoImprovement);
    assert!(verify::improvement(0, 0, 0..1).is_err());
}

#[test]
fn document_check_fires_on_forged_edit_pair() {
    let p = plain();
    let before = parse("Intro.\n\nSecond paragraph.\n", &p);
    // Equal documents pass.
    verify::document(&before, &parse("Intro.\n\nSecond paragraph.\n", &p)).unwrap();
    // Prose-only changes pass.
    verify::document(&before, &parse("Hello.\n\nOther words.\n", &p)).unwrap();
    // A forged pair: the block structure differs.
    let e = verify::document(&before, &parse("Intro.\n\n# Second paragraph.\n", &p)).unwrap_err();
    assert_eq!(e.failed, Check::Document);
    assert_eq!(e.range, 0..before.src.len());
    // A forged pair: the protected bytes differ.
    let code = parse("Run `a` now.\n", &p);
    let e = verify::document(&code, &parse("Run `b` now.\n", &p)).unwrap_err();
    assert_eq!(e.failed, Check::Document);
}
