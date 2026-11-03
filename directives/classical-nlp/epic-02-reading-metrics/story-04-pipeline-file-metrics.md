# Story 04: `FileResult.metrics`, `metrics_before` and `floors`

**Epic:** [epic-02-reading-metrics](../epic-02-reading-metrics.md)
**SPEC Refs:** §11 (Pipeline: `FileResult.metrics`)
**PRD Refs:** FR-3.4, NFR-2
**Component / ADR:** Architecture §4.11 Pipeline; ADR-02, ADR-05
**Wave:** 4

## Scope

`FileResult` (src/pipeline/mod.rs:52) gains `metrics: Counts`, `metrics_before: Option<Counts>` and `floors: Floors` (copied from `source.settings.metrics`). `lint` sets `metrics: a.prose.counts(&a.doc).clone()` and `metrics_before: None`. `fix` clones `prose0.counts(&doc0)` before `prose0` can move, and takes the final metrics from the `Prose` of whichever text survives (§11). Test helpers that build a `FileResult` literal gain the three fields (tests/write.rs:25, tests/report.rs:63, 242). Add tests/text_metrics.rs with the before/after test.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/pipeline/mod.rs` | 52-63, 80-92, 99-196 | three fields; `lint` and `fix` fill them |
| `tests/write.rs` | 25 | literal gains `metrics: Counts::default(), metrics_before: None, floors: Floors::default()` |
| `tests/report.rs` | 63, 242 | same literal fields |
| `tests/text_metrics.rs` | new | tests below |
| `tests/pipeline.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] `lint` gives `metrics_before == None` and the counts of the analysed text (FR-3.4)
- [ ] A rules-only fix of an SLP001-heavy document gives `metrics_before` as Some, with a word count different from `metrics` (FR-3.4)
- [ ] When the whole-document check discards every edit, `metrics == metrics_before` (architecture §7.2)
- [ ] `lint_never_constructs_rewriter` and every existing pipeline test pass
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::text_metrics` | `fix_reports_before_and_after` | Fix carries metrics of the original and the final text | FR-3.4 |
| 2 | `sloperator::pipeline` | `(all existing)` | pipeline behaviour unchanged | NFR-1 |
| 3 | `sloperator::write` | `(all existing)` | the literal still builds | NFR-1 |

## Notes

`text_metrics` uses a temp-dir source so no checkout config applies (tests/pipeline.rs:25-37).

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='text_metrics#fix_reports_before_and_after sloperator::pipeline sloperator::write'`
