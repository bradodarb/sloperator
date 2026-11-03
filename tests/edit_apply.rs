//! `edit`: plan, apply and unified diff (SPEC §23.4).
#![allow(clippy::unwrap_used, clippy::expect_used)] // test helpers may unwrap (architecture §8)

mod common;

use std::path::Path;
use std::sync::Arc;

use common::fixture;
use sloperator::doc::{self, Document, Format, ProtectSettings};
use sloperator::edit::{self, Proposal, ProposalOrigin};
use sloperator::types::RuleCode;
use sloperator::verify::{self, Verified};

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

fn parse(src: &str) -> Document {
    doc::parse(
        Arc::from(src),
        Format::Markdown,
        &ProtectSettings::default(),
    )
}

fn proposal(range: std::ops::Range<usize>, replacement: &str, origin: ProposalOrigin) -> Proposal {
    Proposal {
        range,
        replacement: replacement.to_owned(),
        origin,
    }
}

/// Replaces the first `needle` in `src` and runs the edit through the verify gate.
fn verified(d: &Document, needle: &str, replacement: &str) -> Verified<Proposal> {
    let start = d.src.find(needle).unwrap();
    let p = proposal(
        start..start + needle.len(),
        replacement,
        ProposalOrigin::Model,
    );
    verify::edit(d, p, &[], &ProtectSettings::default(), &[]).unwrap()
}

/// Verifies one edit and applies it with the document's own newline style.
fn replace(src: &str, needle: &str, replacement: &str) -> String {
    let d = parse(src);
    let v = verified(&d, needle, replacement);
    edit::apply(src, &[v], d.newline)
}

#[test]
fn noop_plan_round_trips_every_protect_fixture() {
    for name in FIXTURES {
        let path = fixture(&format!("protect/{name}.md"));
        let bytes = std::fs::read(&path).unwrap();
        let src = String::from_utf8(bytes).unwrap();
        let d = parse(&src);
        let plan = edit::plan(Vec::new(), &[]);
        assert!(plan.kept.is_empty() && plan.dropped.is_empty(), "{name}");
        let out = edit::apply(&src, &[], d.newline);
        assert_eq!(out.as_bytes(), src.as_bytes(), "{name} changed");
    }
}

#[test]
fn crlf_preserved_in_replacements() {
    let src = "Intro text here.\r\nMore text follows.\r\n\r\nNext paragraph.\r\n";
    let out = replace(src, "More text follows.", "Other words\nfollow now.");
    assert_eq!(
        out,
        "Intro text here.\r\nOther words\r\nfollow now.\r\n\r\nNext paragraph.\r\n"
    );
    assert!(!out.contains("\r\r"));
    assert_eq!(out.matches('\n').count(), out.matches("\r\n").count());
}

#[test]
fn lf_replacements_stay_lf() {
    let out = replace(
        "Intro text here.\nMore text.\n",
        "More text.",
        "Other\nwords.",
    );
    assert_eq!(out, "Intro text here.\nOther\nwords.\n");
}

#[test]
fn trailing_newline_preserved() {
    assert_eq!(
        replace("Alpha beta gamma.\n", "beta", "delta"),
        "Alpha delta gamma.\n"
    );
    assert_eq!(
        replace("Alpha beta gamma.", "beta", "delta"),
        "Alpha delta gamma."
    );
    assert_eq!(
        replace("Alpha beta gamma.\r\n", "beta", "delta"),
        "Alpha delta gamma.\r\n"
    );
}

fn model(range: std::ops::Range<usize>, text: &str) -> Proposal {
    proposal(range, text, ProposalOrigin::Model)
}

fn rule(code: u16, range: std::ops::Range<usize>, text: &str) -> Proposal {
    proposal(range, text, ProposalOrigin::Rule(RuleCode(code)))
}

#[test]
fn plan_drops_overlaps_deterministically() {
    let props = vec![
        model(0..10, "m"),
        rule(2, 5..15, "r2"),
        rule(1, 0..10, "r1"),
        rule(3, 20..30, "r3"),
        rule(4, 30..31, "r4"),
    ];
    let expected_kept = vec![
        rule(1, 0..10, "r1"),
        rule(3, 20..30, "r3"),
        rule(4, 30..31, "r4"),
    ];
    let expected_dropped = vec![model(0..10, "m"), rule(2, 5..15, "r2")];

    // Every rotation and the reversal of the input resolves the same way.
    let mut orders: Vec<Vec<Proposal>> = (0..props.len())
        .map(|i| {
            let mut v = props.clone();
            v.rotate_left(i);
            v
        })
        .collect();
    orders.push(props.iter().rev().cloned().collect());
    for order in orders {
        let plan = edit::plan(order, &[]);
        assert_eq!(plan.kept, expected_kept);
        let mut dropped = plan.dropped;
        dropped.sort_by_key(|p| (p.range.start, p.range.end));
        let mut want = expected_dropped.clone();
        want.sort_by_key(|p| (p.range.start, p.range.end));
        assert_eq!(dropped, want);
    }
}

#[test]
fn plan_earlier_start_wins_over_origin() {
    let plan = edit::plan(vec![rule(1, 5..15, "late"), model(0..10, "early")], &[]);
    assert_eq!(plan.kept, vec![model(0..10, "early")]);
    assert_eq!(plan.dropped, vec![rule(1, 5..15, "late")]);
}

#[test]
fn plan_drops_frozen() {
    let frozen = [10..20, 40..50];
    let plan = edit::plan(
        vec![
            rule(1, 0..10, "touches end only"),
            rule(1, 5..12, "overlaps start"),
            rule(1, 12..15, "inside"),
            rule(1, 18..25, "overlaps end"),
            rule(1, 20..25, "starts at end"),
            rule(1, 8..30, "spans it"),
            rule(1, 44..46, "in second range"),
        ],
        &frozen,
    );
    assert_eq!(
        plan.kept,
        vec![
            rule(1, 0..10, "touches end only"),
            rule(1, 20..25, "starts at end"),
        ]
    );
    assert_eq!(plan.dropped.len(), 5);
}

#[test]
fn plan_then_apply_splices_several_edits() {
    let src = "Alpha beta gamma delta.\n";
    let d = parse(src);
    let beta = src.find("beta").unwrap();
    let delta = src.find("delta").unwrap();
    let plan = edit::plan(
        vec![
            model(delta..delta + 5, "omega"),
            model(beta..beta + 4, "zeta"),
        ],
        &[],
    );
    let edits: Vec<Verified<Proposal>> = plan
        .kept
        .into_iter()
        .map(|p| verify::edit(&d, p, &[], &ProtectSettings::default(), &[]).unwrap())
        .collect();
    assert_eq!(
        edit::apply(src, &edits, d.newline),
        "Alpha zeta gamma omega.\n"
    );
}

#[test]
fn unified_diff_has_headers_and_hunks() {
    let before = "one\ntwo\nthree\n";
    let after = "one\n2\nthree\n";
    let diff = edit::unified_diff(Path::new("docs/a.md"), before, after);
    assert!(diff.contains("--- a/docs/a.md"), "{diff}");
    assert!(diff.contains("+++ b/docs/a.md"), "{diff}");
    assert!(diff.contains("@@"), "{diff}");
    assert!(diff.contains("-two"), "{diff}");
    assert!(diff.contains("+2"), "{diff}");
}

#[test]
fn unified_diff_of_equal_text_is_empty() {
    let diff = edit::unified_diff(Path::new("a.md"), "same\n", "same\n");
    assert!(!diff.contains("@@"), "{diff}");
}
