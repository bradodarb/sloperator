//! The model engine without a model (SPEC §16, §23.12): window selection, ⟦n⟧ masking, the output contract,
//! placeholder integrity, SLP902 skips and re-wrapping of hard-wrapped windows. A [`FakeRewriter`] stands in for the
//! runtime.
#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may unwrap (architecture §8)

mod common;

use std::path::Path;
use std::sync::Arc;

use sloperator::text::Prose;

use common::FakeRewriter;
use sloperator::config::{CliOverrides, Resolver, Settings, WindowMode};
use sloperator::doc::{self, Document, Format, ProtectSettings};
use sloperator::engine::mask::{self, propose};
use sloperator::engine::prompt::{self, FEW_SHOT, SYSTEM};
use sloperator::engine::window::{Window, WindowPlan, windows};
use sloperator::engine::{DEFAULT_SEED, RewriteRequest, Rewriter};
use sloperator::model::PromptProfile;
use sloperator::model::registry::QWEN3;
use sloperator::rules::{Finding, Proposal, RuleCode, Severity};
use sloperator::types::ProposalOrigin;
use sloperator::verify::Check;

const AI_VOCAB: RuleCode = RuleCode(2); // FixClass::Model
const REPLACEABLE: RuleCode = RuleCode(1); // FixClass::Safe
const BULLET_SPAM: RuleCode = RuleCode(10); // FixClass::None

/// The built-in default settings (min severity `warning`, paragraph windows, 4000-char cap).
fn settings() -> Settings {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("no-config-here/doc.md");
    let s = Resolver::new(CliOverrides::default(), None)
        .unwrap()
        .for_file(&path)
        .unwrap();
    (*s).clone()
}

fn parse(src: &str) -> Document {
    doc::parse(
        Arc::from(src),
        Format::Markdown,
        &ProtectSettings::default(),
    )
}

/// A finding on the first occurrence of `needle`.
fn finding(d: &Document, needle: &str, code: RuleCode, severity: Severity) -> Finding {
    let start = d.src.find(needle).expect("needle in source");
    Finding {
        code,
        range: start..start + needle.len(),
        message: "test".into(),
        severity,
        fix: None,
    }
}

fn rewrite_plans(plans: &[WindowPlan]) -> Vec<&Window> {
    plans
        .iter()
        .map(|p| match p {
            WindowPlan::Rewrite(w) => w,
            WindowPlan::Skip { reason, .. } => panic!("unexpected skip: {reason}"),
        })
        .collect()
}

/// The one window of a paragraph with a model-class finding on `needle`.
fn window_of(d: &Document, needle: &str) -> Window {
    let prose = Prose::new(d);
    let f = finding(d, needle, AI_VOCAB, Severity::Warning);
    let plans = windows(d, &prose, &[f], &[], &settings());
    let ws = rewrite_plans(&plans);
    assert_eq!(ws.len(), 1);
    ws[0].clone()
}

fn skip_reason(plans: &[WindowPlan]) -> String {
    assert_eq!(plans.len(), 1, "{plans:?}");
    let f = plans[0].skipped_finding().expect("a Skip");
    assert_eq!(f.code, RuleCode::WINDOW_SKIPPED);
    assert_eq!(f.code.to_string(), "SLP902");
    assert_eq!(f.severity, Severity::Warning);
    match &plans[0] {
        WindowPlan::Skip { reason, .. } => reason.clone(),
        WindowPlan::Rewrite(_) => unreachable!(),
    }
}

/// Asks a FakeRewriter for `output` the way the pipeline will, then runs the contract and unmasking.
fn fake_rewrite(d: &Document, w: &Window, output: &str) -> Proposal {
    let mut rw = FakeRewriter {
        output: Some(output.to_string()),
        ..FakeRewriter::default()
    };
    let req = RewriteRequest {
        system: SYSTEM,
        few_shot: FEW_SHOT,
        window: &w.masked,
        profile: &QWEN3,
        max_tokens: prompt::max_tokens(&w.masked, &settings().model),
        seed: DEFAULT_SEED,
    };
    let raw = rw.rewrite(&req).unwrap();
    propose(d, w, &raw, &QWEN3).unwrap_or_else(|r| panic!("rejected: {r:?}"))
}

fn apply(d: &Document, p: &Proposal) -> Document {
    let src = format!(
        "{}{}{}",
        &d.src[..p.range.start],
        p.replacement,
        &d.src[p.range.end..]
    );
    parse(&src)
}

#[test]
fn windows_only_for_findings_at_or_above_min_severity() {
    let d = parse(
        "We delve into the cache.\n\nThe tapestry of options is rich.\n\nWe utilize the tool.\n\nPlain closing words.\n",
    );
    let prose = Prose::new(&d);
    let p = |i: usize| d.blocks[i].range.clone();
    let mut safe_with_fix = finding(&d, "tapestry", REPLACEABLE, Severity::Error);
    safe_with_fix.fix = Some(Proposal {
        range: safe_with_fix.range.clone(),
        replacement: "set".into(),
        origin: ProposalOrigin::Rule(REPLACEABLE),
    });
    let findings = [
        finding(&d, "delve", AI_VOCAB, Severity::Warning), // model class at min → window
        finding(&d, "tapestry", AI_VOCAB, Severity::Suggestion), // below min → none
        safe_with_fix, // safe with a fix: the rules engine applies it → none
        finding(&d, "utilize", REPLACEABLE, Severity::Warning), // safe, no fix → window
    ];
    let mut s = settings();
    assert_eq!(s.min_severity, Severity::Warning);
    let plans = windows(&d, &prose, &findings, &[], &s);
    let ranges: Vec<_> = rewrite_plans(&plans)
        .iter()
        .map(|w| w.range.clone())
        .collect();
    assert_eq!(ranges, vec![p(0), p(2)]);

    s.min_severity = Severity::Suggestion;
    let plans = windows(&d, &prose, &findings, &[], &s);
    let ranges: Vec<_> = rewrite_plans(&plans)
        .iter()
        .map(|w| w.range.clone())
        .collect();
    assert_eq!(ranges, vec![p(0), p(1), p(2)], "one window per block");
    let w = &rewrite_plans(&plans)[1];
    assert_eq!(
        w.weight_before,
        9 + 1,
        "every finding overlapping the window counts"
    );
}

#[test]
fn none_class_selects_no_window() {
    let d = parse("- one\n- two\n- three\n- four\n- five\n");
    let prose = Prose::new(&d);
    let f = finding(&d, "one", BULLET_SPAM, Severity::Error);
    assert!(windows(&d, &prose, &[f], &[], &settings()).is_empty());
}

#[test]
fn mask_replaces_protected_runs_with_numbered_placeholders() {
    let src = "Use `cargo` to [build](https://x.org) it at https://y.org now.\n";
    let d = parse(src);
    let (masked, slots) = mask::mask(&d, d.blocks[0].id, d.blocks[0].range.clone());
    assert_eq!(masked, "Use ⟦1⟧ to ⟦2⟧build⟦3⟧ it at ⟦4⟧ now.");
    let restored: Vec<&str> = slots.iter().map(|r| &src[r.clone()]).collect();
    assert_eq!(
        restored,
        ["`cargo`", "[", "](https://x.org)", "https://y.org"]
    );

    let w = window_of(&d, "now");
    assert_eq!((w.masked.as_str(), &w.slots), (masked.as_str(), &slots));
    assert_eq!(w.wrap, None, "a single-line window needs no re-wrap");
}

#[test]
fn contract_rejects_think_block_preamble_quotes_blank_line() {
    let d = parse("Use `cargo` to build the whole thing now.\n");
    let w = window_of(&d, "build");
    for bad in [
        "<think>plan the edit</think>\nUse ⟦1⟧ to build it.",
        "Here is the rewritten passage: Use ⟦1⟧ to build it.",
        "Sure, here it is:\nUse ⟦1⟧ to build it.",
        "\"Use ⟦1⟧ to build it.\"",
        "“Use ⟦1⟧ to build it.”",
        "```\nUse ⟦1⟧ to build it.\n```",
        "Use ⟦1⟧ to build it.\n\nNote: shorter.",
        "",
        &"Use ⟦1⟧ to build it. ".repeat(10),
    ] {
        let r = propose(&d, &w, bad, &QWEN3).expect_err(bad);
        assert_eq!(r.failed, Check::OutputContract, "{bad:?}: {}", r.detail);
        assert_eq!(r.range, w.range);
    }
    let ok = propose(&d, &w, "Use ⟦1⟧ to build it.", &QWEN3).unwrap();
    assert_eq!(ok.replacement, "Use `cargo` to build it.");
    assert_eq!(ok.origin, ProposalOrigin::Model);
    assert_eq!(ok.range, w.range);
}

#[test]
fn strips_empty_think() {
    let d = parse("Use `cargo` to build the whole thing now.\n");
    let w = window_of(&d, "build");
    let raw = "<think>\n\n</think>\n\nUse ⟦1⟧ to build it.";
    let p = propose(&d, &w, raw, &QWEN3).unwrap();
    assert_eq!(p.replacement, "Use `cargo` to build it.");

    let keeps_think = PromptProfile {
        strip_empty_think: false,
        ..QWEN3
    };
    let r = propose(&d, &w, raw, &keeps_think).unwrap_err();
    assert_eq!(r.failed, Check::OutputContract);
}

#[test]
fn placeholder_dropped_or_duplicated_rejected() {
    let d = parse("Run `a` then `b` before the release goes out.\n");
    let w = window_of(&d, "release");
    assert_eq!(w.slots.len(), 2);
    for (bad, want) in [
        ("Run ⟦1⟧ before release.", "dropped: ⟦2⟧"),
        ("Run ⟦1⟧ then ⟦2⟧ and ⟦2⟧.", "duplicated: ⟦2⟧"),
        ("Run ⟦1⟧ then ⟦3⟧.", "unknown placeholder ⟦3⟧"),
        ("Run ⟦1⟧ then ⟦2 first.", "unclosed"),
        ("Run ⟦1⟧ then 2⟧ ⟦2⟧.", "stray"),
    ] {
        let r = propose(&d, &w, bad, &QWEN3).expect_err(bad);
        assert_eq!(r.failed, Check::Placeholders, "{bad}");
        assert!(r.detail.contains(want), "{bad}: {}", r.detail);
    }
    let moved = propose(&d, &w, "Before release, run ⟦2⟧ after ⟦1⟧.", &QWEN3).unwrap();
    assert_eq!(moved.replacement, "Before release, run `b` after `a`.");
}

#[test]
fn input_with_placeholder_glyph_skipped_slp902() {
    let d = parse("The notation ⟦x⟧ shows a delve into sets.\n");
    let prose = Prose::new(&d);
    let f = finding(&d, "delve", AI_VOCAB, Severity::Warning);
    let plans = windows(&d, &prose, &[f], &[], &settings());
    assert!(skip_reason(&plans).contains("placeholder glyph"));
    let d = parse("A lone ⟧ glyph and a delve.\n");
    let prose = Prose::new(&d);
    let f = finding(&d, "delve", AI_VOCAB, Severity::Warning);
    assert!(
        skip_reason(&windows(&d, &prose, &[f], &[], &settings())).contains("placeholder glyph")
    );
}

#[test]
fn over_long_window_skipped_slp902() {
    let d = parse("We delve into a long passage that keeps going on and on.\n");
    let prose = Prose::new(&d);
    let f = finding(&d, "delve", AI_VOCAB, Severity::Warning);
    let mut s = settings();
    assert_eq!(s.model.max_window_chars, 4000);
    assert_eq!(
        rewrite_plans(&windows(&d, &prose, std::slice::from_ref(&f), &[], &s)).len(),
        1
    );
    s.model.max_window_chars = 20;
    let reason = skip_reason(&windows(&d, &prose, &[f], &[], &s));
    assert!(reason.contains("max_window_chars = 20"), "{reason}");
}

#[test]
fn hard_wrapped_window_rewrapped() {
    let src = "We delve into the cache layer here and\nlook at the `get` path that serves reads\nfrom memory for every request.\n";
    let d = parse(src);
    let w = window_of(&d, "delve");
    assert!(
        !w.masked.contains('\n'),
        "soft breaks fold to spaces: {:?}",
        w.masked
    );
    assert_eq!(
        w.masked,
        "We delve into the cache layer here and look at the ⟦1⟧ path that serves reads from memory for every request."
    );
    let wrap = w.wrap.clone().expect("multi-line window");
    assert_eq!((wrap.prefix.as_str(), wrap.width), ("", 40));

    let p = fake_rewrite(
        &d,
        &w,
        "We look at the cache layer and the ⟦1⟧ path that serves reads from memory for every single request.",
    );
    let lines: Vec<&str> = p.replacement.lines().collect();
    assert!(lines.len() > 1, "{:?}", p.replacement);
    assert!(
        lines.iter().all(|l| l.chars().count() <= wrap.width),
        "{lines:?}"
    );
    assert!(p.replacement.contains("`get`"));
    let after = apply(&d, &p);
    assert_eq!(doc::skeleton(&after), doc::skeleton(&d));
    assert_eq!(after.protected_strings(), d.protected_strings());
}

#[test]
fn wrapped_blockquote_and_list_item_rewrapped_with_prefixes() {
    for (src, prefix) in [
        (
            "> We delve into the cache layer here and\n> look at the `get` path that serves\n> reads from memory for each request.\n",
            "> ",
        ),
        (
            "- We delve into the cache layer here and\n   look at the `get` path that serves\n   reads from memory for each request.\n",
            "   ",
        ),
    ] {
        let d = parse(src);
        let w = window_of(&d, "delve");
        assert!(!w.masked.contains(prefix.trim_end()) || prefix.trim().is_empty());
        assert!(
            !w.masked.contains("  "),
            "prefix never reaches the model: {:?}",
            w.masked
        );
        assert_eq!(w.wrap.as_ref().map(|x| x.prefix.as_str()), Some(prefix));

        let p = fake_rewrite(
            &d,
            &w,
            "We read the cache layer and the ⟦1⟧ path that serves reads from memory for each and every request.",
        );
        let lines: Vec<&str> = p.replacement.lines().collect();
        assert!(lines.len() >= 3, "{:?}", p.replacement);
        assert!(
            lines[1..].iter().all(|l| l.starts_with(prefix)),
            "every continuation line keeps {prefix:?}: {lines:?}"
        );
        let after = apply(&d, &p);
        assert_eq!(doc::skeleton(&after), doc::skeleton(&d), "{}", after.src);
        assert_eq!(after.protected_strings(), d.protected_strings());
        for line in after.src.lines() {
            assert!(line.starts_with(prefix) || line.starts_with(&prefix.replace("   ", "- ")));
        }
    }
}

#[test]
fn inconsistent_prefix_window_skipped_slp902() {
    for src in [
        "> We delve into the cache\nlazy line here\n> back in the quote.\n",
        "- We delve into the cache\n   indented by three\n    indented by four.\n",
    ] {
        let d = parse(src);
        let prose = Prose::new(&d);
        let f = finding(&d, "delve", AI_VOCAB, Severity::Warning);
        let reason = skip_reason(&windows(&d, &prose, &[f], &[], &settings()));
        assert!(
            reason.contains("inconsistent container prefix"),
            "{src:?}: {reason}"
        );
    }
}

#[test]
fn max_tokens_from_setting_or_window_length() {
    let mut m = settings().model;
    assert_eq!(m.max_tokens, 0);
    let masked = "x".repeat(400);
    assert_eq!(prompt::max_tokens(&masked, &m), 400 / 4 * 3 / 2 + 16);
    m.max_tokens = 77;
    assert_eq!(prompt::max_tokens(&masked, &m), 77);
}

#[test]
fn sentence_mode_windows_the_finding_sentence() {
    let src = "> First we set up `x` here. Then we delve into the\n> cache layer, e.g. `y` too. Last words.\n";
    let d = parse(src);
    let prose = Prose::new(&d);
    let f = finding(&d, "delve", AI_VOCAB, Severity::Warning);
    let mut s = settings();
    s.model.window = WindowMode::Sentence;
    let plans = windows(&d, &prose, &[f], &[], &s);
    let w = rewrite_plans(&plans)[0];
    assert_eq!(
        w.masked,
        "Then we delve into the cache layer, e.g. ⟦1⟧ too."
    );
    let wrap = w.wrap.as_ref().expect("the sentence spans two lines");
    assert_eq!((wrap.prefix.as_str(), wrap.indent), ("> ", 26));
}

// ---- The model step of the pipeline (SPEC §17 step 2, epic 06 story-03) ----

const PUFFERY: RuleCode = RuleCode(7); // FixClass::Model, warning

const MODEL_ONLY: sloperator::pipeline::Engines = sloperator::pipeline::Engines {
    rules: false,
    model: true,
};

/// Runs `pipeline::fix` with the model engine only over `text` (default settings), through a LazyRewriter over
/// `factory` with a planted model.
fn fix_model(text: &str, factory: &common::FakeFactory) -> sloperator::pipeline::FileResult {
    use sloperator::input::{Origin, Source};
    let cache = common::TempModelCache::new();
    cache.plant(sloperator::model::spec(sloperator::model::DEFAULT_MODEL).unwrap());
    let mut lazy = sloperator::engine::LazyRewriter::new(factory, &cache.settings()).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("doc.md");
    let settings = Resolver::new(CliOverrides::default(), None)
        .unwrap()
        .for_file(&path)
        .unwrap();
    let a = sloperator::pipeline::analyse(Source {
        origin: Origin::File(path),
        format: Format::Markdown,
        text: Arc::from(text),
        settings,
    })
    .unwrap();
    let r = sloperator::pipeline::fix(a, MODEL_ONLY, Some(&mut lazy), DEFAULT_SEED).unwrap();
    assert_eq!(lazy.loads, factory.opens.get());
    r
}

fn codes(r: &sloperator::pipeline::FileResult) -> Vec<RuleCode> {
    r.findings.iter().map(|l| l.finding.code).collect()
}

fn rejection_message(r: &sloperator::pipeline::FileResult) -> String {
    let rej: Vec<_> = r
        .findings
        .iter()
        .filter(|l| l.finding.code == RuleCode::REWRITE_REJECTED)
        .collect();
    assert_eq!(rej.len(), 1, "{:?}", r.findings);
    assert_eq!(rej[0].finding.severity, Severity::Warning);
    rej[0].finding.message.clone()
}

#[test]
fn no_window_means_no_load() {
    let factory = common::FakeFactory::returning("unused");
    // A suggestion-only finding (SLP002) is below the default min severity, so no window is cut.
    let r = fix_model("A plain sentence about a robust cache.\n", &factory);
    assert_eq!(factory.opens.get(), 0);
    assert!(factory.calls.borrow().is_empty());
    assert_eq!(r.fixed, None);
}

#[test]
fn many_windows_load_once() {
    let factory = common::FakeFactory::default(); // echoes each window
    let text = "The cache plays a crucial role here.\n\nThe log plays a crucial role there.\n\n\
                The test plays a crucial role too.\n";
    let r = fix_model(text, &factory);
    assert_eq!(factory.opens.get(), 1);
    assert_eq!(factory.calls.borrow().len(), 3);
    // An echo is never an improvement, so every window is rejected and the text kept.
    assert_eq!(r.fixed, None);
    assert_eq!(
        codes(&r)
            .iter()
            .filter(|c| **c == RuleCode::REWRITE_REJECTED)
            .count(),
        3
    );
}

#[test]
fn good_rewrite_written() {
    let factory = common::FakeFactory::returning("The cache speeds up the 42 builds.");
    let r = fix_model(
        "The cache plays a crucial role in the 42 builds.\n",
        &factory,
    );
    assert_eq!(
        r.fixed.as_deref(),
        Some("The cache speeds up the 42 builds.\n")
    );
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn number_drop_rejected_original_kept() {
    let factory = common::FakeFactory::returning("The cache speeds up the builds.");
    let r = fix_model(
        "The cache plays a crucial role in the 42 builds.\n",
        &factory,
    );
    assert_eq!(r.fixed, None);
    let msg = rejection_message(&r);
    assert!(msg.starts_with("rewrite rejected (numbers):"), "{msg}");
    assert!(
        codes(&r).contains(&PUFFERY),
        "the original finding is still reported"
    );
}

#[test]
fn equal_score_rewrite_rejected() {
    // One SLP007 warning before, one SLP007 warning after: the same weighted count (DQ-3).
    let factory = common::FakeFactory::returning(
        "The cache marks a significant milestone for the 42 builds.",
    );
    let r = fix_model(
        "The cache plays a crucial role in the 42 builds.\n",
        &factory,
    );
    assert_eq!(r.fixed, None);
    let msg = rejection_message(&r);
    assert!(
        msg.starts_with("rewrite rejected (noImprovement):"),
        "{msg}"
    );
    assert!(msg.contains("3 -> 3"), "{msg}");
}

/// A temp dir holding `doc.md` with a model-class finding and `bad.md` that is not UTF-8.
fn cli_inputs(dir: &Path) -> (std::path::PathBuf, std::path::PathBuf) {
    let doc = dir.join("doc.md");
    std::fs::write(&doc, "The cache plays a crucial role in the 42 builds.\n").unwrap();
    let bad = dir.join("bad.md");
    std::fs::write(&bad, [0xff, 0xfe, b'\n']).unwrap();
    (doc, bad)
}

#[test]
fn missing_model_exits_two_before_any_file() {
    let tmp = common::cli_tempdir();
    let cache = tmp.path().join("empty-cache");
    std::fs::write(
        tmp.path().join("sloperator.toml"),
        format!("[model]\ncache_dir = {:?}\n", cache.to_string_lossy()),
    )
    .unwrap();
    let (doc, bad) = cli_inputs(tmp.path());
    let factory = std::rc::Rc::new(common::FakeFactory::returning("unused"));
    let (code, out, err) = common::run_cli_with(
        &[
            "check",
            "--fix",
            &doc.to_string_lossy(),
            &bad.to_string_lossy(),
        ],
        "",
        Box::new(common::SharedFactory(std::rc::Rc::clone(&factory))),
    );
    assert_eq!(
        code,
        sloperator::cli::ExitCode::Failure,
        "out: {out} err: {err}"
    );
    assert!(
        err.contains("model pull") && err.contains("--rules-only"),
        "{err}"
    );
    // Neither file was read: the non-UTF-8 file would have been reported.
    assert!(!err.contains("bad.md"), "{err}");
    assert_eq!((factory.preflights.get(), factory.opens.get()), (0, 0));
    assert_eq!(
        std::fs::read_to_string(&doc).unwrap(),
        "The cache plays a crucial role in the 42 builds.\n"
    );
}

#[test]
fn disabled_factory_exits_two() {
    let tmp = common::cli_tempdir();
    common::plant_model_config(tmp.path());
    let (doc, bad) = cli_inputs(tmp.path());
    let (code, out, err) = common::run_cli(
        &[
            "check",
            "--fix",
            &doc.to_string_lossy(),
            &bad.to_string_lossy(),
        ],
        "",
    );
    assert_eq!(
        code,
        sloperator::cli::ExitCode::Failure,
        "out: {out} err: {err}"
    );
    assert!(err.contains("without model support"), "{err}");
    assert!(!err.contains("bad.md"), "{err}");
    assert_eq!(
        std::fs::read_to_string(&doc).unwrap(),
        "The cache plays a crucial role in the 42 builds.\n"
    );
}

#[test]
fn runtime_error_exits_two_nothing_written() {
    let tmp = common::cli_tempdir();
    common::plant_model_config(tmp.path());
    // `a.md` has only a safe fix; `b.md` needs the model, which fails.
    let a = tmp.path().join("a.md");
    std::fs::write(&a, "We utilize the cache.\n").unwrap();
    let b = tmp.path().join("b.md");
    std::fs::write(&b, "The cache plays a crucial role in the 42 builds.\n").unwrap();
    let factory = std::rc::Rc::new(common::FakeFactory {
        fail: Some("decode failed".into()),
        ..common::FakeFactory::default()
    });
    let (code, out, err) = common::run_cli_with(
        &["check", "--fix", &a.to_string_lossy(), &b.to_string_lossy()],
        "",
        Box::new(common::SharedFactory(std::rc::Rc::clone(&factory))),
    );
    assert_eq!(
        code,
        sloperator::cli::ExitCode::Failure,
        "out: {out} err: {err}"
    );
    assert!(err.contains("decode failed"), "{err}");
    assert_eq!(factory.calls.borrow().len(), 1);
    assert_eq!(
        std::fs::read_to_string(&a).unwrap(),
        "We utilize the cache.\n"
    );
    assert_eq!(
        std::fs::read_to_string(&b).unwrap(),
        "The cache plays a crucial role in the 42 builds.\n"
    );
}

#[test]
fn seed_passed_through_same_for_every_window() {
    let tmp = common::cli_tempdir();
    common::plant_model_config(tmp.path());
    let doc = tmp.path().join("doc.md");
    std::fs::write(
        &doc,
        "The cache plays a crucial role here.\n\nThe log plays a crucial role there.\n\n\
         The test plays a crucial role too.\n",
    )
    .unwrap();
    let factory = std::rc::Rc::new(common::FakeFactory::default());
    let (code, out, err) = common::run_cli_with(
        &["check", "--diff", "--seed", "1234", &doc.to_string_lossy()],
        "",
        Box::new(common::SharedFactory(std::rc::Rc::clone(&factory))),
    );
    assert_eq!(
        code,
        sloperator::cli::ExitCode::Findings,
        "out: {out} err: {err}"
    );
    let seeds: Vec<u32> = factory.calls.borrow().iter().map(|(_, s)| *s).collect();
    assert_eq!(seeds, vec![1234; 3]);
}

#[test]
fn metric_finding_never_selects_window() {
    let d = parse(&std::fs::read_to_string(common::fixture("rules/SLP018/positive.md")).unwrap());
    let prose = Prose::new(&d);
    let mut s = settings();
    s.min_severity = Severity::Suggestion;
    let rule = sloperator::rules::registry::Registry::builtin()
        .get(RuleCode(18))
        .unwrap();
    let enabled = sloperator::rules::Enabled {
        rule,
        severity: Severity::Suggestion,
        params: rule
            .meta()
            .params
            .iter()
            .map(|p| (p.name, p.default))
            .collect(),
    };
    let findings = sloperator::rules::detect(
        &d,
        &prose,
        &sloperator::directives::Suppressions::default(),
        &[enabled],
    );
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].code, RuleCode(18));
    assert!(windows(&d, &prose, &findings, &[], &s).is_empty());
}

#[test]
fn window_codes_one_per_finding() {
    let d = parse("It is not red but blue, not slow but fast.\n\nOther paragraph.\n");
    let prose = Prose::new(&d);
    let findings = [
        finding(&d, "red", RuleCode(3), Severity::Warning),
        finding(&d, "blue", RuleCode::REWRITE_REJECTED, Severity::Error),
        finding(&d, "slow", RuleCode(3), Severity::Warning),
        finding(&d, "fast", RuleCode(2), Severity::Suggestion),
        finding(&d, "Other", RuleCode(3), Severity::Warning),
    ];
    let plans = windows(&d, &prose, &findings, &[], &settings());
    let ws = rewrite_plans(&plans);
    assert_eq!(ws.len(), 2);
    assert_eq!(ws[0].codes, [RuleCode(3), RuleCode(3), RuleCode(2)]);
    assert_eq!(ws[0].weight_before, 7);
    assert_eq!(ws[1].codes, [RuleCode(3)]);
    assert_eq!(ws[1].weight_before, 3);
}
