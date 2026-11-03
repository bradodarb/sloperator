//! Segmentation counts and compatibility with the original window sentence range.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use sloperator::{
    doc::{self, BlockId, Document, Format, ProtectSettings},
    text::segment::segment,
};
use std::{
    fs,
    ops::Range,
    path::{Path, PathBuf},
    sync::Arc,
};
use unicode_segmentation::UnicodeSegmentation;

fn md(src: &str) -> Document {
    doc::parse(
        Arc::from(src),
        Format::Markdown,
        &ProtectSettings::default(),
    )
}

fn markdown_files() -> Vec<PathBuf> {
    fn visit(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(&path, out);
            } else if path.extension().is_some_and(|e| e == "md") {
                out.push(path);
            }
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    visit(&root.join("fixtures"), &mut files);
    visit(&root.join("eval/paragraphs"), &mut files);
    files.sort();
    assert!(!files.is_empty());
    files
}

#[test]
fn word_count_equals_prose_words_on_every_fixture() {
    for path in markdown_files() {
        let d = md(&fs::read_to_string(&path).unwrap());
        assert_eq!(
            segment(&d).words.len(),
            d.prose_words(),
            "{}",
            path.display()
        );
    }
}

#[test]
fn random_input_never_panics() {
    let alphabet: Vec<char> = "ab Z09.*_~`[]()<>#!?\\|:-\r\n\té\u{fffc}".chars().collect();
    let mut state = 0x1234_5678_u64;
    for case in 0..500 {
        let src: String = (0..200)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                alphabet[state as usize % alphabet.len()]
            })
            .collect();
        let d = md(&src);
        assert_eq!(
            segment(&d).words.len(),
            d.prose_words(),
            "case {case}: {src:?}"
        );
    }
}

#[test]
fn sentence_span_equals_old_sentence_range() {
    for path in markdown_files() {
        let d = md(&fs::read_to_string(&path).unwrap());
        let s = segment(&d);
        for b in &d.blocks {
            let ranges = std::iter::once(b.range.clone()).chain(
                s.words
                    .iter()
                    .filter(|w| b.range.contains(&w.range.start))
                    .map(|w| w.range.clone()),
            );
            for r in ranges {
                assert_eq!(
                    s.sentence_span(&d, b.id, &r),
                    sentence_range(&d, b.id, &r),
                    "{} block {:?} range {:?}",
                    path.display(),
                    b.id,
                    r
                );
            }
        }
    }
}

// Verbatim oracle from engine/window.rs before story 05 removes it.
/// The sentences of `block` that `r` touches, as one source range with surrounding whitespace trimmed.
///
/// Sentences are found on a copy of the block in which line breaks and continuation prefixes are blanked (UAX #29
/// breaks after every line feed otherwise). A boundary inside a protected run is not a boundary.
fn sentence_range(doc: &Document, block: BlockId, r: &Range<usize>) -> Range<usize> {
    let b = doc.blocks[block.0 as usize].range.clone();
    let mut scan = doc.src.as_bytes()[b.clone()].to_vec();
    for byte in scan.iter_mut().filter(|b| **b == b'\n' || **b == b'\r') {
        *byte = b' ';
    }
    for (i, _) in doc.src[b.clone()].match_indices('\n') {
        let ls = b.start + i + 1;
        if ls < b.end {
            let p = doc.continuation_prefix_len(ls, b.end);
            scan[ls - b.start..ls - b.start + p].fill(b' ');
        }
    }
    // Only ASCII bytes were replaced with ASCII bytes, so the copy is still UTF-8.
    let Ok(scan) = String::from_utf8(scan) else {
        return b;
    };
    let mut starts: Vec<usize> = scan
        .split_sentence_bound_indices()
        .map(|(i, _)| i)
        .filter(|&i| i == 0 || !doc.is_protected(&(b.start + i..b.start + i)))
        .collect();
    starts.push(scan.len());
    let (lo, hi) = (
        r.start.saturating_sub(b.start),
        r.end.max(r.start + 1).saturating_sub(b.start),
    );
    let first = starts.partition_point(|&s| s <= lo).saturating_sub(1);
    let last = starts.partition_point(|&s| s < hi).max(first + 1);
    let (s, e) = (starts[first], starts[last.min(starts.len() - 1)]);
    let piece = &scan[s..e];
    let lead = piece.len() - piece.trim_start().len();
    let trail = piece.len() - piece.trim_end().len();
    if lead + trail >= piece.len() {
        return b;
    }
    b.start + s + lead..b.start + e - trail
}
