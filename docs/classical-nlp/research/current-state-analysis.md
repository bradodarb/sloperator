# Current State Analysis: classical (non-neural) NLP for sloperator

**Date:** 2026-11-01
**Status:** Complete
**Related:** [Competitive Analysis](./competitive-analysis.md) | [Feasibility Summary](./feasibility-summary.md)

---

## Purpose

This file inventories what sloperator has today that the seven scope candidates would build on or collide with: the
word-list/regex rule helpers, the safe-versus-model fix split, the verifier, the pipeline, the score, the reports, the
document model and the tests. The PRD reader needs it to see which candidates are small additions (a table in an
existing rule), which need shared infrastructure that does not exist (sentence/token/tag analysis), and which collide
with a pinned contract (the score JSON, the verifier's documented limits). Peer systems and external facts are in the
[Competitive Analysis](./competitive-analysis.md). Priorities, risks and open questions are in the
[Feasibility Summary](./feasibility-summary.md).

**Headline:** a search of `src/` and `tests/` for Flesch, readability, burstiness, TF-IDF, syllable, lemma, stemmer and
POS-tag code finds nothing except one comment (`src/rules/builtin/slp001_ai_vocabulary_replaceable.rs:3`). All seven
candidates are greenfield. The attachment points below are real, and so are the contracts that constrain them.

---

## 1. Rule engine

### 1.1 The `Rule` trait, finding construction and detection

**File(s):** `src/rules/mod.rs`, `src/rules/registry.rs`, `src/rules/builtin/mod.rs`

| Item | Purpose | Status |
|------|---------|--------|
| `Rule` trait (`src/rules/mod.rs:140`) | `meta()` plus `check(&RuleCtx, &mut Vec<Finding>)` | Implemented |
| `RuleCtx` (`src/rules/mod.rs:84-97`) | holds only `doc`, resolved `params`, `severity`; offers `blocks()` and `prose_words()` | Implemented. No slot for shared per-document analysis. |
| `RuleCtx::finding` (`src/rules/mod.rs:107-130`) | builds a `Finding`; the fix is kept only when `meta.fix == FixClass::Safe` (`:115`) | Implemented |
| `FixClass { Safe, Model, None }` (`src/rules/mod.rs:40`) | what a rule's fix may be | Implemented |
| `RuleMeta` (`src/rules/mod.rs:68-81`) | code, name, family, severity, `default_on`, fix class, explain, example, params, `since` | Implemented |
| `Family` (`src/rules/mod.rs:48`) | nine families (AiVocabulary … EmDash) | Implemented. A metrics or redundancy rule has no family yet. |
| `ParamSpec` (`src/rules/mod.rs:62`) | tunable threshold; config validates names against it | Implemented |
| `detect` (`src/rules/mod.rs:158-177`) | runs each enabled rule in turn, drops findings overlapping protected spans or suppressed lines (`:172`), sorts by (start, code) | Implemented |
| `RULE_SET_VERSION = "0.1.0"` (`src/rules/mod.rs:146`) | crate version at which the default list last changed | Implemented. Adding rules is a rule-set change. |
| `builtin::all()` (`src/rules/builtin/mod.rs:23-42`) | the 17 built-in rules SLP001-SLP017 | Implemented |

**Key finding:** each rule re-reads the document by calling `ctx.blocks()` (`src/rules/mod.rs:84-97`), and `detect`
runs the rules one after another (`src/rules/mod.rs:165-175`). Nothing computes sentence splits or tags once and shares
them. A tagger or sentence model used by several rules would be recomputed per rule unless `RuleCtx` or `Document`
gains a cache. That matters for cost (section 6).

### 1.2 Shared detectors

**File(s):** `src/rules/helpers.rs`

| Item | Purpose | Status |
|------|---------|--------|
| `WordList` (`src/rules/helpers.rs:14-17`), `new` (`:24`) | one aho-corasick automaton over a `&'static` table of (phrase, optional replacement); ASCII-case-insensitive (`:26`), leftmost-longest (`:27`) | Implemented |
| `WordList::find` (`src/rules/helpers.rs:38`) | whole-word matches as source ranges with a case-preserving replacement; `is_whole_word` (`:71`), `match_case` (`:79`) | Implemented |
| `PatternRule` (`src/rules/helpers.rs:95`), `find` (`:118`), `find_text` (`:127`) | `RegexSet` plus one `Regex` per pattern | Implemented |
| `starts_sentence` (`src/rules/helpers.rs:141`) | sentence-start test from the preceding character only | Implemented. Heuristic: `.`, `!`, `?` or start. |
| `delete_lead_in` (`src/rules/helpers.rs:165`) | delete a lead phrase and capitalise the next letter | Implemented |
| `markdown_events`, `first_prose` (`src/rules/helpers.rs:181`, `:199`) | structure events; a prose range to report a document-level finding on | Implemented. `first_prose` is how SLP010/SLP011 report. |

**Key finding (candidate 1):** `WordList` matches exact phrases only. It has no stemming or lemma step, and its
replacement is a fixed string chosen per table row. SLP001 already works around this by listing every inflection as its
own row with its own replacement, "never as a lemma" (`src/rules/builtin/slp001_ai_vocabulary_replaceable.rs:1-4`,
rows at `:8-27`). So "stem-aware" detection is not the hard part. The replacement has to be inflected to match
(`make a decision` → `decide`, `made a decision` → `decided`), and a stemmer finds the match but cannot produce the
replacement form.

### 1.3 Rule catalogue: where the fix classes sit

| Rule | Fix class | Evidence | Relevance |
|------|-----------|----------|-----------|
| SLP001 ai-vocabulary-replaceable | Safe | `slp001…rs:42` | The wordiness table already lives here: `in order to` → `to` (`:18`), bare `leverage` has no fix (`:26`). Candidate 1 extends it or adds a sibling. |
| SLP002 ai-vocabulary | Model | `slp002…rs:29` | detection only |
| SLP003 not-x-but-y | Model | `slp003…rs:26`; regex at `:9-10` | Candidate 3. Pattern lets X run up to 80 characters. |
| SLP004 not-just-but-also | Model | `slp004…rs:27`; patterns at `:9-12` | Candidate 3. Pattern ends at `but (also)`, so Y is not captured. |
| SLP005 rule-of-three | Model | `slp005…rs:32`; param default 4.0 at `:41-45` | Density rule: fires only above a per-1,000-words threshold (`:52-76`). The existing precedent for a document-level metric becoming findings. |
| SLP006 copula-avoidance | Safe | `slp006…rs:57` | Candidate 2. Hand-built mini-tagger: `SINGULAR_SUBJECT` (`:22`), `AS_COMPLEMENT` (`:25-27`), `MODALS` (`:30-32`), `ADVERBS` (`:35`), guards in `safe_replacement` (`:112-127`). |
| SLP007 significance-puffery | Model | `slp007…rs:27`; patterns `:7-12` | Candidate 3 |
| SLP008, SLP009, SLP012, SLP013, SLP014, SLP016, SLP017 | Safe | `slp008…rs:27`, `slp009…rs:26`, `slp012…rs:31`, `slp013…rs:24`, `slp014…rs:29`, `slp016…rs:27`, `slp017…rs:145` | Existing safe fixes. SLP009 is a three-row stacked-hedge table (`slp009…rs:7-11`). |
| SLP010, SLP011 | None | `slp010…rs:33`, `slp011…rs:31` | structure rules |
| SLP015 ai-disclaimer | Model | `slp015…rs:25`; patterns `:7-10` | Candidate 3. Its own explain says whether the sentence survives depends on its claim (`slp015…rs:26-29`). |

**Key finding (candidate 3):** the SLP003, SLP004, SLP007 and SLP015 rules state in their own `explain` text that "no
rewrite is safe" or that the outcome depends on the claim (`slp003…rs:27-30`, `slp004…rs:28-30`, `slp007…rs:28-30`,
`slp015…rs:26-29`). A template rewrite for them changes what the rule documents as its reason for being Model-class.

---

## 2. Fix routing: how a finding reaches the model, and how classical fixes displace it

**File(s):** `src/engine/window.rs`, `src/pipeline/mod.rs`, `src/edit/mod.rs`, `src/types.rs`

- `windows()` selects a window only for a finding that is Model-class, or Safe-class **with no fix attached**
  (`src/engine/window.rs:110-118`). A `FixClass::None` finding never selects a window (`:117`).
- Consequence: a rule declared `Safe` that attaches a fix only when its guards pass, and attaches none otherwise,
  already falls through to the model on the hard cases. SLP006 and SLP001 work this way today (SLP001's bare `leverage`
  row, `slp001…rs:26`). This is the existing mechanism for "lean less on the model". No new routing is needed for
  candidates 1, 2 or 3 if the rule is reclassified Safe and attaches fixes only where it is certain.
- `edit::plan` resolves overlapping proposals deterministically: earlier start, then shorter range, then lower origin,
  rules by code before the model (`src/edit/mod.rs:14-16`, `:25`). Two new safe rules that touch the same words do
  not conflict at write time.
- `Proposal` carries `origin: ProposalOrigin` (`src/types.rs:51`, `:58-61`), either `Rule(code)` or `Model`. Verification can therefore apply a different policy per origin.

---

## 3. Verification

**File(s):** `src/verify/mod.rs`, `src/verify/tokens.rs`, `src/engine/mask.rs`

| Item | Purpose | Status |
|------|---------|--------|
| `verify::edit` (`src/verify/mod.rs:65`) | rejects edits touching protected or frozen ranges; compares number multiset (`:91`), URL multiset (`:97`), configured-term multiset (`:100`); re-parses the candidate and compares block skeleton (`:113-`) and protected bytes (`:140`) | Implemented |
| `Check` enum (`src/verify/mod.rs:38-51`) | Numbers, Urls, Terms, Placeholders, Protected, BlockType, NewBlockMarker, OutputContract, NoImprovement, Document | Implemented. Each variant has a name in `pipeline::check_name` (`src/pipeline/mod.rs:331`). |
| `tokens()` (`src/verify/tokens.rs:42`) | numbers, URLs, terms as multisets | Implemented |
| `verify::improvement` (`src/verify/mod.rs:148`) | strictly fewer weighted findings in the window, for model rewrites only | Implemented |
| `verify::document` (`src/verify/mod.rs:162`) | whole-file skeleton and protected-bytes check | Implemented |

**Key finding (candidate 5):** the module header says verification "covers numbers, URLs, terms and block structure,
not names or claim polarity" (`src/verify/mod.rs:5`). `tests/score.rs:102-112` asserts that `docs/REFERENCE.md` states
that limit ("not names or claim polarity"). Adding negation, proper-noun or modal checks changes a documented and
tested limit, so the doc sentence and that test change in the same PR (PRD NFR-9, `docs/PRD.md:323`).

**Key finding (candidate 5, tension):** the SLP003 example is `It's not a bug, it's a feature.` → `It is a feature.`
(`src/rules/builtin/slp003_not_x_but_y.rs:31`). A negation-preservation check would reject the rewrite the rule exists
to make. The same holds for SLP004 (`slp004…rs:31-34`). The check needs an exemption (per rule code through
`ProposalOrigin::Rule`, or a model-origin-only policy) or a smarter polarity rule.

**Key finding:** safe fixes get `verify::edit` but **not** `verify::improvement`. Step 1 only calls `verify::edit`
(`src/pipeline/mod.rs:113-125`). Only model windows are checked for strictly fewer findings (`src/pipeline/mod.rs:264-291`).
A new safe fix that creates a new finding is caught only by the fixture idempotency test
(`tests/rules_fixtures.rs:526`), not at runtime.

---

## 4. Score and reports

**File(s):** `src/score/mod.rs`, `src/report/mod.rs`, `src/report/json.rs`, `src/report/text.rs`, `src/mcp/mod.rs`

- Score = `round2(1000 × Σ weight(severity) / max(prose_words, 1))` (`src/score/mod.rs:26-36`, `:50-56`). Weights are
  Suggestion 1, Warning 3, Error 9 (`:17-23`). SLP9xx system findings are excluded (`:26-30`; SPEC DQ-4, `docs/SPEC.md:2407`).
- `Score { value, prose_words, weighted }` (`src/score/mod.rs:10`) serialises to a pinned shape:
  `{"value":32.52,"proseWords":123,"weighted":4}` (`tests/score.rs:96-98`).
- `score::overall` is word-weighted over raw sums, "not a mean of the per-file scores" (`src/score/mod.rs:39-47`). Any new
  per-file metric needs raw counts (words, sentences, syllables, distinct types) to aggregate the same way.
- Rounding to two decimals keeps the JSON byte-stable (`docs/SPEC.md:1136`).
- Reports: text prints one score line per file (`src/report/text.rs:32`, `score_text` at `:60`); JSON has `File { score,
  score_before, findings }` (`src/report/json.rs:27-33`) under `SCHEMA_VERSION = 1` (`:14`, comment: bump on any breaking
  change). `Summary` carries one overall `Score` (`src/report/mod.rs:35-60`). MCP returns `score` for lint
  (`src/mcp/mod.rs:113`) and `scoreBefore`/`scoreAfter` for fix (`:218-219`). `concise` and `github` formats print no score.
- Fix computes `score_before` on the original text (`src/pipeline/mod.rs:108`) and `score` on the final text
  (`:185`), so before/after metrics need no new plumbing in the pipeline.
- PRD constraints: the score is deterministic and the formula is documented (FR-5.1, `docs/PRD.md:223`); REFERENCE must
  state the score measures style tells, not quality or provenance, and the tool must not claim undetectability
  (FR-5.4, `docs/PRD.md:226`). AI-text detection is out of scope (`docs/PRD.md:58`).

**Key finding (candidate 6):** there are two ways a metric can "feed the score", and the code supports only one cheaply.
(a) A metric becomes a finding from a rule with a severity. It flows through `--min-severity`, `sloperator.toml`
params, directives, `explain`, JSON and the existing formula with no formula change. (b) A metric becomes a new term
in `score()`. That changes the formula (FR-5.1), the pinned JSON (`tests/score.rs:96-98`) and the meaning of every
score people have already recorded. Route (a) has a precedent: SLP005 and SLP014 turn a per-1,000-words density into
findings above a threshold (`slp005…rs:52-76`; `src/rules/builtin/slp014_em_dash_density.rs:2`).

---

## 5. Document model and text segmentation

**File(s):** `src/doc/mod.rs`, `src/engine/window.rs`, `Cargo.toml`

- `Document` holds `src`, spans (prose or protected), blocks and newline style (`src/doc/mod.rs:161-172`).
  `block_text` returns a block's prose with each non-prose run collapsed to one `OBJ` placeholder (`:195`).
  `BlockKind` is Paragraph, Heading, ListItem, TableCell (`:38-52`).
- `prose_words` is the count of `unicode_words()` over prose spans (`src/doc/mod.rs:235-241`). This is the existing
  "word" definition and the score's denominator. Any readability or diversity metric should use the same definition or
  report both.
- Sentence segmentation exists, but only privately: `sentence_range` in `src/engine/window.rs:241` uses UAX #29
  `split_sentence_bound_indices` (`:259`) on a copy of the block with line breaks blanked, and ignores boundaries inside
  protected runs. No public sentence iterator exists. `starts_sentence` (`src/rules/helpers.rs:141`) is a one-position
  heuristic.
- `unicode-segmentation` is already a dependency (`Cargo.toml:49`). No syllable counter, stemmer, stop-word list or
  lexicon is a dependency (`Cargo.toml:30-50`).

**Key finding (candidate 6):** prose is not only paragraphs. Headings, list items and table cells are blocks
(`src/doc/mod.rs:38-52`) and `prose_words` counts them all. Sentence-length and Flesch-Kincaid statistics over
headings and bullet fragments are skewed by construction, so the PRD must say which block kinds count.

---

## 6. End-to-End Wiring (what actually runs at runtime)

```
analyse(source)                         src/pipeline/mod.rs:70-78
  -> detect_in(text, source)            :295-310   parse -> extract directives -> rules::detect
       -> rules::detect(doc, sup, enabled)   src/rules/mod.rs:158   each rule scans ctx.blocks()
 lint(analysis)                          :80       report + score; never touches a rewriter (FR-2.1)
 fix(analysis, engines, rw, seed)        :99
   step 1: findings.fix -> edit::plan -> verify::edit -> apply -> detect_in again   :113-140
   step 2: windows(selects: Model, or Safe without fix) -> model -> mask::propose
           -> verify::edit -> accept(): detect_in(candidate) -> verify::improvement  :142-171, :264-291
   step 3: verify::document                                                         :173
   step 4: score(final findings), score_before from the original                    :183-185
```

- **Wired:** rules, safe fixes, model windows, verification, score, all four reports and MCP.
- **Cost multiplier:** `accept()` calls `detect_in` on the **whole candidate document** for every model window
  (`src/pipeline/mod.rs:264-291`, `detect_in` at `:295`). Anything added to `rules::detect` (a tagger, a similarity
  pass) runs on every window attempt in the model path. The performance contract is a rules-only check of 100 KB in
  under 1 second (NFR-3, `docs/PRD.md:299`), tested by `tests/perf.rs:87`. The model path has its own latency budget
  (`eval/README.md`, Latency budget section).
- **Not wired / absent:** no metrics computation, no tagging, no stemming, no similarity, no polarity or proper-noun
  verification.

---

## 7. Test Coverage

| Area | Test file(s) | Coverage status |
|------|--------------|-----------------|
| Rule fixtures: positive/negative per rule, fixed.md only for Safe | `tests/rules_fixtures.rs:100` (`check_fix`), `:201` (`every_rule_has_fixtures`), `:494` (`only_safe_rules_attach_fixes`), `:526` (idempotency) | Good. A new rule needs a `fixtures/rules/SLPnnn/` directory or `every_rule_has_fixtures` fails. |
| Safe-fix grammaticality | `tests/rules_fixtures.rs:304` (`safe_fix_contexts_grammatical`); `fixtures/rules/SLP006/positive.md` | Partial: hand-written contexts only |
| Verifier | `tests/verify_edits.rs:41` (lost number), `:99` (URL), `:119` (term), `:144` (heading marker), `:197` (protected), `:254` (improvement), `:268` (document) | Good for numbers/URLs/terms/structure. **No test for negation, names or modality**, because the code does not check them. |
| Score | `tests/score.rs:27` (formula), `:62` (overall), `:82` (determinism), `:96-98` (pinned JSON), `:102` (reference limits) | Good. Pins the shape candidate 6 would touch. |
| Pipeline | `tests/pipeline.rs:45` (lint never builds a rewriter), `:76` (rules-only never calls the model), `:99` (second fix is a no-op), `:117` (rejected safe fix → SLP901), `:158` (score/score_before) | Good |
| Reports | `tests/report.rs:123` (JSON shape) | Good for the current shape |
| Layering | `tests/layering.rs:9` (forbidden-use table), `:96-103` (pure dirs) | Good. A new `src/` module needs a row. `src/rules/` may not use `verify` (`:11-17`); `src/score/` may not use `engine`, `write`, `report`, `input` (`:22-29`). |
| Performance | `tests/perf.rs:87` (`rules_only_100kb_under_1s`) | One check, on the existing rules |
| Eval | `eval/paragraphs/p01.md`…`p20.md`; `tests/eval.rs` (ignored; needs a model) | See gap on corpus size |

---

## 8. Gaps Identified

- **Gap:** no text-metrics module. Evidence: grep of `src/` and `tests/` finds no readability, burstiness or diversity code. Impact: candidate 6 is a new module, report fields and (optionally) rules.
- **Gap:** no public sentence/word iterator. Evidence: sentence splitting is private to `src/engine/window.rs:241-259`; `prose_words` is the only word helper (`src/doc/mod.rs:235`). Impact: metrics, redundancy and any tagger need one shared segmenter, which must also respect `OBJ` placeholders and protected spans.
- **Gap:** no shared per-document analysis cache. Evidence: `RuleCtx` has three fields (`src/rules/mod.rs:84-88`); `detect_in` reruns all rules per window (`src/pipeline/mod.rs:264-291`). Impact: a tagger would be recomputed per rule and per window unless cached.
- **Gap:** no human-written reference text and no calibration corpus. Evidence: the eval corpus is 20 model-style paragraphs of about 35-55 words, and the README says nothing in it is a human-written reference (`eval/README.md:9-12`, `:25-28`). Impact: thresholds for burstiness, grade level and diversity cannot be calibrated against human prose from anything in the repo, and a 35-55-word paragraph holds only a few sentences, too few for a stable variance.
- **Gap:** the verifier cannot see polarity, names or modality. Evidence: `src/verify/mod.rs:5`; `Check` has no such variant (`:38-51`). Impact: this is the stated reason classical safe fixes that delete words (intensifiers) or restructure clauses are risky.
- **Gap:** safe fixes skip the improvement check. Evidence: `src/pipeline/mod.rs:113-125` versus `:264-291`. Impact: a new safe rule is guarded only by fixtures and the idempotency test.
- **Gap:** SLP006's "tagger" is three word lists. Evidence: `slp006…rs:22-35`. Impact: any widening of SLP006 either grows these lists or needs real tags.
- **Gap:** no config surface for metric thresholds beyond `ParamSpec`. Evidence: `ParamSpec` is per rule (`src/rules/mod.rs:62`); params exist for SLP005, SLP010, SLP011, SLP014. Impact: metric thresholds fit as rule params without new config schema, if metrics are rules.

---

## 9. References

- [Competitive Analysis](./competitive-analysis.md)
- [Feasibility Summary](./feasibility-summary.md)
- `docs/adr/ADR-05-verify-every-edit.md` — one verifier for every edit; `Verified<T>` type
- `docs/adr/ADR-06-rules-as-code.md` — rules are Rust types in a static registry with fixture directories
- `docs/adr/ADR-01-single-crate-layered-modules.md` — layering enforced by `tests/layering.rs`
- `docs/PRD.md`, `docs/SPEC.md` — requirements cited by FR/NFR number above
- `docs/research/competitive-analysis.md` — the parent initiative's peer study (Ruff, Vale CLI conventions)
