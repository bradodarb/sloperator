//! Counts and multisets for negations, capitalized words and modals (classical-nlp SPEC §23).

use std::collections::BTreeMap;

use unicode_segmentation::UnicodeSegmentation;

use crate::text::lexicon;
use crate::types::{ProposalOrigin, RuleCode};

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Meaning {
    pub(super) negations: u32,
    /// Capitalised, non-sentence-initial words (FR-6.2's proxy); common words skipped only when `skip_common`.
    pub(super) proper: BTreeMap<String, u32>,
    pub(super) modals: BTreeMap<String, u32>,
}

/// D-8: an edit that resolves these may drop one negation per finding.
const NEGATION_EXEMPT: [RuleCode; 2] = [RuleCode(3), RuleCode(4)];

fn bump(map: &mut BTreeMap<String, u32>, key: &str) {
    let slot = map.entry(key.to_owned()).or_insert(0);
    *slot = slot.saturating_add(1);
}

/// `initial_at_start`: whether the text's first word starts a sentence in the document. Inside the text a word is
/// sentence-initial when the nearest non-space text before it (closing quotes and brackets skipped) ends in `.`,
/// `!` or `?`. `skip_common`: whether capitalised common words ([`lexicon::is_common_capitalised`]) are left out of
/// the proper-noun multiset. `verify::edit` sets it for rule-origin edits only (OD-2 option (a)).
pub(super) fn meaning(text: &str, initial_at_start: bool, skip_common: bool) -> Meaning {
    let mut m = Meaning::default();
    let mut prev_end: Option<usize> = None;
    for (at, word) in text.unicode_word_indices() {
        let gap = text[prev_end.unwrap_or(0)..at]
            .trim()
            .trim_end_matches(['"', '\'', '\u{201D}', '\u{2019}', ')', ']']);
        let initial = if gap.is_empty() {
            prev_end.is_none() && initial_at_start
        } else {
            gap.ends_with(['.', '!', '?'])
        };
        m.negations = m.negations.saturating_add(lexicon::negation_count(word));
        if let Some(modal) = lexicon::modal_of(word) {
            bump(&mut m.modals, modal);
        }
        let common = skip_common && lexicon::is_common_capitalised(word);
        if !initial && word.chars().next().is_some_and(char::is_uppercase) && !common {
            bump(&mut m.proper, word);
        }
        prev_end = Some(at + word.len());
    }
    m
}

/// How many negations the edit may drop (D-8): 1 for a safe fix of SLP003/SLP004; for a model rewrite, one per
/// SLP003/SLP004 finding in its window (`resolves`, from `Window.codes`).
pub(super) fn negation_allowance(origin: ProposalOrigin, resolves: &[RuleCode]) -> u32 {
    match origin {
        ProposalOrigin::Rule(code) => u32::from(NEGATION_EXEMPT.contains(&code)),
        ProposalOrigin::Model => u32::try_from(
            resolves
                .iter()
                .filter(|c| NEGATION_EXEMPT.contains(c))
                .count(),
        )
        .unwrap_or(u32::MAX),
    }
}
