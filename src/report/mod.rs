//! Report (SPEC §18, architecture §4.12): renders file results as `text`, `concise`, `json` or `github`.
//!
//! Output order is stable (FR-3.6): files by `Origin`, then findings in source order. `report` never uses `engine`,
//! `write` or config discovery (layering). Colour is a plain `bool` the caller decides (DQ-16, FR-3.5); only `text`
//! and `concise` ever colour, and only when it is `true`.

use std::io::{self, Write};
use std::path::Path;

use crate::input::Origin;
use crate::pipeline::{FileResult, Located};
use crate::rules::registry::Registry;
use crate::rules::{FixClass, RuleCode, Severity};
use crate::score::{self, Score};
use crate::text::{Counts, Floors, Metrics};

mod concise;
mod github;
mod json;
mod text;

/// The `--output-format` values (FR-3.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum OutputFormat {
    #[default]
    Text,
    Concise,
    Json,
    Github,
}

/// The run totals. `by_severity` is indexed suggestion, warning, error. `rejected` and `skipped` count SLP901 and
/// SLP902; `errors` counts paths that failed (an unreadable file, a bad directive).
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub files: usize,
    pub findings: usize,
    pub by_severity: [usize; 3],
    pub fixed_files: usize,
    pub rejected: usize,
    pub skipped: usize,
    pub score: Score,
    pub metrics: Metrics,
    pub errors: usize,
    #[serde(skip)]
    floors: Floors,
}

impl Summary {
    /// Totals over `results`, with the overall word-weighted score (`score::overall`) and `errors` failed paths.
    pub fn of(results: &[FileResult], errors: usize, floors: &Floors) -> Summary {
        let all = || results.iter().flat_map(|r| &r.findings);
        let mut by_severity = [0; 3];
        for l in all() {
            by_severity[severity_index(l.finding.severity)] += 1;
        }
        let count = |code: RuleCode| all().filter(|l| l.finding.code == code).count();
        let scores: Vec<Score> = results.iter().map(|r| r.score).collect();
        let mut counts = Counts::default();
        for r in results {
            counts.merge(&r.metrics);
        }
        Summary {
            files: results.len(),
            findings: all().count(),
            by_severity,
            fixed_files: results.iter().filter(|r| r.fixed.is_some()).count(),
            rejected: count(RuleCode::REWRITE_REJECTED),
            skipped: count(RuleCode::WINDOW_SKIPPED),
            score: score::overall(&scores),
            metrics: counts.derive(floors),
            errors,
            floors: *floors,
        }
    }
}

/// Writes `results` in `fmt` to `out`. Files are ordered by `Origin` and findings by source position, so the output
/// does not depend on the order the caller collected them in (FR-3.6).
///
/// # Errors
/// A failed write, or a finding whose code is neither a built-in rule nor a system code.
pub fn render(
    fmt: OutputFormat,
    results: &[FileResult],
    summary: &Summary,
    color: bool,
    out: &mut dyn Write,
) -> io::Result<()> {
    let files = ordered(results);
    match fmt {
        OutputFormat::Text => text::render(&files, summary, color, out),
        OutputFormat::Concise => concise::render(&files, color, out),
        OutputFormat::Json => json::render(&files, summary, out),
        OutputFormat::Github => github::render(&files, out),
    }
}

/// GitHub workflow-command escaping. Data: `%`→`%25`, `\r`→`%0D`, `\n`→`%0A`. Properties also escape `:`→`%3A`,
/// `,`→`%2C`.
pub fn gh_escape(s: &str, property: bool) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '%' => out.push_str("%25"),
            '\r' => out.push_str("%0D"),
            '\n' => out.push_str("%0A"),
            ':' if property => out.push_str("%3A"),
            ',' if property => out.push_str("%2C"),
            c => out.push(c),
        }
    }
    out
}

/// One file in output order, with its findings in source order.
struct FileView<'a> {
    result: &'a FileResult,
    path: String,
    findings: Vec<&'a Located>,
}

fn ordered(results: &[FileResult]) -> Vec<FileView<'_>> {
    let mut files: Vec<&FileResult> = results.iter().collect();
    files.sort_by(|a, b| a.origin.cmp(&b.origin)); // stable: equal origins keep the caller's order
    files
        .into_iter()
        .map(|result| {
            let mut findings: Vec<&Located> = result.findings.iter().collect();
            findings.sort_by_key(|l| (l.start, l.end)); // stable: ties keep the pipeline's order
            FileView {
                result,
                path: display_path(&result.origin),
                findings,
            }
        })
        .collect()
}

fn display_path(origin: &Origin) -> String {
    let p: &Path = match origin {
        Origin::File(p) => p,
        Origin::Stdin { as_path } => as_path,
    };
    p.display().to_string()
}

/// A finding's rule name and fix class: the registry's for a rule, `rewrite-rejected` / `window-skipped` with class
/// `none` for the system codes (SPEC §18).
fn rule_info(code: RuleCode) -> io::Result<(&'static str, FixClass)> {
    match code {
        RuleCode::REWRITE_REJECTED => Ok(("rewrite-rejected", FixClass::None)),
        RuleCode::WINDOW_SKIPPED => Ok(("window-skipped", FixClass::None)),
        _ => Registry::builtin()
            .get(code)
            .map(|r| (r.meta().name, r.meta().fix))
            .ok_or_else(|| io::Error::other(format!("{code} is not a built-in rule"))),
    }
}

fn severity_name(s: Severity) -> &'static str {
    match s {
        Severity::Suggestion => "suggestion",
        Severity::Warning => "warning",
        Severity::Error => "error",
    }
}

fn severity_index(s: Severity) -> usize {
    match s {
        Severity::Suggestion => 0,
        Severity::Warning => 1,
        Severity::Error => 2,
    }
}

/// `[warning]`, coloured by severity when `color` is set.
fn severity_tag(s: Severity, color: bool) -> String {
    let tag = format!("[{}]", severity_name(s));
    if !color {
        return tag;
    }
    let code = match s {
        Severity::Suggestion => "36",
        Severity::Warning => "33",
        Severity::Error => "31",
    };
    paint(&tag, code)
}

/// Wraps `s` in an SGR sequence. Callers only reach this when colour is on.
fn paint(s: &str, sgr: &str) -> String {
    format!("\x1b[{sgr}m{s}\x1b[0m")
}
