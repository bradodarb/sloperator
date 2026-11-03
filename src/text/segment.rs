//! Sentences and words over prose (classical-nlp SPEC §4, ADR-03). Pure: a function of the `Document`, no I/O.
//! `segment` is the only function that scans a block for sentence bounds; everything else reads its result.

use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

use crate::doc::{BlockId, Document, OBJ, SpanKind};

/// One sentence of one block. `range` is its source bytes with surrounding whitespace trimmed (trimmed on the
/// blanked copy, so a continuation prefix such as `> ` is never inside it); `words` indexes [`Segments::words`]. A
/// sentence with no prose word (only inline code, say) has an empty `words` range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sentence {
    pub block: BlockId,
    pub range: Range<usize>,
    pub words: Range<u32>,
}

/// One `unicode_words()` word of a prose span. `sentence_initial` is true for the first word of a sentence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Word {
    pub range: Range<usize>,
    pub sentence_initial: bool,
}

/// Owned (no borrow of the Document), so it sits beside the Document in pipeline values (ADR-02). `sentences` and
/// `words` are in source order (blocks are ordered and disjoint, `src/doc/mod.rs:166-167`); a word in no block
/// (bytes between blocks) is in `words` but in no sentence.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Segments {
    pub sentences: Vec<Sentence>,
    pub words: Vec<Word>,
    /// Indexed by `BlockId`: the block's raw scan starts relative to its start, with the blanked length appended
    /// (the `starts` vector `src/engine/window.rs:258-263` builds today). Whitespace-only pieces stay in it. Empty
    /// for a block whose blanked copy could not be built.
    bounds: Vec<Vec<usize>>,
}

impl Segments {
    /// The index of the last word that ends at or before source byte `src`.
    pub fn word_before(&self, src: usize) -> Option<usize> {
        self.words
            .partition_point(|w| w.range.end <= src)
            .checked_sub(1)
    }

    /// The index of the word that starts exactly at source byte `src`.
    pub fn word_at(&self, src: usize) -> Option<usize> {
        let i = self.words.partition_point(|w| w.range.start < src);
        (self.words.get(i)?.range.start == src).then_some(i)
    }

    /// The index of the sentence that holds word `word`; `None` for a word in no sentence.
    pub fn sentence_of(&self, word: usize) -> Option<usize> {
        let w = u32::try_from(word).ok()?;
        let i = self.sentences.partition_point(|s| s.words.end <= w);
        self.sentences
            .get(i)
            .filter(|s| s.words.contains(&w))
            .map(|_| i)
    }

    /// Sentence start offsets of `block`, relative to its start (FR-1.2, FR-1.4). A slice of the cache: no scan.
    pub fn sentence_starts(&self, block: BlockId) -> &[usize] {
        match self.bounds.get(block.0 as usize) {
            Some(b) if !b.is_empty() => &b[..b.len() - 1],
            _ => &[0],
        }
    }

    /// The sentences of `block` that `r` touches, as one source range with surrounding whitespace trimmed: today's
    /// `sentence_range` (`src/engine/window.rs:241-277`) as arithmetic over the cached bounds (FR-1.4, NFR-2). The
    /// range arithmetic is unchanged; the trim is the first touched sentence's trimmed start to the last one's
    /// trimmed end, which is what trimming the joined pieces gave. The block range when every touched piece is
    /// whitespace. No scan.
    pub fn sentence_span(&self, doc: &Document, block: BlockId, r: &Range<usize>) -> Range<usize> {
        let Some(b) = doc.blocks.get(block.0 as usize).map(|b| b.range.clone()) else {
            return r.clone();
        };
        let starts = match self.bounds.get(block.0 as usize) {
            Some(s) if s.len() >= 2 => s,
            _ => return b,
        };
        let (lo, hi) = (
            r.start.saturating_sub(b.start),
            r.end.max(r.start + 1).saturating_sub(b.start),
        );
        let first = starts.partition_point(|&s| s <= lo).saturating_sub(1);
        let last = starts.partition_point(|&s| s < hi).max(first + 1);
        let (s, e) = (
            b.start + starts[first],
            b.start + starts[last.min(starts.len() - 1)],
        );
        let i = self.sentences.partition_point(|x| x.range.start < s);
        let j = self.sentences.partition_point(|x| x.range.end <= e);
        match (
            self.sentences.get(i),
            j.checked_sub(1).and_then(|k| self.sentences.get(k)),
        ) {
            (Some(a), Some(z)) if i < j && a.block == block => a.range.start..z.range.end,
            _ => b,
        }
    }
}

#[cfg(test)]
thread_local! {
    /// Block scans run on this thread (NFR-2 test hook, architecture §4.8). Per thread and absent from production
    /// builds, so NFR-8's "no mutable global state" holds.
    static SCANS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// How many block scans this thread has run so far (in-crate tests only).
#[cfg(test)]
pub(crate) fn scans() -> u32 {
    SCANS.with(std::cell::Cell::get)
}

/// The block's bytes with line breaks and continuation prefixes blanked to spaces (UAX #29 breaks after every line
/// feed otherwise). `None` for an unknown block. Only ASCII bytes are replaced with ASCII bytes, so the copy is UTF-8
/// and has the block's length.
fn blanked(doc: &Document, block: BlockId) -> Option<String> {
    let b = doc.blocks.get(block.0 as usize)?.range.clone();
    let mut scan = doc.src.as_bytes().get(b.clone())?.to_vec();
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
    String::from_utf8(scan).ok()
}

/// UAX #29 sentence starts in a blanked block copy, relative to `block_start`; a bound strictly inside a protected
/// run is not a bound.
fn scan_starts(doc: &Document, block_start: usize, scan: &str) -> Vec<usize> {
    #[cfg(test)]
    SCANS.with(|c| c.set(c.get() + 1));
    scan.split_sentence_bound_indices()
        .map(|(i, _)| i)
        .filter(|&i| i == 0 || !doc.is_protected(&(block_start + i..block_start + i)))
        .collect()
}

/// Every block's sentences and every prose word (FR-1.1), with each block's raw bounds cached for window planning.
/// One scan per block. Total: malformed input gives fewer sentences, never a panic.
pub fn segment(doc: &Document) -> Segments {
    let mut sentences: Vec<Sentence> = Vec::new();
    let mut raw: Vec<Range<usize>> = Vec::new(); // untrimmed sentence ranges, parallel to `sentences`
    let mut bounds: Vec<Vec<usize>> = vec![Vec::new(); doc.blocks.len()];
    for block in &doc.blocks {
        let Some(scan) = blanked(doc, block.id) else {
            continue;
        };
        let b0 = block.range.start;
        let mut starts = scan_starts(doc, b0, &scan);
        starts.push(scan.len());
        for pair in starts.windows(2) {
            let (s, e) = (pair[0], pair[1]);
            let piece = &scan[s..e];
            let lead = piece.len() - piece.trim_start().len();
            let trail = piece.len() - piece.trim_end().len();
            if lead + trail >= piece.len() {
                continue;
            }
            sentences.push(Sentence {
                block: block.id,
                range: b0 + s + lead..b0 + e - trail,
                words: 0..0,
            });
            raw.push(b0 + s..b0 + e);
        }
        if let Some(slot) = bounds.get_mut(block.id.0 as usize) {
            *slot = starts;
        }
    }
    let mut words: Vec<Word> = Vec::new();
    let mut k = 0;
    for span in doc.spans.iter().filter(|s| s.kind == SpanKind::Prose) {
        for (off, w) in doc.src[span.range.clone()].unicode_word_indices() {
            let at = span.range.start + off;
            while k < raw.len() && raw[k].end <= at {
                k += 1;
            }
            let i = u32::try_from(words.len()).unwrap_or(u32::MAX);
            let mut initial = false;
            if let Some(s) = sentences.get_mut(k)
                && raw[k].start <= at
                && Some(s.block) == span.block
            {
                if s.words.is_empty() {
                    s.words = i..i + 1;
                    initial = true;
                } else {
                    s.words.end = i + 1;
                }
            }
            words.push(Word {
                range: at..at + w.len(),
                sentence_initial: initial,
            });
        }
    }
    // A sentence with no word gets an empty range at the running end, so `words.end` never decreases and
    // `sentence_of` can binary-search.
    let mut end = 0;
    for s in &mut sentences {
        if s.words.is_empty() {
            s.words = end..end;
        }
        end = s.words.end;
    }
    Segments {
        sentences,
        words,
        bounds,
    }
}

/// Whether the text byte `at` starts a sentence: only whitespace before it, or `.`, `!` or `?` is the nearest
/// character before it. A non-prose run ([`OBJ`]) at the start of a line is block markup (`# `, `- `, `> `), so it
/// is skipped. One in the middle of a line is not, so a phrase after inline code or a link is not sentence-initial.
pub fn starts_sentence(text: &str, at: usize) -> bool {
    let mut before = text.get(..at).unwrap_or_default();
    loop {
        before = before.trim_end();
        if before.is_empty() {
            return true;
        }
        if !before.ends_with(OBJ) {
            return before.ends_with(['.', '!', '?']);
        }
        let line = before.trim_end_matches(OBJ).trim_end_matches([' ', '\t']);
        if !(line.is_empty() || line.ends_with('\n')) {
            return false;
        }
        before = line;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::{self, Format, ProtectSettings};
    use std::sync::Arc;

    fn md(src: &str) -> Document {
        doc::parse(
            Arc::from(src),
            Format::Markdown,
            &ProtectSettings::default(),
        )
    }

    #[test]
    fn two_sentences_words_and_initials() {
        let d = md("It is. We go.");
        let s = segment(&d);
        assert_eq!(s.sentences.len(), 2);
        assert_eq!(s.words.len(), 4);
        assert_eq!(
            s.words
                .iter()
                .map(|w| w.sentence_initial)
                .collect::<Vec<_>>(),
            [true, false, true, false]
        );
        assert_eq!(s.sentences[0].words, 0..2);
        assert_eq!(s.sentences[1].words, 2..4);
    }

    #[test]
    fn protected_span_is_not_a_word_or_bound() {
        let d = md("Use `a. b` now.");
        let s = segment(&d);
        assert_eq!(s.sentences.len(), 1);
        assert_eq!(
            s.words
                .iter()
                .map(|w| &d.src[w.range.clone()])
                .collect::<Vec<_>>(),
            ["Use", "now"]
        );
    }

    #[test]
    fn word_lookup_helpers() {
        let d = md("It is.\n\n`code`\n\nWe go.");
        let s = segment(&d);
        assert_eq!(s.sentences.len(), 3);
        assert!(s.sentences[1].words.is_empty());
        assert_eq!(s.word_before(0), None);
        assert_eq!(s.word_before(2), Some(0));
        assert_eq!(s.word_before(3), Some(0));
        assert_eq!(s.word_before(d.src.len()), Some(3));
        assert_eq!(s.word_at(0), Some(0));
        assert_eq!(s.word_at(1), None);
        assert_eq!(s.word_at(d.src.len()), None);
        assert_eq!(s.sentence_of(0), Some(0));
        assert_eq!(s.sentence_of(2), Some(2));
        assert_eq!(s.sentence_of(4), None);
        assert_eq!(s.sentence_of(usize::MAX), None);
    }

    #[test]
    fn sentence_starts_is_the_cache() {
        let d = md("It is. We go.");
        let s = segment(&d);
        let before = scans();
        assert_eq!(s.sentence_starts(BlockId(0)), [0, 7]);
        assert_eq!(s.sentence_starts(BlockId(99)), [0]);
        assert_eq!(s.sentence_span(&d, BlockId(0), &(1..2)), 0..6);
        assert_eq!(scans(), before);
    }

    #[test]
    fn segment_scans_once_per_block() {
        let d = md("First.\n\nSecond.\n\nThird.");
        assert_eq!(d.blocks.len(), 3);
        let before = scans();
        let segments = segment(&d);
        assert_eq!(scans() - before, 3);
        for b in &d.blocks {
            segments.sentence_span(&d, b.id, &b.range);
        }
        assert_eq!(scans() - before, 3);
    }
}
