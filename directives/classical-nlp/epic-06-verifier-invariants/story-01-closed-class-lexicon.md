# Story 01: `text::lexicon` closed lists; SLP006 uses the shared modals

**Epic:** [epic-06-verifier-invariants](../epic-06-verifier-invariants.md)
**SPEC Refs:** §22 (`text::lexicon`)
**PRD Refs:** FR-6.1, FR-6.2, FR-6.3, FR-9.1, NFR-4
**Component / ADR:** Architecture §4.2 Closed-class lexicon; ADR-07
**Wave:** 1

## Scope

Create src/text/lexicon.rs exactly as §22: `NEGATIONS`, `MODALS`, `HEDGES`, `NOT_NAMES`, `STOP_WORDS`, `negation_count`, `modal_of`, `is_stop_word` and `is_common_capitalised`, with unit tests. Add `pub mod lexicon;` to src/text/mod.rs. In src/rules/builtin/slp006_copula_avoidance.rs, delete the local `MODALS` (lines 29-32) and test `prev == "to" || lexicon::MODALS.contains(&prev)` at lines 120 and 123. SLP006's behaviour is unchanged.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/text/lexicon.rs` | new | tables, helpers, unit tests |
| `src/text/mod.rs` | module list | `pub mod lexicon;` |
| `src/rules/builtin/slp006_copula_avoidance.rs` | 29-32, 120-123 | local `MODALS` deleted; shared list used |
| `tests/rules_fixtures.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] Every table is sorted and unique, apart from `NOT_NAMES`, whose order is free (unit test) (NFR-4)
- [ ] `negation_count("isn't") == 1`, `negation_count("isn’t") == 1`, `negation_count("never") == 1`, `negation_count("few") == 0` (FR-6.1)
- [ ] `modal_of("won't") == Some("will")`, `modal_of("can't") == Some("can")`, `modal_of("we'll") == Some("will")`, `modal_of("I'd") == None` (FR-6.3, DQ-9)
- [ ] `is_common_capitalised("May")`, `("Potentially")`, `("The")` and `("I'm")` are true; `("Redis")` is false (FR-6.2)
- [ ] Every SLP006 fixture test passes unchanged
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::text::lexicon::tests` | `tables_sorted_unique` | table order and uniqueness | NFR-4 |
| 2 | `sloperator::text::lexicon::tests` | `contractions` | negation and modal contractions, ASCII and ’ | FR-6.1, FR-6.3 |
| 3 | `sloperator::text::lexicon::tests` | `common_capitalised_words` | stop words, modals, hedges and NOT_NAMES are common; names are not | FR-6.2 |
| 4 | `sloperator::rules_fixtures` | `slp006_positive_fires / slp006_negative_silent / slp006_fix_matches` | SLP006 unchanged | NFR-1 |

## Notes

`text` may not use `types`; the `NEGATION_EXEMPT` codes live in `verify` (story-03).

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='text::lexicon::tests slp006_'`
