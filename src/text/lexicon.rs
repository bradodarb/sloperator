//! Closed word lists shared by the verifier, the tagger, SLP006 and redundancy (ADR-07: compile-time tables, each
//! sorted and unique, checked by unit tests).

/// FR-6.1's closed list. `n't` forms count through [`negation_count`].
pub const NEGATIONS: &[&str] = &[
    "cannot", "neither", "never", "no", "nobody", "none", "nor", "not", "nothing", "nowhere",
];

/// FR-6.3. (SLP006's local copy, which also held "to", is deleted; SLP006 tests "to" itself.)
pub const MODALS: &[&str] = &[
    "can", "could", "may", "might", "must", "shall", "should", "will", "would",
];

/// Hedging adverbs: capitalised in title case, never names (the proper-noun proxy skips them).
pub const HEDGES: &[&str] = &[
    "arguably",
    "perhaps",
    "possibly",
    "potentially",
    "presumably",
    "probably",
];

/// Capitalised tokens that are never names.
pub const NOT_NAMES: &[&str] = &[
    "I",
    "I'd",
    "I'll",
    "I'm",
    "I've",
    "I\u{2019}d",
    "I\u{2019}ll",
    "I\u{2019}m",
    "I\u{2019}ve",
];

/// Function words: removed before redundancy comparison (FR-9.1), and never names. Negations and modals are not
/// in it.
pub const STOP_WORDS: &[&str] = &[
    "a",
    "about",
    "above",
    "after",
    "again",
    "against",
    "all",
    "also",
    "am",
    "an",
    "and",
    "any",
    "are",
    "as",
    "at",
    "be",
    "because",
    "been",
    "before",
    "being",
    "below",
    "between",
    "both",
    "but",
    "by",
    "did",
    "do",
    "does",
    "doing",
    "down",
    "during",
    "each",
    "few",
    "for",
    "from",
    "further",
    "had",
    "has",
    "have",
    "having",
    "he",
    "her",
    "here",
    "hers",
    "herself",
    "him",
    "himself",
    "his",
    "how",
    "i",
    "if",
    "in",
    "into",
    "is",
    "it",
    "its",
    "itself",
    "just",
    "me",
    "more",
    "most",
    "my",
    "myself",
    "of",
    "off",
    "on",
    "once",
    "only",
    "or",
    "other",
    "our",
    "ours",
    "ourselves",
    "out",
    "over",
    "own",
    "same",
    "she",
    "so",
    "some",
    "such",
    "than",
    "that",
    "the",
    "their",
    "theirs",
    "them",
    "themselves",
    "then",
    "there",
    "these",
    "they",
    "this",
    "those",
    "through",
    "to",
    "too",
    "under",
    "until",
    "up",
    "very",
    "was",
    "we",
    "were",
    "what",
    "when",
    "where",
    "which",
    "while",
    "who",
    "whom",
    "why",
    "with",
    "you",
    "your",
    "yours",
    "yourself",
    "yourselves",
];

fn folded(word: &str) -> String {
    word.to_lowercase().replace('\u{2019}', "'")
}

/// 1 for a word of [`NEGATIONS`] or an `n't` form (ASCII or ’ apostrophe), else 0.
pub fn negation_count(word: &str) -> u32 {
    let w = folded(word);
    u32::from(NEGATIONS.binary_search(&w.as_str()).is_ok() || w.ends_with("n't"))
}

/// The modal a word carries: `won't` → will, `can't`/`cannot` → can, `shan't` → shall, `X'll` → will, `Xn't` → X.
/// `'d` is not mapped (would or had).
pub fn modal_of(word: &str) -> Option<&'static str> {
    let w = folded(word);
    if w.ends_with("'ll") {
        return Some("will");
    }
    let base = match w.as_str() {
        "won't" => "will",
        "can't" | "cannot" => "can",
        "shan't" => "shall",
        other => other.strip_suffix("n't").unwrap_or(other),
    };
    MODALS.iter().copied().find(|m| *m == base)
}

pub fn is_stop_word(lower: &str) -> bool {
    STOP_WORDS.binary_search(&lower).is_ok()
}

/// A capitalised word that is not a name candidate: [`NOT_NAMES`], or a stop word, modal or hedge in any case. The
/// verifier consults it for rule-origin edits only (§23, written for OD-2 option (a)).
pub fn is_common_capitalised(word: &str) -> bool {
    if NOT_NAMES.contains(&word) {
        return true;
    }
    let w = folded(word);
    is_stop_word(&w)
        || MODALS.binary_search(&w.as_str()).is_ok()
        || HEDGES.binary_search(&w.as_str()).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_sorted_unique() {
        for table in [NEGATIONS, MODALS, HEDGES, STOP_WORDS] {
            assert!(!table.is_empty());
            assert!(table.windows(2).all(|pair| pair[0] < pair[1]));
        }
        assert!(!NOT_NAMES.is_empty());
        for (i, word) in NOT_NAMES.iter().enumerate() {
            assert!(!NOT_NAMES[..i].contains(word));
        }
    }

    #[test]
    fn contractions() {
        for word in [
            "isn't", "isn’t", "never", "cannot", "CAN'T", "won’t", "shan't",
        ] {
            assert_eq!(negation_count(word), 1, "{word}");
        }
        assert_eq!(negation_count("few"), 0);
        for (word, modal) in [
            ("won't", "will"),
            ("won’t", "will"),
            ("can't", "can"),
            ("can’t", "can"),
            ("cannot", "can"),
            ("we'll", "will"),
            ("we’ll", "will"),
            ("shan't", "shall"),
            ("shan’t", "shall"),
            ("shouldn't", "should"),
            ("shouldn’t", "should"),
            ("MAY", "may"),
        ] {
            assert_eq!(modal_of(word), Some(modal), "{word}");
        }
        for word in ["I'd", "I’d", "had", "few", "isn't"] {
            assert_eq!(modal_of(word), None, "{word}");
        }
        for modal in MODALS {
            assert_eq!(modal_of(modal), Some(*modal));
        }
    }

    #[test]
    fn common_capitalised_words() {
        for word in ["May", "Potentially", "The", "I'm", "I’m"] {
            assert!(is_common_capitalised(word), "{word}");
        }
        for word in STOP_WORDS
            .iter()
            .chain(MODALS)
            .chain(HEDGES)
            .chain(NOT_NAMES)
        {
            assert!(is_common_capitalised(word), "{word}");
        }
        assert!(!is_common_capitalised("Redis"));
        assert!(is_stop_word("few"));
        for word in NEGATIONS.iter().chain(MODALS) {
            assert!(!is_stop_word(word), "{word}");
        }
    }
}
