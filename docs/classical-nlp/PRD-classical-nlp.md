# PRD: Classical (Non-Neural) NLP for Sloperator

**Version:** 0.1
**Date:** 2026-11-01
**Status:** Draft
**Module:** new pure text-analysis modules (metrics, segmentation, tagging), plus `src/rules/`, `src/verify/`, `src/report/`, `src/mcp/`
**Request:** Add traditional, non-neural NLP techniques so sloperator does more with no model and leans less on the model when one is present. Everything stays local and deterministic, and every rewrite still goes through verify. Text metrics (readability, sentence-length variation, lexical diversity) are Brad's priority.
**Companion Research:**
- [Current State Analysis](./research/current-state-analysis.md) — what exists in the code and which contracts constrain the work
- [Competitive Analysis](./research/competitive-analysis.md) — what Vale and write-good do, and which claims have no source
- [Feasibility Summary](./research/feasibility-summary.md) — pillars, risks, effort, open questions

**Parent / Related PRDs:** [Sloperator PRD](../PRD.md) — the product contract this initiative extends. It supplies the FR/NFR numbers cited below (score FR-5.1 and FR-5.4, rule defaults FR-4.5, performance NFR-3, determinism NFR-6, no shims NFR-9). They are cited as the research cites them.

---

## 1. Problem Statement

Sloperator finds and fixes "AI tells" with 17 rules, one verifier and an optional local model. Without a model, `--fix` handles only what a phrase table or a hand-built guard can prove. Everything else is left in place or sent to the model. The tool also cannot say how a text reads. It reports a score from weighted findings per 1,000 words, so a document with flat sentence rhythm and clean phrasing scores 0. The verifier holds model rewrites to a lower standard than a human editor: it cannot see negation, names or modal strength. Classical NLP techniques are small, deterministic and local. They can close each of these gaps without weakening the verify-every-edit contract.

### Root Causes

1. **No text metrics exist.** A search of `src/` and `tests/` for readability, burstiness, TF-IDF, syllable, lemma, stemmer and POS-tag code finds nothing but one comment (current-state headline, §8). The score counts only weighted findings (`src/score/mod.rs:26-36`).
2. **No shared sentence, token or tag layer.** Sentence splitting is private to the window code (`src/engine/window.rs:241-259`). `prose_words` is the only word helper (`src/doc/mod.rs:235-241`). `RuleCtx` holds only `doc`, `params` and `severity` (`src/rules/mod.rs:84-97`), and `detect` runs every rule independently (`src/rules/mod.rs:158-177`). Nothing is computed once and shared (current-state §1.1, §5, §8).
3. **Phrase tables are small and inflection-blind.** SLP001 has 17 rows, with every inflection spelled out as its own row (`src/rules/builtin/slp001_ai_vocabulary_replaceable.rs:1-4`, `:8-27`). Nominalizations such as `make a decision` have no rows, and `WordList` matches exact phrases with a fixed replacement per row (`src/rules/helpers.rs:14-38`; current-state §1.2).
4. **Safe fixes depend on hand-built guards.** SLP006's "tagger" is three word lists (`src/rules/builtin/slp006_copula_avoidance.rs:22-35`). Widening it means growing lists or having real tags (current-state §8).
5. **Four tells are Model-only, so without a model they are never fixed.** SLP003, SLP004, SLP007 and SLP015 are Model-class (`slp003…rs:26`, `slp004…rs:27`, `slp007…rs:27`, `slp015…rs:25`). Windows are selected from Model findings (`src/engine/window.rs:110-118`).
6. **The verifier cannot see meaning.** Its header says it "covers numbers, URLs, terms and block structure, not names or claim polarity" (`src/verify/mod.rs:5`), and `tests/score.rs:102-112` pins that limit in `docs/REFERENCE.md`. Verification has no polarity, proper-noun or modality check (`src/verify/mod.rs:38-51`).
7. **No sentence-level redundancy detection and no calibration data.** No similarity code exists (current-state §8). The eval corpus is 20 model-style paragraphs of about 35-55 words each, and the README says none is human-written (`eval/README.md:9-12`, `:25-28`). Metric thresholds cannot be calibrated from anything in the repo.

### Impact

- **No-model users get no reading feedback.** A document can be flat, padded and monotone and still score 0 and exit clean.
- **`--fix` without a model leaves most tells.** Only SLP001, SLP006 (after seven listed subjects) and the other Safe rules fix anything (`slp006…rs:22`). Everything else needs a model.
- **Model users pay for cases a rule could settle.** Every Model-class or fix-less Safe finding selects a model window (`src/engine/window.rs:110-118`), and each accepted window reruns all rules on the whole candidate (`src/pipeline/mod.rs:264-291`).
- **Model rewrites can flip meaning unnoticed.** A rewrite that drops a "not", a name or a "must" passes `verify::edit` if numbers, URLs, terms and structure survive (`src/verify/mod.rs:5`).

---

## 2. Scope

### In Scope

- **Text metrics (priority)** — average sentence length, Flesch-Kincaid grade, sentence-length variation (the deterministic proxy for "burstiness") and type/token ratio. They are computed over prose, always reported in text, JSON and MCP output, with before and after values for `fix`. They feed the score through rules that raise findings above thresholds.
- **Shared segmentation core** — one public, protected-span-aware sentence and word iterator, used by metrics, the tagger and redundancy.
- **Plain-language and wordiness tables** — extended phrase tables with explicit per-inflection rows, including nominalizations.
- **Verifier invariants** — negation, proper-noun and modal-strength checks inside the one verifier, covering safe fixes and model rewrites alike.
- **In-tree lexicon-plus-rules (Brill-style) tagger** — used only as a conservative gate for Safe fixes. It widens SLP006 beyond its listed subjects.
- **Narrow template rewrites** for SLP004 and SLP003 where the guards prove the shape.
- **Redundancy detection** — sentence-overlap detection, report only.
- **Measurement of model reliance** — a rules-only baseline of model-eligible findings per 1,000 words on the eval corpus.
- **A licence-clean human-prose calibration corpus** to calibrate metric thresholds (a follow-up of Phase 1, a few MB).

### Out of Scope

- **Extractive summarization (TextRank)** — it produces new text, and the product edits in place and fails on a new block marker (`src/verify/mod.rs:115`; feasibility "NOT building"). No source on its algorithm was read. It would need its own initiative (D-11).
- **Passive-voice rewriting** — needs agent and tense recovery beyond a lexicon tagger. Passive detection and rewriting are not scoped this round.
- **Template rewrites for SLP007 and SLP015** — their own `explain` text says no meaning-preserving rewrite exists (`slp007…rs:28-30`, `slp015…rs:26-29`), so a template cannot be Safe.
- **Perplexity-based or model-based burstiness** — GPTZero's sense of burstiness is variation in model perplexity (competitive §1.2), which needs a language model. Only the sentence-length proxy is in scope.
- **AI-text detection or provenance claims** — out of scope for the product (`docs/PRD.md:58`). Metrics are style indicators (FR-5.4, `docs/PRD.md:226`).
- **Non-English text** — all lexicons, syllable rules and stemmers are English only (`docs/PRD.md:61`).
- **User-authored rules or lexicons** — out of scope for the product (`docs/PRD.md:60`; ADR-06).
- **A new term in the score formula** — would change every recorded score and the pinned JSON (`tests/score.rs:96-98`); see D-2.
- **A `--max-score` failure mode** — out of scope for the product (`docs/PRD.md:65`).
- **Deleting redundant sentences automatically** — a deleted sentence cannot be verified.

---

## 3. Resolved Decisions

### D-1: Priority order of the pillars

Delivery order is: (1) shared segmenter and text metrics; (2) wordiness tables; (3) verifier invariants; (4) tagger and SLP006 widening; (5) SLP004/SLP003 templates; (6) redundancy. TextRank is out.

**Rationale:** The request names metrics as Brad's priority. The segmenter is shared by metrics, the tagger and redundancy, so building it first saves cost (feasibility effort note). Wordiness tables are small and need no new infrastructure. The verifier invariants come before the tagger and the templates because those widen what safe fixes can do. The research recommends this order (feasibility "What Needs Work").

### D-2: Metrics feed the score through rules; the formula and `Score` shape do not change

Metrics are always reported as raw values, and rules turn them into ordinary findings above a threshold. The score formula (`round2(1000 × Σ weight / max(prose_words, 1))`, `src/score/mod.rs:50-56`) and the pinned `Score` JSON (`tests/score.rs:96-98`) stay as they are. Brad accepted the recommendation (option b) at the PRD gate.

**Rationale:** A metric finding flows through `--min-severity`, config params, directives, `explain`, JSON and the existing formula with no formula change. SLP005 and SLP014 already work this way (`slp005…rs:52-76`; feasibility OQ-1). A new term in the formula would invalidate all recorded scores and change the meaning of FR-5.1 (current-state §4). Brad's gate answer: use the recommended option.

### D-3: Sentence-length variation is the user-facing name; "burstiness" is documented as an alias

The metric is computed as the standard deviation and the coefficient of variation of sentence lengths in words. Output and docs call it "sentence-length variation". REFERENCE explains that it is a style indicator and not the perplexity-based burstiness that detectors use. Brad accepted this recommendation at the PRD gate.

**Rationale:** The "known AI tic" claim has no source read (competitive §1.2). GPTZero defines burstiness as variation in model perplexity and contradicts itself on current use. Naming the proxy honestly keeps FR-5.4 true. Brad's gate answer: use the recommended option.

### D-4: Metric rules are default-off, Suggestion, fix-less, with provisional thresholds; raw metrics are always reported

Metric rules follow FR-4.5 (a later-release rule starts off), carry the Suggestion severity and have no fix. A fix-less finding never selects a model window (`src/engine/window.rs:117`). Their thresholds are provisional, documented as such, and exposed as rule params. Brad accepted the recommendations on calibration (provisional thresholds, default-off rules) and on defaults at the PRD gate (D-11).

**Rationale:** There is no human reference text in the repo, so thresholds cannot be calibrated now (current-state §8). Default-off keeps existing scores unchanged. Always-on raw metrics still deliver Brad's priority. Each metric rule's `explain` must state that its threshold is provisional until the calibration corpus exists (Phase 1).

### D-5: Metrics are computed over paragraph blocks only, with minimum-size guards

Headings, list items and table cells are excluded from sentence, grade and diversity statistics. Below a minimum sentence and word count a metric is reported as unavailable (not as a number) and no metric finding is raised. The provisional minimums are 5 sentences and 100 words, configurable as params. The word definition is the one `prose_words` uses (`src/doc/mod.rs:235-241`), so the score denominator and the metrics agree.

**Rationale:** Heading and bullet fragments skew sentence statistics by construction (`src/doc/mod.rs:38-52`; current-state §5). The grade has no upper bound and degenerate input gives extreme values (competitive §1.2). This resolves research OQ-9. The minimums are provisional and the corpus calibrates them.

### D-6: Metrics aggregate from raw counts

Per-file and overall values derive from summed integers (words, sentences, syllables, distinct types, per-sentence lengths), in one fixed summation order, rounded to two decimals at output. The overall value is never a mean of per-file values.

**Rationale:** This matches `score::overall`'s word-weighted raw-sum rule (`src/score/mod.rs:39-47`), keeps output byte-stable (`docs/SPEC.md:1136`) and meets NFR-6 determinism.

### D-7: The tagger is in-tree with an owned lexicon, and it is a gate only

The tagger is a small lexicon (closed-class words plus frequent ambiguous words) with a few hundred contextual rules at most. A Safe fix is attached only where every involved word has a single unambiguous tag. Otherwise no fix is attached and the finding falls through to the model, as SLP001 and SLP006 already do (`src/engine/window.rs:110-118`). No third-party tagger crate or third-party lexicon is used.

**Rationale:** The candidate Rust tagger crates are young and low-adoption, and their licences, lexicons and accuracy were not read (competitive §1.2). The Brill technique is small and deterministic, and the cost is the lexicon and rules (competitive §1.2). Gate-only use keeps the SLP006 standard that "part of speech, agreement and the phrase after as are all certain" (`slp006…rs:4-5`). This resolves research OQ-4.

### D-8: The negation invariant has a defined exemption

The negation check applies to every edit and rejects any edit that reduces the negation-token count. The only exemption is an edit that resolves an SLP003 or SLP004 finding. For rule-origin edits this is keyed by the rule code in the proposal origin (`src/types.rs:58-61`). For model-origin edits, the SPEC defines how the window's finding set determines the exemption.

**Rationale:** SLP003 and SLP004 exist to remove a "not" (`slp003…rs:31`, `slp004…rs:31-34`), so an unconditional check would reject the rewrite the rules make (current-state §3). This resolves research OQ-5. How a model-origin edit carries its window's findings is a SPEC detail.

### D-9: Dependencies are in-tree first; a stemmer is decided at the redundancy phase

No new runtime dependency is added before Phase 6. In Phase 6 the PRD allows `rust-stemmers` (Snowball, English; MIT/BSD-3-Clause; about 2.3 MB; last release 2019-11-17, per competitive §1.2), pinned exactly, if `cargo deny` passes. Otherwise it uses an in-tree suffix-stripping stemmer. Redundancy requirements do not depend on which is chosen. Bundled data is embedded in the binary, so offline builds and `--no-default-features` builds keep every classical feature (feasibility A-4). Total added footprint stays far under the 1 GB limit, expected to be a few MB.

**Rationale:** Only redundancy needs stemming, because the wordiness tables use explicit inflection rows (feasibility Pillar 2). A stemmer detects inflected matches but cannot produce inflected replacements (current-state §1.2). This resolves research OQ-10.

### D-10: Safe means grammatical and truth-conditionally neutral; emphasis-only deletions are detection only

A Safe fix must keep grammaticality and the claim. Deleting an intensifier or an adverb (`very`, `really`, `extremely`) changes strength, and the verifier does not check strength. Those deletions are therefore detection-only in this round. The SLP004 and SLP003 templates stay in scope because they restate the same claim. Brad accepted this recommendation at the PRD gate (option i); no new fix class is added.

**Rationale:** Vale and write-good flag weak words, and neither is shown deleting them (competitive §1.1). The deletion carries the meaning risk. Brad's gate answer: use the recommended option. Intensifier and adverb deletion, which the request listed as candidates, is therefore detection-only this round; a low-confidence fix class would be a public-surface change and needs its own decision later.

---

## 4. Functional Requirements

### FR-1: Shared segmentation

The system SHALL provide one prose segmenter that all classical features use.

- FR-1.1: The segmenter SHALL yield, per paragraph block, the sentences and the words of that sentence, with source ranges.
- FR-1.2: The segmenter SHALL treat protected spans and `OBJ` placeholders as non-words and SHALL NOT split a sentence at a boundary inside a protected span.
- FR-1.3: The segmenter's word count over the whole document SHALL equal `prose_words` for the same document when headings, list items and table cells are included.
- FR-1.4: The window code SHALL use the same sentence segmentation as the segmenter, so there is one definition of a sentence in the crate.
- FR-1.5: Segmentation SHALL be a pure function of the document with no I/O.

### FR-2: Text metrics computation

The system SHALL compute the metrics from raw counts over paragraph blocks.

- FR-2.1: The system SHALL compute, per file, the counts: words, sentences, syllables, distinct word types (case-folded) and the list of per-sentence word lengths.
- FR-2.2: The system SHALL derive the average sentence length as words divided by sentences.
- FR-2.3: The system SHALL derive the Flesch-Kincaid grade as `0.39 × (words/sentences) + 11.8 × (syllables/words) − 15.59` (competitive §1.2).
- FR-2.4: The system SHALL derive sentence-length variation as the population standard deviation of per-sentence word counts and as its coefficient of variation (standard deviation divided by mean).
- FR-2.5: The system SHALL derive the type/token ratio as distinct types divided by words.
- FR-2.6: The syllable counter SHALL be deterministic and offline, and REFERENCE SHALL document it as approximate.
- FR-2.7: When a file has fewer sentences or words than the configured minimums (D-5), each derived metric SHALL be reported as unavailable and not as a number.
- FR-2.8: The overall metrics across files SHALL be derived from summed raw counts, never from a mean of per-file values.
- FR-2.9: Every reported derived value SHALL be rounded to two decimals, and the same input SHALL yield byte-identical output on repeated runs.
- FR-2.10: Metrics SHALL exclude headings, list items, table cells and protected spans.

### FR-3: Metrics in reports

The system SHALL surface the metrics wherever the score is surfaced, without altering the score.

- FR-3.1: The text report SHALL print a metrics line per file, next to the score line, listing the available metrics and marking unavailable ones.
- FR-3.2: The JSON report SHALL include a `metrics` object per file and one overall, as a sibling of the existing score object. The `Score` object's shape SHALL be unchanged.
- FR-3.3: The MCP lint result SHALL include the metrics. The MCP fix result SHALL include metrics before and after.
- FR-3.4: The `fix` flow SHALL compute metrics on the original and on the final text, and report both.
- FR-3.5: The `concise` and `github` formats SHALL continue to print no metrics, as they print no score today.
- FR-3.6: REFERENCE SHALL state the formulas, the block kinds counted, the minimum-size guards and that metrics are style indicators and not quality or provenance (FR-5.4).

### FR-4: Metric rules

The system SHALL turn metrics into findings through rules with thresholds.

- FR-4.1: The system SHALL add one rule per metric family: grade level too high, sentence-length variation too low, average sentence length too long, and lexical diversity too low. Each is a document-level finding reported on a prose range, as SLP010 and SLP011 report (`src/rules/helpers.rs:199`).
- FR-4.2: Each metric rule SHALL be default-off, at Suggestion severity, with no fix, and SHALL declare its threshold and minimums as params so config validates names against them.
- FR-4.3: A metric rule SHALL raise no finding when its metric is unavailable under FR-2.7.
- FR-4.4: A metric finding SHALL NOT select a model window.
- FR-4.5: Each metric rule's `explain` SHALL say that its threshold is provisional until calibrated against human prose.
- FR-4.6: Enabling a metric rule SHALL change the score only through the existing formula, by adding its Suggestion-weighted finding.
- FR-4.7: Each metric rule SHALL have a fixture directory with a positive and a negative case. A rule with no fixture fails `every_rule_has_fixtures`.

### FR-5: Wordiness and plain-language tables

The system SHALL replace padded phrasing from extended tables.

- FR-5.1: The system SHALL extend the Safe wordiness table with explicit rows for each inflection of each covered phrase, including nominalizations (for example `make a decision` → `decide`, `made a decision` → `decided`, `making a decision` → `deciding`) and phrase reductions (for example `in order to` → `to`).
- FR-5.2: Each table row SHALL have a positive fixture, and each Safe row SHALL appear in the rule's `fixed.md`, with the fixed output grammatical.
- FR-5.3: A row whose replacement is not certain SHALL carry no fix, so the finding falls through to the model.
- FR-5.4: Intensifier and weak-adverb entries SHALL be detection-only, with no replacement (D-10).
- FR-5.5: Applying the safe fixes of the extended tables to their own `fixed.md` output SHALL produce no further edit (idempotency, as `tests/rules_fixtures.rs:526` checks).

### FR-6: Verifier invariants

The system SHALL reject edits that damage polarity, names or modality, for every edit origin.

- FR-6.1: `verify::edit` SHALL reject an edit that reduces the multiset of negation tokens (`not`, `no`, `never`, `n't` forms and the SPEC's closed list), except as D-8 allows.
- FR-6.2: `verify::edit` SHALL reject an edit that removes or changes a capitalised token that is not sentence-initial (a proper-noun proxy), and the check SHALL ignore sentence-initial words so SLP008's lead-in deletion and its capitalisation (`src/rules/helpers.rs:165`) still pass.
- FR-6.3: `verify::edit` SHALL reject an edit that changes the multiset of modal verbs (`can`, `could`, `may`, `might`, `must`, `shall`, `should`, `will`, `would`).
- FR-6.4: Each new check SHALL be a distinct `Check` variant, with a name in the pipeline's check-name mapping, so a rejected edit reports which invariant failed.
- FR-6.5: A rejected safe fix SHALL surface as SLP901, as today (`tests/pipeline.rs:117`).
- FR-6.6: Over every existing Safe fixture, no fix that passes verification today SHALL be newly rejected.
- FR-6.7: The verifier module header, the REFERENCE sentence about verification limits, and `tests/score.rs:102-112` SHALL be updated together in the same change, to state the new coverage and its remaining limits (it does not detect all meaning change).
- FR-6.8: Every `Check` variant added SHALL have a rejection test and an acceptance test in `tests/verify_edits.rs`.

### FR-7: Tagger and widened Safe fixes

The system SHALL tag prose deterministically and use tags only as a gate.

- FR-7.1: The tagger SHALL assign each word one tag from its lexicon, mark unknown words with a default tag and an "unknown" flag, and apply contextual rules in a fixed order.
- FR-7.2: The tagger SHALL be a pure function of the segmented text, with embedded data and no I/O.
- FR-7.3: Tagging SHALL be computed at most once per document per detection pass and shared by rules, not recomputed per rule.
- FR-7.4: Tagging SHALL be lazy: a document with no candidate phrase hit SHALL NOT be tagged.
- FR-7.5: SLP006 SHALL attach a Safe fix for a subject outside its current listed subjects only when the tagger gives every involved word a single unambiguous tag and the agreement and following-phrase guards (`slp006…rs:112-127`) pass.
- FR-7.6: Where any involved word is unknown or ambiguous, SLP006 SHALL attach no fix, so the finding falls through to the model.
- FR-7.7: Every existing SLP006 fixture SHALL still produce the same output.
- FR-7.8: `tests/safe_fix_contexts_grammatical` SHALL gain a context for each newly covered subject class, and each class SHALL have a negative fixture where the tagger must decline.

### FR-8: Narrow template rewrites

The system SHALL fix SLP004 and SLP003 only where the shape is proven (D-10).

- FR-8.1: SLP004 SHALL attach a Safe fix only for the shape `not just X, but also Y` (or the equivalent in its patterns, `slp004…rs:9-12`) when X and Y are each a single word with the same tag and the match lies within one sentence. The fix is `X and Y`.
- FR-8.2: SLP003 SHALL attach a Safe fix only for the shape `it's not X, it's Y` when it lies within one sentence and X and Y are each at most a configured number of words (provisionally 3). The fix is `it's Y`.
- FR-8.3: Both rules SHALL attach no fix, and so fall through to the model, when any guard fails.
- FR-8.4: Both rules SHALL change their rule class to Safe and their `explain` text to say when a fix is attached and when not.
- FR-8.5: The SLP003 and SLP004 fixes SHALL pass the negation check only through the D-8 exemption, and SHALL pass the proper-noun and modal checks unmodified.
- FR-8.6: SLP007 and SLP015 SHALL keep their Model class and `explain` text unchanged.
- FR-8.7: Fixtures for both rules SHALL include a `fixed.md` and negative cases where the guards decline, and the pair SHALL pass the idempotency test.

### FR-9: Redundancy detection

The system SHALL flag sentences that repeat earlier content in the same document.

- FR-9.1: The system SHALL add a document-level rule that compares sentences within a paragraph block range by overlap of stemmed content words, after removing a stop-word list, using Jaccard or cosine over term counts.
- FR-9.2: The rule SHALL raise a finding on the later sentence when its overlap with an earlier one is at or above a threshold, declared as a rule param.
- FR-9.3: The rule SHALL be default-off, at Suggestion severity, with no fix.
- FR-9.4: The rule SHALL skip sentences with fewer content words than a minimum param, and documents with fewer sentences than a minimum param.
- FR-9.5: The rule SHALL run in time linear in sentences times the paragraph's sentence count (pairwise within a block range), and SHALL be covered by the performance test (NFR-2).
- FR-9.6: The rule's `explain` SHALL say that it is detection only because deleting a sentence cannot be verified.

### FR-10: Model-reliance measurement

The system SHALL measure how much the classical work reduces what reaches the model.

- FR-10.1: The system SHALL provide a no-model test or script that runs a rules-only fix over the eval corpus and reports the residual count of Model-class and fix-less Safe findings per 1,000 prose words.
- FR-10.2: The baseline value SHALL be recorded before the first rule or table change, and the value after each phase SHALL be recorded beside it in the same file.
- FR-10.3: The residual per 1,000 words after the final phase SHALL be no higher than the baseline.

### FR-11: Rule-set and defaults

- FR-11.1: Adding rules SHALL bump `RULE_SET_VERSION` (`src/rules/mod.rs:146`) and add a CHANGELOG entry.
- FR-11.2: New rules SHALL follow FR-4.5 of the parent PRD (start off), except the rows added to the existing wordiness rule, which keep that rule's current default (D-11).
- FR-11.3: Each new rule SHALL appear in `explain`, in the rule catalogue and in REFERENCE, with its params.

---

## 5. Non-Functional Requirements

### NFR-1: No Behavioral Regression

With every new rule at its default and no new row triggered, the output for existing inputs is unchanged. Verified by: the existing tests of the touched modules pass unmodified, apart from the single documented edit in FR-6.7 and the pinned JSON test updated for the added `metrics` object.

### NFR-2: Performance

Rules-only check of 100 KB of prose finishes in under 1 second with all new rules enabled, and the model path does not recompute tagging or segmentation per window. Verified by: `tests/perf.rs:87` extended to enable the new rules (parent NFR-3), plus a test that counts tagger invocations across a multi-window fix and finds one per document pass.

### NFR-3: Determinism

Same input, same config and same build produce byte-identical metrics, findings and reports. Counts accumulate as integers in one fixed order, and floats are formatted at fixed precision. Verified by: a repeated-run test on the same document asserting identical JSON, as `tests/score.rs:82` does for the score (parent NFR-6).

### NFR-4: Fail Loudly

Config validation rejects an unknown param name or an out-of-range value (a negative threshold, a minimum of zero sentences) for a new rule, naming the param and the rule. A corrupt or missing embedded lexicon or syllable table is a build-time or startup failure, never a silent fallback to untagged behaviour. Verified by: config tests with bad params, and a unit test that loads each embedded data file and asserts it parses and is non-empty.

### NFR-5: Local and Offline

No classical feature opens a network connection or reads data from outside the binary and the document. Bundled data is embedded in the binary. Total added footprint (binary and repo) stays under 50 MB and far under the 1 GB limit. Verified by: the existing no-network test pattern run over a lint and a fix of a metrics-enabled document, and a recorded size check of the release binary before and after.

### NFR-6: Every Edit Verified

No new rewrite path bypasses `verify::edit`. Verified by: `tests/pipeline.rs` cases showing a failing new-rule fix is rejected and reported as SLP901, and a review of the diff showing no new call to apply an edit that skips verification (ADR-05).

### NFR-7: Layering

New pure modules (segmentation, metrics, tagging, similarity) depend on no I/O, engine, write, report or input module, and each gets a row in the layering test. `src/rules/` still does not use `verify`. Verified by: `tests/layering.rs` with the new rows.

### NFR-8: Thread Safety

Analysis runs in parallel across files while the model runs serially (ADR-09). The new modules hold no mutable global state; any per-document cache lives in the per-document analysis context. Verified by: the existing parallel-analysis tests pass, and the new types are `Send + Sync` by a compile-time assertion.

### NFR-9: Clean Breaking Changes

Changes to the verification doc sentence, the `Check` set, rule classes and the report JSON update every call site, with no compatibility shims or deprecated aliases (parent NFR-9, `docs/PRD.md:323`). Verified by: grep for the old sentence and old rule classes returns nothing, and the build has no `allow(dead_code)` added for compatibility.

### NFR-10: Test Coverage

Required: a unit test per metric formula against hand-computed values; a unit test for the syllable counter on a fixed word list; a fixture directory per new rule; one rejection and one acceptance test per new `Check`; tagger tests for known, unknown and ambiguous words; an end-to-end lint and fix test with metrics on; a rules-only baseline run (FR-10). Verified by: the listed tests exist and pass under `make -f "$AEGIS_DIR/Makefile" test`.

---

## 6. Architecture Overview

```
BEFORE:
  source
    -> parse (spans, blocks)
    -> each rule scans blocks on its own (phrase tables, regexes, three word lists in one rule)
    -> findings -> safe fixes -> verify (numbers, URLs, terms, structure)
                -> Model findings -> model window -> verify -> keep if fewer findings
    -> score = weighted findings per 1,000 words
    -> report: score, findings

AFTER:
  source
    -> parse (spans, blocks)
    -> segment once (sentences, words; protected spans excluded)
         |-> metrics (raw counts -> derived values)      [always computed]
         |-> tagger (lazy, once per document)           [only when a candidate phrase hits]
         '-> sentence similarity                        [only when the redundancy rule is on]
    -> rules: phrase tables (more rows), tag-gated Safe fixes, template Safe fixes,
              metric rules (default off), redundancy rule (default off)
    -> findings -> safe fixes -> verify (numbers, URLs, terms, structure,
                                         + negation, proper nouns, modals)
                -> hard cases (no certain fix) -> model window -> same verify -> keep if fewer findings
    -> score = weighted findings per 1,000 words        [formula unchanged]
    -> report: score, metrics (before/after on fix), findings
```

```
Where the model is needed, before and after:

  Finding class           Before                    After
  ----------------------  ------------------------  -------------------------------------------
  wordy phrase            fixed by 17 table rows    fixed by more rows; uncertain rows fall through
  SLP006 copula           7 listed subjects         tag-certain subjects; uncertain fall through
  SLP004 / SLP003         model only                template when shape is proven, else model
  SLP007 / SLP015         model only                model only (unchanged)
  intensifiers, adverbs   not handled               detection only
  metrics, redundancy     not handled               report and Suggestion findings, never model
```

### Module Impact

```
crate
  ├── text segmentation: new — one sentence and word iterator over prose
  ├── text metrics: new — raw counts, derived values, aggregation
  ├── tagger and lexicon: new — lexicon, contextual rules, embedded data
  ├── similarity: new — content-word overlap between sentences (Phase 6)
  ├── rules: updated — new tables, new metric and redundancy rules, SLP004/SLP003/SLP006 changes, per-document analysis slot
  ├── engine (windows): updated — uses the shared sentence definition
  ├── verify: updated — three new invariants, new Check variants
  ├── pipeline: updated — metrics before and after, check names
  ├── report and mcp: updated — metrics line and object
  ├── tests and fixtures: updated — layering rows, fixtures, perf, baseline
  └── docs: updated — REFERENCE, CHANGELOG, rule catalogue
```

---

## 7. Implementation Strategy

### Phase 1: Baseline, segmenter and metrics

1. Capture the rules-only residual baseline on the eval corpus (FR-10.1, FR-10.2) before any rule change.
2. Add the shared segmenter, with layering rows, and point the window code at it (FR-1.1 to FR-1.5).
3. Add the syllable counter and the metric computation from raw counts, with hand-computed unit tests (FR-2.1 to FR-2.10).
4. Surface metrics in the text report, JSON and MCP, with before and after for `fix`, and update the pinned JSON test (FR-3.1 to FR-3.6).
5. Add the four metric rules, default off, with params, `explain` and fixtures (FR-4.1 to FR-4.7); bump `RULE_SET_VERSION` and the CHANGELOG (FR-11.1).
6. Assemble the licence-clean human-prose calibration corpus and record the metric values on it; record the values beside the provisional thresholds; changing a shipped default threshold is a separate, explicit change (D-4).

### Phase 2: Wordiness tables

7. Write the extended phrase and nominalization rows with explicit inflections, fixtures and `fixed.md` (FR-5.1 to FR-5.3, FR-5.5).
8. Add detection-only intensifier and weak-adverb entries (FR-5.4).
9. Record the FR-10 value for the phase.

### Phase 3: Verifier invariants

10. Add the negation, proper-noun and modal checks as new `Check` variants with check names (FR-6.1 to FR-6.4).
11. Define the SLP003/SLP004 exemption path for rule-origin and model-origin edits (D-8).
12. Update the verifier header, REFERENCE sentence and `tests/score.rs:102-112` (FR-6.7).
13. Add rejection and acceptance tests, and run the regression over every Safe fixture (FR-6.5, FR-6.6, FR-6.8).

### Phase 4: Tagger and SLP006

14. Build the lexicon and contextual rules with embedded data, loader tests and tag tests (FR-7.1, FR-7.2, NFR-4).
15. Add the per-document analysis slot, and lazy tagging (FR-7.3, FR-7.4).
16. Widen SLP006 behind the tag gate, with positive, negative and grammatical-context fixtures (FR-7.5 to FR-7.8).
17. Extend the performance test and add the single-tagging count test (NFR-2).
18. Record the FR-10 value for the phase.

### Phase 5: Templates

19. Add the SLP004 template and fixtures (FR-8.1, FR-8.3, FR-8.7).
20. Add the SLP003 template and fixtures (FR-8.2, FR-8.3, FR-8.7).
21. Update both rules' class and `explain`, and check that SLP007 and SLP015 are untouched (FR-8.4 to FR-8.6).
22. Record the FR-10 value for the phase.

### Phase 6: Redundancy

23. Decide the stemmer: run `cargo deny` on `rust-stemmers`; if it passes, pin it exactly, otherwise write the in-tree suffix stripper (D-9).
24. Add the stop-word list, the similarity function and the redundancy rule with fixtures (FR-9.1 to FR-9.6).
25. Extend the performance test to enable the rule (NFR-2), and update the rule catalogue and REFERENCE (FR-11.3).
26. Record the final FR-10 value and compare it with the baseline (FR-10.3).

---

## 8. Risks

### Risk 1: Metric thresholds are uncalibrated

**Risk:** No human reference text exists, eval paragraphs are 35-55 words, and "burstiness is an AI tic" has no source read (current-state §8; competitive §1.2). A threshold set now could flag good human prose or miss slop.
**Mitigation:** Raw metrics are always reported; metric rules are default-off with provisional thresholds and `explain` says so (D-4, FR-4.5); Task 6 builds the calibration corpus.

### Risk 2: Syllable counting is heuristic

**Risk:** The grade formula needs a syllable count and the formula does not say how to count (competitive §1.2). Errors move the grade, which is unbounded on degenerate input.
**Mitigation:** The counter is deterministic and documented as approximate (FR-2.6), the minimum-size guard (FR-2.7) removes degenerate inputs, and the unit tests use a fixed word list (NFR-10).

### Risk 3: Type/token ratio depends on text length

**Risk:** Distinct types over tokens falls as a text grows, because it is a ratio of a slowly growing count to a growing one. Comparing a 100-word and a 5,000-word document by raw ratio would mislead.
**Mitigation:** The minimum word count (D-5) is a floor only. The SPEC must define the exact measure and document its length dependence; if a length-robust variant is chosen (for example, a fixed-size window), it must still derive from raw counts (D-6). Calibration (Task 6) covers several lengths.

### Risk 4: Tag errors become wrong Safe fixes the verifier cannot catch

**Risk:** A mis-tag attaches a fix that is ungrammatical or wrong, and the verifier does not check grammar (`src/verify/mod.rs:5`). Safe fixes also skip the improvement check (`src/pipeline/mod.rs:113-125`).
**Mitigation:** Gate-only use with single-tag words (D-7, FR-7.5, FR-7.6), negative fixtures (FR-7.8), the idempotency test (FR-5.5), and the grammatical-context test (`tests/rules_fixtures.rs:304`).

### Risk 5: The new negation check rejects the rewrites SLP003 and SLP004 exist to make

**Risk:** An unconditional check would reject `It's not a bug, it's a feature.` → `It is a feature.` (`slp003…rs:31`).
**Mitigation:** The D-8 exemption, and FR-6.6, which requires that no Safe fixture is newly rejected.

### Risk 6: Proper-noun check false rejects on capitalised words

**Risk:** A capitalised-word proxy can reject legitimate edits where capitalisation moves, such as SLP008's lead-in deletion (`src/rules/helpers.rs:165`), or flag titles and acronyms.
**Mitigation:** Sentence-initial words are excluded (FR-6.2), and FR-6.6 runs the full Safe fixture set. A rejected safe fix is reported loudly as SLP901 (FR-6.5), never silently dropped.

### Risk 7: Added analysis multiplies in the model path

**Risk:** Every accepted model window reruns all rules on the whole candidate (`src/pipeline/mod.rs:264-291`), so a tagger or similarity pass could run once per window.
**Mitigation:** Lazy tagging and one tagging pass per document pass (FR-7.3, FR-7.4), and the performance test with all new rules on (NFR-2).

### Risk 8: Score drift and JSON breakage

**Risk:** New rules change recorded scores, and a `metrics` object changes the pinned JSON (`tests/score.rs:96-98`) and possibly `SCHEMA_VERSION` (`src/report/json.rs:14`).
**Mitigation:** New rules default-off (D-4), `Score` unchanged (D-2), the metrics object as a sibling (FR-3.2), `RULE_SET_VERSION` bump and CHANGELOG (FR-11.1). The schema version does not bump (D-11).

### Risk 9: Dependency maintenance

**Risk:** `rust-stemmers` has had no release since 2019-11-17, and the Rust tagger crates are young (competitive §1.2).
**Mitigation:** No dependency before Phase 6 (D-9), an exact pin plus `cargo deny` there, an in-tree fallback, and in-tree tagging (D-7).

### Risk 10: Redundancy false positives

**Risk:** Two sentences can share content words without being redundant (a claim and its counterexample), and TF-IDF weights computed over one document are only meaningful in longer ones (feasibility Pillar 6).
**Mitigation:** Detection only with no fix (FR-9.3), a minimum content-word and sentence count (FR-9.4), a tunable threshold, and default-off.

---

## 9. Follow-up Questions

None open. Brad answered the seven questions of the first pass with "use your default recommendations", and they are recorded as resolved decisions (D-11, mapping to D-2, D-3, D-4 and D-10). The questions that were asked are kept below for traceability.

- [ ] **Q1 — How do metrics feed the score?** (a) report only; (b) metric rules raise ordinary findings above a threshold and flow through the existing formula, with raw metrics always reported; (c) a new term in the score formula, which invalidates every recorded score and changes the pinned JSON. **Recommendation: (b)** (D-2, FR-4, FR-3). Affects the meaning of the public score.
- [ ] **Q2 — Ship metric rules with uncalibrated thresholds?** (i) metrics reported only until calibrated; (ii) rules default-off with provisional documented thresholds; (iii) build the calibration corpus before shipping any rule. **Recommendation: (ii) now, with the corpus built in Phase 1 (Task 6), so Brad's priority ships without waiting.** No source establishes that low sentence-length variation marks AI text (competitive §1.2), and the repo has no human reference text.
- [ ] **Q3 — Which new rules and rows are on by default?** The parent PRD says a later-release rule starts off (FR-4.5, `docs/PRD.md:215`), so default users get no new fixes. **Recommendation: all new rules start off, except the added wordiness rows, which take the default of the rule they extend, and the SLP004/SLP003/SLP006 fix widening, which takes the default of those rules.** Defaults change recorded scores, so this is a product trade-off.
- [ ] **Q4 — What is a "safe" fix?** Grammatical only, grammatical and claim-preserving, or also emphasis-preserving. The verifier cannot check strength (`src/verify/mod.rs:5`). (i) safe = grammatical and truth-conditionally neutral, so intensifier and adverb deletion is detection only, and the SLP004/SLP003 templates are in; (ii) allow intensifier deletion with a distinct low-confidence class. **Recommendation: (i)** (D-10, FR-5.4, FR-8). Choosing (ii) adds a fix class, which is a public surface change. Brad's request listed "guarded intensifier deletion" and "adverb deletions" as candidates, so this drops them from Safe unless he chooses (ii).
- [ ] **Q5 — What do we call the "burstiness" metric?** The request says "burstiness (sentence-length variance, a known AI tic)". The only source read defines burstiness as variation in model perplexity, not sentence length (competitive §1.2). **Recommendation: name the output and rule "sentence-length variation", document "burstiness" as a colloquial alias, and make no AI-provenance claim** (D-3). Rule codes and config param names are assigned in the SPEC and follow the existing naming style.
- [ ] **Q6 — Drop extractive summarization (TextRank)?** It would insert new text, which `verify` cannot accept, and no source on its algorithm was read. **Recommendation: out of scope for this initiative; if wanted later, a report-only "key sentences" listing in its own PRD.**
- [ ] **Q7 — Does the additive `metrics` object in the JSON report need a `schemaVersion` bump?** `SCHEMA_VERSION` is bumped on "any breaking change" (`src/report/json.rs:13-14`), and an added sibling object is not breaking for a reader that ignores unknown keys. **Recommendation: additive, no bump, documented in REFERENCE.** Option: bump anyway to signal the new field to consumers. This is a wire-protocol decision.

---

## Revision Log

none — first pass
tence-length variance, a known AI tic)". The only source read defines burstiness as variation in model perplexity, not sentence length (competitive §1.2). **Recommendation: name the output and rule "sentence-length variation", document "burstiness" as a colloquial alias, and make no AI-provenance claim** (D-3). Rule codes and config param names are assigned in the SPEC and follow the existing naming style.
- [ ] **Q6 — Drop extractive summarization (TextRank)?** It would insert new text, which `verify` cannot accept, and no source on its algorithm was read. **Recommendation: out of scope for this initiative; if wanted later, a report-only "key sentences" listing in its own PRD.**
- [ ] **Q7 — Does the additive `metrics` object in the JSON report need a `schemaVersion` bump?** `SCHEMA_VERSION` is bumped on "any breaking change" (`src/report/json.rs:13-14`), and an added sibling object is not breaking for a reader that ignores unknown keys. **Recommendation: additive, no bump, documented in REFERENCE.** Option: bump anyway to signal the new field to consumers. This is a wire-protocol decision.

---

## Revision Log

none — first pass
