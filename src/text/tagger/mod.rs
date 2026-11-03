// tagger/mod.rs
//! A Brill-style tagger (D-7): lexicon tags, suffix guesses for unknown words, then contextual rules in table
//! order. It is used only as a gate for safe fixes; `certain` is the only property a gate may rely on.

use crate::doc::Document;

use super::segment::Segments;

mod lexicon_data;
mod rules_data;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tag {
    Det,
    Pron,
    Noun,
    ProperNoun,
    Verb,
    VerbPast,
    VerbPart,
    Verb3s,
    Modal,
    Adj,
    Adv,
    Prep,
    Conj,
    Num,
    Other,
}

/// One word's tag. `certain`: the lexicon gives exactly one tag and no rule changed it. `unknown`: not in the
/// lexicon; the tag is a suffix guess (FR-7.1). An unknown word is never certain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tagged {
    pub tag: Tag,
    pub certain: bool,
    pub unknown: bool,
}

/// When a contextual rule applies; positions are within one sentence.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Context {
    PrevTag(Tag),
    NextTag(Tag),
    PrevWord(&'static str),
    NextWord(&'static str),
}

/// Change `from` to `to` where `when` holds, only on a word whose lexicon tags include `to`, or an unknown word.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ContextRule {
    pub(crate) from: Tag,
    pub(crate) to: Tag,
    pub(crate) when: Context,
}

fn lookup(lower: &str) -> Option<&'static [Tag]> {
    let lex = lexicon_data::LEXICON;
    lex.binary_search_by(|(w, _)| (*w).cmp(lower))
        .ok()
        .map(|i| lex[i].1)
}

fn guess(word: &str, sentence_initial: bool) -> Tag {
    let lower = word.to_lowercase();
    if word
        .chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c == ',')
    {
        Tag::Num
    } else if !sentence_initial && word.chars().next().is_some_and(char::is_uppercase) {
        Tag::ProperNoun
    } else if lower.ends_with("ly") {
        Tag::Adv
    } else if lower.ends_with("ing") {
        Tag::VerbPart
    } else if lower.ends_with("ed") {
        Tag::VerbPast
    } else {
        Tag::Noun
    }
}

/// One tag per entry of `seg.words`, in order (FR-7.1, FR-7.2).
pub fn tag(doc: &Document, seg: &Segments) -> Vec<Tagged> {
    let mut out = Vec::with_capacity(seg.words.len());
    let mut options: Vec<&'static [Tag]> = Vec::with_capacity(seg.words.len());
    for w in &seg.words {
        let text = &doc.src[w.range.clone()];
        match lookup(&text.to_lowercase()) {
            Some(tags) if !tags.is_empty() => {
                out.push(Tagged {
                    tag: tags[0],
                    certain: tags.len() == 1,
                    unknown: false,
                });
                options.push(tags);
            }
            _ => {
                out.push(Tagged {
                    tag: guess(text, w.sentence_initial),
                    certain: false,
                    unknown: true,
                });
                options.push(&[]);
            }
        }
    }
    let word = |i: usize| &doc.src[seg.words[i].range.clone()];
    for s in &seg.sentences {
        let r = s.words.start as usize..s.words.end as usize;
        for rule in rules_data::RULES {
            for i in r.clone() {
                if out[i].tag != rule.from
                    || !(options[i].is_empty() || options[i].contains(&rule.to))
                {
                    continue;
                }
                let hit = match rule.when {
                    Context::PrevTag(t) => i > r.start && out[i - 1].tag == t,
                    Context::NextTag(t) => i + 1 < r.end && out[i + 1].tag == t,
                    Context::PrevWord(x) => i > r.start && word(i - 1).eq_ignore_ascii_case(x),
                    Context::NextWord(x) => i + 1 < r.end && word(i + 1).eq_ignore_ascii_case(x),
                };
                if hit {
                    out[i] = Tagged {
                        tag: rule.to,
                        certain: false,
                        unknown: out[i].unknown,
                    };
                }
            }
        }
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexicon_sorted_unique_lowercase_nonempty() {
        let lex = lexicon_data::LEXICON;
        assert!(!lex.is_empty() && lex.len() <= 3_000);
        assert!(lex.windows(2).all(|w| w[0].0 < w[1].0));
        for (word, tags) in lex {
            assert!(!word.is_empty());
            assert_eq!(*word, word.to_lowercase());
            assert!(!tags.is_empty());
            assert!(!tags.contains(&Tag::ProperNoun));
        }
        assert!(rules_data::RULES.len() <= 300);
    }

    #[test]
    fn required_entries() {
        use Tag::*;
        for word in ["tool", "service", "guide"] {
            assert_eq!(lookup(word), Some([Noun, Verb].as_slice()));
        }
        assert_eq!(lookup("fast"), Some([Adj, Adv].as_slice()));
        assert_eq!(lookup("library"), Some([Noun].as_slice()));
        for word in ["main", "careful", "ready", "cheap", "quick", "simple"] {
            assert_eq!(lookup(word), Some([Adj].as_slice()));
        }
        for word in ["these", "this"] {
            let tags = lookup(word).unwrap_or(&[]);
            assert!(tags.contains(&Det) && tags.contains(&Pron));
        }
    }
}
