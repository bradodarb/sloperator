//! Sentence content-word counts and weighted Jaccard similarity (classical-nlp SPEC §33).

use std::collections::BTreeMap;

use crate::doc::Document;

use super::lexicon;
use super::segment::{Segments, Sentence};
use super::stem::stem;

/// Stemmed content words of one sentence with their counts; stop words and words with no letter removed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Bag(BTreeMap<Box<str>, u32>);

impl Bag {
    /// The number of content words (with repeats).
    pub fn size(&self) -> u32 {
        self.0.values().sum()
    }
}

pub fn bag(doc: &Document, seg: &Segments, s: &Sentence) -> Bag {
    let mut map: BTreeMap<Box<str>, u32> = BTreeMap::new();
    for w in &seg.words[s.words.start as usize..s.words.end as usize] {
        let lower = doc.src[w.range.clone()].to_lowercase();
        if lexicon::is_stop_word(&lower) || !lower.chars().any(char::is_alphabetic) {
            continue;
        }
        *map.entry(stem(&lower).into_owned().into_boxed_str())
            .or_insert(0) += 1;
    }
    Bag(map)
}

/// Weighted Jaccard in [0, 1]; 0 when both bags are empty.
pub fn jaccard(a: &Bag, b: &Bag) -> f64 {
    let (mut min, mut max) = (0u64, 0u64);
    let mut keys: Vec<&Box<str>> = a.0.keys().chain(b.0.keys()).collect();
    keys.sort();
    keys.dedup();
    for k in keys {
        let (x, y) = (
            a.0.get(k).copied().unwrap_or(0),
            b.0.get(k).copied().unwrap_or(0),
        );
        min += u64::from(x.min(y));
        max += u64::from(x.max(y));
    }
    if max == 0 {
        0.0
    } else {
        min as f64 / max as f64
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::doc::BlockId;
    use crate::doc::{self, Format, ProtectSettings};
    use crate::text::Word;

    #[test]
    fn jaccard_hand_values() {
        let a = Bag(BTreeMap::from([(Box::from("a"), 2), (Box::from("b"), 1)]));
        let b = Bag(BTreeMap::from([(Box::from("a"), 1), (Box::from("c"), 1)]));
        assert_eq!(jaccard(&a, &b), 0.25);
        assert_eq!(jaccard(&b, &a), 0.25);
        assert_eq!(jaccard(&a, &a), 1.0);
        assert_eq!(jaccard(&a, &Bag::default()), 0.0);
        assert_eq!(jaccard(&Bag::default(), &Bag::default()), 0.0);
        assert_eq!(a.size(), 3);
    }

    #[test]
    fn stop_words_removed() {
        let d = doc::parse(
            Arc::from("The cache caches cached caching 123 and the.\n"),
            Format::Markdown,
            &ProtectSettings::default(),
        );
        let mut seg = Segments::default();
        for (i, word) in d.src.split_whitespace().enumerate() {
            let start = word.as_ptr() as usize - d.src.as_ptr() as usize;
            seg.words.push(Word {
                range: start..start + word.trim_end_matches('.').len(),
                sentence_initial: i == 0,
            });
        }
        let sentence = Sentence {
            block: BlockId(0),
            range: 0..d.src.len(),
            words: 0..seg.words.len() as u32,
        };
        let words = bag(&d, &seg, &sentence);
        assert_eq!(words.size(), 4);
        assert_eq!(words.0, BTreeMap::from([(Box::from("cach"), 4)]));
    }
}
