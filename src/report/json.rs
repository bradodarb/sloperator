//! `json` (FR-3.3, FR-5.2, NFR-9): one document, camelCase keys, `schemaVersion: 1` (architecture §6, SPEC §18).
//! Every finding carries its own `path`, equal to its file's, so a consumer that flattens findings needs no join.

use std::io::{self, Write};

use serde::Serialize;

use super::{FileView, Summary, rule_info};
use crate::rules::registry::Registry;
use crate::rules::{FixClass, RuleCode, Severity};
use crate::score::Score;
use crate::text::Metrics;

/// The JSON schema version; bump it on any breaking change to the shape.
pub(super) const SCHEMA_VERSION: u32 = 1;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Doc<'a> {
    schema_version: u32,
    rule_set_version: &'static str,
    files: Vec<File<'a>>,
    summary: &'a Summary,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct File<'a> {
    path: &'a str,
    score: Score,
    score_before: Option<Score>,
    metrics: Metrics,
    metrics_before: Option<Metrics>,
    findings: Vec<Finding<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Finding<'a> {
    code: RuleCode,
    name: &'static str,
    severity: Severity,
    path: &'a str,
    start: Pos,
    end: Pos,
    message: &'a str,
    matched: &'a str,
    fix: Fix,
}

#[derive(Serialize)]
struct Pos {
    line: u32,
    column: u32,
}

#[derive(Serialize)]
struct Fix {
    available: bool,
    class: FixClass,
}

pub(super) fn render(
    files: &[FileView<'_>],
    summary: &Summary,
    out: &mut dyn Write,
) -> io::Result<()> {
    let files = files
        .iter()
        .map(|f| {
            let findings = f
                .findings
                .iter()
                .map(|l| {
                    let (name, class) = rule_info(l.finding.code)?;
                    Ok(Finding {
                        code: l.finding.code,
                        name,
                        severity: l.finding.severity,
                        path: &f.path,
                        start: Pos {
                            line: l.start.0,
                            column: l.start.1,
                        },
                        end: Pos {
                            line: l.end.0,
                            column: l.end.1,
                        },
                        message: &l.finding.message,
                        matched: &l.matched,
                        // A safe fix exists when the rule attached one; a model fix whenever the rule is model-class.
                        // Safe-class findings without a fix may still be rewritten by the model.
                        fix: Fix {
                            available: match class {
                                FixClass::Safe => l.finding.fix.is_some(),
                                FixClass::Model => true,
                                FixClass::None => false,
                            },
                            class,
                        },
                    })
                })
                .collect::<io::Result<Vec<_>>>()?;
            Ok(File {
                path: &f.path,
                score: f.result.score,
                score_before: f.result.score_before,
                metrics: f.result.metrics.derive(&f.result.floors),
                metrics_before: f
                    .result
                    .metrics_before
                    .as_ref()
                    .map(|c| c.derive(&f.result.floors)),
                findings,
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    let doc = Doc {
        schema_version: SCHEMA_VERSION,
        rule_set_version: Registry::builtin().version,
        files,
        summary,
    };
    serde_json::to_writer(&mut *out, &doc)?;
    writeln!(out)
}
