# Story 05: Wire `Prose` through `detect_in`, `rules::detect` and `windows`

**Epic:** [epic-01-sentence-seam](../epic-01-sentence-seam.md)
**SPEC Refs:** §5 (Window sentence ranges; `starts_sentence` moves), §10 (`RuleCtx.prose` and `rules::detect`), §11 (Pipeline: `Analysis.prose`, the `Detected` tuple, `model_windows`; the `metrics` fields come in epic-02)
**PRD Refs:** FR-1.4, FR-7.3, NFR-2, NFR-9
**Component / ADR:** Architecture §4.9 Rule integration, §4.11 Pipeline and windows; ADR-02
**Wave:** 3

## Scope

`detect_in` (src/pipeline/mod.rs:295) builds `text::Prose::new(&d)` after `directives::extract` and returns the four-value `Detected`. `Analysis` gains `prose`. `fix` carries each pass's `Prose` with its `Document` (§11), and `model_windows` (:212) takes `prose`. `RuleCtx` gains `prose`, and `rules::detect` (src/rules/mod.rs:158) takes `&Prose`. `windows` (src/engine/window.rs:75) takes `&Prose` and uses `prose.segments().sentence_span` in Sentence mode. `sentence_range` and the `unicode_segmentation` import are deleted. `starts_sentence` moves into `text::segment`, and src/rules/helpers.rs:141 re-exports it. Test call sites gain `let prose = Prose::new(&d);`. `FileResult` metrics are NOT added here.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/pipeline/mod.rs` | 17, 33-39, 69-77, 99-196, 212-227, 264-292, 295-309 | `Analysis.prose`, `Detected` alias, `detect_in` builds `Prose`, `fix` pairs `Prose` with its `Document`, `model_windows(doc, prose, …)`, `accept` destructures four values |
| `src/rules/mod.rs` | 9, 84-98, 158-177 | `RuleCtx.prose`; `detect(doc, prose, sup, rules)` |
| `src/engine/window.rs` | 7-15, 71-108, 237-278, end | `windows(doc, prose, …)`; `sentence_range` and the unicode import deleted; `#[cfg(test)] mod tests` with `planning_does_not_rescan` |
| `src/rules/helpers.rs` | 138-160 | `starts_sentence` body replaced by `pub use crate::text::segment::starts_sentence;` |
| `src/text/segment.rs` | end | `starts_sentence` moved here, unchanged |
| `tests/engine.rs` | 71, 137, 145, 163, 248, 252, 262, 266 | `&prose` added to each `windows` call |
| `tests/rules_core.rs` | 80, 98, 103, 119 | `&prose` added to each `detect` call |
| `tests/rules_fixtures.rs` | 53 | `findings` builds a `Prose` |
| `tests/pipeline.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] Every existing test in tests/engine.rs, tests/rules_core.rs, tests/rules_fixtures.rs, tests/pipeline.rs and tests/cli*.rs passes, with only the added `&prose` argument as an edit (FR-1.4, NFR-1)
- [ ] In Sentence mode, `windows` plans from `Segments::sentence_span`: four Model findings in two sentences of one paragraph plan two windows with zero block scans (NFR-2, Gap 1 regression)
- [ ] `sentence_range` and `mask::prefix_len` are gone, and `starts_sentence` has one definition (in `text::segment`) (NFR-9)
- [ ] `lint` still names nothing from the model port (`lint_never_constructs_rewriter` passes)
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::engine::window::tests` | `planning_does_not_rescan` | `Prose::new` scans the one block once; `windows` adds zero scans and plans two Rewrite windows | NFR-2, FR-1.4 |
| 2 | `sloperator::engine` | `(all existing window and mask tests)` | window ranges are unchanged | FR-1.4 |
| 3 | `sloperator::rules_core` | `(all existing)` | detection is unchanged with the shared `Prose` | FR-7.3 |
| 4 | `sloperator::rules_fixtures` | `(all existing)` | every rule fixture still passes | NFR-1 |
| 5 | `sloperator::pipeline` | `(all existing, incl. lint_never_constructs_rewriter)` | lint/fix behaviour unchanged end to end | NFR-1 |

## Notes

`Prose::new(` may appear only in src/pipeline/mod.rs, src/text/mod.rs, and the `#[cfg(test)]` modules of src/engine/window.rs and src/rules/mod.rs (§6). The test path `no-config-here/doc.md` follows tests/engine.rs:29-36. Clippy's `type_complexity` is why the `Detected` alias exists.

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='engine::window::tests::planning_does_not_rescan sloperator::engine sloperator::rules_core sloperator::rules_fixtures sloperator::pipeline'`
