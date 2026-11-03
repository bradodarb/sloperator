//! Approximate English syllable counts (FR-2.6): vowel groups, silent-e / -ed / -es rules,
//! and a sorted exception table. Deterministic and offline.

/// Words the heuristic miscounts, sorted by word (binary search).
static EXCEPTIONS: &[(&str, u32)] = &[
    ("being", 2),
    ("business", 2),
    ("create", 2),
    ("created", 3),
    ("creates", 2),
    ("every", 2),
    ("idea", 3),
    ("ideas", 3),
    ("poem", 2),
    ("quiet", 2),
    ("science", 2),
    ("society", 4),
    ("variety", 4),
];

/// The syllables of one word: 0 for a word with no letter (`42`), otherwise at least 1.
/// Non-ASCII letters count as vowels (`café` = 2).
pub fn syllables(word: &str) -> u32 {
    let w: Vec<char> = word
        .chars()
        .filter(|c| c.is_alphabetic())
        .flat_map(char::to_lowercase)
        .collect();
    if w.is_empty() {
        return 0;
    }
    let lower: String = w.iter().collect();
    if let Ok(i) = EXCEPTIONS.binary_search_by(|(e, _)| (*e).cmp(lower.as_str())) {
        return EXCEPTIONS[i].1;
    }
    let vowel = |i: usize| match w[i] {
        'a' | 'e' | 'i' | 'o' | 'u' => true,
        'y' => i > 0,
        c => !c.is_ascii(),
    };
    let n = w.len();
    let mut groups: u32 = 0;
    let mut prev = false;
    for i in 0..n {
        let v = vowel(i);
        if v && !prev {
            groups += 1;
        }
        prev = v;
    }
    let consonant = |i: usize| !vowel(i);
    let silent = if n > 2 && w[n - 1] == 'e' && consonant(n - 2) {
        !(w[n - 2] == 'l' && n > 3 && consonant(n - 3)) // "table" 2
    } else if n > 3 && w[n - 2] == 'e' && w[n - 1] == 'd' {
        consonant(n - 3) && !matches!(w[n - 3], 't' | 'd') // "jumped" 1, "wanted" 2
    } else if n > 3 && w[n - 2] == 'e' && w[n - 1] == 's' {
        consonant(n - 3)
            && !matches!(w[n - 3], 's' | 'x' | 'z' | 'c' | 'g')
            && !(w[n - 3] == 'h' && n > 4 && matches!(w[n - 4], 'c' | 's'))
    } else {
        false
    };
    if silent {
        groups = groups.saturating_sub(1);
    }
    groups.max(1)
}

#[cfg(test)]
mod tests {
    use super::{EXCEPTIONS, syllables};

    #[test]
    fn fixed_list() {
        let words = [
            "the",
            "make",
            "table",
            "free",
            "jumped",
            "wanted",
            "played",
            "freed",
            "makes",
            "boxes",
            "matches",
            "uses",
            "goes",
            "people",
            "idea",
            "every",
            "café",
            "readability",
            "sentence",
            "cat",
            "42",
            "I",
            "simple",
            "sloperator",
        ];
        let expected = [
            1, 1, 2, 1, 1, 2, 1, 1, 1, 2, 2, 2, 1, 2, 3, 2, 2, 5, 2, 1, 0, 1, 2, 4,
        ];
        for (word, count) in words.into_iter().zip(expected) {
            assert_eq!(syllables(word), count, "{word}");
        }
    }

    #[test]
    fn exceptions_sorted_unique() {
        assert!(!EXCEPTIONS.is_empty());
        assert!(EXCEPTIONS.windows(2).all(|pair| pair[0].0 < pair[1].0));
        for &(word, count) in EXCEPTIONS {
            assert!(count > 0, "{word}");
            assert_eq!(syllables(word), count, "{word}");
            assert_eq!(syllables(&word.to_uppercase()), count, "{word}");
        }
    }

    #[test]
    fn letters_and_normalization() {
        for word in ["", "42", "123-456", "?!", "🦀"] {
            assert_eq!(syllables(word), 0, "{word}");
        }
        for word in ["b", "rhythm", "y", "é", "中", "b42"] {
            assert!(syllables(word) >= 1, "{word}");
        }
        assert_eq!(syllables("TABLE"), 2);
        assert_eq!(syllables("'wanted!'"), 2);
    }
}
