# Story 01: `text::syllable`

**Epic:** [epic-02-reading-metrics](../epic-02-reading-metrics.md)
**SPEC Refs:** §7 (`text::syllable`), §36.2
**PRD Refs:** FR-2.3, FR-2.6, NFR-4, NFR-10
**Component / ADR:** Architecture §4.3 Syllable counter; ADR-07
**Wave:** 1

## Scope

Create src/text/syllable.rs exactly as SPEC §7 (vowel groups, silent -e/-ed/-es, a sorted `EXCEPTIONS` table), with unit tests, and add `pub mod syllable;` to src/text/mod.rs. Nothing calls it yet.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/text/syllable.rs` | new | `syllables` and unit tests |
| `src/text/mod.rs` | module list | `pub mod syllable;` |

## Acceptance Criteria

- [ ] The fixed list of §36.2 (the … sloperator) gives 1,1,2,1,1,2,1,1,1,2,2,2,1,2,3,2,2,5,2,1,0,1,2,4 (FR-2.6, NFR-10)
- [ ] `EXCEPTIONS` is sorted and unique (unit test) (NFR-4)
- [ ] A word with no letter gives 0, and any other word gives at least 1
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::text::syllable::tests` | `fixed_list` | the 24-word list of §36.2 | FR-2.6, NFR-10 |
| 2 | `sloperator::text::syllable::tests` | `exceptions_sorted_unique` | table order and uniqueness | NFR-4 |

## Notes

If a listed value disagrees with the §7 algorithm, fix the algorithm or add an exception; never change the asserted list.

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='text::syllable::tests'`
