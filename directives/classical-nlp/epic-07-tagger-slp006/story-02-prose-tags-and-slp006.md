# Story 02: `Prose::tags` and the SLP006 tag-gated subject branch

**Epic:** [epic-07-tagger-slp006](../epic-07-tagger-slp006.md)
**SPEC Refs:** §27 (`Prose::tags`, single-tagging in-crate tests), §28 (SLP006 tag-gated subjects)
**PRD Refs:** FR-7.3, FR-7.4, FR-7.5, FR-7.6, FR-7.7, FR-7.8, NFR-2
**Component / ADR:** Architecture §4.8 Per-document context, §4.9 row SLP006; ADR-02
**Wave:** 2

## Scope

`Prose` (src/text/mod.rs) gains `tags: OnceLock<Vec<Tagged>>`, a `#[cfg(test)] tag_runs: AtomicU32`, and `tags(&self, doc)`, which tags on first call only. In src/rules/builtin/slp006_copula_avoidance.rs, add `SINGULAR_DET`, `AS_IDIOMS`, `tag_certain_subject` and `complement_ok` (split out of `safe_replacement`), and the `or_else` branch in `check` exactly as §28. The existing branches are unchanged. Update `explain` and the module comment. Extend positive.md and fixed.md by one paragraph each. Add the `safe_fix_contexts_grammatical` cases, and the in-crate `rules::tests` module (src/rules/mod.rs end) with `tagging_runs_once_per_pass` and `no_candidate_no_tagging`.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/text/mod.rs` | `Prose` | `tags` field, `tag_runs` counter, `tags()` |
| `src/rules/builtin/slp006_copula_avoidance.rs` | 1-13, 49-67, 111-127, 134-148 | tag-gated branch, guards, explain |
| `src/rules/mod.rs` | end | `#[cfg(test)] mod tests` with the two tagging tests |
| `fixtures/rules/SLP006/positive.md` | end | "The library serves as a guide. The main library serves as a guide." |
| `fixtures/rules/SLP006/fixed.md` | end | "The library is a guide. The main library is a guide." |
| `tests/rules_fixtures.rs` | 303-356 | two `fixed` and four `unchanged` cases |

## Acceptance Criteria

- [ ] The existing lines of SLP006's positive.md and fixed.md are byte-for-byte unchanged; each gains one paragraph (FR-7.7, NFR-1)
- [ ] "The library serves as a guide." → "The library is a guide." and "The main library serves as a guide." → "The main library is a guide." (FR-7.5)
- [ ] No fix is attached for "Export the library functions as a module.", "The library functions as a module are deprecated.", "The library serves as a whole.", "The fast library serves as a guide." and "The tool serves as a bridge." (FR-7.6, FR-7.7, FR-7.8)
- [ ] With SLP004 and SLP006 enabled on "The library serves as a guide. It is not only cheap but also quick.", the tagger runs once (FR-7.3, NFR-2)
- [ ] "Plain prose with nothing to fix." is never tagged (FR-7.4)
- [ ] SLP006's class, default and severity are unchanged
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::rules::tests` | `tagging_runs_once_per_pass` | one tagger run for two consumers | FR-7.3, NFR-2 |
| 2 | `sloperator::rules::tests` | `no_candidate_no_tagging` | no candidate, zero runs (retained per Gap 9) | FR-7.4 |
| 3 | `sloperator::rules_fixtures` | `slp006_fix_matches / slp006_positive_fires / slp006_negative_silent` | fixtures extended | FR-7.7 |
| 4 | `sloperator::rules_fixtures` | `safe_fix_contexts_grammatical` | class A/B fixes and declines | FR-7.5, FR-7.6, FR-7.8 |
| 5 | `sloperator::rules_fixtures` | `no_safe_fixture_fix_newly_rejected` | the new SLP006 fixes pass the verifier | FR-6.6 |

## Notes

`tagging_runs_once_per_pass` enables SLP004, which reads no tags until epic-08. With SLP006 reading them it already expects exactly 1, and epic-08 keeps it at 1. Tags are read only after every cheaper guard passes (FR-7.4). `AS_IDIOMS` applies to the new branch only (DQ-11).

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='rules::tests::tagging_runs_once_per_pass rules::tests::no_candidate_no_tagging slp006_ safe_fix_contexts_grammatical no_safe_fixture_fix_newly_rejected'`
