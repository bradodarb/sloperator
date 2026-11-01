# Acceptance — epic-02-inference-spike (sloperator)

Auditor: reviewer. Diff: `git diff f6b6076dcc71cea5abad489afa9bed00ddfdb2f6` (26 files: spike record, 20 paragraphs, one test, directive/context docs). No `src/` change, as the story requires.

**Verdict: met.** No blocking gap. One non-blocking gap is deferred.

## Epic acceptance criteria

| Criterion | Evidence | Result |
|---|---|---|
| Record has every SPEC §1 section; numbers in every FR-9.9 field for the chosen crate+model | `docs/research/spike-results.md`: Machine/Toolchain line 3; crate × model table lines 12-22 (cold build, binary, GPU, load, tok/s for llama-cpp-2 4B on CPU and Metal); 20-paragraph table lines 37-60; integrity/survival/grammar lines 79-81; Licence l.85; TESTS propagation l.115; Decision l.131. Matches the SPEC §1 form (SPEC-sloperator.md:82-101). | met |
| Decision names the crate, default model, masking mode and `MODEL_WINDOW_BUDGET_S` | spike-results.md:133-138 (`llama-cpp-2` =0.1.160, `"qwen3-4b"`, ADR-04 Option 1, `MODEL_WINDOW_BUDGET_S = 3.0`) | met |
| Verdict PASS (integrity 20/20 after verify) or FAIL escalated | spike-results.md:133 PASS; l.79 integrity 20/20; l.80 survival 23/23 | met |
| Story Test Cases pass together | `make test TESTS='sloperator::eval_corpus'` run this session: 1 passed (`twenty_paragraphs_present_without_placeholder_glyphs`); workflow tests_output PASS | met |

## Story 01 criteria

| Criterion | Evidence | Result |
|---|---|---|
| Every crate × model row has build, binary, GPU, load, tok/s | spike-results.md:14-22. The mistralrs Metal row reads "build FAILED", with the cause cited at l.30 | met (see deferred D1) |
| 20-paragraph table has placeholders kept, numbers/URLs, contract, accepted, plus survival before verify | l.37-60, l.80 | met |
| Licence section quotes the licence text of the chosen model | l.87-109 quote the Apache-2.0 header and the Alibaba Cloud notice | met |
| TESTS propagation shows the real Makefile, one selector → one testcase | l.117-127. I reproduced the same output this session | met |
| Decision sets crate, DEFAULT_MODEL, masking, budget | l.131-138 | met |
| `eval/paragraphs/` holds exactly p01–p20 with no ⟦/⟧ | `ls` shows exactly p01.md…p20.md, and `grep -l '[⟦⟧]'` matches nothing. The paragraphs include a table row (p10), links/URLs (p02, p07, p12, p14, p16, p17, p19) and code spans (16 files), as SPEC §22 asks | met |
| Gate passes | the test passes, and `grep -q '^## Decision'` matches l.131 | met |

## Test discrimination

`tests/eval_corpus.rs:15-36` fails if a paragraph is missing or empty (l.19-21), if any paragraph contains ⟦ or ⟧ (l.22-25), or if the directory holds any file other than p01–p20 (l.27-35). It would fail if the requirement regressed.

## Reachability

This epic builds no runtime code. It delivers a decision record and a test corpus. Both are consumed by later epics of this plan: epic-07-llama-runtime (`docs/plan/epic-07-llama-runtime.md`) takes the crate, model and masking choices, and epic-08-eval (`docs/plan/epic-08-eval/story-01-eval-harness.md:11`) takes the paragraphs and `MODEL_WINDOW_BUDGET_S`. Both epic files exist. `eval/README.md` is assigned to epic 08 (story-01-eval-harness.md:17), not to this epic.

## Gaps

| # | Gap | Where | Severity | Disposition |
|---|---|---|---|---|
| D1 | FR-9.9 says "for each crate and model … tokens/s … on GPU". mistralrs on Metal was never measured, because the machine lacks the Metal Toolchain. The per-paragraph table covers only the chosen pair; the other pairs appear as summary rows. This does not change the decision: the chosen pair is fully measured, and the architecture §12 gate is about the chosen pair. | spike-results.md:20, :30 | minor | DEFER → `docs/backlog/epic-02-mistralrs-metal-unmeasured.md` |
