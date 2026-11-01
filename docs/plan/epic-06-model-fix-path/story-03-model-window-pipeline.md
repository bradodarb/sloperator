# Story 03: Pipeline model step: rewrite, verify, improvement, SLP901/902

**Epic:** [epic-06-model-fix-path](../epic-06-model-fix-path.md)
**SPEC Refs:** §17 (step 2), §16 (LazyRewriter use), §8 (`improvement`), §23.12, §23.13, §23.8, DQ-3, DQ-4
**PRD Refs:** FR-9.2, FR-9.4, FR-9.5, FR-9.6, FR-9.7, FR-9.8, FR-8.7, FR-6.6, FR-2.9, FR-2.8, D-3, D-4, Brad Q3 (a)
**Component / ADR:** §4.11 pipeline; ADR-05, ADR-09
**Wave:** 2

## Scope

Fill step 2 of `pipeline::fix`: windows → `rw.get()?.rewrite` → `mask::propose` → `verify::edit` → re-detect on the candidate → `verify::improvement`; a rejection is SLP901 and a Skip is SLP902. `--model-only` skips step 1. In `cli`, SLP901 and SLP902 bypass `--min-severity` and still print under `--exit-zero`, and `--seed` reaches every request.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/pipeline/mod.rs` | from epic-05 story-04 | step 2 |
| `src/cli/mod.rs` | from story-02 | SLP9xx filter/exit rules, seed |
| `tests/directives.rs` | from epic-04 story-03 | suppressed_region_not_edited_by_either_engine |
| `tests/engine.rs` | existing | tests below (methods added) |
| `tests/pipeline.rs` | existing | tests below (methods added) |
| `tests/cli_exit.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] Deferred item 3 (DQ-3): a FakeRewriter rewrite whose window has the *same* weighted count as before is rejected as SLP901 and the original is kept; a rewrite with strictly fewer is written (FR-9.5).
- [ ] Deferred item 4 (DQ-4): SLP901 is always `warning`; it is reported under `--min-severity error`, it makes the run exit 1, and under `--exit-zero` the run exits 0 but SLP901 is still printed (FR-8.7, FR-2.8, FR-2.9).
- [ ] A missing model or a model-less build exits 2 before any file is read or written (FR-9.6).
- [ ] A runtime error exits 2 and writes nothing (FR-9.7, NFR-5).
- [ ] The same seed goes to every window (FR-9.8).
- [ ] No window means no `open`; many windows mean one `open` (FR-9.4).
- [ ] Suppressed regions are edited by neither engine (FR-6.6).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::engine` | `no_window_means_no_load` | lazy load | FR-9.4 |
| 2 | `sloperator::engine` | `many_windows_load_once` | single load | FR-9.4 |
| 3 | `sloperator::engine` | `good_rewrite_written` | verified rewrite written | FR-9.5, D-4 |
| 4 | `sloperator::engine` | `number_drop_rejected_original_kept` | number drop rejected | FR-9.5, D-4 |
| 5 | `sloperator::engine` | `equal_score_rewrite_rejected` | equal weighted count → SLP901, original kept | FR-9.5, DQ-3 (deferred 3) |
| 6 | `sloperator::engine` | `missing_model_exits_two_before_any_file` | missing model | FR-9.6 |
| 7 | `sloperator::engine` | `disabled_factory_exits_two` | model-less build | FR-9.6, ADR-02 |
| 8 | `sloperator::engine` | `runtime_error_exits_two_nothing_written` | runtime error | FR-9.7, NFR-5 |
| 9 | `sloperator::engine` | `seed_passed_through_same_for_every_window` | seed | FR-9.8 |
| 10 | `sloperator::pipeline` | `default_runs_rules_then_model_on_remaining` | engine order | D-3, FR-9.2 |
| 11 | `sloperator::pipeline` | `model_only_skips_safe_fixes` | --model-only | FR-2.10 |
| 12 | `sloperator::pipeline` | `rejection_is_slp901_warning_counts_for_exit` | SLP901 severity and exit | FR-8.7, DQ-4 |
| 13 | `sloperator::cli_exit` | `slp901_survives_min_severity_and_exit_zero` | SLP901 reported under --min-severity error; --exit-zero exits 0 but prints it | FR-8.7, FR-2.8, FR-2.9, DQ-4 (deferred 4) |
| 14 | `sloperator::directives` | `suppressed_region_not_edited_by_either_engine` | frozen regions | FR-6.6 |

## Notes

`equal_score_rewrite_rejected` and `slp901_survives_min_severity_and_exit_zero` are not in SPEC §23. They are added so deferred items 3 and 4 each get an end-to-end fixture. SLP901/902 are excluded from the score (DQ-4).

Gate: `make test TESTS='sloperator::engine#no_window_means_no_load sloperator::engine#many_windows_load_once sloperator::engine#good_rewrite_written sloperator::engine#number_drop_rejected_original_kept sloperator::engine#equal_score_rewrite_rejected sloperator::engine#missing_model_exits_two_before_any_file sloperator::engine#disabled_factory_exits_two sloperator::engine#runtime_error_exits_two_nothing_written sloperator::engine#seed_passed_through_same_for_every_window sloperator::pipeline#default_runs_rules_then_model_on_remaining sloperator::pipeline#model_only_skips_safe_fixes sloperator::pipeline#rejection_is_slp901_warning_counts_for_exit sloperator::cli_exit#slp901_survives_min_severity_and_exit_zero sloperator::directives#suppressed_region_not_edited_by_either_engine'`
