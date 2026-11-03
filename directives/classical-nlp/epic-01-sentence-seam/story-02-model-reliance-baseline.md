# Story 02: Model-reliance harness and baseline row

**Epic:** [epic-01-sentence-seam](../epic-01-sentence-seam.md)
**SPEC Refs:** §2 (Model-reliance harness)
**PRD Refs:** FR-10.1, FR-10.2, FR-10.3
**Component / ADR:** Architecture §4.14 Model-reliance harness
**Wave:** 1

## Scope

Add `tests/model_reliance.rs` exactly as SPEC §2: a rules-only fix of `eval/paragraphs/p01.md`–`p20.md` analysed as temp-dir paths, counting Model-class findings and fix-less Safe findings per 1,000 prose words. Add `eval/model-reliance.md` with a short header (what is measured, `make test TESTS='model_reliance'`) and the table, and fill in the `baseline` row from the printed value and today's date. No rule, table or `src/` change.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `eval/model-reliance.md` | new | header + `| phase | residual per 1,000 prose words | date |` table with the baseline row |
| `tests/model_reliance.rs` | new | tests below |

## Acceptance Criteria

- [ ] The test computes the residual with the predicate of src/engine/window.rs:110-119, rebuilt from `Registry` metadata (no new public API) (FR-10.1)
- [ ] `eval/model-reliance.md` has exactly one `| baseline |` row, holding the printed value (two decimals) and a 2026-11 date (FR-10.2)
- [ ] The test fails if the residual exceeds the baseline (FR-10.3)
- [ ] No file under `src/` changes
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::model_reliance` | `residual_does_not_exceed_baseline` | the residual per 1,000 prose words is at most the recorded baseline, and the value is printed | FR-10.1, FR-10.2, FR-10.3 |

## Notes

Run the gate once with the row value set to a placeholder large number, read the printed line with `--no-capture` or from the junit output, then write the real value. Later phases (2, 4, 5, 6) append rows; only the baseline row is read. Temp-dir paths keep any checkout `sloperator.toml` out (tests/pipeline.rs:25-37).

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='model_reliance'`
