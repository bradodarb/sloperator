# Story 04: Redundancy catalogue, perf config, binary size and the phase-6 row

**Epic:** [epic-09-redundancy](../epic-09-redundancy.md)
**SPEC Refs:** §35 (Performance test extension), §18 (catalogue, binary size), §2 (phase row)
**PRD Refs:** NFR-2, NFR-5, FR-9.5, FR-10.2, FR-10.3, FR-11.2, FR-11.3
**Component / ADR:** Architecture §8 Performance budgets; §12 phase 6 gate
**Wave:** 4

## Scope

tests/perf.rs (:105-112) writes `extend_select = ["SLP018", "SLP019", "SLP020", "SLP021", "SLP022", "SLP023"]`. CHANGELOG.md adds SLP023 and the release-binary size after Phase 6. docs/REFERENCE.md's catalogue and docs/SPEC.md §10 add SLP023 with params. eval/model-reliance.md appends a `phase 6` row.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `tests/perf.rs` | 105-112 | `"SLP023"` added |
| `CHANGELOG.md` | 0.2.0 section | SLP023; binary size after |
| `docs/REFERENCE.md` | 174-219 | SLP023 row |
| `docs/SPEC.md` | 1013-1041 | §10 row |
| `eval/model-reliance.md` | table | `phase 6` row |
| `tests/model_reliance.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] The perf test passes in its 1-second budget with SLP018–SLP023 on (NFR-2, FR-9.5)
- [ ] CHANGELOG records the release-binary size after Phase 6 beside the Phase 1c value (NFR-5)
- [ ] The `phase 6` row is recorded and the final residual is ≤ baseline (FR-10.2, FR-10.3)
- [ ] REFERENCE and docs/SPEC.md §10 list SLP023 with its params (FR-11.2, FR-11.3)
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::perf` | `rules_only_100kb_under_1s` | budget holds with every new rule on | NFR-2, FR-9.5 |
| 2 | `sloperator::model_reliance` | `residual_does_not_exceed_baseline` | final value ≤ baseline | FR-10.3 |

## Notes

Measure the binary size with `make -f "$AEGIS_DIR/Makefile" build UNIT=sloperator`. Write only the repo's own `make build` form into CHANGELOG.

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='sloperator::perf model_reliance'`
