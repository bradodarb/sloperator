//! Calibration corpus guard (classical-nlp SPEC §19): public-domain prose above the metric floors.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::sync::Arc;

use sloperator::doc::{self, BlockKind, Format, ProtectSettings};
use sloperator::text::{Counts, Floors, Metrics, Prose};

fn metric_row(name: &str, m: Metrics) -> String {
    format!(
        "| {name} | {} | {} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} |",
        m.words,
        m.sentences,
        m.avg_sentence_length.unwrap(),
        m.grade_level.unwrap(),
        m.sentence_length_sd.unwrap(),
        m.sentence_length_cv.unwrap(),
        m.type_token_ratio.unwrap(),
    )
}

#[test]
fn calibration_corpus_is_licensed_and_measurable() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("eval/calibration");
    let readme = std::fs::read_to_string(dir.join("README.md")).unwrap();
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "md") && !p.ends_with("README.md"))
        .collect();
    files.sort();
    assert!(files.len() >= 12, "the corpus has {} files", files.len());
    let mut total = Counts::default();
    let mut bytes = 0;
    let mut rows = Vec::new();
    for path in files {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let row = readme
            .lines()
            .find(|l| l.starts_with(&format!("| {name} |")))
            .expect("file not in README");
        assert!(
            row.contains("| public domain |"),
            "{name}: licence is not public domain"
        );
        let text = std::fs::read_to_string(&path).unwrap();
        bytes += text.len();
        let d = doc::parse(Arc::from(text), Format::Markdown, &ProtectSettings::default());
        assert!(
            d.blocks.iter().all(|b| b.kind == BlockKind::Paragraph),
            "{name}: non-paragraph block"
        );
        let prose = Prose::new(&d);
        let counts = prose.counts(&d);
        let m = counts.derive(&Floors::default());
        assert!(
            m.avg_sentence_length.is_some(),
            "{name} is below the metric floors"
        );
        assert!(
            row.ends_with(&format!("| {} |", m.words)),
            "{name}: licence table word count is stale"
        );
        println!("{name}: {}", serde_json::to_string(&m).unwrap());
        rows.push(metric_row(&name, m));
        total.merge(counts);
    }
    assert!(
        (1_000_000..=3_000_000).contains(&bytes),
        "corpus is {bytes} bytes"
    );
    let overall = total.derive(&Floors::default());
    println!("overall: {}", serde_json::to_string(&overall).unwrap());
    rows.push(metric_row("overall", overall));
    for row in rows {
        println!("{row}");
        assert!(
            readme.lines().any(|line| line == row),
            "README metric row missing or stale: {row}"
        );
    }
}
