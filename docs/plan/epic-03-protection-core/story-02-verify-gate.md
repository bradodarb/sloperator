# Story 02: `verify`: token, structure and improvement checks

**Epic:** [epic-03-protection-core](../epic-03-protection-core.md)
**SPEC Refs:** §8, §23.5, DQ-3
**PRD Refs:** FR-8.5, FR-8.6, FR-8.1, FR-6.6, FR-9.5, D-4, D-7
**Component / ADR:** §4.9 verify; ADR-05
**Wave:** 2

## Scope

Implement `src/verify/{mod,tokens}.rs`: `Verified<T>` with a private constructor, `Tokens`, `Check`, `Rejection`, `tokens`, `edit`, `improvement` and `document`, as §8 specifies. No caller is wired yet; the pipeline uses these in epic 05.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/verify/mod.rs` | new | Verified, Check, Rejection, edit, improvement, document |
| `src/verify/tokens.rs` | new | number/URL/term multisets |
| `tests/verify_edits.rs` | new | tests below |

## Acceptance Criteria

- [ ] An edit that loses, adds or changes a number, percentage, digit date, URL or configured term is rejected with the matching `Check` (FR-8.5).
- [ ] An edit that introduces a heading marker, list marker or fence is rejected (FR-8.6).
- [ ] An edit overlapping a Protected span or a frozen range is rejected (FR-8.1, FR-6.6).
- [ ] Deferred item 3 (DQ-3): `improvement(before, after, ..)` returns `Err` when `after == before` and when `after > before`, and `Ok` only when `after < before` (the equal-score fixture is in the test) (FR-9.5).
- [ ] `Verified<T>` cannot be built outside `verify` (no public constructor; ADR-05).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::verify_edits` | `rejects_lost_number` | number loss rejected | FR-8.5 |
| 2 | `sloperator::verify_edits` | `rejects_added_number` | number addition rejected | FR-8.5 |
| 3 | `sloperator::verify_edits` | `rejects_changed_percentage` | percentage change rejected | FR-8.5 |
| 4 | `sloperator::verify_edits` | `rejects_changed_digit_date` | date change rejected | FR-8.5 |
| 5 | `sloperator::verify_edits` | `rejects_changed_url` | URL change rejected | FR-8.5 |
| 6 | `sloperator::verify_edits` | `rejects_lost_term` | term loss rejected | FR-8.5 |
| 7 | `sloperator::verify_edits` | `rejects_new_heading_marker` | new heading rejected | FR-8.6 |
| 8 | `sloperator::verify_edits` | `rejects_list_marker` | new list marker rejected | FR-8.6 |
| 9 | `sloperator::verify_edits` | `rejects_fence` | new fence rejected | FR-8.6 |
| 10 | `sloperator::verify_edits` | `rejects_edit_into_protected_or_frozen` | protected/frozen edits rejected | FR-8.1, FR-6.6 |
| 11 | `sloperator::verify_edits` | `accepts_reordered_placeholders_restored` | multiset comparison tolerates reorder | ADR-04 |
| 12 | `sloperator::verify_edits` | `improvement_requires_strictly_fewer` | equal or higher weighted count is rejected | FR-9.5, DQ-3 |
| 13 | `sloperator::verify_edits` | `document_check_fires_on_forged_edit_pair` | whole-document skeleton check | architecture §7.2 |

## Notes

Verification covers numbers, URLs, configured terms, placeholders, block structure, negations, modal verbs and capitalised words as a proxy for names. It does not detect every change of meaning: a reworded claim with the same negations, modals and capitalised words passes (PRD Risk 1, architecture R8; coverage updated by classical-nlp). R6 is accepted: a reparse difference rejects a fix, and that is reported, never written.

Gate: `make test TESTS='sloperator::verify_edits'`
