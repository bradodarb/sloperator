# Acceptance audit: epic-06-model-fix-path

Auditor: reviewer. Base: `dd782ee9`. Workflow results: tests PASS, discriminate PASS.

**Verdict: met.** No blocking gaps.

## Epic acceptance criteria

| Criterion | Evidence | Met |
|---|---|---|
| FakeRewriter: a good rewrite is written; a rewrite that drops a number is rejected and the original kept (D-4) | `pipeline::model_windows`/`accept` (src/pipeline/mod.rs:205-283): mask::propose → verify::edit → re-detect → verify::improvement. Tests: tests/engine.rs `good_rewrite_written` (asserts the fixed text) and `number_drop_rejected_original_kept` (asserts `fixed == None` and an SLP901 "(numbers)" finding) | yes |
| No window means no load; many windows load once (FR-9.4) | `rw.get()?` is called only inside the window loop (src/pipeline/mod.rs:247). Tests: `no_window_means_no_load` (opens == 0), `many_windows_load_once` (opens == 1, 3 calls) | yes |
| `model pull` against a local server verifies sha256 and removes a corrupt `.part` (FR-2.14) | src/model/pull.rs:44-73. Tests: tests/model_cache.rs `pull_from_local_server_verifies_sha256` and `pull_rejects_corrupt_file_and_removes_part` (local TcpListener, `Error::Download` naming the URL, no `.part`, no installed file) | yes |
| Every story's Test Cases pass on the merged epic | All 29 named tests exist (grep: engine.rs:118-602, model_cache.rs:207/228/277, layering.rs:204, pipeline.rs:235/258/276, cli_exit.rs:255, directives.rs:299). Workflow tests state: PASS | yes |

## Story criteria (summary)

- **Story 01** (windows/mask/prompt): src/engine/{window,mask,prompt}.rs, with the 11 engine tests. The Skip→SLP902 path goes through `skipped_finding()` in the pipeline (src/pipeline/mod.rs:225-236).
- **Story 02** (pull/list/path): `Command::Model(ModelCmd)` (src/cli/args.rs:37-51) is dispatched in `cli::run` (src/cli/mod.rs:69). `model path` with nothing cached exits 2 through `model::resolve` → ModelMissing (`list_and_path_report_presence`). `pull` uses `HF_BASE` and prints "up to date" when the sidecar matches. `ureq` appears only in pull.rs (`network_only_in_pull`).
- **Story 03** (pipeline step 2):
  - DQ-3: equal weight is rejected with noImprovement "3 -> 3" (`equal_score_rewrite_rejected`).
  - DQ-4: `reported()` lets system codes through `--min-severity` (src/pipeline/mod.rs:303-308), and SLP901 is always Warning (:311-319). Tested end to end through the CLI by `slp901_survives_min_severity_and_exit_zero` (exit 1, then `--exit-zero` exit 0 with the finding still printed).
  - FR-9.6: `LazyRewriter::new` preflights before any file is read (src/cli/mod.rs:156-162). Covered by `missing_model_exits_two_before_any_file` and `disabled_factory_exits_two` (a non-UTF-8 file is never reported).
  - FR-9.7: a per-file error blocks `write::commit` (src/cli/mod.rs:214-221). Covered by `runtime_error_exits_two_nothing_written` (the safe fix in a.md is not written either).
  - FR-9.8: `seed_passed_through_same_for_every_window` (CLI `--seed 1234`, three windows).
  - FR-6.6: `suppressed_region_not_edited_by_either_engine`.

## Reachable as shipped

Everything this epic builds is reached from `cli::run`: the `model` subcommands, and `pipeline::fix` step 2 through `check --fix/--diff`.

src/main.rs:19 still wires `DisabledFactory`. That matches the plan: the epic file names epic-07-llama-runtime as the place the real llama.cpp adapter arrives, and docs/plan/epic-07-llama-runtime.md exists. Until then a shipped `--fix` without `--rules-only` exits 2 with "without model support", as FR-9.6 requires for a model-less build.

## Would the tests fail on regression?

Yes. Each test asserts the observable outcome: the written or kept bytes, the open/call counts, the exit codes, the SLP901 check name, and that no `.part` remains. Removing the improvement check breaks `equal_score_rewrite_rejected`. Making the model load eagerly breaks `no_window_means_no_load`. Dropping the system-code bypass breaks `slp901_survives_min_severity_and_exit_zero`. The workflow's discriminate step also reported PASS.

## Gaps

None blocking. Nothing deferred.
