//! `concise` (FR-3.2): one line per finding, `path:line:col: SLP001 [warning] message`, and nothing else.

use std::io::{self, Write};

use super::{FileView, severity_tag};

pub(super) fn render(files: &[FileView<'_>], color: bool, out: &mut dyn Write) -> io::Result<()> {
    for f in files {
        for l in &f.findings {
            writeln!(
                out,
                "{}:{}:{}: {} {} {}",
                f.path,
                l.start.0,
                l.start.1,
                l.finding.code,
                severity_tag(l.finding.severity, color),
                one_line(&l.finding.message)
            )?;
        }
    }
    Ok(())
}

/// A message on one line: line breaks become spaces, so a finding is always exactly one line.
fn one_line(s: &str) -> String {
    s.replace(['\r', '\n'], " ")
}
