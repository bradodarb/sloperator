# Competitive Analysis: classical NLP techniques and the tools that use them

**Date:** 2026-11-01
**Status:** Complete
**Related:** [Current State Analysis](./current-state-analysis.md) | [Feasibility Summary](./feasibility-summary.md)

---

## Purpose

The request names techniques (readability formulas, burstiness, Brill-style tagging, stemming, TextRank) and asks which
are worth adding. This file records what two prose linters (Vale, write-good) do with them, what Rust building blocks
exist, and what the external sources do and do not establish. The question for the PRD: which techniques are proven
linter practice, which are only available as libraries, and which claims in the request have no source.
**Limits:** only the pages listed below were read. Vale's readability and metric check pages returned 404 at the URLs
tried, so Vale's readability formula is **not verified**. The TextRank paper was opened for its bibliographic record only;
its algorithm is not described from the paper here. The `nlprule` crate was inspected through crates.io metadata only.
No Rust tagger crate's lexicon, licence text or accuracy was read.

---

## Sources Surveyed

| Peer system | Version window | Primary doc |
|-------------|----------------|-------------|
| Vale (prose linter) | docs as served 2026-11-01, "last updated 1 month ago" | [Styles][^vale-styles] |
| write-good (naive prose linter, JS) | GitHub master README as served 2026-11-01 | [README][^write-good] |
| Flesch-Kincaid readability tests | Wikipedia revision `oldid=1382449133` | [article][^fk] |
| Brill tagger | Wikipedia revision `oldid=1338528977` | [article][^brill] |
| rust-stemmers 1.2.0 | released 2019-11-17 | [docs.rs][^stemmers], [crates.io][^stemmers-api] |
| Rust POS-tagger crates (search result) | crates.io, 2026-11-01 | [search][^tagger-search] |
| nlprule 0.6.4 | released 2021-04-24 | [crates.io][^nlprule] |
| GPTZero on perplexity and burstiness | page as served 2026-11-01 | [article][^gptzero] |
| TextRank (Mihalcea and Tarau, EMNLP 2004) | 2004 | [ACL Anthology][^textrank] |

---

## 1. Comparison Matrix

### 1.1 What linters do with classical techniques

| Technique | Vale | write-good | Implication for sloperator |
|-----------|------|------------|----------------------------|
| Phrase substitution with a carried replacement | `substitution` check: "Report a pattern and carry its replacement"; the rule's `action` is a fix "an editor or an agent applies without deciding anything"[^vale-styles] | `tooWordy` check for "wordy or unneeded" phrases; the API returns `index`, `offset`, `reason` per suggestion, with no replacement field in the README example[^write-good] | Substitution tables are universal and cheap. sloperator already does this (`WordList` with replacement) and additionally verifies every edit. |
| Weak-word / intensifier flags | `existence` and `occurrence` checks (count a pattern, report too few or too many)[^vale-styles] | `adverb` check "adverbs that can weaken meaning: really, very, extremely"; `weasel` check[^write-good] | Both peers flag intensifiers. Neither is shown deleting them, which is the part with a meaning risk. |
| Passive voice | not shown in the page read | `passive` check, example `'was stolen' is passive voice`[^write-good] | Peers detect passive. Rewriting it is a different problem (needs agent and tense). |
| POS-aware matching | `sequence` check: "Report a sequence of tokens, with part-of-speech tags, in a sentence"[^vale-styles] | not shown | A prose linter in production uses POS tags for matching, so tags are practical. How Vale tags (lexicon or model) was not read. |
| Readability score | `readability` check: "Score a block's prose, and report when the grade is too high"; `metric` check: "Evaluate a formula over the counts of a block"[^vale-styles] | Lists "Flesch–Kincaid readability" under reading resources only, with no check[^write-good] | Readability as a linter check has precedent. The check reports against a threshold, which matches the rule-with-severity route (route (a) in the Feasibility Summary). Vale's exact formula is not verified. |
| Repetition | `repetition` check and built-in `Vale.Repetition` for "the the"[^vale-styles] | `illusion` check for repeated words ("lexical illusions")[^write-good] | Adjacent-word repetition is a different, much cheaper problem than sentence redundancy (candidate 4). |
| Severity and exit code | `level` default `warning`; "Only errors set a non-zero exit code"[^vale-styles] | not shown | sloperator's exit follows findings (FR-2.7 family); metric findings at Suggestion would behave like SLP005. |
| Fix actions | rule-level `action` (replace, remove, edit, suggest)[^vale-styles] | none: suggestions only[^write-good] | sloperator is ahead of both on verified rewriting; this is the reason to keep every new rewrite behind `verify`. |

### 1.2 Building blocks

| Building block | Fact (source) | Implication |
|----------------|---------------|-------------|
| Flesch-Kincaid grade = `0.39 × (words/sentences) + 11.8 × (syllables/words) − 15.59`; Reading Ease = `206.835 − 1.015 × (words/sentences) − 84.6 × (syllables/words)`[^fk] | Both need a syllable count. Neither formula says how to count syllables. | The syllable counter is an implementation choice and a source of error. It must be deterministic (heuristic or dictionary). |
| The grade formula "does not have an upper bound"; the lowest theoretical grade is −3.40; Reading Ease has no lower bound[^fk] | Scores can be extreme on degenerate input. | A metric that feeds the score needs clamping and a minimum-text guard. |
| Readability formulas "neglect between-reader differences and effects of content, layout and retrieval aids"[^fk] | Developed for school books. | Report as a style indicator, not as quality. Fits FR-5.4. |
| Brill tagger: lexicon assigns each known word its most frequent tag, unknown words default to noun, then contextual rules `tag1 → tag2 IF condition` correct errors; "typical Brill taggers use a few hundred rules"; rules may be hand-written or learned from a tagged corpus[^brill] | The technique is small and fully deterministic. The cost is the lexicon and rules, not code. | Hand-written rules plus a lexicon is a legitimate Brill-style design. No accuracy figure was read, so none is claimed. |
| `rust-stemmers` 1.2.0 implements Snowball stemmers incl. English; input must be lowercase; deps `serde`, `serde_derive`; crate 2,351,301 bytes; licence "MIT/BSD-3-Clause"; last release 2019-11-17[^stemmers][^stemmers-api] | Footprint is a few MB. Licence terms appear on sloperator's allow-list (`deny.toml:2-3`), though `cargo deny` was not run. Maintenance has been quiet since 2019. | A stemmer detects inflected matches; it does not produce inflected replacements (see Current State §1.2). |
| Rust Brill-style crates on crates.io: `pos-tagger` 0.2.0 ("lexicon lookup + 8 contextual rewrite rules", 432 downloads), `verbora-tagger` 0.3.0 ("bring your own lexicon", 66 downloads), `english-pos-tagger` 0.2.0 (tagging and lemmatization, 860 downloads), `postagger` 0.0.3 (NLTK-inspired, 4,256 downloads)[^tagger-search] | All are new or low-adoption. Their licences, lexicon provenance and accuracy were not read. | Depending on one is a supply-chain and maintenance risk. Building a small tagger in-tree is also plausible. Evaluating either is a spike, not a finding. |
| `nlprule` 0.6.4: "fast, low-resource NLP and error correction library"; MIT OR Apache-2.0; last release 2021-04-24; default feature `regex-onig`[^nlprule] | A rule-based grammar-correction crate exists. The `onig` feature name suggests a C regex dependency, which was not verified. | Not evaluated beyond metadata. It would add a second regex stack next to `regex = 1.13.1` (`Cargo.toml:39`). |
| "Burstiness" per GPTZero: "a measure of how much writing patterns and text perplexities vary over the entire document"; perplexity comes from a language model. The same page says that as of autumn 2023 GPTZero "no longer uses perplexity and burstiness" as its detection method, then calls them one of seven indicators[^gptzero] | The detector's burstiness is variation in model-assigned perplexity, **not** sentence-length variance. The page is internally inconsistent about current use. | The request calls sentence-length variance "a known AI tic". No source read here establishes that. Sentence-length variance is a proxy defined by this project. |
| TextRank is a 2004 EMNLP paper by Rada Mihalcea and Paul Tarau[^textrank] | Only the record was read. | No claim about its algorithm or about summarisation quality is made here. |

---

## 2. Analysis

**Universal:** substitution tables with a stated replacement, weak-word flagging and readability or metric checks reported against a
threshold are all present in Vale's check vocabulary[^vale-styles]. write-good ships the weak-word and wordy-phrase checks[^write-good].

**Where peers diverge:** only Vale offers fix actions and POS-aware sequence checks[^vale-styles]. write-good reports only
(`index`, `offset`, `reason`)[^write-good]. Neither peer verifies a rewrite.

**What this implies here:** (1) candidates 1 and 6 follow established linter practice and carry the least novel risk;
(2) POS tagging is feasible in a linter, but the available Rust crates are young, so the lexicon and rules are the real
work; (3) the "burstiness is an AI tic" premise should be treated as a project hypothesis to calibrate, not a sourced
fact; (4) sloperator's differentiator, verified rewriting, is the thing a classical NLP addition must not weaken.

---

## 3. References

- [Current State Analysis](./current-state-analysis.md)
- `docs/research/competitive-analysis.md` — the parent initiative's study of Ruff and Vale CLI conventions

[^vale-styles]: Styles | Vale. https://vale.sh/docs/topics/styles/ (canonical https://docs.vale.sh/topics/styles). Retrieved 2026-11-01. Lists Vale's check types (existence, substitution, occurrence, repetition, consistency, metric, readability, sequence with POS tags, script), rule fields (`level` default warning, `action`), and the exit-code rule. Does not give the readability formula.
[^write-good]: btford/write-good README. https://github.com/btford/write-good. Retrieved 2026-11-01. Lists the checks (passive, illusion, so, thereIs, weasel, adverb, tooWordy, cliches, eprime) and the suggestion shape (index, offset, reason).
[^fk]: Flesch–Kincaid readability tests. https://en.wikipedia.org/wiki/Flesch%E2%80%93Kincaid_readability_tests. Retrieved 2026-11-01. Gives both formulas, the unbounded-score and degenerate-input behaviour, and the limitation that formulas neglect reader and content differences. Wikipedia is a secondary source; it cites Kincaid et al. 1975 (DTIC ADA006655), which was not opened.
[^brill]: Brill tagger. https://en.wikipedia.org/wiki/Brill_tagger. Retrieved 2026-11-01. Describes lexicon initialisation, unknown-word default, contextual rule form, and "a few hundred rules". Secondary source; cites Brill 1992 (doi 10.3115/974499.974526), not opened.
[^stemmers]: rust-stemmers 1.2.0. https://docs.rs/crate/rust-stemmers/latest. Retrieved 2026-11-01. Snowball algorithms in Rust, English included; lowercase input; dependencies; release date.
[^stemmers-api]: crates.io API record for rust-stemmers. https://crates.io/api/v1/crates/rust-stemmers. Retrieved 2026-11-01. Licence "MIT/BSD-3-Clause", crate size 2,351,301 bytes, version 1.2.0 created 2019-11-17.
[^tagger-search]: crates.io search "part of speech tagger". https://crates.io/api/v1/crates?q=part%20of%20speech%20tagger&per_page=8. Retrieved 2026-11-01. Names, versions, descriptions and download counts of the Rust tagger crates listed above. Nothing beyond metadata was read.
[^nlprule]: crates.io API record for nlprule. https://crates.io/api/v1/crates/nlprule. Retrieved 2026-11-01. Description, licence "MIT OR Apache-2.0", version 0.6.4, release date, default feature `regex-onig`.
[^gptzero]: Perplexity, burstiness, and statistical AI detection (GPTZero). https://gptzero.me/news/perplexity-and-burstiness-what-is-it/. Retrieved 2026-11-01. Vendor definition of burstiness as variation of perplexities, and the vendor's statement about no longer relying on it alone. Vendor marketing, not a peer-reviewed source.
[^textrank]: TextRank: Bringing Order into Text. https://aclanthology.org/W04-3252/. Retrieved 2026-11-01. Bibliographic record (EMNLP 2004, pages 404-411) only; the paper body was not read here.

**Retrieval date note:** the environment has no clock tool; 2026-11-01 is the latest timestamp seen in fetched data (a crates.io record updated that day), so retrieval dates may be off by a day.
