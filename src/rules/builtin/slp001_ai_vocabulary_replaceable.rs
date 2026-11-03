//! SLP001 `ai-vocabulary-replaceable` (SPEC §10; classical-nlp §20): padded verbs, nominalisations and phrases.
//!
//! Noun-final rows use cached sentence shapes; "the fact that" rows reject preceding forms of "be".
//! Unproven contexts give no finding: "a decision tree" and "a decision to leave and a plan to return" stay alone.
//!
//! Every verb is listed per inflection with its own replacement, never as a lemma (safe-fix rule, Gap 2). Bare
//! "leverage" is a noun as often as a verb, so it is reported with no fix and left to the model.

use std::ops::Range;

use crate::doc::BlockText;
use crate::rules::helpers::WordList;
use crate::rules::{Family, Finding, FixClass, Rule, RuleCode, RuleCtx, RuleMeta, Severity};
use crate::text::{Segments, Word};

static ENTRIES: &[(&str, Option<&str>)] = &[
    ("delve into", Some("look at")),
    ("delves into", Some("looks at")),
    ("delved into", Some("looked at")),
    ("delving into", Some("looking at")),
    ("utilize", Some("use")),
    ("utilizes", Some("uses")),
    ("utilized", Some("used")),
    ("utilizing", Some("using")),
    ("in order to", Some("to")),
    ("facilitate", Some("help")),
    ("facilitates", Some("helps")),
    ("facilitated", Some("helped")),
    ("facilitating", Some("helping")),
    ("leverages", Some("uses")),
    ("leveraged", Some("used")),
    ("leveraging", Some("using")),
    ("leverage", None),
    ("make a decision", Some("decide")),
    ("makes a decision", Some("decides")),
    ("made a decision", Some("decided")),
    ("making a decision", Some("deciding")),
    ("reach a conclusion", Some("conclude")),
    ("reaches a conclusion", Some("concludes")),
    ("reached a conclusion", Some("concluded")),
    ("reaching a conclusion", Some("concluding")),
    ("take into consideration", Some("consider")),
    ("takes into consideration", Some("considers")),
    ("took into consideration", Some("considered")),
    ("taking into consideration", Some("considering")),
    ("taken into consideration", Some("considered")),
    ("take into account", Some("consider")),
    ("takes into account", Some("considers")),
    ("took into account", Some("considered")),
    ("taking into account", Some("considering")),
    ("taken into account", Some("considered")),
    ("give consideration to", Some("consider")),
    ("gives consideration to", Some("considers")),
    ("gave consideration to", Some("considered")),
    ("giving consideration to", Some("considering")),
    ("given consideration to", Some("considered")),
    ("conduct an analysis of", Some("analyze")),
    ("conducts an analysis of", Some("analyzes")),
    ("conducted an analysis of", Some("analyzed")),
    ("conducting an analysis of", Some("analyzing")),
    ("perform an analysis of", Some("analyze")),
    ("performs an analysis of", Some("analyzes")),
    ("performed an analysis of", Some("analyzed")),
    ("performing an analysis of", Some("analyzing")),
    ("conduct an investigation into", Some("investigate")),
    ("conducts an investigation into", Some("investigates")),
    ("conducted an investigation into", Some("investigated")),
    ("conducting an investigation into", Some("investigating")),
    ("provide assistance to", Some("help")),
    ("provides assistance to", Some("helps")),
    ("provided assistance to", Some("helped")),
    ("providing assistance to", Some("helping")),
    ("make an attempt to", Some("try to")),
    ("makes an attempt to", Some("tries to")),
    ("made an attempt to", Some("tried to")),
    ("making an attempt to", Some("trying to")),
    ("has a tendency to", Some("tends to")),
    ("have a tendency to", Some("tend to")),
    ("had a tendency to", Some("tended to")),
    ("is in agreement with", Some("agrees with")),
    ("are in agreement with", Some("agree with")),
    ("was in agreement with", Some("agreed with")),
    ("were in agreement with", Some("agreed with")),
    ("due to the fact that", Some("because")),
    ("owing to the fact that", Some("because")),
    ("in light of the fact that", Some("because")),
    ("despite the fact that", Some("although")),
    ("in spite of the fact that", Some("although")),
    ("at this point in time", Some("now")),
    ("at the present time", Some("now")),
    ("in the near future", Some("soon")),
    ("on a daily basis", Some("daily")),
    ("on a regular basis", Some("regularly")),
    ("until such time as", Some("until")),
    ("prior to", Some("before")),
    ("for the purpose of", Some("for")),
    ("with the exception of", Some("except")),
    ("in close proximity to", Some("near")),
];

/// Noun-final rows: the noun that ends the phrase, the words that may come right after it, and whether the phrase
/// may open its sentence before a comma (the adverbial rows only). See [`noun_final_allows`] (Gap 2, D-10).
static NOUN_FOLLOW: &[(&str, &[&str], bool)] = &[
    ("basis", CLAUSE, true),
    ("conclusion", CLAUSE, false),
    ("decision", DECISION, false),
    ("future", CLAUSE, true),
    ("time", CLAUSE, true),
];

/// Subordinators that open an adverbial clause or phrase on the verb, never a modifier of the noun. "and", "or",
/// "but", "as", "with", "for", "in", "at", "by", "so" and "when" are absent: each can join or modify the noun
/// ("made a decision and a plan", "a decision as bold as theirs", "a decision with consequences", "at this point in
/// time when costs rose"), and "that" can open a relative clause ("made a decision that surprised us").
static CLAUSE: &[&str] = &[
    "after", "although", "because", "before", "if", "though", "unless", "until",
];

/// [`CLAUSE`] plus the complements "decide" takes with the noun's meaning: "a decision about/on X" = "decide about/on
/// X", "a decision to leave" = "decide to leave". "conclusion" gets no such words: "reached a conclusion about the
/// data" → "concluded about the data" is not the same claim.
static DECISION: &[&str] = &[
    "about", "after", "although", "because", "before", "if", "on", "though", "to", "unless",
    "until",
];

/// A last word that makes the follow word open a relative clause on the deleted noun: "made a decision about which."
static RELATIVE: &[&str] = &["which", "whom", "whose"];

/// A last word that makes "to" a preposition on the noun, not an infinitive marker: "made a decision to them" →
/// "decided to them". A capitalised last word after "to" ("to Paris") is refused too.
static NOT_AFTER_TO: &[&str] = &[
    "a", "an", "her", "him", "his", "it", "its", "me", "my", "our", "that", "the", "their", "them",
    "these", "this", "those", "us", "you", "your",
];

/// Subject pronouns that open a main clause after a fronted adverbial row: "At this point in time, we stop."
static SUBJECT: &[&str] = &["he", "i", "it", "she", "they", "we", "you"];

/// Characters that may stand between the last accounted-for word and the end of the sentence.
static CLOSERS: &[char] = &['.', '!', '?', '"', '\'', ')', '\u{2019}', '\u{201D}'];

/// Forms of "be" that may not precede a "… the fact that" row: "The delay is due to the fact that we left." would
/// become "The delay is because we left."
static BE_FORMS: &[&str] = &["am", "are", "be", "been", "being", "is", "was", "were"];

/// Whether a noun-final match at source range `m` of `src` may be reported and fixed (Gap 2, D-10); `true` for any
/// other row. The replacement deletes the noun and its verb, so every word after the noun in its sentence (from the
/// pass's segmentation) must be accounted for. The sentence must take one of three shapes, and anything else drops the
/// match:
/// 1. the noun ends the sentence: only [`CLOSERS`] follow it ("We will ship in the near future.");
/// 2. exactly two words follow, separated by spaces only, then only [`CLOSERS`]: a word from the noun's list, then one
///    word that is not a relative pronoun, and after "to" a lowercase word that is not in [`NOT_AFTER_TO`] ("They made
///    a decision to leave.");
/// 3. an adverbial row opens the sentence, then a comma and a subject pronoun ("At this point in time, we stop."). The
///    replacement is itself an adverbial and the main clause has its own subject, so nothing after the comma depends
///    on the deleted noun.
/// No other word after the noun is allowed, so a coordinated, compared or appended phrase ("… to leave and a plan to
/// return") is refused however it is joined: there is no room for it (review round 3).
fn noun_final_allows(src: &str, seg: &Segments, m: &Range<usize>) -> bool {
    let phrase = src[m.clone()].to_ascii_lowercase();
    let last = phrase.split_whitespace().next_back().unwrap_or_default();
    let Some(&(_, follow, fronted)) = NOUN_FOLLOW.iter().find(|(noun, _, _)| *noun == last) else {
        return true;
    };
    let Some(noun) = seg
        .word_before(m.end)
        .filter(|&i| seg.words[i].range.end == m.end)
    else {
        return false;
    };
    let Some(sentence) = seg.sentence_of(noun).and_then(|i| seg.sentences.get(i)) else {
        return false;
    };
    let tail = &seg.words[noun + 1..sentence.words.end as usize];
    let text = |w: &Word| src[w.range.clone()].to_lowercase();
    let spaces = |r: Range<usize>| src[r].chars().all(char::is_whitespace);
    let closes = |from: usize| {
        src[from..sentence.range.end]
            .chars()
            .all(|c| CLOSERS.contains(&c))
    };
    match tail {
        [] => closes(m.end),
        [w1, w2] if follow.contains(&text(w1).as_str()) => {
            let (word, then) = (text(w1), text(w2));
            spaces(m.end..w1.range.start)
                && spaces(w1.range.end..w2.range.start)
                && closes(w2.range.end)
                && !RELATIVE.contains(&then.as_str())
                && !(word == "to"
                    && (NOT_AFTER_TO.contains(&then.as_str())
                        || !src[w2.range.clone()].starts_with(|c: char| c.is_lowercase())))
        }
        [w1, ..] if fronted => {
            seg.word_at(m.start)
                .is_some_and(|i| seg.words[i].sentence_initial)
                && src[m.end..w1.range.start].trim() == ","
                && SUBJECT.contains(&text(w1).as_str())
        }
        _ => false,
    }
}

/// Whether a "… the fact that" match at text range `m` of `text` (a `BlockText`'s text) may be reported and fixed:
/// not after a form of "be". `true` for any other row.
fn fact_that_allows(text: &str, m: Range<usize>) -> bool {
    if !text[m.clone()]
        .to_ascii_lowercase()
        .ends_with("the fact that")
    {
        return true;
    }
    let before = text[..m.start].trim_end();
    let start = before
        .char_indices()
        .rev()
        .take_while(|(_, c)| c.is_alphabetic() || *c == '\'' || *c == '\u{2019}')
        .last()
        .map_or(before.len(), |(i, _)| i);
    let prev = before[start..].to_lowercase().replace('\u{2019}', "'");
    let contracted_be = ["'s", "'re", "'m"].iter().any(|s| prev.ends_with(s));
    !(BE_FORMS.contains(&prev.as_str()) || contracted_be)
}

/// Both guards for a source range `WordList::find` returned. `false` when the range has no text offset. Rows that are
/// neither noun-final nor "the fact that" rows always pass, so the 17 existing rows behave as today.
fn allowed(ctx: &RuleCtx<'_>, bt: &BlockText, src: &Range<usize>) -> bool {
    let in_text = match bt.to_text(src.start) {
        Some(s) => {
            // The exclusive end may be closing markup or EOF, which to_text deliberately rejects.
            // WordList returns contiguous prose; verify that range before using its byte length.
            let m = s..s + src.len();
            bt.src_range(m.clone()).as_ref() == Some(src) && fact_that_allows(&bt.text, m)
        }
        None => false,
    };
    in_text && noun_final_allows(&ctx.doc.src, ctx.prose.segments(), src)
}

struct AiVocabularyReplaceable {
    meta: RuleMeta,
    list: WordList,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(AiVocabularyReplaceable {
        meta: RuleMeta {
            code: RuleCode(1),
            name: "ai-vocabulary-replaceable",
            family: Family::AiVocabulary,
            severity: Severity::Warning,
            default_on: true,
            fix: FixClass::Safe,
            explain: "Flags verbs and phrases that pad plain wording: delve into, utilize, in order to, facilitate and \
                      leverage. Each inflection with a safe replacement is fixed; a bare `leverage` is reported with \
                      no fix because it may be a noun. Also fixes nominalisations (make a decision) and padded \
                      phrases (due to the fact that). Guarded phrases are left alone where the replacement might \
                      not fit, such as a decision tree or a decision to leave and a plan to return.",
            example: (
                "We utilize the cache in order to look things up.",
                "We use the cache to look things up.",
            ),
            params: &[],
            since: "0.1.0",
        },
        list: WordList::new(ENTRIES),
    })
}

impl Rule for AiVocabularyReplaceable {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        for bt in ctx.blocks() {
            for (range, replacement) in self.list.find(&bt) {
                if !allowed(ctx, &bt, &range) {
                    continue;
                }
                let message = match &replacement {
                    Some(r) => format!(
                        "`{}` is padded wording; use `{r}`",
                        &ctx.doc.src[range.clone()]
                    ),
                    None => format!(
                        "`{}` is padded wording; plainer wording is needed",
                        &ctx.doc.src[range.clone()]
                    ),
                };
                let fix = replacement.map(|r| (range.clone(), r));
                out.push(ctx.finding(&self.meta, range, message, fix));
            }
        }
    }
}
