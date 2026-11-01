# Story 04: Neutral protected term in `rejects_lost_term` (fix: standalone scan)

**Epic:** [epic-03-protection-core](../epic-03-protection-core.md)
**SPEC Refs:** §5, §8
**PRD Refs:** FR-8.5
**Component / ADR:** §4.9 verify
**Wave:** 4

## Scope

Acceptance audit gap G1. `tests/verify_edits.rs:120-140` (`rejects_lost_term`) uses the host project's name as its configured protected term, so `standalone#no_host_references` (`tests/standalone.rs:72`) finds a host reference in the crate and fails, breaking SPEC §5. Replace every occurrence of that term in the test (lines 120, 126, 132, 133, 139, 140) with a neutral term, `Acme`, and keep every assertion as it is. Test-only change; no `src/` file changes.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `tests/verify_edits.rs` | 120-140 | Swap the protected term for `Acme` in `ProtectSettings::new`, the three edit strings and the `tokens` call/assertion. |

## Acceptance Criteria

- [ ] `rejects_lost_term` configures `Acme` as its term and still asserts: adding the term is rejected with `Check::Terms`; removing it is rejected with `Check::Protected` or `Check::Terms`; `tokens` counts 2 occurrences (FR-8.5).
- [ ] A case-insensitive search for the host name finds nothing under `src`, `tests` or `fixtures`, so `standalone#no_host_references` passes (SPEC §5).
- [ ] The rest of `sloperator::verify_edits` still passes (no assertion removed or loosened).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::verify_edits` | `rejects_lost_term` | term add/remove rejected with neutral term | FR-8.5 |
| 2 | `sloperator::standalone` | `no_host_references` | crate holds no host reference | SPEC §5 |
| 3 | `sloperator::verify_edits` | (whole binary) | no regression in other verify tests | FR-8.5, FR-8.6 |

## Notes

The marker list in `tests/standalone.rs` is built with `concat!` so it does not match itself; do not edit that file or add exemptions. Do not write the host name anywhere in the fix, including comments.

Gate: `make test TESTS='sloperator::verify_edits sloperator::standalone#no_host_references'`
