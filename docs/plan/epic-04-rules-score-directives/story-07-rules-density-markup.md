# Story 07: Rules SLP010, SLP011, SLP014 (markup and density)

**Epic:** [epic-04-rules-score-directives](../epic-04-rules-score-directives.md)
**SPEC Refs:** §10
**PRD Refs:** FR-4.2, FR-4.3, FR-4.6, FR-4.7, D-4
**Component / ADR:** §4.6 rules; ADR-06
**Wave:** 5

## Scope

Add bullet spam and over-bolding (fix class `none`, detection only) and em-dash density (safe; fires only above `max_per_1000_words`, then one finding per dash). Add their fixtures, register them, and extend the macro call.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/rules/builtin/slp010_bullet_spam.rs` | new | class none |
| `src/rules/builtin/slp011_over_bolding.rs` | new | class none |
| `src/rules/builtin/slp014_em_dash_density.rs` | new | density; safe fix → ", " |
| `fixtures/rules/SLP010/, SLP011/, SLP014/` | new | fixtures |
| `src/rules/builtin/mod.rs` | from story-06 | register three rules |
| `tests/rules_fixtures.rs` | from story-06 | macro gains 10, 11, 14; em_dash_fires_on_density_not_single |

## Acceptance Criteria

- [ ] A single em dash in a long file reports nothing; a file above 3 per 1,000 words reports one finding per dash (FR-4.3).
- [ ] SLP010/SLP011 never attach a fix (PRD Deviation Note D-4).
- [ ] SLP014's fixes bring `positive.md` to `fixed.md` (FR-4.7).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::rules_fixtures` | `slp010_*` | SLP010 fixtures | FR-4.2 |
| 2 | `sloperator::rules_fixtures` | `slp011_*` | SLP011 fixtures | FR-4.2 |
| 3 | `sloperator::rules_fixtures` | `slp014_* (incl. fix_matches)` | SLP014 fixtures | FR-4.2, FR-4.7 |
| 4 | `sloperator::rules_fixtures` | `em_dash_fires_on_density_not_single` | density threshold | FR-4.3 |

## Notes

none

Gate: `make test TESTS='slp010_ slp011_ slp014_ em_dash_fires_on_density_not_single'`
