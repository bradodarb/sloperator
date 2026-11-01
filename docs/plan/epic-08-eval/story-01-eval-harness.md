# Story 01: `tests/eval.rs` and `eval/README.md`

**Epic:** [epic-08-eval](../epic-08-eval.md)
**SPEC Refs:** §22
**PRD Refs:** FR-10.1, FR-10.2, NFR-3
**Component / ADR:** architecture §6 eval
**Wave:** 1

## Scope

Add `eval/README.md` (how the paragraphs were written; they are not training pairs) and `tests/eval.rs::eval_report_per_model`. The test runs `pipeline::fix` with both engines for each cached registry model, writes `target/eval/report.json`, prints the table, and asserts 20/20 integrity and mean s/window ≤ `MODEL_WINDOW_BUDGET_S`, a constant taken from the spike's Decision.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `eval/README.md` | new | corpus provenance (FR-10.1) |
| `tests/eval.rs` | new | tests below |

## Acceptance Criteria

- [ ] The report has, per model: mean score before/after, integrity n/20, accepted rewrites, rejections per `Check`, SLP902 count and mean seconds per window (FR-10.2).
- [ ] Integrity is 20/20 for the default model.
- [ ] `MODEL_WINDOW_BUDGET_S` equals the value in `docs/research/spike-results.md` and is not `0.0` (NFR-3).
- [ ] Without the default model, the test fails naming `sloperator model pull`.
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::eval` | `eval_report_per_model` | eval report and budget | FR-10.2, NFR-3 |

## Notes

The docs hand-off (architecture Phase 7) happens outside these stories.

Gate: `make test-integration TESTS='sloperator::eval#eval_report_per_model'`
