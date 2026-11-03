# Epic 02: Reading metrics in every report, with configurable floors

**Goal:** Every `check`/`fix` text, JSON and MCP report shows per-file and overall reading metrics computed from integer counts, with availability floors set by a `[metrics]` config table (Brad's Gap 5 decision (a)).
**PRD Refs:** FR-2.1–FR-2.10, FR-3.1–FR-3.6, NFR-3, NFR-5, D-5, D-6, D-11
**SPEC Refs:** §7 (text::syllable), §8 (text::metrics), §9 (Prose::counts), §11 (FileResult.metrics), §12 (Text and JSON reports), §13 (MCP results), §14 (REFERENCE: metrics), Open Decisions OD-3 option (a), §36.2, §36.3
**Modules:** `sloperator`

## Waves

Copied from this epic's line in `index.md`. Stories in one wave run in parallel; each wave waits for the one before.

| Wave | Stories | Why this order |
|------|---------|----------------|
| 1 | [story-01-syllable-counter](./epic-02-reading-metrics/story-01-syllable-counter.md) | the syllable counter has no dependency |
| 2 | [story-02-metric-counts-and-floors](./epic-02-reading-metrics/story-02-metric-counts-and-floors.md) | metrics call `syllables` |
| 3 | [story-03-metrics-config-table](./epic-02-reading-metrics/story-03-metrics-config-table.md) | `[metrics]` resolves into the `Floors` type from story-02 |
| 4 | [story-04-pipeline-file-metrics](./epic-02-reading-metrics/story-04-pipeline-file-metrics.md) | `FileResult` carries the floors from `Settings.metrics` |
| 5 | [story-05-text-json-reports](./epic-02-reading-metrics/story-05-text-json-reports.md), [story-06-mcp-metrics](./epic-02-reading-metrics/story-06-mcp-metrics.md) | reports and MCP each read `FileResult.metrics`/`floors`; they touch disjoint files |

## Acceptance Criteria

- [ ] `sloperator check doc.md` prints `doc.md: metrics …` after each score line, and `overall metrics …` after the summary line (FR-3.1)
- [ ] `--output-format json` has `files[].metrics`, `files[].metricsBefore` and `summary.metrics`, `schemaVersion` stays 1, and two runs give identical bytes (FR-3.2, NFR-3, D-11)
- [ ] MCP `lint` returns `metrics`, and MCP `fix` returns `metricsBefore` and `metricsAfter` (FR-3.3)
- [ ] A `sloperator.toml` with `[metrics]\nmin_sentences = 10` makes a 6-sentence, 150-word document report `unavailable (…; needs 10 sentences and 100 words)`, with every metric rule off (D-5, FR-2.7, Gap 5 (a))
- [ ] `concise` and `github` output is unchanged (FR-3.5)
- [ ] REFERENCE documents the formulas, the counting basis, `[metrics]` and the JSON/MCP keys (FR-3.6, FR-2.6)
- [ ] The release-binary size before and after is recorded in CHANGELOG.md's 0.2.0 section (NFR-5)
- [ ] The tests named in every story's Test Cases pass together on the merged epic.

## Notes

Gap 5 is decided (a), which the SPEC did not write. The SPEC's §8 `MIN_SENTENCES`/`MIN_WORDS` constants become the defaults of `text::metrics::Floors { sentences: u64, words: u64 }` (`Floors::default()` = 5/100), and `Counts::derive(&self, floors: &Floors)` replaces `derive(&self)`. `text` may not use `config` (ADR-01), so `config` holds a `text::metrics::Floors` (config → text is allowed). Floors reach reports through `FileResult.floors` (per file, from that file's `Settings.metrics`), and the summary uses the run root's floors (the `--config` chain, else the nearest `sloperator.toml` up from the working directory, else defaults). The architecture's §4.4, §8 and ADR-09 still say 'constants'. Those notes are now stale against Brad's decision; this slice builds the decision, and the architecture text is not edited here. ADR-09's rule-param side lands in epic-03 story-02.
