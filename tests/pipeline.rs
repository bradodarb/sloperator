//! The pipeline with the rules engine (SPEC §17, §23.13): lint is model-free, --rules-only never loads the model, a
//! rejected safe fix is SLP901 with the original kept, fix is idempotent, and Fix sets both scores.
#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may unwrap (architecture §8)

mod common;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use common::{FakeFactory, TempModelCache};
use sloperator::config::{CliOverrides, Resolver};
use sloperator::engine::{DEFAULT_SEED, LazyRewriter};
use sloperator::input::{Format, Origin, Source};
use sloperator::model;
use sloperator::pipeline::{self, Engines, FileResult};
use sloperator::rules::{RuleCode, Severity};
use sloperator::score;

const RULES_ONLY: Engines = Engines {
    rules: true,
    model: false,
};

/// A source at `name` under a temp dir (no `sloperator.toml` above it), with the built-in default settings.
fn source(dir: &Path, text: &str) -> Source {
    let path: PathBuf = dir.join("doc.md");
    let settings = Resolver::new(CliOverrides::default(), None)
        .unwrap()
        .for_file(&path)
        .unwrap();
    Source {
        origin: Origin::File(path),
        format: Format::Markdown,
        text: Arc::from(text),
        settings,
    }
}

fn fix_rules_only(dir: &Path, text: &str) -> FileResult {
    let a = pipeline::analyse(source(dir, text)).unwrap();
    pipeline::fix(a, RULES_ONLY, None, DEFAULT_SEED).unwrap()
}

#[test]
fn lint_never_constructs_rewriter() {
    // Structural: `lint` takes no rewriter, and its body names nothing from the model port.
    let src =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/pipeline/mod.rs"))
            .unwrap();
    let start = src.find("pub fn lint(").expect("lint is defined");
    let body = &src[start..start + src[start..].find("\n}\n").expect("lint body ends")];
    for banned in ["Rewriter", "rewrite", "engine", "rw", "verify::"] {
        assert!(!body.contains(banned), "lint mentions `{banned}`:\n{body}");
    }

    // Behavioural: a lint run with a model available opens and preflights nothing, and reports without fixing.
    let factory = FakeFactory::default();
    let tmp = tempfile::tempdir().unwrap();
    let r = pipeline::lint(
        pipeline::analyse(source(
            tmp.path(),
            "We utilize the cache in order to look things up.\n",
        ))
        .unwrap(),
    );
    assert_eq!((factory.preflights.get(), factory.opens.get()), (0, 0));
    assert!(r.fixed.is_none());
    assert!(
        r.findings.iter().any(|l| l.finding.code == RuleCode(1)),
        "{:?}",
        r.findings
    );
}

#[test]
fn rules_only_never_calls_model() {
    let cache = TempModelCache::new();
    cache.plant(model::spec(model::DEFAULT_MODEL).unwrap());
    let factory = FakeFactory::default();
    let mut lazy = LazyRewriter::new(&factory, &cache.settings()).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let a = pipeline::analyse(source(
        tmp.path(),
        "We utilize the cache in order to look things up.\n",
    ))
    .unwrap();

    let r = pipeline::fix(a, RULES_ONLY, Some(&mut lazy), DEFAULT_SEED).unwrap();

    assert_eq!(lazy.loads, 0);
    assert_eq!(factory.opens.get(), 0);
    assert_eq!(
        r.fixed.as_deref(),
        Some("We use the cache to look things up.\n")
    );
}

#[test]
fn second_fix_changes_nothing_rules_engine() {
    let text = std::fs::read_to_string(common::fixture("rules/SLP001/positive.md")).unwrap();
    let tmp = tempfile::tempdir().unwrap();

    let first = fix_rules_only(tmp.path(), &text);
    let once = first.fixed.expect("the fixture has safe fixes");
    assert_ne!(once, text);

    let second = fix_rules_only(tmp.path(), &once);
    assert_eq!(second.fixed, None, "a second fix changed the text");
    assert!(
        second.findings.iter().all(|l| l.finding.fix.is_none()),
        "a safe fix is still pending: {:?}",
        second.findings
    );
}

#[test]
fn rejected_safe_fix_is_slp901_original_kept() {
    let text = "We utilize 42 servers here.\n";
    let tmp = tempfile::tempdir().unwrap();
    let mut a = pipeline::analyse(source(tmp.path(), text)).unwrap();
    // Forge SLP001's safe fix so that it swallows the number.
    let f = a
        .findings
        .iter_mut()
        .find(|f| f.code == RuleCode(1))
        .expect("SLP001 fires on `utilize`");
    let p = f.fix.as_mut().expect("`utilize` has a safe fix");
    let start = p.range.start;
    p.range = start..start + "utilize 42".len();
    p.replacement = "use".to_string();

    let r = pipeline::fix(a, RULES_ONLY, None, DEFAULT_SEED).unwrap();

    assert_eq!(r.fixed, None, "the original text must be kept");
    assert_eq!(&*r.original, text);
    let rej: Vec<_> = r
        .findings
        .iter()
        .filter(|l| l.finding.code == RuleCode::REWRITE_REJECTED)
        .collect();
    assert_eq!(rej.len(), 1, "{:?}", r.findings);
    assert_eq!(rej[0].finding.severity, Severity::Warning);
    assert!(
        rej[0]
            .finding
            .message
            .starts_with("rewrite rejected (numbers):"),
        "{}",
        rej[0].finding.message
    );
    assert_eq!(rej[0].matched, "utilize 42");
    assert_eq!(rej[0].start, (1, 4));
    // The finding the fix belonged to is still reported against the kept text.
    assert!(r.findings.iter().any(|l| l.finding.code == RuleCode(1)));
}

#[test]
fn fix_sets_score_and_score_before() {
    let text = std::fs::read_to_string(common::fixture("rules/SLP001/positive.md")).unwrap();
    let tmp = tempfile::tempdir().unwrap();

    let a = pipeline::analyse(source(tmp.path(), &text)).unwrap();
    let min = a.source.settings.min_severity;
    let reported: Vec<_> = a
        .findings
        .iter()
        .filter(|f| f.severity >= min)
        .cloned()
        .collect();
    let expected_before = score::score(&reported, a.doc.prose_words());
    let r = pipeline::fix(a, RULES_ONLY, None, DEFAULT_SEED).unwrap();

    let fixed = r.fixed.clone().expect("the fixture has safe fixes");
    let after = pipeline::analyse(source(tmp.path(), &fixed)).unwrap();
    let reported_after: Vec<_> = after
        .findings
        .iter()
        .filter(|f| f.severity >= min)
        .cloned()
        .collect();
    assert_eq!(r.score_before, Some(expected_before));
    assert_eq!(
        r.score,
        score::score(&reported_after, after.doc.prose_words())
    );
    assert!(
        r.score.value < expected_before.value,
        "{} !< {}",
        r.score.value,
        expected_before.value
    );

    let lint = pipeline::lint(pipeline::analyse(source(tmp.path(), &text)).unwrap());
    assert_eq!(lint.score_before, None);
    assert_eq!(lint.score, expected_before);
}

const BOTH: Engines = Engines {
    rules: true,
    model: true,
};

const MODEL_ONLY: Engines = Engines {
    rules: false,
    model: true,
};

/// `pipeline::fix` over `text` with `engines`, through a LazyRewriter over `factory` with a planted model.
fn fix_with_model(
    text: &str,
    engines: Engines,
    factory: &FakeFactory,
    overrides: CliOverrides,
) -> FileResult {
    let cache = TempModelCache::new();
    cache.plant(model::spec(model::DEFAULT_MODEL).unwrap());
    let mut lazy = LazyRewriter::new(factory, &cache.settings()).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("doc.md");
    let settings = Resolver::new(overrides, None)
        .unwrap()
        .for_file(&path)
        .unwrap();
    let a = pipeline::analyse(Source {
        origin: Origin::File(path),
        format: Format::Markdown,
        text: Arc::from(text),
        settings,
    })
    .unwrap();
    pipeline::fix(a, engines, Some(&mut lazy), DEFAULT_SEED).unwrap()
}

#[test]
fn default_runs_rules_then_model_on_remaining() {
    // SLP001 (safe) and SLP007 (model) in one paragraph: the model sees the paragraph after the safe fix.
    let factory = FakeFactory::returning("We use the cache. It speeds up the build.");
    let r = fix_with_model(
        "We utilize the cache. It plays a crucial role in the build.\n",
        BOTH,
        &factory,
        CliOverrides::default(),
    );
    let calls = factory.calls.borrow();
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls[0].0,
        "We use the cache. It plays a crucial role in the build."
    );
    assert_eq!(
        r.fixed.as_deref(),
        Some("We use the cache. It speeds up the build.\n")
    );
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn model_only_skips_safe_fixes() {
    let factory = FakeFactory::returning("It speeds up the build.");
    let r = fix_with_model(
        "We utilize the cache.\n\nIt plays a crucial role in the build.\n",
        MODEL_ONLY,
        &factory,
        CliOverrides::default(),
    );
    // The safe-fix paragraph never reaches the model (its finding carries a fix) and is not fixed either.
    assert_eq!(factory.calls.borrow().len(), 1);
    assert_eq!(
        r.fixed.as_deref(),
        Some("We utilize the cache.\n\nIt speeds up the build.\n")
    );
    assert!(r.findings.iter().any(|l| l.finding.code == RuleCode(1)));
}

#[test]
fn rejection_is_slp901_warning_counts_for_exit() {
    // SLP015 is an error, so its window is cut under --min-severity error; the rewrite drops the number.
    let factory = FakeFactory::returning("I count apples.");
    let r = fix_with_model(
        "As an AI language model, I count 3 apples.\n",
        MODEL_ONLY,
        &factory,
        CliOverrides {
            min_severity: Some(Severity::Error),
            ..CliOverrides::default()
        },
    );
    assert_eq!(r.fixed, None);
    let rej: Vec<_> = r
        .findings
        .iter()
        .filter(|l| l.finding.code == RuleCode::REWRITE_REJECTED)
        .collect();
    assert_eq!(rej.len(), 1, "{:?}", r.findings);
    assert_eq!(rej[0].finding.severity, Severity::Warning);
    assert!(rej[0].finding.message.contains("(numbers)"));
    // A warning passes --min-severity error because it is a system code, so the file has findings and exits 1.
    assert!(!r.findings.is_empty());
    // SLP901 is not scored (DQ-4): the score is that of the kept text's SLP015 alone.
    assert_eq!(r.score.weighted, r.score_before.as_ref().unwrap().weighted);
}

#[test]
fn lint_reports_counts_and_resolved_floors() {
    let tmp = tempfile::tempdir().unwrap();
    let mut src = source(tmp.path(), "# Heading\n\nCat cat. Dog!\n");
    Arc::make_mut(&mut src.settings).metrics = sloperator::text::Floors {
        sentences: 10,
        words: 200,
    };
    let a = pipeline::analyse(src).unwrap();
    let expected = a.prose.counts(&a.doc).clone();
    let r = pipeline::lint(a);
    assert_eq!(r.metrics, expected);
    assert_eq!(r.metrics.words, 3);
    assert_eq!(r.metrics.sentences, 2);
    assert_eq!(r.metrics_before, None);
    assert_eq!(
        r.floors,
        sloperator::text::Floors {
            sentences: 10,
            words: 200
        }
    );
}

#[test]
fn whole_document_rejection_keeps_original_metrics() {
    let tmp = tempfile::tempdir().unwrap();
    let mut a = pipeline::analyse(source(tmp.path(), "utilize cat utilize dog.")).unwrap();
    let expected = a.prose.counts(&a.doc).clone();
    // Individually unmatched backticks pass per-edit checks; together they protect inline code.
    let mut proposals = 0;
    for f in &mut a.findings {
        if f.code == RuleCode(1) {
            f.fix.as_mut().unwrap().replacement = "`".into();
            proposals += 1;
        }
    }
    assert_eq!(proposals, 2);
    let r = pipeline::fix(a, RULES_ONLY, None, DEFAULT_SEED).unwrap();
    assert!(r.findings.iter().any(|l| {
        l.finding
            .message
            .starts_with("rewrite rejected (document):")
    }));
    assert_eq!(r.fixed, None);
    assert_eq!(r.metrics_before, Some(expected.clone()));
    assert_eq!(r.metrics, expected);
}
