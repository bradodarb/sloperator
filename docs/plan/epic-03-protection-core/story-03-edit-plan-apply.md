# Story 03: `edit`: plan, apply, unified diff

**Epic:** [epic-03-protection-core](../epic-03-protection-core.md)
**SPEC Refs:** §7, §23.4
**PRD Refs:** FR-8.4, FR-6.6, FR-2.5, NFR-5 (content side)
**Component / ADR:** §4.8 edit; ADR-03, ADR-05
**Wave:** 3

## Scope

Implement `src/edit/mod.rs`: `Plan` and `plan`, which drops overlaps deterministically and drops edits into frozen ranges; `apply`, which takes only `&[Verified<Proposal>]` and keeps CRLF and the trailing newline; and `unified_diff`. Re-export `Proposal` and `ProposalOrigin` from `types`.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/edit/mod.rs` | new | plan, apply, unified_diff |
| `tests/edit_apply.rs` | new | tests below |

## Acceptance Criteria

- [ ] A no-op plan round-trips every `fixtures/protect/*` file byte-identical (FR-8.4).
- [ ] Replacements in a CRLF file use CRLF, and a missing or present trailing newline is preserved (FR-8.4).
- [ ] Overlapping proposals are resolved the same way on every run (§7), and proposals into frozen ranges are dropped (FR-6.6).
- [ ] `unified_diff` output has `---`/`+++` headers and `@@` hunks (FR-2.5).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::edit_apply` | `noop_plan_round_trips_every_protect_fixture` | byte-identical round trip | FR-8.4, ADR-03 |
| 2 | `sloperator::edit_apply` | `crlf_preserved_in_replacements` | CRLF kept | FR-8.4 |
| 3 | `sloperator::edit_apply` | `trailing_newline_preserved` | trailing newline kept | FR-8.4 |
| 4 | `sloperator::edit_apply` | `plan_drops_overlaps_deterministically` | overlap resolution | §7 |
| 5 | `sloperator::edit_apply` | `plan_drops_frozen` | frozen ranges untouched | FR-6.6 |
| 6 | `sloperator::edit_apply` | `unified_diff_has_headers_and_hunks` | diff format | FR-2.5 |

## Notes

`edit` must not use `crate::rules` (layering); it gets `RuleCode` through `types`.

Gate: `make test TESTS='sloperator::edit_apply'`
