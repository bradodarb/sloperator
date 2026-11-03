# Story 04: Verification-limits sentence in REFERENCE, SPEC, the module header and the score test

**Epic:** [epic-06-verifier-invariants](../epic-06-verifier-invariants.md)
**SPEC Refs:** §25 (Verification-limits sentence in three places), §18 (CHANGELOG)
**PRD Refs:** FR-6.7, NFR-9
**Component / ADR:** Architecture §4.10 Verifier invariants
**Wave:** 3

## Scope

Replace the old sentence ("… not names or claim polarity.") with §25's text, in the same change, at docs/REFERENCE.md:443 (and the 'Edit gate' section near :128), in the tests/score.rs:107-111 assertion, in the src/verify/mod.rs:4-5 module header (its own shorter wording) and at docs/SPEC.md:238. CHANGELOG.md's 0.2.0 section adds the three verifier invariants and the D-8 allowance.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/verify/mod.rs` | 4-5 | module header |
| `docs/REFERENCE.md` | 128, 443 | new sentence |
| `tests/score.rs` | 107-111 | assertion updated |
| `docs/SPEC.md` | 238 | new sentence |
| `CHANGELOG.md` | 0.2.0 section | verifier invariants |
| `tests/standalone.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] `grep -rn 'not names or claim polarity' src tests docs` (excluding docs/classical-nlp/, which quotes it on purpose) returns nothing (NFR-9)
- [ ] REFERENCE states the new coverage and that a reworded claim with the same negations, modals and capitalised words passes (FR-6.7)
- [ ] `reference_states_limits` passes with the new text
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::score` | `reference_states_limits` | REFERENCE carries the new limits sentence | FR-6.7 |
| 2 | `sloperator::standalone` | `no_host_references` | doc edits use the repo's own form | NFR-1 |

## Notes

docs/classical-nlp/ quotes the old sentence on purpose (SPEC §25, PRD). Leave those initiative documents alone: the grep criterion applies to product docs, src/ and tests/.

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='reference_states_limits sloperator::standalone#no_host_references'`
