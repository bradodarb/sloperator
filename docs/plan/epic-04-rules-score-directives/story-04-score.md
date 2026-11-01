# Story 04: Score and the README formula and limits

**Epic:** [epic-04-rules-score-directives](../epic-04-rules-score-directives.md)
**SPEC Refs:** §11, §2 (README)
**PRD Refs:** FR-5.1, FR-5.2, FR-5.3, FR-5.4
**Component / ADR:** §4.7 score
**Wave:** 2

## Scope

Implement `src/score/mod.rs` (`Score`, `score`, `overall`) as §11 specifies. SLP901 and SLP902 are excluded (DQ-4). Fill the README's Score formula section and the Limits section, which carries FR-5.4 word for word and the PRD Risk 1 limit.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/score/mod.rs` | new | Score, score, overall |
| `README.md` | from epic-01 story-01 | Score formula and Limits sections |
| `tests/score.rs` | new | tests below |

## Acceptance Criteria

- [ ] The score of a fixture equals the README formula computed by hand in the test (FR-5.1).
- [ ] The overall score is word-weighted across files (FR-5.1).
- [ ] The same input gives the same score (FR-5.3).
- [ ] The README contains the FR-5.4 sentence verbatim (FR-5.4).
- [ ] System findings do not change the score (DQ-4).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::score` | `formula_matches_readme` | formula | FR-5.1 |
| 2 | `sloperator::score` | `overall_is_word_weighted` | overall weighting | FR-5.1 |
| 3 | `sloperator::score` | `identical_input_identical_score` | determinism | FR-5.3 |
| 4 | `sloperator::score` | `readme_states_limits` | README limits text | FR-5.4 |

## Notes

`score` must not use engine, write, report or input (layering).

Gate: `make test TESTS='sloperator::score#formula_matches_readme sloperator::score#overall_is_word_weighted sloperator::score#identical_input_identical_score sloperator::score#readme_states_limits'`
