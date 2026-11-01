//! Rule core (SPEC §9): `detect` filtering and the `WordList` helper, with test-only rules (no built-in exists yet).
#![allow(clippy::unwrap_used, clippy::expect_used)] // test helpers may unwrap (architecture §8)

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use sloperator::directives::Suppressions;
use sloperator::doc::{self, Document, Format, ProtectSettings};
use sloperator::rules::helpers::WordList;
use sloperator::rules::{
    Enabled, Family, Finding, FixClass, Rule, RuleCode, RuleCtx, RuleMeta, Severity, detect,
};

/// Reports every `foo` in the raw source, prose or not, so `detect` is the only thing keeping it out of code.
struct FooEverywhere {
    meta: RuleMeta,
}

impl FooEverywhere {
    fn new(code: u16) -> Self {
        FooEverywhere {
            meta: RuleMeta {
                code: RuleCode(code),
                name: "foo-everywhere",
                family: Family::AiVocabulary,
                severity: Severity::Warning,
                default_on: false,
                fix: FixClass::None,
                explain: "test rule",
                example: ("foo", "bar"),
                params: &[],
                since: "0.1.0",
            },
        }
    }
}

impl Rule for FooEverywhere {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        for (i, _) in ctx.doc.src.match_indices("foo") {
            out.push(Finding {
                code: self.meta.code,
                range: i..i + 3,
                message: "foo".to_string(),
                severity: ctx.severity,
                fix: None,
            });
        }
    }
}

fn md(src: &str) -> Document {
    doc::parse(
        Arc::from(src),
        Format::Markdown,
        &ProtectSettings::default(),
    )
}

fn enabled(rule: &dyn Rule) -> Enabled<'_> {
    Enabled {
        rule,
        severity: rule.meta().severity,
        params: BTreeMap::new(),
    }
}

#[test]
fn detect_skips_protected_and_suppressed() {
    // line 1: prose foo + code foo; line 3: foo on a suppressed line; line 5: prose foo.
    let src = "foo `foo`\n\nfoo\n\nfoo\n";
    let d = md(src);
    let line3 = d.line_range(3);
    let sup = Suppressions::default().with_lines(3..4, None, line3);
    let rule = FooEverywhere::new(1);
    let found = detect(&d, &sup, &[enabled(&rule)]);
    let starts: Vec<usize> = found.iter().map(|f| f.range.start).collect();
    assert_eq!(
        starts,
        vec![0, src.rfind("foo").unwrap()],
        "only the prose foo on lines 1 and 5 survive"
    );
}

#[test]
fn detect_suppresses_only_named_codes_and_never_system_codes() {
    let src = "foo\n";
    let d = md(src);
    let named: BTreeSet<RuleCode> = [RuleCode(1)].into();
    let sup = Suppressions::default().with_lines(1..2, Some(named), d.line_range(1));
    let one = FooEverywhere::new(1);
    let two = FooEverywhere::new(2);
    let sys = FooEverywhere::new(901);
    let found = detect(&d, &sup, &[enabled(&one), enabled(&two), enabled(&sys)]);
    let codes: Vec<RuleCode> = found.iter().map(|f| f.code).collect();
    assert_eq!(codes, vec![RuleCode(2), RuleCode(901)]);

    let all = Suppressions::default().with_file_ignored();
    let codes: Vec<RuleCode> = detect(&d, &all, &[enabled(&one), enabled(&sys)])
        .iter()
        .map(|f| f.code)
        .collect();
    assert_eq!(
        codes,
        vec![RuleCode(901)],
        "ignore-file never hides a system code"
    );
}

#[test]
fn detect_sorts_by_start_then_code() {
    let d = md("foo foo\n");
    let a = FooEverywhere::new(7);
    let b = FooEverywhere::new(3);
    let found = detect(&d, &Suppressions::default(), &[enabled(&a), enabled(&b)]);
    let keys: Vec<(usize, u16)> = found.iter().map(|f| (f.range.start, f.code.0)).collect();
    assert_eq!(keys, vec![(0, 3), (0, 7), (4, 3), (4, 7)]);
}

static ENTRIES: &[(&str, Option<&str>)] = &[
    ("utilize", Some("use")),
    ("moreover", None),
    ("in order to", Some("to")),
];

fn find_in(src: &str) -> Vec<(String, Option<String>)> {
    let d = md(src);
    let list = WordList::new(ENTRIES);
    d.blocks
        .iter()
        .flat_map(|b| {
            let bt = d.block_text(b.id);
            list.find(&bt)
                .into_iter()
                .map(|(r, fix)| (d.src[r].to_string(), fix))
                .collect::<Vec<_>>()
        })
        .collect()
}

#[test]
fn wordlist_none_entry_has_no_fix() {
    let found = find_in("Moreover, we utilize it.\n");
    assert_eq!(
        found,
        vec![
            ("Moreover".to_string(), None),
            ("utilize".to_string(), Some("use".to_string())),
        ]
    );
}

#[test]
fn wordlist_matches_whole_words_and_preserves_case() {
    let found = find_in("Utilize it, UTILIZE it, utilized it, In order to win.\n");
    assert_eq!(
        found,
        vec![
            ("Utilize".to_string(), Some("Use".to_string())),
            ("UTILIZE".to_string(), Some("USE".to_string())),
            ("In order to".to_string(), Some("To".to_string())),
        ]
    );
}

#[test]
fn wordlist_skips_matches_inside_code() {
    assert!(find_in("a `utilize` b\n").is_empty());
}
