# Classical (Non-Neural) NLP — Epics and Stories

**Source:** [PRD](../../docs/classical-nlp/PRD-classical-nlp.md) | [SPEC](../../docs/classical-nlp/SPEC-classical-nlp.md) | [Architecture](../../docs/classical-nlp/ARCHITECTURE-classical-nlp.md)
**Goal:** Sloperator gains one sentence/word definition, always-on reading metrics with configurable floors, default-off metric, intensifier and redundancy rules, more no-model fixes (SLP001 wordiness, tag-gated SLP006, SLP003/SLP004 templates) and meaning invariants in the verifier. It stays offline and deterministic, and model reliance never rises above the recorded baseline.
**Date:** 2026-11-02

## Delivery Plan

One line per epic, in delivery order (the architecture's phases 1a–6, locked by PRD D-1). Stories separated by spaces run in parallel; `|` starts the next wave.

```slice
epic-01-sentence-seam: story-01-prd-host-reference story-02-model-reliance-baseline story-03-doc-prefix-and-to-text | story-04-text-segment-and-prose | story-05-wire-prose-through-detect-and-windows | story-06-layering-rows
epic-02-reading-metrics: story-01-syllable-counter | story-02-metric-counts-and-floors | story-03-metrics-config-table | story-04-pipeline-file-metrics | story-05-text-json-reports story-06-mcp-metrics
epic-03-metric-rules: story-01-param-bounds | story-02-metric-rule-helpers | story-03-slp018-021-rules | story-04-metric-rules-catalogue
epic-04-calibration-corpus: story-01-calibration-corpus
epic-05-wordiness: story-01-slp001-wordiness-rows | story-02-slp022-weak-intensifier | story-03-wordiness-catalogue
epic-06-verifier-invariants: story-01-closed-class-lexicon story-02-window-codes | story-03-meaning-checks | story-04-verification-limits-sentence
epic-07-tagger-slp006: story-01-tagger-and-spike-gate | story-02-prose-tags-and-slp006 | story-03-multi-window-pass-test
epic-08-template-fixes: story-01-slp004-template | story-02-slp003-template | story-03-class-change-fallout
epic-09-redundancy: story-01-stemmer-decision | story-02-similarity | story-03-slp023-redundant-sentence | story-04-redundancy-catalogue-and-perf
```

What each epic needs before it may start. Parallel delivery is on: an epic needs exactly what its line lists, and an
epic without a line needs nothing. Every later epic shares files with the one before it (`tests/rules_fixtures.rs`,
`src/rules/builtin/mod.rs`, `CHANGELOG.md`, `docs/REFERENCE.md`, `eval/model-reliance.md`, `src/text/mod.rs`), and the
PRD locks the phase order (D-1), so the chain is real. The one independent epic is epic-04 (calibration). It touches
only `eval/calibration/` and `tests/calibration.rs`, so it runs beside epic-03.

```needs
order: explicit
epic-02-reading-metrics: epic-01-sentence-seam
epic-03-metric-rules: epic-02-reading-metrics
epic-04-calibration-corpus: epic-02-reading-metrics
epic-05-wordiness: epic-01-sentence-seam epic-03-metric-rules
epic-06-verifier-invariants: epic-01-sentence-seam epic-05-wordiness
epic-07-tagger-slp006: epic-01-sentence-seam epic-06-verifier-invariants
epic-08-template-fixes: epic-06-verifier-invariants epic-07-tagger-slp006 docs/classical-nlp/spikes/G4-slp006-tagger-spike.md
epic-09-redundancy: epic-06-verifier-invariants epic-08-template-fixes
```

## Epics

| Epic | Outcome | PRD Refs | Stories |
|------|---------|----------|---------|
| [epic-01-sentence-seam](./epic-01-sentence-seam.md) | Walking skeleton: the model-reliance baseline is recorded, and every `check`/`fix` builds one `Prose` per document pass. Rules and window planning read its cached segmentation, and behaviour is unchanged | FR-1, FR-10, NFR-1, NFR-2, NFR-5, NFR-7, NFR-8, NFR-9 | 6 |
| [epic-02-reading-metrics](./epic-02-reading-metrics.md) | Every text, JSON and MCP report shows reading metrics, with report floors configurable in `[metrics]` | FR-2, FR-3, NFR-3, NFR-5 | 6 |
| [epic-03-metric-rules](./epic-03-metric-rules.md) | Default-off SLP018–SLP021 feed metrics into the score, with bounded params | FR-4, FR-11, NFR-4 | 4 |
| [epic-04-calibration-corpus](./epic-04-calibration-corpus.md) | A public-domain corpus records the metric values beside the provisional thresholds | D-4 follow-up, FR-4.5 | 1 |
| [epic-05-wordiness](./epic-05-wordiness.md) | SLP001 fixes padded phrases without a model; SLP022 flags intensifiers | FR-5, FR-10.2, FR-11 | 3 |
| [epic-06-verifier-invariants](./epic-06-verifier-invariants.md) | The verifier rejects dropped negations, lost names and changed modals | FR-6, NFR-6, NFR-9 | 4 |
| [epic-07-tagger-slp006](./epic-07-tagger-slp006.md) | A Brill-style tagger, run at most once per pass, lets SLP006 fix `Det (Adj) N serves as` | FR-7, NFR-2 | 3 |
| [epic-08-template-fixes](./epic-08-template-fixes.md) | SLP003 and SLP004 become Safe and attach template fixes | FR-8, NFR-9 | 3 |
| [epic-09-redundancy](./epic-09-redundancy.md) | Default-off SLP023 reports repeated sentences within a paragraph | FR-9, FR-10.3, NFR-2, NFR-5 | 4 |

34 stories in total.

## Brad's decisions applied

From `decisions.md` (spec gate). Each is built as given, and no story reopens it.

| Gap | Decision | Where it lands |
|-----|----------|----------------|
| Gap 4 (OD-1) | (a) Wordiness rows extend default-on SLP001; intensifiers go in a new default-off detection-only SLP022 | epic-05 story-01, story-02 |
| Gap 5 (OD-3) | (a) Report minimums are configurable and apply consistently, even with every metric rule off. A top-level `[metrics]` table (`min_sentences`, `min_words`, integers ≥ 1, defaults 5/100) goes into `Settings`. `Counts::derive` takes the floors. Each file's metrics use its own settings; `summary.metrics` uses the run's root config (the `--config` chain, else the nearest `sloperator.toml` up from the working directory, else defaults). Metric-rule minimums default to the configured floors and may not go below them | epic-02 story-02 to story-05; epic-03 story-02 |
| Gap 11 (OD-2) | (a) Only rule-origin edits skip capitalised common words; model rewrites get FR-6.2's literal proxy | epic-06 story-01, story-03 |

## Open question for Brad

**Planning files versus the standalone scan.** `tests/standalone.rs::no_host_references` scans every file in the
repository except `.git` and `target` (tests/standalone.rs:37). Among its markers are the host's name and `UNIT=`
(:15-28). This `directives/` tree is tracked on the feature branch, and every story's mandated gate line contains
both markers. So the test fails as soon as these files exist, and on `main` too if they land there. SPEC §1 found only
the PRD line.

- **(a)** Add `directives` to `NOT_SCANNED`, so planning artifacts are not scanned as shipped content. Simple, and the
  scan still covers `src/`, `tests/`, `docs/`, `eval/` and `fixtures/`.
- **(b)** Keep the scan whole, and have the landing remove `directives/` from the branch before it merges.

Recommendation: **(a)**. It needs no change to the delivery workflow, and the planning tree is not product content.
epic-01 story-01 is written for (a), and its Notes say what changes under (b).

## Deferred items (from `deferred.md`)

| Gap | Owner | How it lands |
|-----|-------|--------------|
| Gap 8 — stemmer inflections | epic-09 story-01 | `stem::tests::inflections_meet` runs make/makes/making and caches/cached/caching against the backend that ships. The assertion is never weakened; the fallback is repaired if it is chosen |
| Gap 9 — multi-window pipeline cache test | epic-07 story-03 (and story-02 for the no-candidate assertion) | `pipeline::tests::multi_window_fix_analyses_each_pass_once` goes through the fake rewriter seam. It tells the original pass apart from each candidate pass, and fails when one pass segments or tags twice. `rules::tests::no_candidate_no_tagging` is kept |
| Gap 1 (segmentation-work regression, named in Gap 9's item) | epic-01 story-05 | `engine::window::tests::planning_does_not_rescan` |

## Accepted Gaps

| Gap | Where it matters | Note |
|-----|------------------|------|
| Gap 10 | all epics | No `docs/context/` conventions pack exists. Stories follow `docs/DEVELOPMENT.md`, `Cargo.toml` lints, `clippy.toml` and `tests/layering.rs`, and no story creates a conventions pack |
| Gap 12 | epic-02, epic-06 | Sufficient statistics (`sentence_len_sq`, merged type set), contraction-to-modal mapping (with `'d` rejected loudly) and static purity scans are the accepted equivalents. The binary-size record stays a deliverable of epic-02 and epic-09 |
