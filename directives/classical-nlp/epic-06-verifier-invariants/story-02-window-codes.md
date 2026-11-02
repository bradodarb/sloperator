# Story 02: `Window.codes`: the codes of the findings a window resolves

**Epic:** [epic-06-verifier-invariants](../epic-06-verifier-invariants.md)
**SPEC Refs:** §24 (`Window.codes`)
**PRD Refs:** D-8, FR-6.4
**Component / ADR:** Architecture §4.11 Pipeline and windows; ADR-06
**Wave:** 1

## Scope

`Window` (src/engine/window.rs:17-30) gains `pub codes: Vec<RuleCode>`: one entry per non-system finding overlapping the window range, duplicates kept, in finding order. `plan` (:194-206) builds `weight_before` and `codes` from one `overlapping` closure. Nothing reads `codes` yet.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/engine/window.rs` | 17-30, 194-206 | `codes` field and its computation |
| `tests/engine.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] Two SLP003 findings in one paragraph give a window whose `codes` holds two `RuleCode(3)` entries (D-8)
- [ ] System findings are not in `codes`
- [ ] `weight_before` is unchanged for every existing window test
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::engine` | `window_codes_one_per_finding` | duplicates kept, system codes excluded | D-8 |
| 2 | `sloperator::engine` | `(all existing window tests)` | planning unchanged | NFR-1 |

## Notes

If any test or module builds a `Window` literal, it gains `codes: Vec::new()`.

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='window_codes_one_per_finding sloperator::engine'`
