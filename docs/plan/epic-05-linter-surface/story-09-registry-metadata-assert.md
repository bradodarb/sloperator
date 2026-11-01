# Story 09: `rules_registry` checks each rule's metadata against the SPEC §10 catalogue

**Epic:** [epic-05-linter-surface](../epic-05-linter-surface.md)
**SPEC Refs:** §10 (built-in rule catalogue), §9 (registry)
**PRD Refs:** FR-4.1, FR-4.5, FR-4.7
**Component / ADR:** §4.6 rules; ADR-06
**Wave:** 6

## Scope

Fix the no-op assertion that the epic-04 acceptance audit deferred (G2, `docs/backlog/epic-04-registry-noop-assert.md`). In `tests/rules_registry.rs`, `every_rule_has_code_name_severity_state_fix` ends with `let _ = (m.severity, m.default_on, m.fix);` (line 66), which asserts nothing at runtime. Replace it with real checks against the SPEC §10 table. This is a test-only change: no file under `src/` changes, unless a rule's metadata turns out to disagree with the SPEC. In that case, fix the rule's `RuleMeta` and say so in the story's PR.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `tests/rules_registry.rs` | 65-66 | replace the no-op `let _ = …` with a literal table of (code, severity, fix class) from SPEC §10, asserted per rule; assert `m.default_on == Registry::builtin().default_set().contains(&m.code)` |

## Acceptance Criteria

- [ ] For each of SLP001–SLP016, the test asserts `m.severity` and `m.fix` against a literal table copied from SPEC §10. For example, SLP001 is warning/safe, SLP002 suggestion/model, SLP010 suggestion/none, SLP012 error/safe and SLP015 error/model (FR-4.1, FR-4.7).
- [ ] The literal table covers every registered code: a code in the registry that is missing from the table, or the other way round, fails the test.
- [ ] For each rule, `m.default_on` equals whether `Registry::builtin().default_set()` contains `m.code` (FR-4.5).
- [ ] The line `let _ = (m.severity, m.default_on, m.fix);` is gone from the file.
- [ ] Changing any one rule's severity or fix class in `src/rules/builtin/` makes the test fail. Check this once by hand and revert, or let the discriminate step show it.
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::rules_registry` | `every_rule_has_code_name_severity_state_fix` | severity, fix class and default state match the catalogue | FR-4.1, FR-4.5, FR-4.7 |

## Notes

Source: epic-04 acceptance audit G2 (CH-9 in `changes.md`). This story runs in parallel with story-08: it touches only `tests/rules_registry.rs`, and nothing else in this epic edits that file. `Severity` and `FixClass` derive `PartialEq` and `Debug` (`src/rules/mod.rs:28-45`), so `assert_eq!` works on them directly.

Gate: `make test TESTS='sloperator::rules_registry#every_rule_has_code_name_severity_state_fix'`
