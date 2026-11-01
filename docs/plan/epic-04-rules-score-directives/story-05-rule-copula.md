# Story 05: Rule SLP006 copula avoidance with its guards

**Epic:** [epic-04-rules-score-directives](../epic-04-rules-score-directives.md)
**SPEC Refs:** §10 (SLP006 row and the safe-fix design note)
**PRD Refs:** FR-4.2, FR-4.6, FR-4.7
**Component / ADR:** §4.6 rules; ADR-06
**Wave:** 3

## Scope

Add `slp006_copula_avoidance.rs`, which uses the `SINGULAR_SUBJECT` guard on the previous token and the `AS_COMPLEMENT` allow-list on the next token. Add its fixtures, register it, and add 6 to the macro call. Add `safe_fix_contexts_grammatical`, which asserts the exact pairs from §10.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/rules/builtin/slp006_copula_avoidance.rs` | new | guards, fix → is / be |
| `fixtures/rules/SLP006/` | new | positive, negative, fixed |
| `src/rules/builtin/mod.rs` | from story-02 | register SLP006 |
| `tests/rules_fixtures.rs` | from story-02 | macro gains 6; add safe_fix_contexts_grammatical |

## Acceptance Criteria

- [ ] "It serves as a bridge" becomes "It is a bridge", and "These files can serve as the templates" becomes "These files can be the templates".
- [ ] "They served as mentors", "The tool serves as a bridge", "export these functions as a module" (Gap 14), "It functions as follows" and "It serves as well as the old one" (Gap 16) fire with no fix and stay unchanged in `fixed.md`.
- [ ] With no next word token in the same block, no fix is attached (Gap 16).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::rules_fixtures` | `slp006_positive_fires / slp006_negative_silent / slp006_fix_matches` | SLP006 fixtures | FR-4.2, FR-4.6 |
| 2 | `sloperator::rules_fixtures` | `safe_fix_contexts_grammatical` | the exact SLP001/SLP006 context pairs from §10 | FR-4.7 (Gaps 2, 14, 16) |

## Notes

FixClass stays `Safe`. A Safe finding with `fix: None` selects a model window later (§16).

Gate: `make test TESTS='slp006_ safe_fix_contexts_grammatical'`
