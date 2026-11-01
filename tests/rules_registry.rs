//! Rule registry (SPEC §9): what is and is not a registered rule, the catalogue and the default set.

use std::collections::{BTreeMap, BTreeSet};

use sloperator::rules::registry::Registry;
use sloperator::rules::{Family, FixClass, RuleCode, Severity};

#[test]
fn system_codes_not_in_registry() {
    let reg = Registry::builtin();
    assert!(reg.get(RuleCode::REWRITE_REJECTED).is_none());
    assert!(reg.get(RuleCode::WINDOW_SKIPPED).is_none());
    assert!(reg.all().all(|r| !r.meta().code.is_system()));
    assert!(reg.default_set().iter().all(|c| !c.is_system()));
}

/// FR-4.2: at least 15 rules, and every one of the nine families has a rule.
#[test]
fn at_least_15_rules_covering_nine_families() {
    let reg = Registry::builtin();
    let codes: BTreeSet<RuleCode> = reg.all().map(|r| r.meta().code).collect();
    assert_eq!(codes.len(), reg.all().count(), "duplicate rule codes");
    assert!(codes.len() >= 15, "{} rules", codes.len());
    assert_eq!(codes.len(), 16, "the v0.1.0 catalogue has 16 rules");
    let families = [
        Family::AiVocabulary,
        Family::NegativeParallelism,
        Family::RuleOfThree,
        Family::CopulaAvoidance,
        Family::SignificancePuffery,
        Family::Hedging,
        Family::Markup,
        Family::ChatbotResidue,
        Family::EmDash,
    ];
    for family in families {
        assert!(
            reg.all().any(|r| r.meta().family == family),
            "no rule in family {family:?}"
        );
    }
}

/// FR-4.1: every rule carries a code, a name, a severity, an on/off state and a fix class.
#[test]
fn every_rule_has_code_name_severity_state_fix() {
    use FixClass::{Model, None as NoFix, Safe};
    use Severity::{Error, Suggestion, Warning};
    // SPEC §10 built-in rule catalogue: (code, severity, fix class), copied literally (FR-4.1, FR-4.7).
    let catalogue: BTreeMap<u16, (Severity, FixClass)> = [
        (1, (Warning, Safe)),
        (2, (Suggestion, Model)),
        (3, (Warning, Model)),
        (4, (Warning, Model)),
        (5, (Suggestion, Model)),
        (6, (Warning, Safe)),
        (7, (Warning, Model)),
        (8, (Warning, Safe)),
        (9, (Suggestion, Safe)),
        (10, (Suggestion, NoFix)),
        (11, (Suggestion, NoFix)),
        (12, (Error, Safe)),
        (13, (Error, Safe)),
        (14, (Warning, Safe)),
        (15, (Error, Model)),
        (16, (Suggestion, Safe)),
    ]
    .into_iter()
    .collect();
    // FR-4.5: a rule's default_on state is exactly its membership in the default set.
    let default_set = Registry::builtin().default_set();
    for rule in Registry::builtin().all() {
        let m = rule.meta();
        assert!(
            (1..900).contains(&m.code.0),
            "{} is not a rule code",
            m.code
        );
        assert!(!m.name.is_empty(), "{} has no name", m.code);
        assert!(
            m.name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            "{} name {:?} is not kebab-case",
            m.code,
            m.name
        );
        assert!(!m.explain.is_empty(), "{} has no explanation", m.code);
        assert!(!m.since.is_empty(), "{} has no since", m.code);
        let (severity, fix) = catalogue.get(&m.code.0).unwrap_or_else(|| {
            panic!(
                "{} is registered but missing from the SPEC §10 table",
                m.code
            )
        });
        assert_eq!(m.severity, *severity, "{} severity", m.code);
        assert_eq!(m.fix, *fix, "{} fix class", m.code);
        assert_eq!(
            m.default_on,
            default_set.contains(&m.code),
            "{} default_on disagrees with the default set",
            m.code
        );
    }
    let registered: BTreeSet<u16> = Registry::builtin().all().map(|r| r.meta().code.0).collect();
    let listed: BTreeSet<u16> = catalogue.keys().copied().collect();
    assert_eq!(
        registered, listed,
        "registry codes and the SPEC §10 table differ"
    );
    let names: BTreeSet<&str> = Registry::builtin().all().map(|r| r.meta().name).collect();
    assert_eq!(
        names.len(),
        Registry::builtin().all().count(),
        "duplicate rule names"
    );
}

/// FR-4.5: the default set is compared with a literal list, so any change to the defaults shows up as a diff.
#[test]
fn default_set_is_explicit() {
    let want: BTreeSet<RuleCode> = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
        .into_iter()
        .map(RuleCode)
        .collect();
    assert_eq!(Registry::builtin().default_set(), want);
}

/// FR-4.5: a rule added after the first release starts off.
#[test]
fn new_rules_start_off() {
    let first = version("0.1.0");
    for rule in Registry::builtin().all() {
        let m = rule.meta();
        if version(m.since) > first {
            assert!(
                !m.default_on,
                "{} (since {}) is on by default",
                m.code, m.since
            );
        }
    }
}

/// `major.minor.patch` as a comparable tuple; a malformed version fails the test.
fn version(v: &str) -> (u32, u32, u32) {
    let parts: Vec<u32> = v
        .split('.')
        .map(|p| p.parse().unwrap_or_else(|_| panic!("bad version {v:?}")))
        .collect();
    match parts[..] {
        [a, b, c] => (a, b, c),
        _ => panic!("bad version {v:?}"),
    }
}
