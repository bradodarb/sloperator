# sloperator — change log

One entry per change item surfaced during delivery, never deleted.

## CH-1: Integration-tier JUnit report (from deferred.md #1, PRD gate)
- **What:** Integration JUnit must land in its own report, `target/nextest/it/junit.xml`, not the unit tier's.
- **Route:** append → epic-01-walking-skeleton story-02 (planned at slice time)
- **Landed in:** epic-01 story-02 AC; acceptance audit shows the integration-tier JUnit XML holding `builds_when_extracted` (commit 69675e8).
- **Status:** done

## CH-2: `TESTS` selectors reach cargo (from deferred.md #2, PRD gate)
- **What:** the Makefile must turn TESTS selectors into one nextest filterset; one selector gives exactly one testcase, and a selector that matches nothing fails loudly.
- **Route:** append → epic-01-walking-skeleton story-01 (planned at slice time)
- **Landed in:** epic-01 story-01 AC; acceptance audit ran `TESTS='sloperator::standalone#builds_when_extracted'` and got exactly that testcase.
- **Status:** done

## CH-3: FR-9.5 means strictly fewer weighted findings (from deferred.md #3, PRD gate / DQ-3)
- **What:** A rewrite with an equal weighted score is rejected.
- **Route:** append → epic-03-protection-core story-02 (`improvement_requires_strictly_fewer`) and epic-06-model-fix-path story-03 (`equal_score_rewrite_rejected`)
- **Landed in:** existing ACs in those stories (slice time). Epic 03 delivered the unit level (`improvement_requires_strictly_fewer`). Epic 06 delivered the end-to-end level: `tests/engine.rs:495` `equal_score_rewrite_rejected` (story-03, commit 2d80023, merged 5cae5d6); the epic-06 acceptance audit confirms the noImprovement "3 -> 3" rejection.
- **Status:** done

## CH-4: SLP901 severity and exit behaviour (from deferred.md #4, PRD gate / DQ-4)
- **What:** SLP901 is always `warning`, `--min-severity` never filters it, it counts toward exit 1, and it still prints under `--exit-zero`.
- **Route:** append → epic-06-model-fix-path story-03
- **Landed in:** existing AC in that story (slice time). Delivered by epic-06 story-03: `reported()` passes system codes through `--min-severity` and pins SLP901 to Warning (`src/pipeline/mod.rs:303-319`); `tests/cli_exit.rs:255` `slp901_survives_min_severity_and_exit_zero` proves exit 1, then exit 0 under `--exit-zero` with the finding still printed (commit 2d80023, merged 5cae5d6).
- **Status:** done

## CH-5: perf.rs proves the model cache is never read (from deferred.md #5, spec-review Gap 12)
- **What:** `rules_only_100kb_under_1s` points `model.cache_dir` at an unreadable directory (mode 000) and still exits 0 within budget. Under uid 0 the test skips loudly.
- **Route:** append → epic-05-linter-surface story-08
- **Landed in:** epic-05 story-08: `tests/perf.rs:87` `rules_only_100kb_under_1s`, run in the release profile through the Makefile's perf pass (commit 5a54c32, merged 9c2c4b2). The epic-05 acceptance audit confirmed the mode-000 cache directory and the loud skip under uid 0.
- **Status:** done

## CH-6: NFR-1 diff scope includes docs outside the crate (from epic-01 acceptance audit, Note 1)
- **What:** The workflow's record commit (bb0b6f1) changed three context docs outside the crate. Neither story merge did.
- **Route:** drop
- **Landed in:** n/a
- **Status:** dropped. The audit accepted it: NFR-1 governs the crate's own footprint, and context-map upkeep is required repo practice. No later epic changes.

## CH-7: `RewriterFactory::preflight` / `DisabledFactory` not exercised yet (from epic-01 acceptance audit, Note 2)
- **What:** No `check` path exercises the skeleton's engine port yet.
- **Route:** append → epic-05-linter-surface story-03 (`lazy_rewriter_preflights_without_opening`, the full port) and story-07 (`fix_without_model_exits_two` through DisabledFactory); epic-07-llama-runtime story-01 (RuntimeFactory preflight)
- **Landed in:** existing stories and tests (slice time); nothing to change. Epic 05 delivered its part: `tests/model_cache.rs:117` `lazy_rewriter_preflights_without_opening` (commit 19594b1) and `tests/e2e.rs:216` `fix_without_model_exits_two` (commit 8b1a5a3). The epic-07 story-01 part (RuntimeFactory preflight) is still to come.
- **Status:** done. Epic-07 story-01 delivered `RuntimeFactory` (`src/engine/runtime.rs`), which keeps the port's default `preflight` (`runtime.rs:47`: `model::resolve` already found the file, nothing loads before `open`); `loads_once_for_many_windows` drives it through the CLI (epic-07 acceptance audit, round 2 verdict met).

## CH-8: mistralrs Metal row unmeasured (from epic-02 acceptance audit, D1; `docs/backlog/epic-02-mistralrs-metal-unmeasured.md`)
- **What:** `docs/research/spike-results.md:20,30`: the `mistralrs =0.8.1` Metal build failed (no Metal Toolchain on the machine), so that FR-9.9 row has no numbers.
- **Route:** drop
- **Landed in:** n/a
- **Status:** dropped. The chosen pair (llama-cpp-2 + Qwen3-4B) is fully measured, and the architecture §12 gate concerns only that pair. No later epic uses the mistralrs numbers. The backlog file stays in case the runtime choice is ever reopened.

## CH-9: no-op metadata assertion in rules_registry (from epic-04 acceptance audit, G2; `docs/backlog/epic-04-registry-noop-assert.md`)
- **What:** `tests/rules_registry.rs:66` `let _ = (m.severity, m.default_on, m.fix);` asserts nothing. It should check each rule's severity and fix class against SPEC §10, and that `default_on` agrees with `default_set()`.
- **Route:** append → epic-05-linter-surface story-09-registry-metadata-assert (new, wave 6, parallel with story-08; test-only, touches only `tests/rules_registry.rs`)
- **Landed in:** `docs/plan/epic-05-linter-surface/story-09-registry-metadata-assert.md` (commit 4b3a569, merged 6fbe523). `tests/rules_registry.rs:46-110` now checks every rule's severity and fix class against a literal SPEC §10 table, checks `default_on` against `default_set()`, and checks that the registry codes equal the table's codes. The backlog file `docs/backlog/epic-04-registry-noop-assert.md` is resolved by this commit.
- **Status:** done

## CH-10: `score::score` / `score::overall` unreachable from `src/` (from epic-04 acceptance audit, G1, resolved by naming the wiring epic)
- **What:** Only `tests/score.rs` calls the score. The epic-04 Notes name epic-05 story-04-pipeline (step 4, FR-5.2) as the wiring point, but that story had no test proving the pipeline computes the score.
- **Route:** append → epic-05-linter-surface story-04-pipeline (new test `fix_sets_score_and_score_before`, added to the Test Cases and the Gate)
- **Landed in:** `docs/plan/epic-05-linter-surface/story-04-pipeline.md` (Test Cases #5, Notes, Gate); `tests/pipeline.rs:158` `fix_sets_score_and_score_before` (commit 0f22501). The epic-05 audit shows `pipeline::fix` reached from `src/cli/mod.rs`.
- **Status:** done

## CH-11: `verify` and `edit` unreachable from `src/` (from epic-03 acceptance audit, G2)
- **What:** Only tests reach `verify::edit`, `edit::plan` and `edit::apply`. The audit asked for the epic-03 Notes to name the wiring epics, but epic-03 has landed and cannot be edited.
- **Route:** append → epic-05-linter-surface story-04-pipeline (fix step 1: plan → verify → apply; tests `rejected_safe_fix_is_slp901_original_kept`, `second_fix_changes_nothing_rules_engine`) and epic-06-model-fix-path story-03 (`verify::edit`, `verify::improvement`)
- **Landed in:** existing stories and tests (slice time); this entry is the pointer the audit asked for. In epic 04, `check_fix` (`tests/rules_fixtures.rs:100-109`) already runs plan → verify → apply under test. Epic 05 delivered step 1 from `src/`: `tests/pipeline.rs:99` `second_fix_changes_nothing_rules_engine` and `:117` `rejected_safe_fix_is_slp901_original_kept` (commit 0f22501), reached from `src/cli/mod.rs` (`pipeline::fix`). Epic 06 story-03 delivered the model path: `pipeline::model_windows`/`accept` (`src/pipeline/mod.rs:205-283`) runs mask → `verify::edit` → re-detect → `verify::improvement`, reached from `cli::run` through `check --fix/--diff`; tests `good_rewrite_written` (`tests/engine.rs:468`) and `number_drop_rejected_original_kept` (`:482`) (commit 2d80023, merged 5cae5d6).
- **Status:** done

## CH-12: perf.rs skips under uid 0 (from epic-05 acceptance audit, note 1)
- **What:** `tests/perf.rs` skips with a loud stderr message when run as root, because root ignores mode 000 on the cache directory.
- **Route:** drop
- **Landed in:** n/a
- **Status:** dropped. The audit calls it a documented limit, not a defect: the skip is loud, never a silent pass (fail loudly). This was planned at slice time (CH-5) and nothing depends on it.

## CH-13: FR-7 mostly tested through the library, not the binary (from epic-05 acceptance audit, note 2)
- **What:** `tests/config.rs` drives `Resolver` directly. Only `config_command_prints_resolved_json` and `exit_two_on_bad_config` go through the CLI.
- **Route:** drop
- **Landed in:** n/a
- **Status:** dropped. The audit judged the coverage enough for the epic criterion and deferred nothing. The CLI composes `Resolver::new` (`src/cli/mod.rs` ~119), so the library tests cover the same code.

## CH-14: shipped binary still wires DisabledFactory (from epic-06 acceptance audit, "Reachable as shipped")
- **What:** `src/main.rs:19` wires `DisabledFactory`, so a shipped `--fix` without `--rules-only` exits 2 ("without model support"). The model step is reached only through FakeRewriter in tests until the real adapter lands.
- **Route:** append → epic-07-llama-runtime story-01-runtime-adapter (planned at slice time)
- **Landed in:** epic-07 story-01: `src/main.rs:19-20` constructs `RuntimeFactory::new(cli.verbose > 0)` under `cfg(feature = "model")`; `model` is a default feature. The epic-07 audit's "Reachable as shipped" confirms `no_network_during_check` rewrites through the shipped binary.
- **Status:** done

## CH-15: `make check` failed at `cargo fmt --check` (from epic-07 acceptance audit, Gap 1)
- **What:** rustfmt drift from earlier epics (`src/pipeline/mod.rs`, `tests/cli_exit.rs`, `tests/engine.rs`) stopped `check` before either clippy run, so epic-07 criterion 2 was unmet.
- **Route:** append → epic-07-llama-runtime story-02-fmt-check-green (added by the epic's revise loop before it landed)
- **Landed in:** epic-07 story-02 (HEAD 1009d99); audit round 2: `make check` exits 0 with both clippy runs.
- **Status:** done

## CH-16: `no_network_during_check` cannot catch a direct socket (from epic-07 acceptance audit, Gap 2; `docs/backlog/epic-07-no-network-test-weak.md`)
- **What:** `tests/model_real.rs:249-279` only sets HTTP(S)/ALL proxy variables, which llama.cpp never reads; a non-proxied connect would still pass. FR-9.1 is therefore weakly proven.
- **Route:** append → epic-08-eval story-02-no-network-sandbox (new, wave 1, parallel; touches only `tests/model_real.rs`). Nothing later depends on it, so it does not block.
- **Landed in:** `docs/plan/epic-08-eval/story-02-no-network-sandbox.md`: `no_network_under_sandbox` runs the shipped binary under `sandbox-exec` (macOS) / `unshare -rn` (Linux), with a control proving the sandbox denies connects and a loud failure when no sandbox is available.
- **Status:** done. `tests/model_real.rs:280-365` `no_network_under_sandbox` passes. Before it runs the binary, an `nc -z` control shows a live listener is reachable unsandboxed and unreachable inside the sandbox, and it panics naming both tools when neither exists. Confirmed by the epic-08 acceptance audit round 2, criterion 3. The backlog file `sloperator-epic-07-no-network-test-weak.md` is resolved.

## CH-17: SPEC §21 still describes `Box::leak` and `void_logs` (from epic-07 acceptance audit, Gap 3; `docs/backlog/epic-07-spec21-leak-stale.md`)
- **What:** `docs/SPEC.md` §21 (2040, 2075, 2081) and open question 4 (2549) describe a leaked model, `void_logs` and one reused context; the shipped `runtime.rs` owns the model (Metal teardown assert, `runtime.rs:5-7`), uses `send_logs_to_tracing` and a fresh context per window.
- **Route:** append → epic-08-eval story-03-spec21-runtime-text (new, wave 1, docs only). Not a re-plan: behaviour is correct and already recorded in the index's Accepted Gap 13 row; only the text is stale.
- **Landed in:** `docs/plan/epic-08-eval/story-03-spec21-runtime-text.md`, Gate greps that `Box::leak`/`void_logs` are gone and `send_logs_to_tracing` is present.
- **Status:** done. The epic-08 acceptance audit (criterion 4) found no `Box::leak`, `void_logs` or `clear_kv_cache` in the SPEC. `send_logs_to_tracing` appears at SPEC:2039, 2055, 2070 and 2089, the owned model and Metal teardown reason at SPEC:2164-2165, and open question 4 at SPEC:2574. The backlog file `sloperator-epic-07-spec21-leak-stale.md` is resolved. The index's Accepted Gap 13 row still calls the SPEC wording "stale (deferred)"; that note is now historical and the index row is left as is.

## CH-18: `cli_exit` tests fail when `$TMPDIR` has an ignore-all `.gitignore` (from epic-08 acceptance audit, Deferred D1; `docs/backlog/cli-exit-tmpdir-gitignore.md`)
- **What:** On the dev host, `$TMPDIR/.gitignore` contains `*`. Inside `standalone#builds_when_extracted`'s extracted copy, six `cli_exit` tests (`tests/cli_exit.rs:21,44,61,79,119,140`) pass fixture paths that resolve under `$TMPDIR`. The CLI's gitignore handling (`src/input/mod.rs:199`) skips those paths, so the tests fail and the integration tier is red unless `TMPDIR` is clean. The audit found the same failure at base 62c04bb, so it predates epic 08 and comes from the environment.
- **Route:** append → epic-09-backlog story-01-cli-exit-tmpdir-independent (new trailing backlog epic). All epics through 08 have landed, so no unlanded epic exists to take the story, and nothing depends on it.
- **Landed in:** `docs/plan/epic-09-backlog/story-01-cli-exit-tmpdir-independent.md`. The six tests check fixtures copied into `common::cli_tempdir()` (`tests/common/mod.rs:139-143`, which writes `!*`). An extra Gate leg reruns `builds_when_extracted` under a `TMPDIR` whose `.gitignore` is `*`. `src/`, `standalone.rs` and `common/mod.rs` stay unchanged. The needs line `epic-09-backlog: epic-08-eval` is in index.md.
- **Status:** done. Epic-09 story-01 added `fixture_copy` (`tests/cli_exit.rs:14-21`), which copies each fixture into `common::cli_tempdir()`, and the six tests use it (commit b9b7c9b, merged 9f36ec2). The epic-09 acceptance audit (verdict met) ran the integration tier under a `TMPDIR` whose `.gitignore` is `*`: exit 0, 7/7, including `standalone#builds_when_extracted`, and `cli_exit` 15/15. `src/`, `standalone.rs` and `common/mod.rs` are unchanged. The backlog file `sloperator-cli-exit-tmpdir-gitignore.md` is resolved by this commit.

## CH-19: resolved backlog file still on disk (from epic-09 acceptance audit, Gaps, housekeeping note)
- **What:** `docs/backlog/cli-exit-tmpdir-gitignore.md` still exists after its fix landed. The same holds for the other resolved sloperator backlog files (CH-8, CH-9, CH-16, CH-17).
- **Route:** drop
- **Landed in:** n/a
- **Status:** dropped. This is not a gap in the plan, and no epic depends on it. Each file's resolution is recorded here (CH-8/9/16/17/18). Removing backlog files belongs to the backlog owner, not to a delivery story.

## CH-20: story-01's Files to Touch row for `tests/standalone.rs` (from the epic-09 story review, minor note)
- **What:** The landed story's Files to Touch table lists `tests/standalone.rs` with "methods added". Its Scope and acceptance criteria say the file stays unchanged, and it did.
- **Route:** drop
- **Landed in:** n/a
- **Status:** dropped. The epic has landed, so its files are not edited. The delivered code matches the acceptance criteria, and nothing depends on the table row.
