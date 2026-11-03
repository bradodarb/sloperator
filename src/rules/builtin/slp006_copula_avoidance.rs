//! SLP006 `copula-avoidance` (SPEC §10): "serve/stand/function as" in every inflection, where plain "is" or "be"
//! would do.
//!
//! Every match is reported. A fix is attached only where part of speech, agreement and the phrase after "as" are
//! all certain (safe-fix rule, Gaps 2, 14 and 16):
//! - "serves/stands/functions as" becomes "is" when the word before it is in [`SINGULAR_SUBJECT`];
//! - the base form after a modal or "to" (with at most one adverb between) becomes "be";
//! - both only when the word after "as" is in [`AS_COMPLEMENT`], so "as follows" and "as well as" are never fixed.
//!
//! A new branch accepts sentence-initial Det N or Det Adj N subjects only when every subject word is certain,
//! the determiner is singular-compatible, and the complement has no following verb or modal in its clause.
//! Other forms are reported with no fix and left to the model.

use crate::doc::Document;
use crate::rules::helpers::PatternRule;
use crate::rules::{Family, Finding, FixClass, Rule, RuleCode, RuleCtx, RuleMeta, Severity};
use crate::text::{Prose, Tag, lexicon};

/// Every inflection of the three verbs, then "as". The match is the verb, the whitespace and "as".
const PATTERNS: &[&str] = &[
    r"(?i)\b(?:serves?|served|serving|stands?|stood|standing|functions?|functioned|functioning)\s+as\b",
];

/// The only words before "serves/stands/functions as" that make the verb a singular present-tense verb. Any other
/// word may be a noun ("the tool", "these functions") and the match gets no fix (Gap 14).
const SINGULAR_SUBJECT: &[&str] = &["it", "this", "that", "which", "who", "he", "she"];

/// The only words after "as" that open a noun phrase, so "is"/"be" plus that phrase is grammatical (Gap 16).
const AS_COMPLEMENT: &[&str] = &[
    "a", "an", "the", "its", "his", "her", "their", "our", "your", "my", "one", "another",
];

/// Adverbs allowed between a modal and the base form, besides any word ending in "ly".
const ADVERBS: &[&str] = &["also", "still", "just", "never", "always", "even", "not"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Form {
    ThirdPerson,
    Base,
    Other,
}

struct CopulaAvoidance {
    meta: RuleMeta,
    patterns: PatternRule,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(CopulaAvoidance {
        meta: RuleMeta {
            code: RuleCode(6),
            name: "copula-avoidance",
            family: Family::CopulaAvoidance,
            severity: Severity::Warning,
            default_on: true,
            fix: FixClass::Safe,
            explain: "Flags \"serves as\", \"stands as\" and \"functions as\" (any inflection) where \"is\" or \"be\" \
                      says the same. The fix applies only after it/this/that/which/who/he/she or after a modal or \
                      \"to\", or after a sentence-initial determiner and noun (optionally with one adjective) \
                      that the tagger is certain of, when no verb follows in the clause. A noun phrase must \
                      follow \"as\"; every other form is reported with no fix.",
            example: ("It serves as a bridge.", "It is a bridge."),
            params: &[],
            since: "0.1.0",
        },
        patterns: PatternRule::new(PATTERNS),
    })
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '\'' || c == '’'
}

/// The lower-case word that ends `text[..at]` (whitespace skipped) and where it starts. `None` when punctuation, a
/// non-prose run or the start of the block comes first.
fn word_before(text: &str, at: usize) -> Option<(usize, String)> {
    let before = text.get(..at)?.trim_end();
    let start = before
        .char_indices()
        .rev()
        .take_while(|&(_, c)| is_word_char(c))
        .last()
        .map(|(i, _)| i)?;
    let word = before
        .get(start..)?
        .trim_start_matches(['\'', '’'])
        .to_lowercase();
    (!word.is_empty()).then_some((start, word))
}

/// The lower-case word that starts `text[at..]` (whitespace skipped). `None` when punctuation, a non-prose run or
/// the end of the block comes first.
fn word_after(text: &str, at: usize) -> Option<String> {
    let rest = text.get(at..)?.trim_start();
    let end = rest.find(|c: char| !is_word_char(c)).unwrap_or(rest.len());
    let word = rest.get(..end)?.trim_matches(['\'', '’']).to_lowercase();
    (!word.is_empty()).then_some(word)
}

fn form_of(verb: &str) -> Form {
    match verb.to_lowercase().as_str() {
        "serves" | "stands" | "functions" => Form::ThirdPerson,
        "serve" | "stand" | "function" => Form::Base,
        _ => Form::Other,
    }
}

fn is_adverb(word: &str) -> bool {
    ADVERBS.contains(&word) || (word.len() > 2 && word.ends_with("ly"))
}

/// The safe replacement for the match at `at` in `text`, or `None` when any guard fails.
fn complement_ok(text: &str, at: &std::ops::Range<usize>) -> bool {
    word_after(text, at.end).is_some_and(|next| AS_COMPLEMENT.contains(&next.as_str()))
}

fn safe_replacement(text: &str, at: &std::ops::Range<usize>, form: Form) -> Option<&'static str> {
    if !complement_ok(text, at) {
        return None;
    }
    let (start, prev) = word_before(text, at.start)?;
    match form {
        Form::ThirdPerson => SINGULAR_SUBJECT.contains(&prev.as_str()).then_some("is"),
        Form::Base if prev == "to" || lexicon::MODALS.contains(&prev.as_str()) => Some("be"),
        Form::Base if is_adverb(&prev) => {
            let (_, before) = word_before(text, start)?;
            (before == "to" || lexicon::MODALS.contains(&before.as_str())).then_some("be")
        }
        Form::Base | Form::Other => None,
    }
}

impl Rule for CopulaAvoidance {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        for bt in ctx.blocks() {
            for (_, m) in self.patterns.find_text(&bt.text) {
                let Some(range) = bt.src_range(m.clone()) else {
                    continue;
                };
                let verb = bt.text[m.clone()].split_whitespace().next().unwrap_or("");
                let form = form_of(verb);
                let message = format!("`{}` can be a plain copula", &ctx.doc.src[range.clone()]);
                let fix = safe_replacement(&bt.text, &m, form)
                    .or_else(|| {
                        (form == Form::ThirdPerson
                            && complement_ok(&bt.text, &m)
                            && tag_certain_subject(ctx.doc, ctx.prose, range.start))
                        .then_some("is")
                    })
                    .map(|r| (range.clone(), r.to_string()));
                out.push(ctx.finding(&self.meta, range, message, fix));
            }
        }
    }
}

const SINGULAR_DET: &[&str] = &[
    "a", "an", "another", "each", "every", "her", "his", "its", "my", "our", "that", "the",
    "their", "this", "your",
];
const AS_IDIOMS: &[&str] = &["a matter", "a result", "a rule", "a whole"];

fn tag_certain_subject(doc: &Document, prose: &Prose, verb: usize) -> bool {
    let seg = prose.segments();
    let Some(v) = seg.word_at(verb) else {
        return false;
    };
    let Some(noun) = v.checked_sub(1) else {
        return false;
    };
    let lower = |i: usize| doc.src[seg.words[i].range.clone()].to_lowercase();
    if seg.words.get(v + 1).is_none() || lower(v + 1) != "as" {
        return false;
    }
    let tags = prose.tags(doc);
    let certain = |i: usize, t: Tag| tags[i].certain && tags[i].tag == t;
    let joined = |a: usize| {
        let gap = seg.words[a].range.end..seg.words[a + 1].range.start;
        doc.src[gap.clone()].trim().is_empty() && !doc.is_protected(&gap)
    };
    if !certain(noun, Tag::Noun) || !joined(noun) {
        return false;
    }
    let det = match noun.checked_sub(1) {
        Some(d) if certain(d, Tag::Det) => d,
        Some(a) if certain(a, Tag::Adj) && joined(a) => match a.checked_sub(1) {
            Some(d) if certain(d, Tag::Det) => d,
            _ => return false,
        },
        _ => return false,
    };
    if !joined(det)
        || !seg.words[det].sentence_initial
        || !SINGULAR_DET.contains(&lower(det).as_str())
    {
        return false;
    }
    let Some(s) = seg.sentence_of(v) else {
        return false;
    };
    let end = seg.sentences[s].words.end as usize;
    let first = v + 2;
    if first + 1 < end
        && AS_IDIOMS.contains(&format!("{} {}", lower(first), lower(first + 1)).as_str())
    {
        return false;
    }
    for (i, tagged) in tags.iter().enumerate().take(end).skip(first) {
        if i > first {
            let gap = &doc.src[seg.words[i - 1].range.end..seg.words[i].range.start];
            if gap.contains([',', ';', ':', '(', '\u{2014}', '\u{2013}']) {
                break;
            }
        }
        if matches!(
            tagged.tag,
            Tag::Verb | Tag::Verb3s | Tag::VerbPast | Tag::Modal
        ) {
            return false;
        }
    }
    true
}
