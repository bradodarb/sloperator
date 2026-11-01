# Story 06: Rules SLP003, SLP004, SLP005, SLP007, SLP012, SLP013, SLP015

**Epic:** [epic-04-rules-score-directives](../epic-04-rules-score-directives.md)
**SPEC Refs:** §10
**PRD Refs:** FR-4.2, FR-4.6, FR-4.7
**Component / ADR:** §4.6 rules; ADR-06
**Wave:** 4

## Scope

Add the pattern rules (negative parallelism, rule of three, puffery) and the chatbot residue rules (closer, opener, AI disclaimer) with their fixtures. Register them and extend the macro call. Only SLP012 and SLP013 are safe and have a `fixed.md`.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/rules/builtin/slp003_not_x_but_y.rs` | new | PatternRule |
| `src/rules/builtin/slp004_not_just_but_also.rs` | new | PatternRule |
| `src/rules/builtin/slp005_rule_of_three.rs` | new | triad density param `max_per_1000_words` |
| `src/rules/builtin/slp007_significance_puffery.rs` | new | PatternRule |
| `src/rules/builtin/slp012_chatbot_closer.rs` | new | safe: delete the sentence |
| `src/rules/builtin/slp013_chatbot_opener.rs` | new | safe: delete and capitalise |
| `src/rules/builtin/slp015_ai_disclaimer.rs` | new | model class |
| `fixtures/rules/SLP003/ … SLP015/` | new | fixtures for the seven rules |
| `src/rules/builtin/mod.rs` | from story-05 | register seven rules |
| `tests/rules_fixtures.rs` | from story-05 | macro gains 3, 4, 5, 7, 12, 13, 15 |

## Acceptance Criteria

- [ ] Each of the seven rules fires on `positive.md` and is silent on `negative.md` (FR-4.6).
- [ ] SLP012/SLP013 fixes match `fixed.md`; the others have no `fixed.md` (FR-4.7).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::rules_fixtures` | `slp003_* ` | SLP003 fixtures | FR-4.2 |
| 2 | `sloperator::rules_fixtures` | `slp004_*` | SLP004 fixtures | FR-4.2 |
| 3 | `sloperator::rules_fixtures` | `slp005_*` | SLP005 fixtures | FR-4.2 |
| 4 | `sloperator::rules_fixtures` | `slp007_*` | SLP007 fixtures | FR-4.2 |
| 5 | `sloperator::rules_fixtures` | `slp012_* (incl. fix_matches)` | SLP012 fixtures | FR-4.2, FR-4.7 |
| 6 | `sloperator::rules_fixtures` | `slp013_* (incl. fix_matches)` | SLP013 fixtures | FR-4.2, FR-4.7 |
| 7 | `sloperator::rules_fixtures` | `slp015_*` | SLP015 fixtures | FR-4.2 |

## Notes

Each rule is one small file with a regex set, so seven rules fit one session.

Gate: `make test TESTS='slp003_ slp004_ slp005_ slp007_ slp012_ slp013_ slp015_'`
