# Initial Feasibility Summary: classical (non-neural) NLP for sloperator

**Date:** 2026-11-01
**Status:** Feasible with caveats — the metrics, wordiness, verifier and tagger-gated pieces are feasible with owned code; thresholds for burstiness and readability cannot be calibrated from anything in the repo, and several "safe" rewrites need a policy decision from Brad.
**Scope:** `src/rules/` (new rules, helpers), a new pure NLP/metrics module, `src/score/`, `src/report/`, `src/verify/`, `src/mcp/`
**Next Step:** PRD
**Related:** [Current State Analysis](./current-state-analysis.md) | [Competitive Analysis](./competitive-analysis.md)

---

## Vision

With no model, sloperator reports how a text reads (grade level, sentence length, sentence-length variation, vocabulary
diversity), catches more padded phrasing, and applies more verified fixes. With a model, fewer windows reach it
because the deterministic engine already took the cases that have one right answer, and the verifier guards the
model's rewrites against more kinds of damage. The work is additive: new deterministic capability layered on the
existing rule, verify and report pipeline, with no change to the local-only, verify-every-edit contract.

---

## Why This Matters

1. **No text metrics exist, and Brad's priority is metrics.** Nothing in `src/` computes readability, burstiness or
   diversity (Current State headline). The score counts only weighted findings per 1,000 words
   (`src/score/mod.rs:26-36`), so a text with flat sentence rhythm and clean phrasing scores 0. Impact: the report
   cannot say how a document reads, and the score cannot reward or penalise rhythm.
2. **Most findings are still model-bound.** Of 17 rules, four tells are Model-class (SLP003, SLP004, SLP007, SLP015)
   plus SLP002 and SLP005 (`slp002…rs:29`, `slp005…rs:32`); SLP006 fixes only after seven listed subjects
   (`slp006…rs:22`). Windows are selected from exactly these findings (`src/engine/window.rs:110-118`). Impact: without
   a model, `--fix` leaves them all.
3. **The verifier cannot see meaning.** It checks numbers, URLs, terms and block structure, "not names or claim
   polarity" (`src/verify/mod.rs:5`). Impact: this limit holds model rewrites to a lower standard than a human editor
   and is the main brake on widening safe fixes.

---

## What Already Works

| Component | Status | Notes |
|-----------|--------|-------|
| `WordList` / `PatternRule` detectors | Production-ready | `src/rules/helpers.rs:14`, `:95`. Phrase tables with case-preserving replacement. |
| Safe-with-fallthrough routing | Production-ready | A Safe rule that attaches no fix selects a model window (`src/engine/window.rs:110-118`). This is the "lean less on the model" mechanism and needs no new routing. |
| Density rules with `ParamSpec` thresholds | Production-ready | SLP005, SLP014: metric → threshold → findings (`slp005…rs:41-76`). The template for metric rules. |
| `verify::edit` on every edit | Production-ready | `src/verify/mod.rs:65`; `origin` on `Proposal` allows per-origin policy (`src/types.rs:58-61`). |
| Before/after score plumbing | Production-ready | `src/pipeline/mod.rs:108`, `:185`; MCP `scoreBefore`/`scoreAfter` (`src/mcp/mod.rs:218-219`). |
| `unicode-segmentation` | Dependency present | `Cargo.toml:49`; used for words (`src/doc/mod.rs:235`) and private sentence splitting (`src/engine/window.rs:259`). |
| Fixture and idempotency test harness | Production-ready | `tests/rules_fixtures.rs:100`, `:201`, `:526` |

---

## What Needs Work

Pillars are in the order this research recommends. The PRD sets the final priority.

### Pillar 1: Text metrics (candidate 6) (M, high — Brad's priority)

**Problem:** no readability, average-sentence-length, burstiness or lexical-diversity measure exists, and no public
sentence/word iterator to build one on (Current State §5, §8).

**Solution (directional):** a new pure module (for example `src/metrics/`, with a row in `tests/layering.rs:9` and
`:96`) that segments prose blocks once and returns raw counts (words, sentences, syllables, distinct word types,
per-sentence lengths) so per-file and overall values aggregate from sums, as `score::overall` does
(`src/score/mod.rs:39-47`). Derive: Flesch-Kincaid grade, average sentence length, sentence-length standard deviation or
coefficient of variation, and type/token ratio. Surface them in the text, JSON and MCP outputs and in before/after for
fix. Feed the score through rules that raise findings above thresholds (route (a), see OQ-1), at `Suggestion`/
`FixClass::None` so they never select a model window (`src/engine/window.rs:117`). Round to fixed decimals for
byte-stable output (`docs/SPEC.md:1136`). Guard short inputs (minimum sentence and word counts) because the grade is
unbounded and degenerate input gives extreme values[^fk-short].

**Effort:** ~5-7 days, including the syllable counter (heuristic or small dictionary) and its fixtures.

### Pillar 2: Wordiness and plain-language tables (candidate 1) (S, medium)

**Problem:** SLP001 has 17 rows (`slp001…rs:8-27`); nominalizations (`make a decision`) and guarded intensifier deletion
are absent. Replacement must be inflected, not just matched.

**Solution:** extend `WordList` tables with explicit per-inflection rows (the existing SLP001 convention,
`slp001…rs:1-4`). A stemmer is optional here: it helps detect candidates when authoring tables, not at runtime, so
`rust-stemmers` is not required for this pillar. Put intensifier deletion behind a context guard (needs Pillar 4's tags
or a conservative left/right-word list) and flag it for OQ-7, because deleting "very" or "really" changes strength and
the verifier does not check that (`src/verify/mod.rs:5`).

**Effort:** ~2-3 days for tables, fixtures and `fixed.md`; +1 day for guarded intensifier deletion.

### Pillar 3: Verifier invariants (candidate 5) (M, high — guards the model too)

**Problem:** polarity, proper nouns and modal strength are unchecked (`src/verify/mod.rs:5`).

**Solution:** add checks inside `verify::edit` so they cover safe fixes and model rewrites alike (model rewrites reach
it through `pipeline::accept`, `src/pipeline/mod.rs:264-291`): negation-token multiset, capitalised non-sentence-initial
tokens, and modal verb multiset (can/could/may/might/must/shall/should/will/would). New `Check` variants
(`src/verify/mod.rs:38-51`) also need entries in `check_name` (`src/pipeline/mod.rs:331`). Update the doc sentence and
the test that pins it (`tests/score.rs:102-112`). **The negation check must exempt SLP003 and SLP004**, whose purpose
is to remove a "not" (`slp003…rs:31`, `slp004…rs:31-34`), either by `ProposalOrigin::Rule(code)` or by applying the
check to model-origin edits with a rule-aware allowance (OQ-5). Capitalisation moves in SLP008's lead-in deletion
(`src/rules/helpers.rs:165`), so sentence-initial words must be excluded from the proper-noun check.

**Effort:** ~3-4 days. Tests: one rejection test per new `Check` in `tests/verify_edits.rs`, plus a regression run
over every Safe fixture to prove no existing fix is newly rejected.

### Pillar 4: Shared sentence/token/tag core and a Brill-style tagger (candidate 2) (L, medium)

**Problem:** SLP006's "tagger" is hand lists (`slp006…rs:22-35`) and a general tagger and its lexicon do not exist.

**Solution:** a pure `src/nlp/`-style module with a shared segmenter (also used by Pillar 1) and a lexicon+contextual-rule
tagger[^brill-short]. Use it only as a **gate**: attach a Safe fix only where the lexicon gives every involved word a
single unambiguous tag, otherwise attach none and let the existing fallthrough send the case to the model. This keeps
the SLP006 header's standard ("part of speech, agreement and the phrase after as are all certain", `slp006…rs:4-5`).
Cache analysis per document (add a slot to `RuleCtx`, `src/rules/mod.rs:84-88`) so it is not recomputed per rule and
per model window (`src/pipeline/mod.rs:264-291`). The lexicon source and licence are unresolved (OQ-4). Passive-voice
rewrites need agent and tense recovery; treat them as out of this round.

**Effort:** ~6-8 days for segmenter, tagger, lexicon, cache; +3-4 days to widen SLP006 and add adverb deletions with fixtures.

### Pillar 5: Template rewrites for Model-only rules (candidate 3) (M, medium-low)

**Problem:** SLP003, SLP004, SLP007, SLP015 are Model-class by their own documented reasons (Current State §1.3).

**Solution:** only narrow, unambiguous shapes. Candidates in order of plausibility: SLP004 `not just ADJ, but also ADJ` →
`ADJ and ADJ` (the rule's own example, `slp004…rs:31-34`) when both slots are single words with matching tags;
SLP003 `it's not X, it's Y` → `it's Y` only when the pattern is within one sentence and X, Y are short; SLP007 and SLP015
have no meaning-preserving template (`explain` texts, `slp007…rs:28-30`, `slp015…rs:26-29`), so recommend no template for them.
Implementation: reclassify the rule `Safe`, attach a fix only when the template guards pass, and rely on the existing
fallthrough. Rule meta changes need fixtures with a `fixed.md` (`tests/rules_fixtures.rs:201`), and the rule text in
`explain` must change.

**Effort:** ~3-4 days for SLP003/SLP004 only.

### Pillar 6: Redundancy detection (candidate 4) (M, low-medium)

**Problem:** no sentence-level similarity exists.

**Solution:** a document-level rule (the `RuleCtx` sees the whole document, `src/rules/mod.rs:84-88`) that compares
sentences by overlap of stemmed content words (Jaccard or cosine over term counts) above a threshold. TF-IDF weights
are computed over the document's own sentences, so they are only meaningful in longer documents. Detection only
(`FixClass::None`): deleting a sentence is not verifiable. This is the one pillar that justifies a stemmer and a
stop-word list.

**Effort:** ~3 days, plus a stemmer decision (OQ-10).

---

## What We're NOT Building

- **Extractive summarization / TextRank (candidate 7):** the product edits text in place and never inserts new blocks (a new block marker fails `verify::edit`, `src/verify/mod.rs:115`); a summary would be new text, and PRD scope excludes verifying facts beyond numbers, URLs and terms (`docs/PRD.md:66`). If Brad wants it, it is a report-only "key sentences" listing, a later initiative.
- **Passive-voice rewriting:** needs agent/tense recovery beyond a lexicon tagger; detection-only is possible but not scoped here.
- **A model-based or perplexity-based burstiness:** the detector sense of burstiness needs a language model[^gptzero-short], which this initiative excludes; only the deterministic sentence-length proxy is in scope.
- **AI-text detection claims:** out of scope for the product (`docs/PRD.md:58`); metrics are style indicators (FR-5.4, `docs/PRD.md:226`).
- **Non-English text:** out of scope (`docs/PRD.md:61`); lexicons, syllable rules and stemmers here are English only.
- **User-authored rules or lexicons:** out of scope (`docs/PRD.md:60`; ADR-06).
- **A score-threshold failure mode (`--max-score`):** out of scope (`docs/PRD.md:65`).

---

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Metric thresholds are uncalibrated: no human reference text exists in the repo and eval paragraphs are 35-55 words (`eval/README.md:9-12`, `:25-28`) | High | High | Ship metrics report-only first, or ship rules default-off with documented provisional thresholds; PRD adds a calibration corpus task (licence-clean human prose, a few MB). |
| "Burstiness is a known AI tic" has no source read; GPTZero's burstiness is perplexity-based and the page contradicts itself on current use[^gptzero-short] | Med | Med | Name the metric "sentence-length variation" in output; call it a style indicator; keep FR-5.4 wording. |
| Syllable counting is heuristic; grade level inherits its error | High | Low | Document it as approximate; fixtures on known sentences; use the grade only through thresholds with margin. |
| Metric on headings/lists/tables skews results (`src/doc/mod.rs:38-52`) | High | Med | PRD decides block kinds counted; default paragraphs only. |
| Tag errors become wrong Safe fixes that the verifier cannot catch (`src/verify/mod.rs:5`) | Med | High | Use tags only as a conservative gate (single-tag words); fixtures for negative cases; idempotency test (`tests/rules_fixtures.rs:526`). |
| New negation check rejects SLP003/SLP004 model rewrites | High if unmanaged | Med | Exemption by origin/rule code; regression test over all fixtures (Pillar 3). |
| Added work in `rules::detect` multiplies in the model path (`src/pipeline/mod.rs:264-291`) | Med | Med | Per-document cache; lazy tagging gated on a cheap pattern hit; extend `tests/perf.rs:87` to cover new rules (NFR-3, `docs/PRD.md:299`). |
| Adding fields to `Score` breaks the pinned JSON (`tests/score.rs:96-98`) and `schemaVersion` (`src/report/json.rs:14`) | Med | Med | Keep `Score` unchanged; add metrics as a separate sibling object; PRD decides whether that needs a schema bump (OQ-6). |
| Score drift: new rules change recorded scores; new rules must start off (FR-4.5, `docs/PRD.md:215`) | High | Med | Bump `RULE_SET_VERSION` (`src/rules/mod.rs:146`); document in CHANGELOG; see OQ-3. |
| New dependency licence or maintenance (`rust-stemmers` last released 2019[^stemmers-short]; Rust tagger crates are young) | Med | Low | Prefer in-tree code and bundled data; if a crate is added, pin exact (`Cargo.toml:30-50` convention) and run `cargo deny` (`deny.toml`). |
| Float output differs across platforms | Low | Med | Fixed rounding (as `round2`, `src/score/mod.rs:54`), integer accumulation of counts, one fixed summation order (NFR-6, `docs/PRD.md:311`). |

---

## Effort Estimate

Rough, unmeasured estimates by a reader of the code, not a track record.

| Pillar | Scope | Effort |
|--------|-------|--------|
| 1. Text metrics | module, syllables, outputs, metric rules, fixtures | ~5-7 days |
| 2. Wordiness tables | tables, inflections, guarded intensifiers | ~3-4 days |
| 3. Verifier invariants | three checks, exemptions, docs, tests | ~3-4 days |
| 4. Segmenter + tagger | shared core, lexicon, cache, SLP006, adverbs | ~9-12 days |
| 5. Template rewrites | SLP003, SLP004 only | ~3-4 days |
| 6. Redundancy | similarity rule, stemmer decision | ~3 days |
| **Total** | | **~26-34 days** |

Pillar 4 shares the segmenter with Pillar 1, so building Pillar 1 first saves part of its cost.

---

## Assumptions

- **A-1:** English-only, Markdown and plain text, as the existing product (`docs/PRD.md:61-62`) — nothing in the request asks for more.
- **A-2:** New rules ship at `Suggestion` or `Warning` severity and follow FR-4.5 (start off) unless the PRD decides otherwise — the conservative reading, because defaults change recorded scores.
- **A-3:** Metrics are computed on prose only (protected spans excluded), using the same word definition as `prose_words` (`src/doc/mod.rs:235`) — keeps the score denominator and metrics consistent.
- **A-4:** Bundled data stays in the repository and the binary (`include_str!`-style), so NFR-2 (`docs/PRD.md:295`) and offline builds hold, and the `--no-default-features` build still has every classical feature (`Cargo.toml:23-28`).
- **A-5:** "Leans less on the model" is measured as model-eligible findings per 1,000 words remaining after a rules-only fix, on the eval corpus. This has not been measured; the baseline is a PRD task.
- **A-6:** The existing `Score` formula and JSON shape stay unchanged unless Brad chooses route (c) in OQ-1.

---

## Resolved Decisions

| # | Decision | Rationale |
|---|----------|-----------|
| RD-1 | Everything is local and deterministic; no network, no model in the classical path | The request's constraint; NFR-6 (`docs/PRD.md:311`). |
| RD-2 | Every edit, classical or model, still goes through `verify::edit` | The request's constraint; ADR-05; `Verified<T>` (`src/verify/mod.rs:65`). |
| RD-3 | Text metrics (candidate 6) are the first pillar | Brad's stated priority in the request. |
| RD-4 | TextRank (candidate 7) is out of this round | Lowest priority in the request, cannot pass verification as an insertion, no source read for it. |
| RD-5 | Classical routing reuses the existing Safe-with-fallthrough rule (`src/engine/window.rs:110-118`) | No new routing needed; keeps hard cases going to the model. |
| RD-6 | New pure modules get rows in `tests/layering.rs` and follow ADR-01 | Layering is test-enforced (`tests/layering.rs:9`, `:96`). |
| RD-7 | Metrics are labelled style indicators, never provenance | FR-5.4 (`docs/PRD.md:226`); AI detection out of scope (`docs/PRD.md:58`). |

---

## Open Questions

| # | Question | Context | Options and recommendation | Decider |
|---|----------|---------|----------------------------|---------|
| OQ-1 | How do metrics "feed the score"? | Changes what the score means (FR-5.1, FR-5.4) and the pinned JSON (`tests/score.rs:96-98`). | (a) separate, report only; (b) metric findings from rules at Suggestion, flowing through the existing formula, `--min-severity` and params; (c) a new term in the formula. **Recommend (b), with raw metrics always reported (a).** (c) invalidates all recorded scores. | Brad |
| OQ-2 | Ship uncalibrated thresholds? | No human corpus in repo; eval paragraphs are short (`eval/README.md:9-12`); "known AI tic" is unsourced. | (i) report-only until calibrated; (ii) default-off rules with provisional thresholds; (iii) build a calibration corpus first. **Recommend (i) now, (iii) as a follow-up task.** | Brad |
| OQ-3 | Should new rules be on by default? | FR-4.5 says a later-release rule starts off (`docs/PRD.md:215`), but then default users get no new fixes. | Keep FR-4.5 (off) or flip for selected safe, low-risk rules (e.g. the plain wordiness table). **Recommend: wordiness table on, metric and tagger rules off.** | Brad |
| OQ-4 | Tagger lexicon source and licence | No lexicon was evaluated; Rust crates are young (Competitive §1.2). | (i) small owned lexicon of closed-class words and frequent ambiguous words, tagger as gate only; (ii) bundle a third-party lexicon (needs licence review); (iii) depend on a crate. **Recommend (i).** | PRD author |
| OQ-5 | How should the negation invariant treat SLP003/SLP004? | They exist to remove a "not" (`slp003…rs:31`). | Exempt by rule code via `ProposalOrigin::Rule`; or apply only to model-origin edits and require model output to keep every negation unless the window's finding is SLP003/SLP004. **Recommend the second.** | PRD author |
| OQ-6 | Does adding a metrics object to JSON need a `schemaVersion` bump? | `SCHEMA_VERSION` bumps on "any breaking change" (`src/report/json.rs:13-14`); NFR-9 forbids shims (`docs/PRD.md:323`). | An additive sibling object is not breaking; bump anyway for clarity. **Recommend additive, no bump, documented in REFERENCE.** | PRD author |
| OQ-7 | What counts as a "safe" fix: grammatical only, or meaning-preserving? | The verifier cannot check meaning (`src/verify/mod.rs:5`); intensifier deletion and SLP004 joins can shift emphasis. | (i) safe = grammatical and behaviour-neutral, so exclude intensifier deletion; (ii) allow with a distinct low-confidence class. **Recommend (i) for now, intensifier deletion detection-only.** | Brad |
| OQ-8 | TextRank (candidate 7) | Insertion fails `verify`; no source read. | Out; or report-only later. **Recommend out.** | Brad |
| OQ-9 | Which blocks and what granularity for metrics? | `prose_words` counts headings, list items and table cells (`src/doc/mod.rs:235`, `:38-52`). | Paragraph blocks only; minimum sentence and word counts; per-file plus overall from raw counts. **Recommend that.** | PRD author |
| OQ-10 | Add `rust-stemmers`? | Last release 2019, MIT/BSD-3-Clause, ~2.3 MB[^stemmers-short]; only Pillar 6 needs it. | Add for Pillar 6 only, or skip and use a suffix-strip heuristic. **Recommend decide at Pillar 6, not now.** | PRD author |

---

## References

- [Current State Analysis](./current-state-analysis.md) — source for all local code citations
- [Competitive Analysis](./competitive-analysis.md) — source for all peer-system and external claims

[^fk-short]: Flesch–Kincaid readability tests. https://en.wikipedia.org/wiki/Flesch%E2%80%93Kincaid_readability_tests. Retrieved 2026-11-01. The grade formula has no upper bound and degenerate input gives extreme values. Full citation in the Competitive Analysis.
[^brill-short]: Brill tagger. https://en.wikipedia.org/wiki/Brill_tagger. Retrieved 2026-11-01. Lexicon plus contextual rules; "a few hundred rules". Full citation in the Competitive Analysis.
[^gptzero-short]: Perplexity, burstiness, and statistical AI detection. https://gptzero.me/news/perplexity-and-burstiness-what-is-it/. Retrieved 2026-11-01. Burstiness defined as variation in perplexities from a language model; the page is inconsistent about current use. Full citation in the Competitive Analysis.
[^stemmers-short]: rust-stemmers 1.2.0. https://docs.rs/crate/rust-stemmers/latest and https://crates.io/api/v1/crates/rust-stemmers. Retrieved 2026-11-01. Last release 2019-11-17; licence "MIT/BSD-3-Clause"; crate size 2,351,301 bytes. Full citation in the Competitive Analysis.
