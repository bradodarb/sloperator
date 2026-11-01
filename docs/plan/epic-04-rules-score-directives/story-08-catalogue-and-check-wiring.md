# Story 08: Registry-wide tests and wiring rules into `check`

**Epic:** [epic-04-rules-score-directives](../epic-04-rules-score-directives.md)
**SPEC Refs:** §9, §10 (fixture contract), §20 (skeleton exit codes), §23.6
**PRD Refs:** FR-4.1, FR-4.2, FR-4.5, FR-4.6, FR-4.7, FR-8.9, FR-2.7, D-8
**Component / ADR:** §4.6 rules, §4.1 cli
**Wave:** 6

## Scope

Complete `tests/rules_registry.rs` and add `every_rule_has_fixtures`, `only_safe_rules_attach_fixes` and `fix_is_idempotent_over_all_fixtures` to `tests/rules_fixtures.rs`. Wire `doc::parse` → `directives::extract` → `rules::detect` with the default set into the skeleton `check` in `cli/mod.rs`. It prints one `path:line:col: CODE [severity] message` line per finding and exits 1 when any finding is a warning or worse, 0 otherwise, and 2 on a directive error.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `tests/rules_registry.rs` | from story-01 | registry tests |
| `tests/rules_fixtures.rs` | from story-07 | registry-wide fixture tests |
| `src/cli/mod.rs` | from epic-03 story-01 | skeleton check runs detect and sets the exit code |
| `tests/cli_exit.rs` | from epic-01 story-01 | exit_one_on_warning, exit_zero_on_suggestion_only |

## Acceptance Criteria

- [ ] The registry has 16 rules covering all nine families, each with code, name, severity, state and fix class (FR-4.1, FR-4.2).
- [ ] The default set is an explicit list, and a rule added later starts off (FR-4.5).
- [ ] A rule without a fixture directory, a `positive.md` or `negative.md`, a safe rule without `fixed.md`, or a non-safe rule with one fails `every_rule_has_fixtures` (FR-4.6).
- [ ] Applying the safe fixes twice changes nothing the second time, on every fixture (FR-8.9).
- [ ] `sloperator check` exits 1 on a warning finding and 0 when only suggestions are found (FR-2.7, D-8).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::rules_registry` | `at_least_15_rules_covering_nine_families` | catalogue size and families | FR-4.2 |
| 2 | `sloperator::rules_registry` | `every_rule_has_code_name_severity_state_fix` | metadata | FR-4.1 |
| 3 | `sloperator::rules_registry` | `default_set_is_explicit` | explicit default set | FR-4.5 |
| 4 | `sloperator::rules_registry` | `new_rules_start_off` | new rules are off | FR-4.5 |
| 5 | `sloperator::rules_fixtures` | `every_rule_has_fixtures` | fixture contract | FR-4.6 |
| 6 | `sloperator::rules_fixtures` | `only_safe_rules_attach_fixes` | fix class honoured | FR-4.7 |
| 7 | `sloperator::rules_fixtures` | `fix_is_idempotent_over_all_fixtures` | second fix pass is a no-op | FR-8.9 |
| 8 | `sloperator::cli_exit` | `exit_one_on_warning` | warning → exit 1 | FR-2.7, D-8 |
| 9 | `sloperator::cli_exit` | `exit_zero_on_suggestion_only` | suggestion only → exit 0 | FR-2.7, D-8 |

## Notes

The idempotence test applies fixes through `verify::edit` and `edit::apply`, the path that epic 05's pipeline uses.

Gate: `make test TESTS='sloperator::rules_registry#at_least_15_rules_covering_nine_families sloperator::rules_registry#every_rule_has_code_name_severity_state_fix sloperator::rules_registry#default_set_is_explicit sloperator::rules_registry#new_rules_start_off sloperator::rules_fixtures#every_rule_has_fixtures sloperator::rules_fixtures#only_safe_rules_attach_fixes sloperator::rules_fixtures#fix_is_idempotent_over_all_fixtures sloperator::cli_exit#exit_one_on_warning sloperator::cli_exit#exit_zero_on_suggestion_only'`
