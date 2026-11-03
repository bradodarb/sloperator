# Story 01: SLP001 wordiness/nominalisation rows with the sentence-shape guard

**Epic:** [epic-05-wordiness](../epic-05-wordiness.md)
**SPEC Refs:** §20 (SLP001 wordiness and nominalisation rows)
**PRD Refs:** FR-5.1, FR-5.2, FR-5.3, FR-5.5, D-9, D-10
**Component / ADR:** Architecture §4.9 Rule integration, row SLP001; ADR-08 (a)
**Wave:** 1

## Scope

Append every context-free and guarded row of §20's table to `ENTRIES` in src/rules/builtin/slp001_ai_vocabulary_replaceable.rs, one row per inflection. Add `NOUN_FOLLOW`, `CLAUSE`, `DECISION`, `RELATIVE`, `NOT_AFTER_TO`, `SUBJECT`, `CLOSERS`, `BE_FORMS`, `noun_final_allows` (reads `ctx.prose.segments()`), `fact_that_allows` and `allowed` exactly as §20, and skip a match in `check` when `allowed` is false. Widen the module comment and `explain`. Extend positive.md, fixed.md and negative.md, and add the tests. The 17 existing rows behave exactly as before.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/rules/builtin/slp001_ai_vocabulary_replaceable.rs` | 1-4, 6-7, 9-27, 43-49, 62-79 | rows, guard lists, guard functions, `check` skip, `explain` |
| `fixtures/rules/SLP001/positive.md` | end | a paragraph using every new phrase once, guarded rows in admitted shapes |
| `fixtures/rules/SLP001/fixed.md` | end | the same paragraph with replacements |
| `fixtures/rules/SLP001/negative.md` | end | the twenty counter-context sentences |
| `tests/rules_fixtures.rs` | 303-356, new tests | `safe_fix_contexts_grammatical` cases; `slp001_every_row_in_positive`; `slp001_guard_drops_unprovable_contexts` with `GUARD_COUNTER_CONTEXTS` |
| `tests/model_reliance.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] Every `ENTRIES` phrase gives exactly one finding on positive.md (FR-5.2)
- [ ] positive.md fixes to fixed.md, and fixed.md is grammatical (FR-5.1, FR-5.2)
- [ ] "We make a decision about it." → "We decide about it."; "They made a decision to leave." → "They decided to leave."; "We reached a conclusion before noon." → "We concluded before noon."; "At this point in time, we stop." → "Now, we stop."; "We stayed due to the fact that it rained." → "We stayed because it rained." (FR-5.2)
- [ ] Each of the twenty `GUARD_COUNTER_CONTEXTS` of §20 (seven from round 1, six from round 2, seven from round 3) gives no finding, and `apply_fixes(1, src) == src`; the failure message names the input (D-10, FR-5.3, Gap 2)
- [ ] No row adds or removes a modal or negation, and no `None` row is added (DQ-17)
- [ ] Fixing is idempotent over every fixture (FR-5.5)
- [ ] The model-reliance residual stays ≤ baseline (FR-10.3)
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::rules_fixtures` | `slp001_fix_matches` | positive.md fixes to fixed.md | FR-5.1, FR-5.2 |
| 2 | `sloperator::rules_fixtures` | `slp001_negative_silent` | negative.md, with the counter-contexts, gives no finding | D-10 |
| 3 | `sloperator::rules_fixtures` | `slp001_every_row_in_positive` | one finding per `ENTRIES` row on positive.md | FR-5.2 |
| 4 | `sloperator::rules_fixtures` | `slp001_guard_drops_unprovable_contexts` | the twenty counter-contexts: no finding, unchanged by fix | D-10, FR-5.3 |
| 5 | `sloperator::rules_fixtures` | `safe_fix_contexts_grammatical` | the five new `fixed` cases | FR-5.2 |
| 6 | `sloperator::rules_fixtures` | `fix_is_idempotent_over_all_fixtures` | a second fix pass changes nothing | FR-5.5 |
| 7 | `sloperator::model_reliance` | `residual_does_not_exceed_baseline` | the new rows do not raise model reliance | FR-10.3 |

## Notes

The counter-contexts must not go in `safe_fix_contexts_grammatical`'s `unchanged` list: it asserts a fix-less finding exists (tests/rules_fixtures.rs:350-354). `WordList::find` yields source ranges (src/rules/helpers.rs:37-65), and `BlockText::to_text` (epic-01 story-03) maps them back. Rejected candidates and the recall given up are listed in §20's design notes; do not add them.

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='slp001_ safe_fix_contexts_grammatical fix_is_idempotent_over_all_fixtures model_reliance'`
