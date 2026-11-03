//! Classical text analysis (classical-nlp SPEC §4, §9, ADR-01).
//!
//! `stem::stem` normalises lower-case inflections with an in-tree suffix stripper (D-9).
//! It applies one suffix rule and drops a final `e` only when at least three letters remain.
//! `similarity` builds sentence bags of stemmed content words, excluding stop words and letterless tokens.
//! Weighted Jaccard compares their integer counts using summed minima over summed maxima; empty pairs score zero.
//!
//! `lexicon` provides compiled closed word lists and contraction-aware negation/modal helpers (SPEC §22).
//! SLP006 shares its modal list; `to` remains a separate infinitive guard in the rule.
//!
//! `tagger::tag` tags cached words using owned tables and contextual rules. Only unchanged single-tag
//! lexicon entries are certain; ambiguous and unknown words cannot authorize safe fixes. `Prose::tags` memoises
//! the tag vector on first use; rules run cheap candidate guards first, so plain prose never invokes the tagger.
//!
//! Build one owned `Prose` per document pass. Segmentation eagerly caches each block's sentence bounds and
//! every prose word; consumers can query ranges without scanning again. Pair it with its original document.
//! The pipeline builds it after directive extraction; rules and model windows share its cached segmentation.
//!
//! `syllable::syllables` separately estimates English syllables using vowel groups, silent suffixes,
//! and a compiled exception table. It returns zero for words without letters and at least one otherwise;
//! non-ASCII letters count as vowels. The counter is approximate and has no pipeline callers yet.
//! `Prose::counts` memoises integer paragraph statistics. `Counts::derive` applies sample floors and rounds
//! metrics once to two decimals; values below either floor are unavailable. Merging counts unions word types.

use std::sync::OnceLock;
#[cfg(test)]
use std::sync::atomic::{AtomicU32, Ordering};

use crate::doc::Document;

pub mod lexicon;
pub mod metrics;
pub mod segment;
pub mod similarity;
pub mod stem;
pub mod syllable;
pub mod tagger;

pub use metrics::{Counts, Floors, Metrics};
pub use segment::{Segments, Sentence, Word};
pub use tagger::{Tag, Tagged};

/// One document pass's owned analysis (ADR-02), paired with the document it was built from.
pub struct Prose {
    segments: Segments,
    tags: OnceLock<Vec<Tagged>>,
    #[cfg(test)]
    pub(crate) tag_runs: AtomicU32,
    counts: OnceLock<Counts>,
    src_len: usize,
}

impl Prose {
    /// Segments every block once.
    pub fn new(doc: &Document) -> Prose {
        Prose {
            segments: segment::segment(doc),
            tags: OnceLock::new(),
            #[cfg(test)]
            tag_runs: AtomicU32::new(0),
            counts: OnceLock::new(),
            src_len: doc.src.len(),
        }
    }

    pub fn segments(&self) -> &Segments {
        &self.segments
    }

    /// One tag per cached word, computed lazily once per document pass.
    pub fn tags(&self, doc: &Document) -> &[Tagged] {
        debug_assert_eq!(
            doc.src.len(),
            self.src_len,
            "a Prose is used with another Document"
        );
        self.tags.get_or_init(|| {
            #[cfg(test)]
            self.tag_runs.fetch_add(1, Ordering::Relaxed);
            tagger::tag(doc, &self.segments)
        })
    }

    /// Paragraph metric counts, computed once on first use with the paired document.
    pub fn counts(&self, doc: &Document) -> &Counts {
        debug_assert_eq!(
            doc.src.len(),
            self.src_len,
            "a Prose is used with another Document"
        );
        self.counts
            .get_or_init(|| metrics::counts(doc, &self.segments))
    }
}

/// NFR-8: analysis can cross worker threads.
const _: fn() = || {
    fn ok<T: Send + Sync>() {}
    ok::<Prose>();
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::{self, Format, ProtectSettings};

    #[test]
    fn counts_are_cached_without_rescanning() {
        let doc = doc::parse(
            "Cat cat. Dog!".into(),
            Format::PlainText,
            &ProtectSettings::default(),
        );
        let prose = Prose::new(&doc);
        assert!(prose.counts.get().is_none());
        let before = segment::scans();
        let first = prose.counts(&doc);
        assert_eq!(first.words, 3);
        assert_eq!(first.sentences, 2);
        assert!(std::ptr::eq(first, prose.counts(&doc)));
        assert_eq!(segment::scans(), before);
    }
}
