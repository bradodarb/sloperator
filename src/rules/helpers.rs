//! Shared detectors (ADR-06). One aho-corasick automaton per word-list rule (NFR-3).

use std::ops::Range;

use aho_corasick::{AhoCorasick, AhoCorasickBuilder, Input, MatchKind};
use pulldown_cmark::{Event, Options, Parser};

use crate::doc::{BlockText, Document, Format, SpanKind};

type Entry = (&'static str, Option<&'static str>);

/// A table of phrases, each with an optional safe replacement. `None` means the phrase is flagged but has no safe
/// fix, so it is left to the model (Gap 2, FR-4.7).
pub struct WordList {
    ac: AhoCorasick,
    entries: &'static [Entry],
}

impl WordList {
    /// `entries`: (phrase, safe replacement). Matching ignores ASCII case and takes the leftmost, longest phrase.
    ///
    /// # Panics
    /// If the automaton cannot be built. The tables are static, so that is a programming error, not an input error.
    pub fn new(entries: &'static [Entry]) -> WordList {
        let built = AhoCorasickBuilder::new()
            .ascii_case_insensitive(true)
            .match_kind(MatchKind::LeftmostLongest)
            .build(entries.iter().map(|(phrase, _)| *phrase));
        match built {
            Ok(ac) => WordList { ac, entries },
            Err(e) => panic!("word list does not build: {e}"),
        }
    }

    /// Whole-word matches in one block as source ranges, with a case-preserving replacement when the entry has one.
    /// A match that covers a non-prose run (an `OBJ`) has no source range and is skipped. A longest match that is not
    /// a whole word does not hide a shorter phrase that starts at the same place.
    pub fn find(&self, bt: &BlockText) -> Vec<(Range<usize>, Option<String>)> {
        let text = bt.text.as_str();
        let mut out = Vec::new();
        let mut pos = 0;
        while pos <= text.len() {
            let Some(m) = self.ac.find(Input::new(text).span(pos..text.len())) else {
                break;
            };
            let range = m.start()..m.end();
            let step = text[range.start..].chars().next().map_or(1, char::len_utf8);
            if range.is_empty() || !is_whole_word(text, &range) {
                pos = range.start + step;
                continue;
            }
            pos = range.end;
            let Some(src) = bt.src_range(range.clone()) else {
                continue;
            };
            let replacement = self
                .entries
                .get(m.pattern().as_usize())
                .and_then(|(_, r)| *r)
                .map(|r| match_case(&text[range], r));
            out.push((src, replacement));
        }
        out
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn is_whole_word(text: &str, range: &Range<usize>) -> bool {
    let before = text[..range.start].chars().next_back();
    let after = text[range.end..].chars().next();
    !before.is_some_and(is_word_char) && !after.is_some_and(is_word_char)
}

/// `replacement` in the case of `matched`: all caps (two or more letters) gives upper case, a capital first letter
/// capitalises the replacement's first letter, anything else leaves it as written.
fn match_case(matched: &str, replacement: &str) -> String {
    let letters: Vec<char> = matched.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.len() > 1 && letters.iter().all(|c| c.is_uppercase()) {
        return replacement.to_uppercase();
    }
    if letters.first().is_some_and(|c| c.is_uppercase()) {
        let mut chars = replacement.chars();
        return match chars.next() {
            Some(first) => first.to_uppercase().chain(chars).collect(),
            None => String::new(),
        };
    }
    replacement.to_string()
}

/// Regex-based detection: a set to skip blocks cheaply, and one regex per pattern to locate matches.
pub struct PatternRule {
    pub set: regex::RegexSet,
    pub regexes: Vec<regex::Regex>,
}

impl PatternRule {
    /// # Panics
    /// If a pattern is not a valid regex. Patterns are static rule data, so that is a programming error.
    pub fn new(patterns: &[&str]) -> PatternRule {
        match (
            regex::RegexSet::new(patterns),
            patterns
                .iter()
                .map(|p| regex::Regex::new(p))
                .collect::<Result<Vec<_>, _>>(),
        ) {
            (Ok(set), Ok(regexes)) => PatternRule { set, regexes },
            (Err(e), _) | (_, Err(e)) => panic!("rule pattern does not compile: {e}"),
        }
    }

    /// Matches in one block as (pattern index, source range), in text order within each pattern. A match that covers
    /// a non-prose run has no source range and is skipped.
    pub fn find(&self, bt: &BlockText) -> Vec<(usize, Range<usize>)> {
        self.find_text(&bt.text)
            .into_iter()
            .filter_map(|(i, r)| bt.src_range(r).map(|src| (i, src)))
            .collect()
    }

    /// Matches in a block's text as (pattern index, text range), in text order within each pattern. For rules that
    /// look at the text around a match before they map it to the source with [`BlockText::src_range`].
    pub fn find_text(&self, text: &str) -> Vec<(usize, Range<usize>)> {
        let mut out = Vec::new();
        for i in self.set.matches(text).iter() {
            for m in self.regexes[i].find_iter(text) {
                out.push((i, m.range()));
            }
        }
        out
    }
}

pub use crate::text::segment::starts_sentence;

/// The safe fix for a lead-in phrase (SLP008, SLP016): delete `lead` (a text range that includes the whitespace after
/// it) and capitalise the next letter. Returns the source range to replace and its replacement.
///
/// The range grows by one letter when that letter is lower case, so the capital is part of the same edit. When the
/// next character is a capital letter or a digit, only `lead` is deleted. Anything else follows (punctuation, a
/// non-prose run, the end of the block), where deleting would leave broken prose, so there is no fix.
pub fn delete_lead_in(bt: &BlockText, lead: Range<usize>) -> Option<(Range<usize>, String)> {
    let next = bt.text.get(lead.end..)?.chars().next()?;
    if next.is_lowercase() {
        let src = bt.src_range(lead.start..lead.end + next.len_utf8())?;
        return Some((src, next.to_uppercase().collect()));
    }
    if next.is_alphanumeric() {
        return bt.src_range(lead).map(|src| (src, String::new()));
    }
    None
}

/// The Markdown events of `doc` with their source ranges, for rules that need structure the span map does not keep
/// (lists, strong spans: SLP010, SLP011). Plain text has no Markdown structure, so it yields nothing. The options
/// match `doc::markdown`'s with every protection on; an event inside a protected run gives a finding that `detect`
/// drops anyway.
pub fn markdown_events(doc: &Document) -> Vec<(Event<'_>, Range<usize>)> {
    if doc.format != Format::Markdown {
        return Vec::new();
    }
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
        | Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS
        | Options::ENABLE_MATH;
    Parser::new_ext(&doc.src, options)
        .into_offset_iter()
        .collect()
}

/// The first prose run inside `range`, trimmed of whitespace: where a structure rule reports, since a finding that
/// touches markup is dropped. `None` when `range` holds no prose.
pub fn first_prose(doc: &Document, range: &Range<usize>) -> Option<Range<usize>> {
    doc.spans
        .iter()
        .filter(|s| s.kind == SpanKind::Prose)
        .filter(|s| s.range.start < range.end && s.range.end > range.start)
        .find_map(|s| {
            let r = s.range.start.max(range.start)..s.range.end.min(range.end);
            let text = &doc.src[r.clone()];
            let lead = text.len() - text.trim_start().len();
            let trimmed = text.trim();
            (!trimmed.is_empty()).then(|| r.start + lead..r.start + lead + trimmed.len())
        })
}
