# Story 01: Rule trait, registry, helpers and `detect`

**Epic:** [epic-04-rules-score-directives](../epic-04-rules-score-directives.md)
**SPEC Refs:** §9
**PRD Refs:** FR-4.1, FR-4.4, FR-4.5, FR-4.7, D-4
**Component / ADR:** §4.6 rules; ADR-06
**Wave:** 1

## Scope

Implement `src/rules/{mod,registry,helpers}.rs`: `Severity`, `FixClass`, `Family`, `ParamSpec`, `RuleMeta`, `RuleCtx`, `Finding`, the `Rule` trait, `Enabled`, `detect`, `Registry` (with `builtin()`, `get` and `default_set`), `WordList` and `PatternRule`. Add `src/rules/builtin/mod.rs` with an empty `all()` list. `detect` takes a `Suppressions`, so add the `Suppressions` struct to `src/directives/mod.rs` with `Default` and the two query methods. `extract` comes in story 03. Re-export `RuleCode` from `types`.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/rules/mod.rs` | new | types, Rule, Finding, Enabled, detect |
| `src/rules/registry.rs` | new | Registry |
| `src/rules/helpers.rs` | new | WordList, PatternRule |
| `src/rules/builtin/mod.rs` | new | `all()`, empty for now |
| `src/directives/mod.rs` | new | `Suppressions` struct, `is_suppressed`, `frozen_ranges` only |
| `tests/rules_registry.rs` | new | tests below |
| `tests/rules_core.rs` | new | tests below |

## Acceptance Criteria

- [ ] `detect` never reports a finding whose range overlaps a Protected span (FR-4.4).
- [ ] `detect` drops findings on suppressed lines and never suppresses a system code (§12).
- [ ] `WordList::find` returns the fix `None` for entries without a replacement (Gap 2).
- [ ] SLP901 and SLP902 are never in `Registry::builtin()` (§9).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::rules_registry` | `system_codes_not_in_registry` | SLP9xx is not a registry rule | §9 |
| 2 | `sloperator::rules_core` | `detect_skips_protected_and_suppressed` | a test-only rule matching inside code and on a suppressed line reports nothing there | FR-4.4 |
| 3 | `sloperator::rules_core` | `wordlist_none_entry_has_no_fix` | WordList entries without replacement yield no fix | FR-4.7, Gap 2 |

## Notes

`tests/rules_core.rs` is not in SPEC §23. It is added here so the core has a gate before any built-in rule exists. `rules` must not use `crate::edit` (layering): use `types::Proposal`.

Gate: `make test TESTS='sloperator::rules_registry#system_codes_not_in_registry sloperator::rules_core'`
