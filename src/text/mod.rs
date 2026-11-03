//! Classical text analysis (classical-nlp SPEC §4, §9, ADR-01).
//!
//! Build one owned `Prose` per document pass. Segmentation eagerly caches each block's sentence bounds and
//! every prose word; consumers can query ranges without scanning again. Pair it with its original document.
//! The pipeline builds it after directive extraction; rules and model windows share its cached segmentation.

use crate::doc::Document;

pub mod segment;

pub use segment::{Segments, Sentence, Word};

/// One document pass's owned analysis (ADR-02), paired with the document it was built from.
pub struct Prose {
    segments: Segments,
    // Retained for document-pair assertions when lazy analyses are added.
    #[allow(dead_code)]
    src_len: usize,
}

impl Prose {
    /// Segments every block once.
    pub fn new(doc: &Document) -> Prose {
        Prose {
            segments: segment::segment(doc),
            src_len: doc.src.len(),
        }
    }

    pub fn segments(&self) -> &Segments {
        &self.segments
    }
}

/// NFR-8: analysis can cross worker threads.
const _: fn() = || {
    fn ok<T: Send + Sync>() {}
    ok::<Prose>();
};
