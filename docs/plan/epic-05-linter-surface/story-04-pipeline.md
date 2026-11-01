# Story 04: `pipeline`: analyse, lint, fix with the rules engine

**Epic:** [epic-05-linter-surface](../epic-05-linter-surface.md)
**SPEC Refs:** §17 (steps 1, 3, 4), §23.13 (rules-engine tests)
**PRD Refs:** FR-2.1, FR-2.10, FR-4.7, FR-8.7, FR-8.9, FR-5.2, D-3, D-4
**Component / ADR:** §4.11 pipeline; ADR-03, ADR-05, ADR-09
**Wave:** 3

## Scope

Implement `src/pipeline/mod.rs`: Mode, Engines, Analysis, Located, FileResult, `analyse`, `lint`, and `fix` with step 1 (safe fixes → plan → verify → apply → re-detect; a rejection becomes SLP901), step 3 (whole-document check) and step 4 (report and scores). Step 2 (model windows) is left as a call that does nothing when `engines.model` is off or `rw` is `None`, and epic 06 story-03 fills it.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/pipeline/mod.rs` | new | analyse, lint, fix (rules engine) |
| `tests/pipeline.rs` | new | tests below |

## Acceptance Criteria

- [ ] `lint` never constructs or calls a Rewriter (FR-2.1, D-3).
- [ ] `--rules-only` never calls the model (FR-2.10).
- [ ] A rejected safe fix becomes SLP901 `rewrite-rejected` at warning, and the original text is kept (FR-8.7).
- [ ] A second `fix` over its own output changes nothing (FR-8.9).
- [ ] `score_before` and `score` are set in Fix/Diff mode (FR-5.2, DQ-12).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::pipeline` | `lint_never_constructs_rewriter` | lint is model-free | FR-2.1, D-3 |
| 2 | `sloperator::pipeline` | `rules_only_never_calls_model` | --rules-only | FR-2.10 |
| 3 | `sloperator::pipeline` | `second_fix_changes_nothing_rules_engine` | idempotence through the pipeline | FR-8.9 |
| 4 | `sloperator::pipeline` | `rejected_safe_fix_is_slp901_original_kept` | a forged safe fix that drops a number becomes SLP901 | FR-8.7 |
| 5 | `sloperator::pipeline` | `fix_sets_score_and_score_before` | `fix` on a fixture with a safe-fixable finding returns `score_before` equal to `score::score` on the original and `score` on the final text, lower than `score_before`; `lint` returns `score` with `score_before` = None. This is the first `src/` caller of `score::score` (epic-04 audit G1, CH-10) | FR-5.2, DQ-12 |

## Notes

`rejected_safe_fix_is_slp901_original_kept` is added here; SPEC §23.13's `rejection_is_slp901_warning_counts_for_exit` covers the model path in epic 06 story-03. `pipeline` must not use cli, report, write or engine::runtime (layering). `fix_sets_score_and_score_before` was added at the epic-04 re-plan: the epic-04 audit (G1) accepted unwired `score::score`/`score::overall` because this story wires them, so the wiring needs its own test. Step 4 is where `score::score` is first called from `src/`; `score::overall` is reached through the report summary (story-05).

Gate: `make test TESTS='sloperator::pipeline#lint_never_constructs_rewriter sloperator::pipeline#rules_only_never_calls_model sloperator::pipeline#second_fix_changes_nothing_rules_engine sloperator::pipeline#rejected_safe_fix_is_slp901_original_kept sloperator::pipeline#fix_sets_score_and_score_before'`
