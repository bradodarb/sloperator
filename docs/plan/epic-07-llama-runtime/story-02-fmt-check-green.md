# Story 02: `make check` green: rustfmt drift and both clippy runs

**Epic:** [epic-07-llama-runtime](../epic-07-llama-runtime.md)
**SPEC Refs:** §1 (Decision), §21
**PRD Refs:** NFR-7, D-1
**Component / ADR:** Makefile `check` target; ADR-02
**Wave:** 2

## Scope

Fix story from the epic-07 acceptance audit (epic criterion 2). `make check` exits 2 at `cargo fmt --check` so neither clippy run (`--no-default-features` and default features) is reached. The drift predates this epic and sits in files epic-07 did not change.

1. Run `cargo fmt` in the crate (formatting only, no hand edits to logic).
2. Run `make check`. If either clippy run now reports warnings (they were hidden behind the fmt failure), fix them in place with the smallest change that keeps behavior. Do not use `#[allow(...)]` unless the lint is a false positive, and if you do, add a one-line reason comment.
3. Do not change the Makefile: the gate stays as written (no weakening of `-D warnings`, no dropping `fmt --check`).

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/pipeline/mod.rs` | 9, 15, 147 | rustfmt only |
| `tests/cli_exit.rs` | 268, 282 | rustfmt only |
| `tests/engine.rs` | 488, 495, 502, 536, 561, 588, 615 | rustfmt only |
| any `**/*.rs` | as reported | only if a clippy run reports a warning once fmt passes |

## Acceptance Criteria

- [ ] `cargo fmt --check` in the crate reports no diff.
- [ ] `make check` exits 0, and its output shows both clippy runs (`--no-default-features` and default features) completed with `-D warnings`.
- [ ] The Makefile is unchanged.
- [ ] The reformatted test binaries still pass: `sloperator::engine`, `sloperator::cli_exit`, and `sloperator::layering#runtime_crate_only_in_adapter`.
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::engine` | (whole binary) | reformatted tests still pass | NFR-7 |
| 2 | `sloperator::cli_exit` | (whole binary) | reformatted tests still pass | NFR-7 |
| 3 | `sloperator::layering` | `runtime_crate_only_in_adapter` | runtime crate still confined | ADR-02 |
| 4 | contract `check` | fmt + clippy (both feature sets) | lint gate green | epic criterion 2, ADR-02 |

## Notes

No model needed: these are unit-tier binaries plus the lint gate. Deferred items from the audit (`no_network_during_check` scope, SPEC §21 `Box::leak` wording) stay in `docs/backlog/` and are not part of this story.

Gate: `make test TESTS='sloperator::engine sloperator::cli_exit sloperator::layering#runtime_crate_only_in_adapter' && make check`
