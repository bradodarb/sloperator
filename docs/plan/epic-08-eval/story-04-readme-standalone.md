# Story 04: `eval/README.md` names no host project (standalone scan green)

> Historical record: the one-time repository text scan and its planted-reference test have since been removed. Current standalone checks cover path dependencies and extracted builds (SPEC §5).
**Epic:** [epic-08-eval](../epic-08-eval.md)
**SPEC Refs:** §5, §22
**PRD Refs:** FR-10.1, FR-10.2
**Component / ADR:** architecture §6 eval
**Wave:** 2

## Scope

Fix story for acceptance-audit Gap 1 (`acceptance.md`). `eval/README.md:34` tells the reader to run the eval with a build command the crate does not own. The standalone scan (`tests/standalone.rs:14-21` `HOST_MARKERS`, `:72-75` the retired repository text check, SPEC §5) forbids the host name in every crate file, case-insensitively, so the unit tier is red. Rewrite the "Running the eval" block in terms the crate owns, using cargo only:

```sh
sloperator model pull     # the default model must be in the cache
cargo test --test eval -- --ignored eval_report_per_model
```

(`model` is a default feature, `Cargo.toml:12`, and `tests/eval.rs:8` is `#![cfg(feature = "model")]`; the test is `#[ignore = "integration tier"]`, `tests/eval.rs:325`, so `--ignored` is required.) Keep the rest of the README as it is. Do not touch `tests/standalone.rs`: the scan, its markers and `NOT_EXTRACTED` stay unchanged.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `eval/README.md` | ~31-35 | run instruction rewritten with cargo; no host reference |

## Acceptance Criteria

- [ ] `eval/README.md` contains no `HOST_MARKERS` entry in any case; its run instruction is `cargo test --test eval -- --ignored eval_report_per_model` (after `sloperator model pull`).
- [ ] `git diff` shows no change to `tests/standalone.rs`.
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|

## Notes

Docs only; no `src/` or test change. Unblocks epic criteria 2 (unit tier exits 0) and 5 (all story tests pass together).

Gate: verify that `eval/README.md` documents `cargo test --test eval -- --ignored eval_report_per_model`.
