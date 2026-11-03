# Story 03: Class-change fallout: docs, JSON comment, SLP007/SLP015 guard, SLP901 test, phase-5 row

**Epic:** [epic-08-template-fixes](../epic-08-template-fixes.md)
**SPEC Refs:** §31 (Class change fallout), §18 (catalogue), §36.7, §36.9, §2 (phase row)
**PRD Refs:** FR-8.4, FR-8.6, FR-10.2, FR-10.3, NFR-6, NFR-9
**Component / ADR:** Architecture §4.9; §14 risks (`fix.available`)
**Wave:** 3

## Scope

src/report/json.rs:88 gains the comment "Safe-class findings without a fix may still be rewritten by the model."; no code change. CHANGELOG.md, docs/REFERENCE.md and docs/SPEC.md §10 record the SLP003/SLP004 class change, the `max_words` param and the `fix.available` meaning. Add `slp007_slp015_unchanged` (tests/rules_registry.rs) and `failing_template_fix_is_slp901` (tests/pipeline.rs). Append a `phase 5` row to eval/model-reliance.md.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/report/json.rs` | 88-96 | comment only |
| `CHANGELOG.md` | 0.2.0 section | class change and its JSON consequence |
| `docs/REFERENCE.md` | SLP003/SLP004 entries; JSON `fix.available` | updated |
| `docs/SPEC.md` | §10 rows | SLP003/SLP004 rows |
| `eval/model-reliance.md` | table | `phase 5` row |
| `tests/rules_registry.rs` | existing | tests below (methods added) |
| `tests/pipeline.rs` | existing | tests below (methods added) |
| `tests/model_reliance.rs` | existing | tests below (methods added) |
| `tests/report.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] SLP007 and SLP015 are `FixClass::Model`, their `explain` equals the literal text at slp007…rs:28-30 and slp015…rs:26-29, and the epic diff shows no change under those files (FR-8.6)
- [ ] A forged SLP004 fix whose replacement drops Y's capitalised word becomes SLP901 with the original bytes kept (NFR-6)
- [ ] REFERENCE and CHANGELOG say `fix.available` is false for an unfixed SLP003/SLP004 match that the model may still rewrite (FR-8.4, NFR-9)
- [ ] The `phase 5` row is recorded and ≤ baseline (FR-10.2, FR-10.3)
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::rules_registry` | `slp007_slp015_unchanged` | class and explain unchanged | FR-8.6 |
| 2 | `sloperator::pipeline` | `failing_template_fix_is_slp901` | a failing template fix is rejected loudly | NFR-6 |
| 3 | `sloperator::model_reliance` | `residual_does_not_exceed_baseline` | phase-5 value ≤ baseline | FR-10.2, FR-10.3 |
| 4 | `sloperator::report` | `(all existing)` | JSON output unchanged by the comment | NFR-9 |

## Notes

The macro arm moves for 3 and 4 are in story-01/02, as the documented test edits FR-8.4 makes necessary (NFR-1).

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='slp007_slp015_unchanged failing_template_fix_is_slp901 model_reliance sloperator::report'`
