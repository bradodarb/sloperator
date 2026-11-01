# Epic 02: Inference spike and results record (Phase 1 gate)

**Goal:** `docs/research/spike-results.md` records the measured runtime crate, default model, masking mode and model latency budget, with a pass/fail verdict that epic 07 needs.
**PRD Refs:** FR-9.9, D-1, D-6, PRD Phase 1 tasks 1-4, Brad Q4 (a)
**SPEC Refs:** §1 (Inference spike and results record)
**Status:** Delivered. Story 01 merged: `docs/research/spike-results.md` (verdict PASS: `llama-cpp-2` =0.1.160, `DEFAULT_MODEL` `qwen3-4b`, masking ADR-04 Option 1, `MODEL_WINDOW_BUDGET_S = 3.0`), the 20 paragraphs `eval/paragraphs/p01.md … p20.md` and `tests/eval_corpus.rs`. Epic 07's spike dependency is met.

**Modules:** `sloperator` (eval corpus only); `docs/research/`

## Waves

Copied from this epic's line in `index.md`. Stories in one wave run in parallel; each wave waits for the one before.

| Wave | Stories | Why this order |
|------|---------|----------------|
| 1 | [story-01-spike-record](./epic-02-inference-spike/story-01-spike-record.md) | One investigation, one record; runs in parallel with epics 03-06. |

## Acceptance Criteria

- [ ] The record has every section of the SPEC §1 form, with numbers in every FR-9.9 field for the chosen crate and model.
- [ ] The Decision section names the crate, the default model, the masking mode (ADR-04 Option 1 or 2) and `MODEL_WINDOW_BUDGET_S`.
- [ ] The verdict is PASS (protected-span integrity 20/20 after verify), or FAIL with the result taken to Brad (D-1 fallback reopens).
- [ ] The tests named in every story's Test Cases pass together on the merged epic.

## Notes

The spike harness lives in `$TMPDIR/sloperator-spike/` and is not committed (architecture §10). Brad's Q4: Qwen3-4B Q4_K_M is the default unless the spike or the licence reading rules it out; Phi-4-mini (MIT) is the fallback. Llama 3.2 is excluded.
