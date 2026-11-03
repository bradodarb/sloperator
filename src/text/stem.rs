//! One stemming function for redundancy (D-9). Input is lower case.
use std::borrow::Cow;

const SUFFIXES: &[(&str, &str, usize)] = &[
    ("sses", "ss", 2),
    ("ies", "y", 2),
    ("ing", "", 3),
    ("ed", "", 3),
    ("ly", "", 3),
    ("s", "", 3),
];

/// Applies the first eligible suffix rule, then drops final `e` if three letters remain.
pub fn stem(word_lower: &str) -> Cow<'_, str> {
    let mut w: Cow<'_, str> = Cow::Borrowed(word_lower);
    if !word_lower.ends_with("ss") {
        for (suffix, with, keep) in SUFFIXES {
            if let Some(base) = word_lower.strip_suffix(suffix)
                && base.chars().count() >= *keep
            {
                w = Cow::Owned(format!("{base}{with}"));
                break;
            }
        }
    }
    match w.strip_suffix('e') {
        Some(base) if base.chars().count() >= 3 => Cow::Owned(base.to_string()),
        _ => w,
    }
}

#[cfg(test)]
mod tests {
    use super::stem;

    #[test]
    fn inflections_meet() {
        for family in [
            &["make", "makes", "making"][..],
            &["cache", "caches", "cached", "caching"][..],
        ] {
            for word in family {
                assert_eq!(stem(word), stem(family[0]), "{word}");
            }
        }
    }

    #[test]
    fn suffix_guards() {
        for word in ["", "s", "ss", "class", "be", "the", "sing"] {
            assert_eq!(stem(word), word);
        }
        for (word, expected) in [
            ("classes", "class"),
            ("ponies", "pony"),
            ("quickly", "quick"),
            ("cats", "cat"),
            ("café", "café"),
        ] {
            assert_eq!(stem(word), expected);
        }
    }
}
