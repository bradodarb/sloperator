# Story 02: SLP022 `weak-intensifier` (detection only)

**Epic:** [epic-05-wordiness](../epic-05-wordiness.md)
**SPEC Refs:** §21 (SLP022 `weak-intensifier`)
**PRD Refs:** FR-5.4, FR-11.2, D-10
**Component / ADR:** Architecture §4.9 Rule integration, row SLP022; ADR-08 (a)
**Wave:** 2

## Scope

Add src/rules/builtin/slp022_weak_intensifier.rs: a `WordList` of the twelve §21 intensifiers, every entry `None`. `Family::Wordiness`, `Severity::Suggestion`, `default_on: false`, `FixClass::None`, `since: "0.2.0"`, no params. The message is `` `{word}` adds emphasis, not meaning; consider deleting it ``, and `explain` says it is detection only because the verifier does not check strength. Add the builtin/mod.rs entry, fixtures and a `model(22, …)` macro entry.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/rules/builtin/slp022_weak_intensifier.rs` | new | SLP022 |
| `src/rules/builtin/mod.rs` | module and `rule()` lists | SLP022 entry |
| `fixtures/rules/SLP022/positive.md` | new | intensifiers in prose |
| `fixtures/rules/SLP022/negative.md` | new | plain prose plus a fenced block, inline code and a link (an intensifier inside code) |
| `tests/rules_fixtures.rs` | 135-198 | `model(22, slp022_positive_fires, slp022_negative_silent)` |
| `tests/rules_registry.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] SLP022 fires on positive.md and is silent on negative.md (FR-5.4)
- [ ] SLP022 attaches no fix (`only_safe_rules_attach_fixes` passes) (FR-5.4, D-10)
- [ ] SLP022 is off by default (`new_rules_start_off`, `default_set_is_explicit` pass) (FR-11.2)
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::rules_fixtures` | `slp022_positive_fires / slp022_negative_silent` | SLP022 fixtures | FR-5.4 |
| 2 | `sloperator::rules_fixtures` | `only_safe_rules_attach_fixes` | SLP022 never carries a fix | FR-5.4, D-10 |
| 3 | `sloperator::rules_registry` | `(all existing)` | default-off, fixtures present | FR-11.2 |

## Notes

`check` mirrors SLP001's loop (slp001…rs:62-79) without the guard.

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='slp022_ only_safe_rules_attach_fixes sloperator::rules_registry'`
