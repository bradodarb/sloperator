//! Rule fixtures (SPEC §10, FR-4.6): every built-in rule has `positive.md`, `negative.md` and, if safe, `fixed.md`.
//!
//! `rule_fixtures!` takes each rule's code and its test names. `macro_rules!` cannot join `slp` and a number into an
//! identifier and the crate takes no `paste` dependency, so the names are passed in. The macro checks the rule's
//! fix class against the arm it is listed under, so a rule filed under the wrong arm fails.
#![allow(clippy::unwrap_used, clippy::expect_used)] // test helpers may unwrap (architecture §8)

mod common;

use std::path::PathBuf;
use std::sync::Arc;

use common::fixture;
use sloperator::directives::Suppressions;
use sloperator::doc::{self, Document, Format, ProtectSettings};
use sloperator::edit::{self, Proposal};
use sloperator::rules::registry::Registry;
use sloperator::rules::{Enabled, Finding, FixClass, RuleCode, detect};
use sloperator::verify::{self, Verified};

fn dir(code: u16) -> PathBuf {
    fixture(&format!("rules/SLP{code:03}"))
}

fn read(code: u16, name: &str) -> String {
    let path = dir(code).join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn parse(src: &str) -> Document {
    doc::parse(
        Arc::from(src),
        Format::Markdown,
        &ProtectSettings::default(),
    )
}

/// What `detect` reports for only rule `code`, with no suppression.
fn findings(code: u16, d: &Document) -> Vec<Finding> {
    let rule = Registry::builtin()
        .get(RuleCode(code))
        .unwrap_or_else(|| panic!("SLP{code:03} is not registered"));
    let enabled = Enabled {
        rule,
        severity: rule.meta().severity,
        params: rule
            .meta()
            .params
            .iter()
            .map(|p| (p.name, p.default))
            .collect(),
    };
    detect(d, &Suppressions::default(), &[enabled])
}

fn fix_class(code: u16) -> FixClass {
    Registry::builtin().get(RuleCode(code)).unwrap().meta().fix
}

/// Applies every fix of rule `code` through the real path: plan, verify gate, apply.
fn apply_fixes(code: u16, src: &str) -> String {
    let d = parse(src);
    let props: Vec<Proposal> = findings(code, &d)
        .into_iter()
        .filter_map(|f| f.fix)
        .collect();
    let plan = edit::plan(props, &[]);
    assert!(plan.dropped.is_empty(), "fixes overlap: {:?}", plan.dropped);
    let protect = ProtectSettings::default();
    let verified: Vec<Verified<Proposal>> = plan
        .kept
        .into_iter()
        .map(|p| verify::edit(&d, p, &[], &protect).unwrap())
        .collect();
    edit::apply(src, &verified, d.newline)
}

fn check_positive(code: u16) {
    let src = read(code, "positive.md");
    let found = findings(code, &parse(&src));
    assert!(!found.is_empty(), "SLP{code:03} is silent on positive.md");
    assert!(found.iter().all(|f| f.code == RuleCode(code)));
}

/// Silent on `negative.md`, which must also hold the trigger in inline code, a fence and a link destination (FR-4.6).
fn check_negative(code: u16) {
    let src = read(code, "negative.md");
    assert!(src.contains("```"), "negative.md has no fenced block");
    assert!(
        src.contains("`") && src.contains("](http"),
        "negative.md has no inline code or link"
    );
    let found = findings(code, &parse(&src));
    assert!(
        found.is_empty(),
        "SLP{code:03} fires on negative.md: {found:?}"
    );
}

fn check_fix(code: u16) {
    assert_eq!(fix_class(code), FixClass::Safe, "SLP{code:03} is not safe");
    let fixed = apply_fixes(code, &read(code, "positive.md"));
    assert_eq!(fixed, read(code, "fixed.md"), "SLP{code:03} fixes differ");
    let again = findings(code, &parse(&fixed));
    assert!(
        again.iter().all(|f| f.fix.is_none()),
        "fixed.md still has fixable findings"
    );
}

macro_rules! rule_fixtures {
    ($($class:ident($code:literal, $($name:ident),+)),+ $(,)?) => {
        $(rule_fixtures!(@one $class $code, $($name),+);)+
    };
    (@one safe $code:literal, $pos:ident, $neg:ident, $fix:ident) => {
        #[test]
        fn $pos() { check_positive($code); }
        #[test]
        fn $neg() { check_negative($code); }
        #[test]
        fn $fix() { check_fix($code); }
    };
    (@one model $code:literal, $pos:ident, $neg:ident) => {
        #[test]
        fn $pos() {
            assert_ne!(fix_class($code), FixClass::Safe, "SLP{:03} is safe", $code);
            check_positive($code);
            assert!(findings($code, &parse(&read($code, "positive.md"))).iter().all(|f| f.fix.is_none()));
        }
        #[test]
        fn $neg() { check_negative($code); }
    };
}

rule_fixtures!(
    safe(
        1,
        slp001_positive_fires,
        slp001_negative_silent,
        slp001_fix_matches
    ),
    model(2, slp002_positive_fires, slp002_negative_silent),
    model(3, slp003_positive_fires, slp003_negative_silent),
    model(4, slp004_positive_fires, slp004_negative_silent),
    model(5, slp005_positive_fires, slp005_negative_silent),
    safe(
        6,
        slp006_positive_fires,
        slp006_negative_silent,
        slp006_fix_matches
    ),
    model(7, slp007_positive_fires, slp007_negative_silent),
    safe(
        8,
        slp008_positive_fires,
        slp008_negative_silent,
        slp008_fix_matches
    ),
    safe(
        9,
        slp009_positive_fires,
        slp009_negative_silent,
        slp009_fix_matches
    ),
    model(10, slp010_positive_fires, slp010_negative_silent),
    model(11, slp011_positive_fires, slp011_negative_silent),
    safe(
        12,
        slp012_positive_fires,
        slp012_negative_silent,
        slp012_fix_matches
    ),
    safe(
        13,
        slp013_positive_fires,
        slp013_negative_silent,
        slp013_fix_matches
    ),
    safe(
        14,
        slp014_positive_fires,
        slp014_negative_silent,
        slp014_fix_matches
    ),
    model(15, slp015_positive_fires, slp015_negative_silent),
    safe(
        16,
        slp016_positive_fires,
        slp016_negative_silent,
        slp016_fix_matches
    ),
);

#[test]
fn every_rule_has_fixtures() {
    for rule in Registry::builtin().all() {
        let m = rule.meta();
        let d = dir(m.code.0);
        assert!(d.is_dir(), "{} has no fixture directory", m.code);
        assert!(
            d.join("positive.md").is_file(),
            "{} has no positive.md",
            m.code
        );
        assert!(
            d.join("negative.md").is_file(),
            "{} has no negative.md",
            m.code
        );
        let has_fixed = d.join("fixed.md").is_file();
        match m.fix {
            FixClass::Safe => assert!(has_fixed, "{} is safe and has no fixed.md", m.code),
            _ => assert!(!has_fixed, "{} is not safe and has a fixed.md", m.code),
        }
    }
}

fn one_finding(code: u16, src: &str) -> (Document, Finding) {
    let d = parse(src);
    let mut found = findings(code, &d);
    assert_eq!(found.len(), 1, "{found:?}");
    (d, found.remove(0))
}

#[test]
fn slp001_bare_leverage_has_no_fix() {
    let (d, f) = one_finding(1, "We have leverage over suppliers.\n");
    assert_eq!(&d.src[f.range], "leverage");
    assert!(f.fix.is_none());
}

#[test]
fn slp001_verb_inflection_is_fixed() {
    let src = "She leveraged the API.\n";
    assert_eq!(apply_fixes(1, src), "She used the API.\n");
}

#[test]
fn slp001_keeps_case() {
    assert_eq!(
        apply_fixes(1, "Utilize it. UTILIZE it.\n"),
        "Use it. USE it.\n"
    );
}

#[test]
fn slp008_no_fix_when_nothing_follows_the_preamble() {
    let (d, f) = one_finding(8, "It is worth noting that (the cache) is cold.\n");
    assert_eq!(&d.src[f.range], "It is worth noting that");
    assert!(f.fix.is_none());
}

#[test]
fn slp008_needs_a_sentence_start() {
    let src = "See `x` it is worth noting that y.\n";
    assert!(findings(8, &parse(src)).is_empty());
}

#[test]
fn slp008_fires_after_heading_markup() {
    let src = "# It is worth noting that the cache is cold\n";
    assert_eq!(apply_fixes(8, src), "# The cache is cold\n".to_string());
}

#[test]
fn slp009_keeps_the_modal_case() {
    assert_eq!(
        apply_fixes(9, "It May Potentially fail.\n"),
        "It May fail.\n"
    );
}

#[test]
fn slp016_only_the_last_sentence() {
    let src = "In summary, it works. Overall, it ships.\n";
    let (d, f) = one_finding(16, src);
    assert_eq!(&d.src[f.range], "Overall,");
    assert_eq!(apply_fixes(16, src), "In summary, it works. It ships.\n");
}

#[test]
fn slp016_sentence_end_inside_a_quote_still_breaks() {
    let src = "Overall, he said \"stop.\" Then it ended.\n";
    assert!(findings(16, &parse(src)).is_empty());
}

#[test]
fn slp016_fixes_across_a_link_without_touching_it() {
    let src = "Overall, [the guide](https://example.com/a) wins.\n";
    let (d, f) = one_finding(16, src);
    assert_eq!(&d.src[f.range], "Overall,");
    assert!(f.fix.is_none(), "an OBJ after the lead-in gives no fix");
}

/// The exact SLP001/SLP006 context pairs of SPEC §10 (Gaps 2, 14, 16): each fix is grammatical, and every other
/// context fires with no fix and stays unchanged.
#[test]
fn safe_fix_contexts_grammatical() {
    let fixed = [
        (
            6,
            "These files can serve as the templates.\n",
            "These files can be the templates.\n",
        ),
        (
            6,
            "It is designed to serve as a guide.\n",
            "It is designed to be a guide.\n",
        ),
        (6, "It serves as a bridge.\n", "It is a bridge.\n"),
        (
            6,
            "A cache, which functions as a buffer, is small.\n",
            "A cache, which is a buffer, is small.\n",
        ),
        (
            6,
            "It can also serve as a guide.\n",
            "It can also be a guide.\n",
        ),
        (1, "She leveraged the API.\n", "She used the API.\n"),
    ];
    for (code, src, want) in fixed {
        assert_eq!(apply_fixes(code, src), want, "{src:?}");
    }
    let unchanged = [
        (1, "We have leverage over suppliers.\n"),
        (6, "They served as mentors.\n"),
        (6, "Both serve as anchors.\n"),
        (6, "The tool serves as a bridge.\n"),
        (6, "Please export these functions as a module.\n"),
        (6, "We expose helper functions as a convenience.\n"),
        (6, "It functions as follows.\n"),
        (6, "It serves as well as the old one.\n"),
        (6, "This can serve as well as any.\n"),
        (6, "These files can serve as templates.\n"),
        (6, "Here it serves as\n"),
        (6, "It serves as `code` here.\n"),
        (
            6,
            "The tools can't serve as a bridge, so we stand as one.\n",
        ),
    ];
    for (code, src) in unchanged {
        let found = findings(code, &parse(src));
        assert!(!found.is_empty(), "no finding for {src:?}");
        assert!(found.iter().all(|f| f.fix.is_none()), "{src:?}: {found:?}");
        assert_eq!(apply_fixes(code, src), src);
    }
}

/// FR-4.3: one em dash in a long file is normal prose; above 3 per 1,000 words every dash is reported.
#[test]
fn em_dash_fires_on_density_not_single() {
    let filler = "The cache is cold after a restart so the first build is slow. ".repeat(40);
    let single = format!("{filler}\n\nThe build passed\u{2014}we deploy.\n");
    assert!(findings(14, &parse(&single)).is_empty());

    let dense = format!("{filler}\n\nA\u{2014}b. C\u{2014}d. E -- f. G\u{2014}h.\n");
    let words = parse(&dense).prose_words();
    assert!(4.0 * 1000.0 / words as f64 > 3.0, "{words} words");
    let d = parse(&dense);
    let found = findings(14, &d);
    assert_eq!(found.len(), 4, "{found:?}");
    let marks: Vec<&str> = found.iter().map(|f| &d.src[f.range.clone()]).collect();
    assert_eq!(marks, ["\u{2014}", "\u{2014}", "--", "\u{2014}"]);
    assert!(found.iter().all(|f| f.fix.is_some()));
}

#[test]
fn slp014_no_fix_unless_between_words() {
    let d = parse("\u{2014}Lead. Close \u{2014}. \u{2014} x \u{2014}\n");
    let found = findings(14, &d);
    assert_eq!(found.len(), 4, "{found:?}");
    assert!(found.iter().all(|f| f.fix.is_none()), "{found:?}");
}

#[test]
fn slp010_needs_min_items_and_short_items() {
    let four = "- a\n- b\n- c\n- d\n";
    assert!(findings(10, &parse(four)).is_empty());
    let long = "- one two three four five six seven\n".repeat(5);
    assert!(findings(10, &parse(&long)).is_empty());
    let (d, f) = one_finding(10, "- a\n- b\n- c\n- d\n- e\n");
    assert_eq!(&d.src[f.range], "a");
}

#[test]
fn slp011_counts_per_block() {
    let sparse = format!("{} **slow**.\n", "word ".repeat(60));
    assert!(findings(11, &parse(&sparse)).is_empty());
    let found = findings(11, &parse("A **b** c **d** e.\n"));
    assert_eq!(found.len(), 2);
    assert!(found.iter().all(|f| f.fix.is_none()));
}

/// The fixture files of every registered rule, as (path, text).
fn all_fixtures() -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    for rule in Registry::builtin().all() {
        for name in ["positive.md", "negative.md", "fixed.md"] {
            let path = dir(rule.meta().code.0).join(name);
            if path.is_file() {
                let text = std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
                out.push((path, text));
            }
        }
    }
    out
}

fn safe_codes() -> Vec<u16> {
    Registry::builtin()
        .all()
        .filter(|r| r.meta().fix == FixClass::Safe)
        .map(|r| r.meta().code.0)
        .collect()
}

/// FR-4.7: on every fixture, a model or detection-only rule never carries a fix; a safe rule fixes its positive.md.
#[test]
fn only_safe_rules_attach_fixes() {
    let fixtures = all_fixtures();
    for rule in Registry::builtin().all() {
        let m = rule.meta();
        for (path, text) in &fixtures {
            let found = findings(m.code.0, &parse(text));
            if m.fix != FixClass::Safe {
                assert!(
                    found.iter().all(|f| f.fix.is_none()),
                    "{} ({:?}) attached a fix on {}",
                    m.code,
                    m.fix,
                    path.display()
                );
            }
            for f in found.iter().filter_map(|f| f.fix.as_ref()) {
                assert_eq!(f.origin, sloperator::types::ProposalOrigin::Rule(m.code));
            }
        }
        if m.fix == FixClass::Safe {
            let positive = findings(m.code.0, &parse(&read(m.code.0, "positive.md")));
            assert!(
                positive.iter().any(|f| f.fix.is_some()),
                "{} is safe and attaches no fix on positive.md",
                m.code
            );
        }
    }
}

/// FR-8.9: on every fixture, each safe rule's fixes applied twice change nothing the second time.
#[test]
fn fix_is_idempotent_over_all_fixtures() {
    for (path, text) in all_fixtures() {
        for code in safe_codes() {
            let once = apply_fixes(code, &text);
            let twice = apply_fixes(code, &once);
            assert_eq!(
                twice,
                once,
                "SLP{code:03} is not idempotent on {}",
                path.display()
            );
        }
    }
}
