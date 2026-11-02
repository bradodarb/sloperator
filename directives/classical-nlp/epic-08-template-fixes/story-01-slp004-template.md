# Story 01: SLP004 `X and Y` template fix

**Epic:** [epic-08-template-fixes](../epic-08-template-fixes.md)
**SPEC Refs:** §29 (SLP004 template `X and Y`), §31 (macro arm for 4)
**PRD Refs:** FR-8.1, FR-8.3, FR-8.4, FR-8.5, FR-8.7
**Component / ADR:** Architecture §4.9 row SLP004
**Wave:** 1

## Scope

In src/rules/builtin/slp004_not_just_but_also.rs: the class becomes `FixClass::Safe`, the struct gains `template: regex::Regex` (built from `TEMPLATE`, panicking on a bad pattern like `PatternRule::new`), and `template_fix` and `capitalise` are added exactly as §29. `check` attaches the template fix per detection match, and `explain` changes. Detection (`PATTERNS`) is unchanged. Add fixtures/rules/SLP004/fixed.md (§29's three lines); positive.md is unchanged. In tests/rules_fixtures.rs, move 4 to the safe arm (`safe(4, slp004_positive_fires, slp004_negative_silent, slp004_fix_matches)`, line 144) and add the decline tests.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/rules/builtin/slp004_not_just_but_also.rs` | 1-2, 4-5, 14-17, 19-39, 46-56 | class, template, `template_fix`, `check`, explain |
| `fixtures/rules/SLP004/fixed.md` | new | §29 fixed lines |
| `tests/rules_fixtures.rs` | 144, new tests | safe arm for 4; decline tests |
| `src/rules/mod.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] positive.md fixes to fixed.md: "It is cheap and quick." and "We are careful and ready."; line 1 (`fast`, ambiguous) keeps a fix-less finding (FR-8.1, FR-8.7)
- [ ] "It is not only cheap but also quick tools." has no fix: Y does not end the clause (FR-8.3)
- [ ] X and Y in different sentences give no fix (FR-8.1)
- [ ] A leading "Not" capitalises X
- [ ] Each template fix passes the verifier through the D-8 allowance (one `not` dropped, no capital, no modal) (FR-8.5)
- [ ] The finding still selects a model window when the template declines (FR-8.3)
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::rules_fixtures` | `slp004_positive_fires / slp004_negative_silent / slp004_fix_matches` | safe-arm fixture tests | FR-8.1, FR-8.4, FR-8.7 |
| 2 | `sloperator::rules_fixtures` | `slp004_declines_when_y_continues` | no fix when Y continues | FR-8.3 |
| 3 | `sloperator::rules_fixtures` | `slp004_declines_across_sentences` | no fix across sentences | FR-8.1 |
| 4 | `sloperator::rules_fixtures` | `no_safe_fixture_fix_newly_rejected` | SLP004 template fixes pass `verify::edit` | FR-8.5, FR-6.6 |
| 5 | `sloperator::rules::tests` | `tagging_runs_once_per_pass` | SLP004 and SLP006 still tag once together | NFR-2 |

## Notes

The fix range runs from "not" to the end of Y, wider than the finding range; `Finding`'s fix range is independent (src/rules/mod.rs:107-128). The tag read comes after the regex and sentence guards (FR-7.4).

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='slp004_ no_safe_fixture_fix_newly_rejected rules::tests::tagging_runs_once_per_pass'`
