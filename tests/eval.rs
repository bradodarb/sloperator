//! The eval harness (SPEC §22, FR-10.2, NFR-3). Integration tier only: it runs `pipeline::fix` with both engines on
//! the twenty `eval/paragraphs`, once for every registry model in the cache, writes `target/eval/report.json`, prints
//! one table per model and asserts 20/20 integrity and the per-window latency budget. Without the default model it
//! fails naming `sloperator model pull` (architecture §11).
//!
//! Models run one after another in this one process: each model's rewriter is dropped before the next opens, so the
//! runtime's one-backend-at-a-time rule holds (`engine::runtime`).
#![cfg(feature = "model")]
#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may unwrap (architecture §8)

use std::cell::Cell;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use sloperator::Error;
use sloperator::config::{CliOverrides, ModelSettings, Resolver, Settings, WindowMode};
use sloperator::doc;
use sloperator::engine::runtime::RuntimeFactory;
use sloperator::engine::{self, LazyRewriter, RewriteRequest, Rewriter, RewriterFactory};
use sloperator::input::{Format, Origin, Source};
use sloperator::model::{self, ModelSpec, ResolvedModel};
use sloperator::pipeline::{self, Engines, FileResult};
use sloperator::rules::RuleCode;
use sloperator::verify::Check;

/// Mean seconds per model window, from the spike's Decision (NFR-3). Recorded inside the crate in `eval/README.md`;
/// `tests/eval_corpus.rs::budget_matches_eval_readme` keeps the two equal in the unit tier.
const MODEL_WINDOW_BUDGET_S: f64 = 3.0;

const PARAGRAPHS: usize = 20;

const BOTH: Engines = Engines {
    rules: true,
    model: true,
};
const RULES_ONLY: Engines = Engines {
    rules: true,
    model: false,
};

/// Every `Check`, so the report lists each one, zero included.
const CHECKS: [Check; 10] = [
    Check::Numbers,
    Check::Urls,
    Check::Terms,
    Check::Placeholders,
    Check::Protected,
    Check::BlockType,
    Check::NewBlockMarker,
    Check::OutputContract,
    Check::NoImprovement,
    Check::Document,
];

/// Seconds spent in `rewrite` and how many windows reached it. Model load time is not counted.
#[derive(Clone, Default)]
struct Timing {
    windows: Rc<Cell<u32>>,
    seconds: Rc<Cell<f64>>,
}

struct TimedRewriter {
    inner: Box<dyn Rewriter>,
    timing: Timing,
}

impl Rewriter for TimedRewriter {
    fn rewrite(&mut self, req: &RewriteRequest<'_>) -> Result<String, Error> {
        let start = Instant::now();
        let out = self.inner.rewrite(req);
        let t = &self.timing;
        t.seconds
            .set(t.seconds.get() + start.elapsed().as_secs_f64());
        t.windows.set(t.windows.get() + 1);
        out
    }
}

struct TimedFactory {
    inner: RuntimeFactory,
    timing: Timing,
}

impl RewriterFactory for TimedFactory {
    fn preflight(&self, m: &ResolvedModel) -> Result<(), Error> {
        self.inner.preflight(m)
    }
    fn open(&self, m: &ResolvedModel) -> Result<Box<dyn Rewriter>, Error> {
        Ok(Box::new(TimedRewriter {
            inner: self.inner.open(m)?,
            timing: self.timing.clone(),
        }))
    }
}

/// One model's row of the report (FR-10.2). Serialised camelCase like the CLI's JSON report.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelReport {
    model: String,
    paragraphs: usize,
    mean_score_before: f64,
    mean_score_after: f64,
    /// Paragraphs whose protected spans are byte-identical after the fix.
    integrity: usize,
    windows: u32,
    accepted_rewrites: u32,
    rejected: BTreeMap<String, u32>,
    slp902: u32,
    mean_seconds_per_window: f64,
    budget_seconds_per_window: f64,
}

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn model_settings(name: &str) -> ModelSettings {
    ModelSettings {
        name: name.to_string(),
        path: None,
        profile: None,
        cache_dir: None,
        window: WindowMode::Paragraph,
        max_window_chars: 4000,
        max_tokens: 0,
        seed: engine::DEFAULT_SEED,
    }
}

/// The registry models in the user cache. Absent default → the test fails naming `sloperator model pull`.
fn cached_models() -> Vec<&'static ModelSpec> {
    if let Err(e) = model::resolve(&model_settings(model::DEFAULT_MODEL)) {
        panic!("{e}\nthe eval needs the default model: run `sloperator model pull`");
    }
    model::REGISTRY
        .iter()
        .filter(|spec| model::resolve(&model_settings(spec.name)).is_ok())
        .collect()
}

/// The twenty paragraphs, copied under a temp dir (no `sloperator.toml` above it) so the built-in defaults apply.
fn corpus(dir: &Path, settings: &Arc<Settings>) -> Vec<Source> {
    (1..=PARAGRAPHS)
        .map(|n| {
            let name = format!("p{n:02}.md");
            let text = fs::read_to_string(manifest().join("eval/paragraphs").join(&name))
                .unwrap_or_else(|e| panic!("{name} missing or unreadable: {e}"));
            Source {
                origin: Origin::File(dir.join(&name)),
                format: Format::Markdown,
                text: Arc::from(text),
                settings: Arc::clone(settings),
            }
        })
        .collect()
}

fn protected(text: &str, s: &Settings) -> Vec<String> {
    let d = doc::parse(Arc::from(text), Format::Markdown, &s.protect);
    let mut v: Vec<String> = d
        .protected_strings()
        .into_iter()
        .map(str::to_string)
        .collect();
    v.sort();
    v
}

fn check_name(c: Check) -> String {
    serde_json::to_value(c)
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
}

/// The check named in an SLP901 message, `rewrite rejected (<check>): <detail>` (`pipeline::rejection`).
fn rejected_check(message: &str) -> String {
    let rest = message
        .strip_prefix("rewrite rejected (")
        .unwrap_or_else(|| panic!("an SLP901 message without its check: {message}"));
    rest[..rest.find(')').unwrap()].to_string()
}

/// The check of every SLP901 in `r`.
fn rejections(r: &FileResult) -> Vec<String> {
    r.findings
        .iter()
        .filter(|l| l.finding.code == RuleCode::REWRITE_REJECTED)
        .map(|l| rejected_check(&l.finding.message))
        .collect()
}

fn count(r: &FileResult, code: RuleCode) -> u32 {
    r.findings.iter().filter(|l| l.finding.code == code).count() as u32
}

/// Runs the corpus through one model and builds its report row.
fn eval_model(spec: &ModelSpec, sources: &[Source]) -> ModelReport {
    let timing = Timing::default();
    let factory = TimedFactory {
        inner: RuntimeFactory::new(false),
        timing: timing.clone(),
    };
    let mut rw = LazyRewriter::new(&factory, &model_settings(spec.name))
        .unwrap_or_else(|e| panic!("{}: {e}", spec.name));
    let mut rejected: BTreeMap<String, u32> = CHECKS.iter().map(|&c| (check_name(c), 0)).collect();
    let (mut before, mut after) = (0.0, 0.0);
    let (mut integrity, mut slp902, mut accepted) = (0, 0, 0u32);
    for src in sources {
        let name = format!("{:?}", src.origin);
        let s = Arc::clone(&src.settings);
        // Step 1 is the same with or without the model, so the rules-only SLP901s are the safe-fix rejections and
        // the rest of the both-engines SLP901s are rejected model rewrites.
        let rules = pipeline::fix(
            pipeline::analyse(src.clone()).unwrap(),
            RULES_ONLY,
            None,
            s.model.seed,
        )
        .unwrap_or_else(|e| panic!("{name} rules-only: {e}"));
        let step1 = rejections(&rules);
        assert!(
            !step1.contains(&check_name(Check::Document)),
            "{name}: the safe fixes alone failed the document check, so the eval cannot tell step-1 rejections \
             from model ones"
        );
        let windows_before = timing.windows.get();
        let r = pipeline::fix(
            pipeline::analyse(src.clone()).unwrap(),
            BOTH,
            Some(&mut rw),
            s.model.seed,
        )
        .unwrap_or_else(|e| panic!("{} on {name}: {e}", spec.name));
        let windows = timing.windows.get() - windows_before;

        before += r.score_before.as_ref().unwrap().value;
        after += r.score.value;
        let final_text = r.fixed.as_deref().unwrap_or(&r.original);
        if protected(final_text, &s) == protected(&r.original, &s) {
            integrity += 1;
        }
        slp902 += count(&r, RuleCode::WINDOW_SKIPPED);
        let checks = rejections(&r);
        for c in &checks {
            *rejected
                .get_mut(c)
                .unwrap_or_else(|| panic!("unknown check `{c}`")) += 1;
        }
        // A whole-document failure discards every edit of the paragraph (architecture §7.2): nothing accepted.
        if !checks.contains(&check_name(Check::Document)) {
            let model_rejected = checks.len() - step1.len();
            accepted += windows - model_rejected as u32;
        }
    }
    drop(rw); // free this model before the next one opens (one backend per process)

    let windows = timing.windows.get();
    assert!(windows > 0, "{}: no window reached the model", spec.name);
    let n = sources.len() as f64;
    ModelReport {
        model: spec.name.to_string(),
        paragraphs: sources.len(),
        mean_score_before: before / n,
        mean_score_after: after / n,
        integrity,
        windows,
        accepted_rewrites: accepted,
        rejected,
        slp902,
        mean_seconds_per_window: timing.seconds.get() / f64::from(windows),
        budget_seconds_per_window: MODEL_WINDOW_BUDGET_S,
    }
}

fn table(r: &ModelReport) -> String {
    let mut t = format!("eval: {}\n", r.model);
    let mut row = |k: &str, v: String| t.push_str(&format!("  {k:<28} {v}\n"));
    row(
        "mean score before / after",
        format!("{:.3} / {:.3}", r.mean_score_before, r.mean_score_after),
    );
    row("integrity", format!("{}/{}", r.integrity, r.paragraphs));
    row("windows rewritten", r.windows.to_string());
    row("rewrites accepted", r.accepted_rewrites.to_string());
    for (check, n) in &r.rejected {
        row(&format!("rejected: {check}"), n.to_string());
    }
    row("SLP902 (window skipped)", r.slp902.to_string());
    row(
        "mean s/window (budget)",
        format!(
            "{:.3} ({:.1})",
            r.mean_seconds_per_window, r.budget_seconds_per_window
        ),
    );
    t
}

fn target_dir() -> PathBuf {
    std::env::var_os("CARGO_TARGET_DIR").map_or_else(|| manifest().join("target"), PathBuf::from)
}

#[test]
#[ignore = "integration tier"]
fn eval_report_per_model() {
    // Compile-time: an unset (0.0) budget cannot pass by accident (SPEC §22).
    const {
        assert!(
            MODEL_WINDOW_BUDGET_S > 0.0,
            "MODEL_WINDOW_BUDGET_S is unset (0.0)"
        )
    };
    let models = cached_models();

    let tmp = tempfile::tempdir().unwrap();
    let settings = Resolver::new(CliOverrides::default(), None)
        .unwrap()
        .for_file(&tmp.path().join("p01.md"))
        .unwrap();
    let sources = corpus(tmp.path(), &settings);

    let reports: Vec<ModelReport> = models.iter().map(|m| eval_model(m, &sources)).collect();

    let dir = target_dir().join("eval");
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("report.json");
    fs::write(&out, serde_json::to_string_pretty(&reports).unwrap()).unwrap();
    for r in &reports {
        println!("{}", table(r));
    }
    println!("report: {}", out.display());

    for r in &reports {
        assert_eq!(
            r.integrity,
            PARAGRAPHS,
            "{}: protected-span integrity {}/{PARAGRAPHS}\n{}",
            r.model,
            r.integrity,
            table(r)
        );
        assert!(
            r.mean_seconds_per_window <= MODEL_WINDOW_BUDGET_S,
            "{}: mean {:.3} s/window exceeds the budget {MODEL_WINDOW_BUDGET_S} s (NFR-3)\n{}",
            r.model,
            r.mean_seconds_per_window,
            table(r)
        );
    }
    assert!(
        reports.iter().any(|r| r.model == model::DEFAULT_MODEL),
        "the default model was not evaluated"
    );
}
