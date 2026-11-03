# Epic 01: Sentence seam and model-reliance baseline (walking skeleton)

**Goal:** Record the rules-only model-reliance baseline, then make every shipped `check` and `fix` build one `text::Prose` per document pass, so rules and window planning read one cached segmentation, with no change to any output.
**PRD Refs:** FR-1.1–FR-1.5, FR-2.10 (word basis), FR-7.3 (shared context), FR-10.1–FR-10.3, NFR-1, NFR-2, NFR-5, NFR-7, NFR-8, NFR-9
**SPEC Refs:** §1 (Standalone-scan hygiene), §2 (Model-reliance harness), §3 (doc helpers), §4 (text::segment), §5 (Window sentence ranges), §6 (text module root and layering rows), §9 (Prose, segments only), §10 (RuleCtx.prose and rules::detect), §11 (Pipeline Analysis.prose; metrics fields come in epic-02), §36.1, §36.11
**Modules:** `sloperator`

## Waves

Copied from this epic's line in `index.md`. Stories in one wave run in parallel; each wave waits for the one before.

| Wave | Stories | Why this order |
|------|---------|----------------|
| 1 | [story-01-prd-host-reference](./epic-01-sentence-seam/story-01-prd-host-reference.md), [story-02-model-reliance-baseline](./epic-01-sentence-seam/story-02-model-reliance-baseline.md), [story-03-doc-prefix-and-to-text](./epic-01-sentence-seam/story-03-doc-prefix-and-to-text.md) | independent: PRD doc line, a new test plus an eval file, and doc/mask helpers touch disjoint files |
| 2 | [story-04-text-segment-and-prose](./epic-01-sentence-seam/story-04-text-segment-and-prose.md) | segment uses `Document::continuation_prefix_len` from story-03 |
| 3 | [story-05-wire-prose-through-detect-and-windows](./epic-01-sentence-seam/story-05-wire-prose-through-detect-and-windows.md) | the wiring needs `Prose` and `Segments::sentence_span` from story-04 |
| 4 | [story-06-layering-rows](./epic-01-sentence-seam/story-06-layering-rows.md) | `sentence_scan_only_in_segmenter` passes only after story-05 removes the scan from window.rs |

## Acceptance Criteria

- [ ] `standalone::no_host_references` passes on the branch with the directives tree present (NFR-1, phase-1a gate; see the index's open question)
- [ ] `eval/model-reliance.md` has a `| baseline |` row filled from the test's printed value before any rule or table change, and `model_reliance::residual_does_not_exceed_baseline` passes (FR-10.2, FR-10.3)
- [ ] `sloperator check` and `sloperator fix` run end to end with `Prose` built once in `detect_in`. Every existing test in `tests/engine.rs`, `tests/rules_core.rs`, `tests/rules_fixtures.rs` and `tests/pipeline.rs` passes, edited only to add the `&prose` argument (FR-1.4, phase-1b gate)
- [ ] `sentence_span` equals the old `sentence_range` on every fixture, and window planning runs zero block scans (NFR-2, Gap 1)
- [ ] `split_sentence_bound_indices` appears in `src/` only in `src/text/segment.rs`; `mask::prefix_len` and `window::sentence_range` no longer exist (NFR-9)
- [ ] The tests named in every story's Test Cases pass together on the merged epic.

## Notes

Rule 7 skeleton: the deliverable that ships is the existing CLI and MCP, now with the `Prose` seam in `detect_in` (src/pipeline/mod.rs:295). Every later epic plugs into `Prose`, `RuleCtx.prose` or `Analysis.prose`. The baseline (story-02) must be committed before epic-03 or epic-05 changes any rule. Architecture Risk 1 (block-less prose spans) is covered by `word_count_equals_prose_words_on_every_fixture`: if it fails, `Segments` keeps such words in no sentence, and the story records that. SPEC line cites are a little off for some files (e.g. `starts_sentence` is at src/rules/helpers.rs:141, `detect_in` at src/pipeline/mod.rs:295, `detect` at src/rules/mod.rs:158); stories cite the real lines.
