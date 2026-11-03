//! Report formats (SPEC §18, §23.14): text and concise show where and what (FR-3.2), JSON matches the schema-v1 shape
//! (FR-3.3, FR-5.2, NFR-9), github maps severities and escapes (FR-3.4), no ANSI without colour (FR-3.5), and the order
//! is files by Origin, then source order (FR-3.6).
#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may unwrap (architecture §8)

mod common;

use std::path::PathBuf;
use std::sync::Arc;

use sloperator::cli::ExitCode;
use sloperator::input::Origin;
use sloperator::pipeline::{FileResult, Located};
use sloperator::report::{self, OutputFormat, Summary};
use sloperator::rules::{Finding, Proposal, RuleCode, Severity};
use sloperator::score::Score;
use sloperator::types::ProposalOrigin;

const TEXT: &str = "# Title\n\nWe delve into the data.\n";

fn located(
    code: u16,
    severity: Severity,
    start: (u32, u32),
    end: (u32, u32),
    matched: &str,
    line: &str,
) -> Located {
    Located {
        finding: Finding {
            code: RuleCode(code),
            range: 0..matched.len(),
            message: format!("message for SLP{code:03}"),
            severity,
            fix: None,
        },
        start,
        end,
        matched: matched.to_string(),
        line_text: line.to_string(),
    }
}

/// SLP001 with a safe fix on "delve into" (line 3, columns 4..14).
fn delve() -> Located {
    let mut l = located(
        1,
        Severity::Warning,
        (3, 4),
        (3, 14),
        "delve into",
        "We delve into the data.",
    );
    l.finding.fix = Some(Proposal {
        range: 12..22,
        replacement: "examine".into(),
        origin: ProposalOrigin::Rule(RuleCode(1)),
    });
    l
}

fn result(path: &str, findings: Vec<Located>) -> FileResult {
    FileResult {
        origin: Origin::File(PathBuf::from(path)),
        original: Arc::from(TEXT),
        findings,
        score: Score {
            value: 12.4,
            prose_words: 2984,
            weighted: 37,
        },
        score_before: None,
        fixed: None,
        metrics: sloperator::text::Counts::default(),
        metrics_before: None,
        floors: sloperator::text::Floors::default(),
    }
}

fn render(fmt: OutputFormat, results: &[FileResult], color: bool) -> String {
    let summary = Summary::of(results, 0, &sloperator::text::Floors::default());
    let mut out = Vec::new();
    report::render(fmt, results, &summary, color, &mut out).unwrap();
    String::from_utf8(out).unwrap()
}

#[test]
fn text_shows_path_line_col_code_severity_message_source_line() {
    let out = render(
        OutputFormat::Text,
        &[result("docs/a.md", vec![delve()])],
        false,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(
        lines[0],
        "docs/a.md:3:4 SLP001 [warning] message for SLP001"
    );
    assert_eq!(lines[1], "    We delve into the data.");
    assert_eq!(lines[2], "       ^^^^^^^^^^", "caret under `delve into`");
    assert_eq!(
        lines[3],
        "docs/a.md: score 12.40 (37 weighted / 2,984 words)"
    );
    assert_eq!(
        lines[4],
        "docs/a.md: metrics unavailable (0 sentences, 0 words; needs 5 sentences and 100 words)"
    );
    assert_eq!(lines[5], "");
    assert!(lines[6].starts_with(
        "1 file checked, 1 finding (0 errors, 1 warning, 0 suggestions); overall score "
    ));
    assert_eq!(
        lines[6],
        "1 file checked, 1 finding (0 errors, 1 warning, 0 suggestions); overall score 12.40 (37 weighted / 2,984 words)"
    );
    assert_eq!(
        lines[7],
        "overall metrics unavailable (0 sentences, 0 words; needs 5 sentences and 100 words)"
    );
    assert_eq!(lines.len(), 8);
}

#[test]
fn concise_one_line_per_finding() {
    let mut multi = located(3, Severity::Suggestion, (1, 3), (1, 8), "Title", "# Title");
    multi.finding.message = "line one\nline two".into();
    let results = [result("docs/a.md", vec![delve(), multi])];
    let out = render(OutputFormat::Concise, &results, false);
    assert_eq!(
        out,
        "docs/a.md:1:3: SLP003 [suggestion] line one line two\n\
         docs/a.md:3:4: SLP001 [warning] message for SLP001\n"
    );
}

#[test]
fn json_shape_matches_schema() {
    let mut fixed = result("docs/a.md", vec![delve()]);
    fixed.score_before = Some(Score {
        value: 20.0,
        prose_words: 2984,
        weighted: 60,
    });
    let mut rejected = located(901, Severity::Warning, (1, 1), (1, 2), "x", "x");
    rejected.finding.message = "rewrite rejected (numbers): 42".into();
    let results = [fixed, result("docs/b.md", vec![rejected])];
    let out = render(OutputFormat::Json, &results, true);
    assert!(out.ends_with('\n') && !out.contains('\x1b'));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert_eq!(v["schemaVersion"], 1);
    assert_eq!(v["ruleSetVersion"], "0.2.0");
    let a = &v["files"][0];
    assert_eq!(a["path"], "docs/a.md");
    assert_eq!(
        a["score"],
        serde_json::json!({"value": 12.4, "proseWords": 2984, "weighted": 37})
    );
    assert_eq!(a["scoreBefore"]["value"], 20.0);
    assert_eq!(
        a["findings"][0],
        serde_json::json!({"code": "SLP001", "name": "ai-vocabulary-replaceable", "severity": "warning",
            "path": "docs/a.md", "start": {"line": 3, "column": 4}, "end": {"line": 3, "column": 14},
            "message": "message for SLP001", "matched": "delve into",
            "fix": {"available": true, "class": "safe"}})
    );
    let b = &v["files"][1];
    assert_eq!(b["scoreBefore"], serde_json::Value::Null);
    assert_eq!(
        b["findings"][0]["path"], "docs/b.md",
        "every finding carries its own path"
    );
    assert_eq!(b["findings"][0]["name"], "rewrite-rejected");
    assert_eq!(
        b["findings"][0]["fix"],
        serde_json::json!({"available": false, "class": "none"})
    );

    assert_eq!(
        v["summary"],
        serde_json::json!({"files": 2, "findings": 2, "bySeverity": [0, 2, 0], "fixedFiles": 0, "rejected": 1,
            "skipped": 0, "score": {"value": 12.4, "proseWords": 5968, "weighted": 74},
            "metrics": {"words": 0, "sentences": 0, "avgSentenceLength": null, "gradeLevel": null,
                "sentenceLengthSd": null, "sentenceLengthCv": null, "typeTokenRatio": null}, "errors": 0})
    );
    let keys: Vec<&String> = v.as_object().unwrap().keys().collect();
    assert_eq!(
        keys.len(),
        4,
        "schemaVersion, ruleSetVersion, files, summary: {keys:?}"
    );
}

#[test]
fn github_annotations_and_escaping() {
    let mut err = located(1, Severity::Error, (1, 3), (1, 8), "Title", "# Title");
    err.finding.message = "100% sure\r\nreally: yes, ok".into();
    let sug = located(
        16,
        Severity::Suggestion,
        (3, 1),
        (3, 3),
        "We",
        "We delve into the data.",
    );
    let results = [
        result("docs/a,b:c.md", vec![err]),
        result("docs/z.md", vec![delve(), sug]),
    ];
    let out = render(OutputFormat::Github, &results, false);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(
        lines[0],
        "::error file=docs/a%2Cb%3Ac.md,line=1,col=3,endLine=1,endColumn=8,\
         title=SLP001 ai-vocabulary-replaceable::100%25 sure%0D%0Areally: yes, ok"
    );
    assert!(
        lines[1].starts_with(
            "::notice file=docs/z.md,line=3,col=1,endLine=3,endColumn=3,title=SLP016 "
        )
    );
    assert!(
        lines[2].starts_with(
            "::warning file=docs/z.md,line=3,col=4,endLine=3,endColumn=14,title=SLP001 "
        )
    );
    assert_eq!(lines.len(), 3);

    assert_eq!(report::gh_escape("a%b\r\nc:d,e", false), "a%25b%0D%0Ac:d,e");
    assert_eq!(
        report::gh_escape("a%b\r\nc:d,e", true),
        "a%25b%0D%0Ac%3Ad%2Ce"
    );
}

#[test]
fn no_color_when_not_terminal() {
    let results = [result("docs/a.md", vec![delve()])];
    for fmt in [
        OutputFormat::Text,
        OutputFormat::Concise,
        OutputFormat::Json,
        OutputFormat::Github,
    ] {
        assert!(
            !render(fmt, &results, false).contains('\x1b'),
            "{fmt:?} without colour has ANSI"
        );
    }
    // The flag is what decides: with colour on, text and concise do colour.
    assert!(render(OutputFormat::Text, &results, true).contains('\x1b'));
    assert!(render(OutputFormat::Concise, &results, true).contains('\x1b'));
}

#[test]
fn stable_order() {
    let early = located(16, Severity::Suggestion, (1, 3), (1, 8), "Title", "# Title");
    let stdin = FileResult {
        origin: Origin::Stdin {
            as_path: PathBuf::from("-"),
        },
        ..result("-", vec![delve()])
    };
    // Shuffled files and findings out of source order.
    let results = [
        result("docs/b.md", vec![delve(), early.clone()]),
        stdin,
        result("docs/a.md", vec![delve(), early]),
    ];
    let out = render(OutputFormat::Concise, &results, false);
    let at: Vec<&str> = out.lines().map(|l| l.split(": ").next().unwrap()).collect();
    assert_eq!(
        at,
        [
            "docs/a.md:1:3",
            "docs/a.md:3:4",
            "docs/b.md:1:3",
            "docs/b.md:3:4",
            "-:3:4"
        ],
        "files by Origin (File before Stdin), findings in source order"
    );
    // The same results in any order render byte-identically.
    let mut reversed: Vec<FileResult> = results.into_iter().collect();
    reversed.reverse();
    assert_eq!(render(OutputFormat::Json, &reversed, false), {
        reversed.reverse();
        render(OutputFormat::Json, &reversed, false)
    });
}

/// FR-3.1: an unknown `--output-format` exits 2 and names the value.
#[test]
fn unknown_output_format_exits_two() {
    let clean = common::fixture("clean.md");
    let (code, _, err) = common::run_cli(
        &["check", "--output-format", "yaml", &clean.to_string_lossy()],
        "",
    );
    assert_eq!(code, ExitCode::Failure, "stderr: {err}");
    assert!(err.contains("yaml"), "stderr: {err}");
}

fn metric_counts() -> sloperator::text::Counts {
    sloperator::text::Counts {
        words: 100,
        sentences: 5,
        syllables: 150,
        sentence_len_sq: 2200,
        types: (0..52)
            .map(|i| format!("word{i}").into_boxed_str())
            .collect(),
    }
}

#[test]
fn text_metrics_line_lists_values() {
    let mut f = result("a.md", vec![]);
    f.metrics = metric_counts();
    let out = render(OutputFormat::Text, &[f], false);
    assert!(out.contains("a.md: metrics avg sentence 20.00 words, grade 9.91, sentence-length variation 6.32 (cv 0.32), type/token 0.52\n"));
}

#[test]
fn text_fix_prints_metrics_before() {
    let mut f = result("a.md", vec![]);
    f.metrics_before = Some(metric_counts());
    f.metrics = metric_counts();
    let out = render(OutputFormat::Text, &[f], false);
    let lines: Vec<_> = out.lines().collect();
    assert!(lines[0].starts_with("a.md: score "));
    assert!(lines[1].starts_with("a.md: metrics before avg sentence 20.00"));
    assert!(lines[2].starts_with("a.md: metrics avg sentence 20.00"));
}

#[test]
fn json_file_has_metrics_and_metrics_before() {
    let mut f = result("a.md", vec![]);
    f.metrics = metric_counts();
    f.metrics_before = Some(metric_counts());
    let out = render(OutputFormat::Json, &[f], false);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["schemaVersion"], 1);
    assert_eq!(v["files"][0]["metrics"]["gradeLevel"], 9.91);
    assert_eq!(v["files"][0]["metricsBefore"]["words"], 100);
    assert_eq!(
        v["files"][0]["score"],
        serde_json::json!({"value":12.4,"proseWords":2984,"weighted":37})
    );
    let out = render(OutputFormat::Json, &[result("a.md", vec![])], false);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(v["files"][0].get("metricsBefore").unwrap().is_null());
}

#[test]
fn unavailable_text_names_configured_floors() {
    let mut f = result("a.md", vec![]);
    f.metrics = metric_counts();
    f.metrics.sentences = 6;
    f.floors = sloperator::text::Floors {
        sentences: 10,
        words: 200,
    };
    let out = render(OutputFormat::Text, &[f], false);
    assert!(out.contains(
        "a.md: metrics unavailable (6 sentences, 100 words; needs 10 sentences and 200 words)"
    ));
    let mut f = result("a.md", vec![]);
    f.metrics = metric_counts();
    f.floors = sloperator::text::Floors {
        sentences: 10,
        words: 100,
    };
    let out = render(OutputFormat::Json, &[f], false);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    for key in [
        "avgSentenceLength",
        "gradeLevel",
        "sentenceLengthSd",
        "sentenceLengthCv",
        "typeTokenRatio",
    ] {
        assert!(v["files"][0]["metrics"][key].is_null());
    }
}

#[test]
fn concise_and_github_print_no_metrics() {
    let mut f = result("a.md", vec![delve()]);
    f.metrics = metric_counts();
    f.metrics_before = Some(metric_counts());
    for fmt in [OutputFormat::Concise, OutputFormat::Github] {
        assert!(!render(fmt, std::slice::from_ref(&f), false).contains("metrics"));
    }
}

#[test]
fn summary_merges_counts_and_uses_run_floors() {
    let mut a = result("a.md", vec![]);
    a.metrics = metric_counts();
    let mut b = result("b.md", vec![]);
    b.metrics = metric_counts();
    b.metrics.words = 200;
    b.metrics.sentence_len_sq = 8000;
    let results = [a, b];
    let floors = sloperator::text::Floors {
        sentences: 10,
        words: 100,
    };
    let summary = Summary::of(&results, 0, &floors);
    assert_eq!(summary.metrics.words, 300);
    assert_eq!(summary.metrics.avg_sentence_length, Some(30.0));
    assert_eq!(summary.metrics.type_token_ratio, Some(0.17));
    let floors = sloperator::text::Floors {
        sentences: 11,
        words: 400,
    };
    let summary = Summary::of(&results, 0, &floors);
    let mut out = Vec::new();
    report::render(OutputFormat::Text, &results, &summary, false, &mut out).unwrap();
    assert!(String::from_utf8(out).unwrap().contains(
        "overall metrics unavailable (10 sentences, 300 words; needs 11 sentences and 400 words)"
    ));
}
