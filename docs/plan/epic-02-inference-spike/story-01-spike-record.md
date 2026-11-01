# Story 01: Run the spike, write the record and the 20 eval paragraphs

**Epic:** [epic-02-inference-spike](../epic-02-inference-spike.md)
**SPEC Refs:** §1, §22 (paragraph corpus)
**PRD Refs:** FR-9.9, FR-10.1, D-6, NFR-3 (budget)
**Component / ADR:** architecture §12 Phase 1; ADR-02, ADR-04, ADR-08
**Wave:** 1

## Scope

Build the throwaway harness outside the repo. Measure `llama-cpp-2` =0.1.160 (CPU and Metal) and `mistralrs` =0.8.1 with Qwen3-4B-Q4_K_M and Qwen3-1.7B-Q8_0 on the 20 paragraphs. Read the licences. Re-check the TESTS propagation through the real Makefile from epic 01. Write the record and commit the 20 paragraphs to `eval/paragraphs/`. Add one unit test that keeps the corpus complete. No `src/` change.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `docs/research/spike-results.md` | new | the SPEC §1 form, filled in |
| `eval/paragraphs/p01.md … p20.md` | new | 20 slop paragraphs covering every model-class family, with code spans, links, numbers and a table row |
| `tests/eval_corpus.rs` | new | tests below |

## Acceptance Criteria

- [ ] Every row of the §1 crate × model table holds cold build time, binary size, GPU use, load time and tok/s (FR-9.9).
- [ ] The 20-paragraph table reports placeholders kept, numbers/URLs kept, output-contract result and acceptance per paragraph, plus placeholder survival before verify.
- [ ] The licence section quotes the licence text of the chosen model (PRD Question 4).
- [ ] The TESTS-propagation section shows the real Makefile run with one selector producing one testcase.
- [ ] The Decision section sets the crate, `DEFAULT_MODEL`, the masking mode and `MODEL_WINDOW_BUDGET_S` (seconds per window), or records FAIL and the escalation to Brad.
- [ ] `eval/paragraphs/` holds exactly p01.md–p20.md, none containing `⟦` or `⟧`.
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::eval_corpus` | `twenty_paragraphs_present_without_placeholder_glyphs` | p01–p20 exist, are non-empty, and contain no ⟦/⟧ | FR-10.1 |

## Notes

`eval_corpus.rs` is a new test file not named in SPEC §23; it guards the corpus that §22 reuses. If the spike picks `mistralrs`, record the exact `tokio` pin it needs: epic 07 changes the `model` feature line in Cargo.toml (SPEC §2 design note).

Gate: `make test TESTS='sloperator::eval_corpus' && grep -q '^## Decision' docs/research/spike-results.md`
