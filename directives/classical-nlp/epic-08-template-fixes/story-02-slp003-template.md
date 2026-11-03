# Story 02: SLP003 `it's Y` template fix with the additive-frame guard

**Epic:** [epic-08-template-fixes](../epic-08-template-fixes.md)
**SPEC Refs:** §30 (SLP003 template `it's Y`), §31 (macro arm for 3)
**PRD Refs:** FR-8.2, FR-8.3, FR-8.4, FR-8.5, FR-8.7, D-10
**Component / ADR:** Architecture §4.9 row SLP003
**Wave:** 2

## Scope

In src/rules/builtin/slp003_not_x_but_y.rs: the class becomes `FixClass::Safe`, add the param `max_words` (3, 1, 10), and add `ADDITIVE`, `denies`, `TEMPLATE`, `template: regex::Regex` and `template_fix` exactly as §30. `check` mirrors SLP004's loop, and `explain` and the example change. positive.md gains the two §30 lines, and fixed.md is new with §30's fixed lines. In tests/rules_fixtures.rs, move 3 to the safe arm (line 143) and add the SLP003 tests.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/rules/builtin/slp003_not_x_but_y.rs` | 1-2, 4-5, 13-16, 18-36, 43-53 | class, param, guards, template, explain |
| `fixtures/rules/SLP003/positive.md` | end | "It's not a bug that we can fix in an hour, it's a feature." and "It's not only cheap, it's quick." |
| `fixtures/rules/SLP003/fixed.md` | new | §30 fixed lines |
| `tests/rules_fixtures.rs` | 143, new tests | safe arm for 3; SLP003 tests |

## Acceptance Criteria

- [ ] "It's not a bug, it's a feature." → "It's a feature." (FR-8.2)
- [ ] "I think it's not a bug, it's a feature." → "I think it's a feature." (mid-sentence keeps case) (FR-8.2)
- [ ] "It's not only cheap, it's quick." and "It's not just cheap, it's quick." keep their finding with no fix and are unchanged by a rules-only fix (FR-8.3, D-10, Gap 3)
- [ ] X over `max_words`, a different second subject ("This isn't X — it's Y"), or the two clauses in different sentences give no fix (FR-8.2, FR-8.3)
- [ ] `[rules.SLP003]\nmax_words = 11` is rejected (NFR-4)
- [ ] Every template fix passes the verifier through the D-8 allowance (FR-8.5)
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::rules_fixtures` | `slp003_positive_fires / slp003_negative_silent / slp003_fix_matches` | safe-arm fixture tests | FR-8.2, FR-8.4, FR-8.7 |
| 2 | `sloperator::rules_fixtures` | `slp003_declines_on_long_x_or_other_subject` | length and subject guards | FR-8.2, FR-8.3 |
| 3 | `sloperator::rules_fixtures` | `slp003_mid_sentence_keeps_case` | mid-sentence deletion keeps case | FR-8.2 |
| 4 | `sloperator::rules_fixtures` | `slp003_declines_additive_frame` | Gap 3 additive frames stay fix-less; the denial still fixes | FR-8.2, FR-8.3, D-10 |
| 5 | `sloperator::rules_fixtures` | `fix_is_idempotent_over_all_fixtures` | idempotent with both templates | FR-8.7 |
| 6 | `sloperator::rules_fixtures` | `no_safe_fixture_fix_newly_rejected` | SLP003 template fixes pass `verify::edit` | FR-8.5, FR-6.6 |

## Notes

Sentence-initial matches capitalise through `delete_lead_in` (src/rules/helpers.rs:165). `ADDITIVE` is sorted (binary search).

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='slp003_ fix_is_idempotent_over_all_fixtures no_safe_fixture_fix_newly_rejected'`
