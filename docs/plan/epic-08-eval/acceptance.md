# Acceptance audit: sloperator / epic-08-eval

> Historical record: the one-time repository text scan and its planted-reference test have since been removed. Current standalone checks cover path dependencies and extracted builds (SPEC §5).
Base: 62c04bba2337d1451ef73ce49907af0da9c2fdfa. Reviewer: reviewer. This is round 2, after story-04 fixed round 1's Gap 1.

**Verdict: met** (no blocking gap; one environmental item deferred)

## Criteria

| Criterion | Status | Evidence |
|---|---|---|
| `target/eval/report.json` and the printed table for the default model (FR-10.2) | Met | I ran `make test-integration TESTS='sloperator::eval#eval_report_per_model'`: PASS. It wrote `target/eval/report.json` for `qwen3-4b`: integrity 20/20, mean score 132.28 → 61.83, 11 windows, 1 accepted, rejections per Check (`protected` 9, `placeholders` 1), SLP902 0, 0.676 s/window against a 3.0 budget. Code: `tests/eval.rs:198-302` (report row), `:324-380` (test). |
| Unit and integration tiers exit 0 | Met | Unit tier (`make test`) exit 0, 211 passed. With a clean `TMPDIR`, the integration tier exits 0 with 7/7 passed. With the host's default `TMPDIR`, `standalone#builds_when_extracted` fails, but it fails the same way at base. See Deferred D1. |
| FR-9.1 proven by an OS sandbox, not proxy variables (CH-16) | Met | `tests/model_real.rs:280-365` `no_network_under_sandbox` runs `sandbox-exec` with deny `network*`, or `unshare -rn`. Before it runs the binary, a control checks that `nc -z` reaches a live listener unsandboxed and fails inside the sandbox. If no sandbox exists, the test panics naming both tools. Run: PASS (4.2 s). |
| SPEC §21 describes the shipped adapter (CH-17) | Met | `grep Box::leak\|void_logs\|clear_kv_cache` finds nothing in the SPEC. `send_logs_to_tracing` appears at SPEC:2039, 2055, 2070, 2089. The owned model and the Metal teardown reason appear at SPEC:2164-2165 and open question 4 at SPEC:2574. |
| Every story's Test Cases pass together | Met | `eval_report_per_model` and `no_network_under_sandbox` pass (integration). `layering#runtime_crate_only_in_adapter`, the retired repository text check and the retired planted-reference check pass (unit). |

## Story checks

- Story 01: `MODEL_WINDOW_BUDGET_S = 3.0` (`tests/eval.rs:31`). The test asserts it is non-zero at compile time and checks it at runtime against `spike-results.md:137` (`spike_budget()`, `:307-321`), so the constant cannot drift. When the default model is missing, the test panics naming `sloperator model pull` (`:135-138`).
- Story 04: `eval/README.md:31-35` now gives the run instruction as cargo only and contains no `HOST_MARKERS` entry. `tests/standalone.rs` is unchanged against base (`git diff --quiet` passes). Round 1 Gap 1 is closed.
- Story 03: docs only. `src/` is unchanged against base.

## Would the tests fail on regression?

- Eval: the integrity assert, the budget assert and the default-model-present assert each fail the test when their requirement breaks.
- Sandbox test: the `nc` control makes a no-op sandbox fail, and a socket opened by the binary would surface as a `sloperator:` error or a non-0/1 exit.
- SPEC text: the story Gate greps for the stale names.
- README: the retired repository text check checks it, and the retired planted-reference check passing shows the scan was not weakened.

## Reachability

This epic adds tests and docs only (`git diff --name-status`: `eval/README.md`, `tests/eval.rs`, `tests/model_real.rs`, plus SPEC and directives). No new src/main surface needs wiring.

## Deferred

### D1: the extracted-crate CLI tests fail when `$TMPDIR` has an ignore-all `.gitignore` (DEFER)
- **What:** On this host, `$TMPDIR/.gitignore` contains `*` (dated Oct 25, before this epic). In `builds_when_extracted`, the extracted copy's `cli_exit` tests (6 of them, e.g. `tests/cli_exit.rs:25,48,73`) then print `sloperator: skipped (excluded)` and fail. The cause: those tests write their fixtures under a temp dir that has no `!*` re-include, unlike `common::cli_tempdir` (`tests/common/mod.rs:136-142`).
- **Evidence:** The same 6 failures reproduce on a base checkout (`git archive 62c04bb`, `cargo test --no-default-features --test cli_exit`: 9 passed, 6 failed). With `TMPDIR=/tmp/aud08tmp/` the whole integration tier passes 7/7.
- **Severity:** non-blocking. The failure is pre-existing and environmental, and this epic did not touch `cli_exit.rs` or `src/`.
- **Disposition:** DEFER → `docs/backlog/cli-exit-tmpdir-gitignore.md`.
