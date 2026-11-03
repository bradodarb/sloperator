//! The per-file pipeline (SPEC §17, architecture §7.2): `analyse` → `lint`, or `analyse` → `fix` for Fix/Diff.
//!
//! `pipeline` never uses `cli`, `report`, `write` or `engine::runtime` (layering). `lint` takes no rewriter, so it
//! cannot reach the model (FR-2.1, D-3). `fix` runs the rules engine's safe fixes (step 1), then the model windows
//! (step 2), then the whole-document check (step 3) and the report and scores (step 4).

use std::ops::Range;
use std::sync::Arc;

use crate::Error;
use crate::doc::{self, Document};
use crate::engine::window::{Window, WindowPlan, windows};
use crate::input::{Origin, Source};
use crate::rules::registry::Registry;
use crate::rules::{Finding, RuleCode, Severity};
use crate::text::Prose;
use crate::verify::{self, Check, Rejection, Verified};
use crate::{directives, edit, engine, rules, score};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Lint,
    Fix,
    Diff,
}

/// Which engines a Fix/Diff run uses: both by default; `--rules-only` / `--model-only` turn one off (FR-2.10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Engines {
    pub rules: bool,
    pub model: bool,
}

/// One parsed file with its directives and every finding the enabled rules report.
pub struct Analysis {
    pub source: Source,
    pub doc: Document,
    pub prose: Prose,
    pub sup: directives::Suppressions,
    pub findings: Vec<Finding>,
}

/// A finding placed in the text the report shows: 1-based (line, column) with columns counted in characters.
#[derive(Clone, Debug)]
pub struct Located {
    pub finding: Finding,
    pub start: (u32, u32),
    pub end: (u32, u32),
    pub matched: String,
    /// The line of `start`, without its line ending.
    pub line_text: String,
}

pub struct FileResult {
    pub origin: Origin,
    pub original: Arc<str>,
    /// Findings with severity ≥ `min_severity`, plus every SLP9xx (DQ-4), in source order.
    pub findings: Vec<Located>,
    /// The score of the final text.
    pub score: score::Score,
    /// The score of the original text; Fix/Diff only.
    pub score_before: Option<score::Score>,
    /// The final text; Fix/Diff only, and only when it differs from the original.
    pub fixed: Option<String>,
}

/// Parses the source, extracts its directives and runs the enabled rules. Pure and safe to run in parallel (ADR-09).
///
/// # Errors
/// `Error::UnknownRule` or `Error::Directive` from a bad directive (FR-6.5).
pub fn analyse(source: Source) -> Result<Analysis, Error> {
    let (doc, prose, sup, findings) = detect_in(Arc::clone(&source.text), &source)?;
    Ok(Analysis {
        source,
        doc,
        prose,
        sup,
        findings,
    })
}

/// Reports the analysis as it stands. Lint never touches a Rewriter (FR-2.1, D-3) and never runs `verify` (NFR-3).
pub fn lint(a: Analysis) -> FileResult {
    let min = a.source.settings.min_severity;
    let reported = reported(a.findings, min);
    let score = score::score(&reported, a.doc.prose_words());
    FileResult {
        origin: a.source.origin,
        original: a.source.text,
        findings: locate(&a.doc, reported),
        score,
        score_before: None,
        fixed: None,
    }
}

/// Fix/Diff, run serially (ADR-09). `rw` is `None` when the model engine is off; the model is opened only when a
/// window needs it (FR-9.4).
///
/// # Errors
/// A bad directive in the fixed text, or a model load or runtime error, which exits 2 (FR-9.7).
pub fn fix(
    a: Analysis,
    engines: Engines,
    rw: Option<&mut engine::LazyRewriter<'_>>,
    seed: u32,
) -> Result<FileResult, Error> {
    let min = a.source.settings.min_severity;
    let doc0 = a.doc;
    let prose0 = a.prose;
    let findings0 = a.findings;
    let score_before = score::score(&reported(findings0.clone(), min), doc0.prose_words());
    // Rejections, with ranges in the original text, and the verified edits that produced the fixed text.
    let mut rejected: Vec<Rejection> = Vec::new();
    let mut applied: Vec<Verified<edit::Proposal>> = Vec::new();

    // Step 1: safe fixes (D-4, FR-4.7). A rejected fix becomes SLP901 and its bytes stay as they were (FR-8.7).
    let step1 = if engines.rules {
        let proposals: Vec<_> = findings0.iter().filter_map(|f| f.fix.clone()).collect();
        let frozen = a.sup.frozen_ranges();
        for p in edit::plan(proposals, frozen).kept {
            match verify::edit(&doc0, p, frozen, &a.source.settings.protect) {
                Ok(v) => applied.push(v),
                Err(r) => rejected.push(r),
            }
        }
        if applied.is_empty() {
            None
        } else {
            let text1: Arc<str> = edit::apply(&doc0.src, &applied, doc0.newline).into();
            Some(detect_in(text1, &a.source)?)
        }
    } else {
        None
    };
    // SLP901 for the step-1 rejections, moved into the step-1 text.
    let mut system: Vec<Finding> = rejected
        .into_iter()
        .map(|r| {
            let mut f = rejection(r);
            f.range = shift(&f.range, &applied);
            f
        })
        .collect();

    // Step 2: model windows (FR-9.2). With --model-only step 1 did nothing, so windows come from the original
    // findings. The findings it adds (SLP901/902) are placed in the step-2 text.
    let step2 = match (engines.model, rw) {
        (true, Some(rw)) => {
            let (base, base_prose, base_sup, base_findings) = match &step1 {
                Some((d, p, s, f)) => (d, p, s, f.as_slice()),
                None => (&doc0, &prose0, &a.sup, findings0.as_slice()),
            };
            let m = model_windows(
                base,
                base_prose,
                base_sup.frozen_ranges(),
                base_findings,
                &a.source,
                rw,
                seed,
            )?;
            for f in &mut system {
                f.range = shift(&f.range, &m.applied);
            }
            system.extend(m.system);
            if m.applied.is_empty() {
                step1.map(|(d, p, _, f)| (d, p, f))
            } else {
                let text2: Arc<str> = edit::apply(&base.src, &m.applied, base.newline).into();
                let (d, p, _, f) = detect_in(text2, &a.source)?;
                Some((d, p, f))
            }
        }
        _ => step1.map(|(d, p, _, f)| (d, p, f)),
    };

    // Step 3: whole-document check. A failure discards every edit and reports one SLP901 (architecture §7.2).
    let (final_doc, final_findings, system) = match step2 {
        None => (doc0, findings0, system),
        Some((d, _prose, f)) => match verify::document(&doc0, &d) {
            Ok(()) => (d, f, system),
            Err(r) => (doc0, findings0, vec![rejection(r)]),
        },
    };
    let fixed = (final_doc.src != a.source.text).then(|| final_doc.src.to_string());

    // Step 4: report the findings left in the final text that pass --min-severity, plus every SLP9xx.
    let mut out = reported(final_findings, min);
    let score = score::score(&out, final_doc.prose_words());
    out.extend(system);
    out.sort_by_key(|f| (f.range.start, f.code));
    Ok(FileResult {
        origin: a.source.origin,
        original: a.source.text,
        findings: locate(&final_doc, out),
        score,
        score_before: Some(score_before),
        fixed,
    })
}

/// What step 2 produced: the verified window rewrites (ascending, disjoint) and the SLP901/902 findings, both in
/// the coordinates of the text the windows were cut from.
struct ModelStep {
    applied: Vec<Verified<edit::Proposal>>,
    system: Vec<Finding>,
}

/// Step 2 (SPEC §17). Each window is rewritten, run through the output contract (`mask::propose`), verified as an
/// edit and then checked for improvement on the one-edit candidate (FR-9.5, DQ-3). A Skip is SLP902 and any
/// rejection is SLP901, with the window's bytes kept. The model opens on the first window and never when there is
/// none (FR-9.4); every request carries the same `seed` (FR-9.8).
///
/// # Errors
/// A model load or runtime error (FR-9.7), or a bad directive in a candidate text.
fn model_windows(
    doc: &Document,
    prose: &Prose,
    frozen: &[Range<usize>],
    findings: &[Finding],
    source: &Source,
    rw: &mut engine::LazyRewriter<'_>,
    seed: u32,
) -> Result<ModelStep, Error> {
    let s = &source.settings;
    let mut step = ModelStep {
        applied: Vec::new(),
        system: Vec::new(),
    };
    // The end of the last window sent: a window that overlaps it (a sentence window over two sentences) is skipped.
    let mut sent_to = 0;
    for plan in windows(doc, prose, findings, frozen, s) {
        let w = match plan {
            WindowPlan::Rewrite(w) if w.range.start >= sent_to => w,
            WindowPlan::Rewrite(w) => {
                let skip = WindowPlan::Skip {
                    range: w.range,
                    reason: "the window overlaps an earlier window".into(),
                };
                step.system.extend(skip.skipped_finding());
                continue;
            }
            skip => {
                step.system.extend(skip.skipped_finding());
                continue;
            }
        };
        sent_to = w.range.end;
        let profile = rw.profile();
        let req = engine::RewriteRequest {
            system: engine::prompt::SYSTEM,
            few_shot: engine::prompt::FEW_SHOT,
            window: &w.masked,
            profile,
            max_tokens: engine::prompt::max_tokens(&w.masked, &s.model),
            seed,
        };
        let raw = rw.get()?.rewrite(&req)?;
        match accept(doc, &w, &raw, profile, frozen, source)? {
            Ok(v) => step.applied.push(v),
            Err(r) => step.system.push(rejection(r)),
        }
    }
    Ok(step)
}

/// The checks on one model rewrite: the output contract, per-edit verification, then strictly fewer weighted
/// findings in the rewritten window than in the original one (FR-9.5, DQ-3).
fn accept(
    doc: &Document,
    w: &Window,
    raw: &str,
    profile: &crate::model::PromptProfile,
    frozen: &[Range<usize>],
    source: &Source,
) -> Result<Result<Verified<edit::Proposal>, Rejection>, Error> {
    let verified = match engine::mask::propose(doc, w, raw, profile)
        .and_then(|p| verify::edit(doc, p, frozen, &source.settings.protect))
    {
        Ok(v) => v,
        Err(r) => return Ok(Err(r)),
    };
    let candidate: Arc<str> =
        edit::apply(&doc.src, std::slice::from_ref(&verified), doc.newline).into();
    // The rewritten window in the candidate: `apply` may have normalised the replacement's line endings.
    let start = verified.get().range.start;
    let new_range = start..candidate.len() - (doc.src.len() - verified.get().range.end);
    let (_, _, _, after) = detect_in(candidate, source)?;
    let weight_after = after
        .iter()
        .filter(|f| {
            !f.code.is_system() && f.range.start < new_range.end && new_range.start < f.range.end
        })
        .map(|f| score::weight(f.severity))
        .sum();
    Ok(verify::improvement(w.weight_before, weight_after, w.range.clone()).map(|()| verified))
}

type Detected = (Document, Prose, directives::Suppressions, Vec<Finding>);

/// Parse, extract directives and detect over `text` with the settings of `source`.
fn detect_in(text: Arc<str>, source: &Source) -> Result<Detected, Error> {
    let s = &source.settings;
    let mut d = doc::parse(text, source.format, &s.protect);
    let path = match &source.origin {
        Origin::File(p) => p.clone(),
        Origin::Stdin { as_path } => as_path.clone(),
    };
    let reg = Registry::builtin();
    let sup = directives::extract(&mut d, &path, reg)?;
    let prose = Prose::new(&d);
    let findings = rules::detect(&d, &prose, &sup, &s.enabled(reg));
    Ok((d, prose, sup, findings))
}

/// The findings that pass `--min-severity`. System findings always pass (DQ-4).
fn reported(findings: Vec<Finding>, min: Severity) -> Vec<Finding> {
    findings
        .into_iter()
        .filter(|f| f.code.is_system() || f.severity >= min)
        .collect()
}

/// SLP901 `rewrite-rejected`: always a warning, with no fix (DQ-4, SPEC §8).
fn rejection(r: Rejection) -> Finding {
    Finding {
        code: RuleCode::REWRITE_REJECTED,
        range: r.range,
        message: format!("rewrite rejected ({}): {}", check_name(r.failed), r.detail),
        severity: Severity::Warning,
        fix: None,
    }
}

/// The serialised (camelCase) name of a check.
fn check_name(c: Check) -> &'static str {
    match c {
        Check::Numbers => "numbers",
        Check::Urls => "urls",
        Check::Terms => "terms",
        Check::Placeholders => "placeholders",
        Check::Protected => "protected",
        Check::BlockType => "blockType",
        Check::NewBlockMarker => "newBlockMarker",
        Check::OutputContract => "outputContract",
        Check::NoImprovement => "noImprovement",
        Check::Document => "document",
    }
}

/// Maps a range of the original text into the text with `edits` applied. The range overlaps no applied edit (the
/// plan keeps non-overlapping proposals), so it moves by the length change of every edit that ends before it.
fn shift(r: &Range<usize>, edits: &[Verified<edit::Proposal>]) -> Range<usize> {
    let delta: isize = edits
        .iter()
        .map(Verified::get)
        .filter(|p| p.range.end <= r.start)
        .map(|p| p.replacement.len() as isize - p.range.len() as isize)
        .sum();
    let at = |o: usize| o.saturating_add_signed(delta);
    at(r.start)..at(r.end)
}

fn locate(d: &Document, findings: Vec<Finding>) -> Vec<Located> {
    findings
        .into_iter()
        .map(|finding| {
            let r = finding.range.clone();
            let (line, _) = d.line_col(r.start);
            let line_text = d
                .src
                .get(d.line_range(line))
                .unwrap_or_default()
                .trim_end_matches(['\n', '\r'])
                .to_string();
            Located {
                start: d.line_col(r.start),
                end: d.line_col(r.end),
                matched: d.src.get(r).unwrap_or_default().to_string(),
                line_text,
                finding,
            }
        })
        .collect()
}
