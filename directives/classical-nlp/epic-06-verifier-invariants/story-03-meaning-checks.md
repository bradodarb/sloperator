# Story 03: `verify::meaning`, new `Check` variants and `verify::edit(…, resolves)`

**Epic:** [epic-06-verifier-invariants](../epic-06-verifier-invariants.md)
**SPEC Refs:** §23 (`verify::meaning`, new `Check` variants, `verify::edit(resolves)`), §24 (pipeline call sites, check names), §36.6, §36.7
**PRD Refs:** FR-6.1, FR-6.2, FR-6.3, FR-6.4, FR-6.5, FR-6.6, FR-6.8, FR-8.5, NFR-6, D-8
**Component / ADR:** Architecture §4.10 Verifier invariants; ADR-06
**Wave:** 2

## Scope

Create src/verify/meaning.rs exactly as §23 (`Meaning`, `NEGATION_EXEMPT`, `meaning(text, initial_at_start, skip_common)`, `negation_allowance`). In src/verify/mod.rs, add `Check::Negation`, `ProperNouns` and `Modals`, give `edit` a `resolves: &[RuleCode]` parameter, run the three checks after the term check and before the reparse (`skip_common` true only for `ProposalOrigin::Rule`), and add `starts_at_sentence`. Update every call site: pipeline :118 (`&[]`) and :273 (`&w.codes`); tests/verify_edits.rs:29, 231, 246; tests/rules_fixtures.rs:73; tests/edit_apply.rs:52, 206 (`&[]`). `check_name` (src/pipeline/mod.rs:331) gains `negation`, `properNouns` and `modals`. tests/eval.rs:46-57 `CHECKS` becomes `[Check; 13]`. Add the acceptance/rejection tests and the FR-6.6 gate test.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/verify/meaning.rs` | new | meaning invariants |
| `src/verify/mod.rs` | 1-15, 35-51, 60-105 | `mod meaning;`, `Check` variants, `edit(…, resolves)`, checks, `starts_at_sentence` |
| `src/pipeline/mod.rs` | 118, 273, 331-344 | `resolves` passed; check names |
| `tests/rules_fixtures.rs` | 73, new test | `&[]` at the call; `no_safe_fixture_fix_newly_rejected` |
| `tests/edit_apply.rs` | 52, 206 | `&[]` |
| `tests/eval.rs` | 46-57 | `CHECKS` lists the three new variants |
| `tests/verify_edits.rs` | existing | tests below (methods added) |
| `tests/pipeline.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] Model "It is not ready." → "It is ready." fails `Check::Negation`; "It isn't ready." → "It is not ready." passes (FR-6.1, FR-6.8)
- [ ] `Rule(RuleCode(3))` may drop one `not` but not two; a model edit with `resolves = [SLP003]` may drop one, and with `resolves = []` none (D-8)
- [ ] "We use Redis here." → "We use it here." fails `Check::ProperNouns`; SLP008-style "It is worth noting that t" → "T" passes; rule-origin SLP009 "May Potentially" → "May" passes (FR-6.2)
- [ ] A model edit "We watched It and a robust sequel." → "We watched something and a sequel." fails `Check::ProperNouns` (FR-6.2, Gap 11 (a))
- [ ] "It must ship." → "It should ship." fails `Check::Modals`; "It will not ship." → "It won't ship." passes (FR-6.3)
- [ ] A forged SLP001 fix that drops "not" yields SLP901 `rewrite rejected (negation)`, and the bytes stay (FR-6.4, FR-6.5, NFR-6)
- [ ] For every Safe rule and every fixture, every planned fix passes `verify::edit(…, &[])` (FR-6.6 gate; fail = any)
- [ ] `check_name` is exhaustive, so a missing arm fails the build
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::verify_edits` | `rejects_dropped_negation` | Check::Negation on a model drop | FR-6.1, FR-6.8 |
| 2 | `sloperator::verify_edits` | `accepts_kept_negation` | isn't → is not passes | FR-6.8 |
| 3 | `sloperator::verify_edits` | `slp003_origin_may_drop_one_negation` | rule-origin allowance 1 | D-8 |
| 4 | `sloperator::verify_edits` | `model_allowance_counts_window_findings` | model allowance from `resolves` | D-8 |
| 5 | `sloperator::verify_edits` | `rejects_removed_proper_noun` | Redis → it rejected | FR-6.2, FR-6.8 |
| 6 | `sloperator::verify_edits` | `accepts_sentence_initial_capital_change` | lead-in deletion passes | FR-6.2 |
| 7 | `sloperator::verify_edits` | `accepts_title_case_hedge_removal` | rule-origin May Potentially → May passes | FR-6.2 |
| 8 | `sloperator::verify_edits` | `rejects_model_removal_of_capitalised_common_word` | model drop of title "It" rejected | FR-6.2, Gap 11 |
| 9 | `sloperator::verify_edits` | `rejects_changed_modal` | must → should rejected | FR-6.3, FR-6.8 |
| 10 | `sloperator::verify_edits` | `accepts_modal_contraction` | will not → won't passes | FR-6.3 |
| 11 | `sloperator::pipeline` | `rejected_new_check_is_slp901_with_name` | SLP901 message names the check; bytes kept | FR-6.4, FR-6.5, NFR-6 |
| 12 | `sloperator::rules_fixtures` | `no_safe_fixture_fix_newly_rejected` | FR-6.6 gate over every Safe fixture | FR-6.6 |
| 13 | `sloperator::edit_apply` | `(all existing)` | call sites updated | NFR-1 |
| 14 | `sloperator::eval` | `(all existing)` | 13 checks listed | FR-6.4 |

## Notes

`apply_fixes` in tests/rules_fixtures.rs unwraps every `verify::edit`, so every existing fixture test also exercises the new checks. If the FR-6.6 gate fails on an epic-05 row, fix the row (no row may change a modal or negation, DQ-17); never weaken the check. Model-path tests use `common::FakeFactory` (tests/common/mod.rs).

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='sloperator::verify_edits sloperator::pipeline sloperator::rules_fixtures sloperator::edit_apply sloperator::eval'`
