//! `github` (FR-3.4): one GitHub Actions workflow command per finding, e.g.
//! `::warning file=docs/a.md,line=3,col=5,endLine=3,endColumn=15,title=SLP001 ai-vocabulary-replaceable::message`.

use std::io::{self, Write};

use super::{FileView, gh_escape, rule_info};
use crate::rules::Severity;

pub(super) fn render(files: &[FileView<'_>], out: &mut dyn Write) -> io::Result<()> {
    for f in files {
        for l in &f.findings {
            let (name, _) = rule_info(l.finding.code)?;
            writeln!(
                out,
                "::{} file={},line={},col={},endLine={},endColumn={},title={}::{}",
                command(l.finding.severity),
                gh_escape(&f.path, true),
                l.start.0,
                l.start.1,
                l.end.0,
                l.end.1,
                gh_escape(&format!("{} {name}", l.finding.code), true),
                gh_escape(&l.finding.message, false)
            )?;
        }
    }
    Ok(())
}

/// suggestion → `notice`, warning → `warning`, error → `error`.
fn command(s: Severity) -> &'static str {
    match s {
        Severity::Suggestion => "notice",
        Severity::Warning => "warning",
        Severity::Error => "error",
    }
}
