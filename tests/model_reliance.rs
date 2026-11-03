//! FR-10 (classical-nlp SPEC §2): what a rules-only fix leaves for the model, per 1,000 prose words, over
//! `eval/paragraphs`, compared with the baseline recorded in `eval/model-reliance.md`. No model, no network.
#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may unwrap (architecture §8)

use std::path::PathBuf;
use std::sync::Arc;

use sloperator::config::{CliOverrides, Resolver};
use sloperator::engine::DEFAULT_SEED;
use sloperator::input::{Format, Origin, Source};
use sloperator::pipeline::{self, Engines};
use sloperator::rules::FixClass;
use sloperator::rules::registry::Registry;

const RULES_ONLY: Engines = Engines {
    rules: true,
    model: false,
};

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Residual model-eligible findings per 1,000 prose words, rounded to two decimals. Each paragraph is analysed as
/// a file in a temp dir, so no `sloperator.toml` of the checkout applies (as `tests/pipeline.rs:25-37` does).
fn residual_per_1000() -> f64 {
    let tmp = tempfile::tempdir().unwrap();
    let (mut residual, mut words) = (0u64, 0u64);
    for n in 1..=20 {
        let name = format!("p{n:02}.md");
        let text = std::fs::read_to_string(manifest().join("eval/paragraphs").join(&name)).unwrap();
        let path = tmp.path().join(&name);
        let settings = Resolver::new(CliOverrides::default(), None)
            .unwrap()
            .for_file(&path)
            .unwrap();
        let source = Source {
            origin: Origin::File(path),
            format: Format::Markdown,
            text: Arc::from(text),
            settings,
        };
        let r = pipeline::fix(
            pipeline::analyse(source).unwrap(),
            RULES_ONLY,
            None,
            DEFAULT_SEED,
        )
        .unwrap();
        words += r.score.prose_words as u64;
        residual += r
            .findings
            .iter()
            .filter(|l| !l.finding.code.is_system())
            .filter(|l| {
                match Registry::builtin().get(l.finding.code).map(|x| x.meta().fix) {
                    Some(FixClass::Model) => true,
                    Some(FixClass::Safe) => l.finding.fix.is_none(),
                    Some(FixClass::None) | None => false,
                }
            })
            .count() as u64;
    }
    (100_000.0 * residual as f64 / words.max(1) as f64).round() / 100.0
}

/// The value of the `| baseline |` row of `eval/model-reliance.md`.
fn baseline() -> f64 {
    let text = std::fs::read_to_string(manifest().join("eval/model-reliance.md")).unwrap();
    let row = text
        .lines()
        .find(|l| l.starts_with("| baseline |"))
        .expect("no `| baseline |` row");
    row.split('|')
        .nth(2)
        .unwrap()
        .trim()
        .parse()
        .expect("baseline value is a number")
}

/// FR-10.3: never above the baseline. The current value is printed so each phase records it (FR-10.2).
#[test]
fn residual_does_not_exceed_baseline() {
    let (now, base) = (residual_per_1000(), baseline());
    println!("model reliance: {now:.2} per 1,000 prose words (baseline {base:.2})");
    assert!(
        now <= base,
        "residual {now:.2} per 1,000 words is above the baseline {base:.2}"
    );
}
