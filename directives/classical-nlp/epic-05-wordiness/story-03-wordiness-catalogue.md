# Story 03: Wordiness catalogue, CHANGELOG, perf config and the phase-2 row

**Epic:** [epic-05-wordiness](../epic-05-wordiness.md)
**SPEC Refs:** §18 (catalogue), §35 (perf, Phase 2), §2 (phase rows)
**PRD Refs:** FR-10.2, FR-10.3, FR-11.2, FR-11.3, NFR-2
**Component / ADR:** Architecture §4.9; §12 phase 2 gate
**Wave:** 3

## Scope

CHANGELOG.md's 0.2.0 section adds the SLP001 rows (default-on, so default output gains findings and fixes on padded phrasing) and SLP022. docs/REFERENCE.md's catalogue and docs/SPEC.md §10 update SLP001's row and add SLP022. tests/perf.rs's `extend_select` gains "SLP022". eval/model-reliance.md appends a `phase 2` row with the printed value and date.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `CHANGELOG.md` | 0.2.0 section | SLP001 rows, SLP022 |
| `docs/REFERENCE.md` | 174-219 | SLP001 row widened; SLP022 row |
| `docs/SPEC.md` | 1013-1041 | §10 rows |
| `tests/perf.rs` | 105-112 | `"SLP022"` added |
| `eval/model-reliance.md` | table | `phase 2` row |
| `tests/model_reliance.rs` | existing | tests below (methods added) |
| `tests/standalone.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] `eval/model-reliance.md` has a `phase 2` row, and `residual_does_not_exceed_baseline` passes (FR-10.2, FR-10.3)
- [ ] The perf test passes with SLP018–SLP022 enabled (NFR-2)
- [ ] CHANGELOG says SLP001's new rows are default-on (FR-11.2), and REFERENCE/SPEC §10 describe SLP001's wider scope and SLP022 (FR-11.3)
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::model_reliance` | `residual_does_not_exceed_baseline` | phase-2 value ≤ baseline | FR-10.2, FR-10.3 |
| 2 | `sloperator::perf` | `rules_only_100kb_under_1s` | budget holds | NFR-2 |
| 3 | `sloperator::standalone` | `no_host_references` | docs use the repo's own form | NFR-1 |

## Notes

Changes accumulate under `0.2.0 (unreleased)`; no version bump (DQ-3).

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='model_reliance sloperator::perf sloperator::standalone#no_host_references'`
