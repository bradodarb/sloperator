//! `text` (FR-3.2, FR-5.2): each finding with its source line and a caret line, a score line per file, then the run
//! summary.

use std::io::{self, Write};

use super::{FileView, Summary, paint, severity_tag};
use crate::pipeline::Located;
use crate::score::Score;
use crate::text::{Floors, Metrics};

pub(super) fn render(
    files: &[FileView<'_>],
    summary: &Summary,
    color: bool,
    out: &mut dyn Write,
) -> io::Result<()> {
    for f in files {
        for l in &f.findings {
            let at = format!("{}:{}:{}", f.path, l.start.0, l.start.1);
            let at = if color { paint(&at, "1") } else { at };
            writeln!(
                out,
                "{at} {} {} {}",
                l.finding.code,
                severity_tag(l.finding.severity, color),
                l.finding.message
            )?;
            writeln!(out, "    {}", l.line_text)?;
            let caret = "^".repeat(caret_width(l));
            let caret = if color { paint(&caret, "1;31") } else { caret };
            writeln!(out, "    {}{caret}", caret_indent(l))?;
        }
        writeln!(out, "{}: score {}", f.path, score_text(&f.result.score))?;
        let floors = &f.result.floors;
        if let Some(before) = &f.result.metrics_before {
            writeln!(
                out,
                "{}: metrics before {}",
                f.path,
                metrics_text(&before.derive(floors), floors)
            )?;
        }
        writeln!(
            out,
            "{}: metrics {}",
            f.path,
            metrics_text(&f.result.metrics.derive(floors), floors)
        )?;
    }
    writeln!(out)?;
    writeln!(out, "{}", summary_line(summary))?;
    writeln!(
        out,
        "overall metrics {}",
        metrics_text(&summary.metrics, &summary.floors)
    )
}

/// Whitespace up to the start column, keeping tabs so the caret lines up under a tab-indented line.
fn caret_indent(l: &Located) -> String {
    let skip = (l.start.1 as usize).saturating_sub(1);
    l.line_text
        .chars()
        .take(skip)
        .map(|c| if c == '\t' { '\t' } else { ' ' })
        .collect()
}

/// The characters the finding covers on its first line; at least one.
fn caret_width(l: &Located) -> usize {
    let (start, end) = (l.start.1 as usize, l.end.1 as usize);
    let width = if l.end.0 == l.start.0 {
        end.saturating_sub(start)
    } else {
        (l.line_text.chars().count() + 1).saturating_sub(start)
    };
    width.max(1)
}

/// `12.40 (37 weighted / 2,984 words)`.
fn score_text(s: &Score) -> String {
    format!(
        "{:.2} ({} weighted / {} words)",
        s.value,
        thousands(s.weighted as usize),
        thousands(s.prose_words)
    )
}

fn summary_line(s: &Summary) -> String {
    let [sug, warn, err] = s.by_severity;
    let mut line = format!(
        "{} checked, {} ({}, {}, {}); overall score {}",
        plural(s.files, "file"),
        plural(s.findings, "finding"),
        plural(err, "error"),
        plural(warn, "warning"),
        plural(sug, "suggestion"),
        score_text(&s.score)
    );
    for (n, what) in [
        (s.fixed_files, "fixed"),
        (s.rejected, "rewrite(s) rejected"),
        (s.skipped, "window(s) skipped"),
        (s.errors, "path(s) failed"),
    ] {
        if n > 0 {
            line.push_str(&format!("; {n} {what}"));
        }
    }
    line
}

fn plural(n: usize, word: &str) -> String {
    format!("{} {word}{}", thousands(n), if n == 1 { "" } else { "s" })
}

/// `2984` → `2,984`.
fn thousands(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Values derived with the same floors used to explain an unavailable sample.
fn metrics_text(m: &Metrics, floors: &Floors) -> String {
    match (
        m.avg_sentence_length,
        m.grade_level,
        m.sentence_length_sd,
        m.sentence_length_cv,
        m.type_token_ratio,
    ) {
        (Some(avg), Some(grade), Some(sd), Some(cv), Some(ttr)) => format!(
            "avg sentence {avg:.2} words, grade {grade:.2}, sentence-length variation {sd:.2} (cv {cv:.2}), type/token {ttr:.2}"
        ),
        _ => format!(
            "unavailable ({}, {}; needs {} sentences and {} words)",
            plural(
                usize::try_from(m.sentences).unwrap_or(usize::MAX),
                "sentence"
            ),
            plural(usize::try_from(m.words).unwrap_or(usize::MAX), "word"),
            floors.sentences,
            floors.words,
        ),
    }
}
