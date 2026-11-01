# Story 01: Crate, error type, minimal `check`, and Makefile

**Epic:** [epic-01-walking-skeleton](../epic-01-walking-skeleton.md)
**SPEC Refs:** §2, §3, §4, §20 (skeleton subset), §23.2
**PRD Refs:** FR-1.1, FR-1.2, FR-1.3, FR-1.4, FR-1.5, FR-1.6, FR-1.9, FR-2.15, FR-2.16, NFR-1, NFR-4, NFR-7, D-11
**Component / ADR:** §4.15 module shell, §4.1 cli (skeleton); ADR-02, ADR-07
**Wave:** 1

## Scope

Create the crate exactly as SPEC §2 pins it: Cargo.toml with `default = ["model"]`, Cargo.lock, toolchain, clippy, deny and nextest configs, .gitignore, the README section headings and both licence files. Add `src/lib.rs` with every module from §4 declared, where modules other than `error` and `cli` are empty stubs that later stories fill. Add `src/error.rs` with the 11 variants. Add `src/main.rs`, the single `std::process::exit`, which uses `engine::DisabledFactory` until epic 07. Add a minimal `cli`: clap parsing of `check PATHS...`, where each path is read, a missing path exits 2 with `Error::Io`, and otherwise the result is exit 0. Add the `Makefile` from SPEC §3, plus `tests/common/mod.rs` with `fixture` and `run_cli`. No rules, config or report yet.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `Cargo.toml` | new | SPEC §2 pins; features model/metal/cuda |
| `Cargo.lock` | new | generated with `cargo generate-lockfile`; committed |
| `rust-toolchain.toml` | new | channel 1.99.1 |
| `clippy.toml` | new | tests may unwrap |
| `deny.toml` | new | licence allow-list (NFR-7) |
| `.config/nextest.toml` | new | profiles ci, ci-perf, it with JUnit |
| `.gitignore` | new | `target/` |
| `README.md` | new | section headings from SPEC §2; Building section with Phase 0 prerequisites |
| `LICENSE-MIT` | new | licence text |
| `LICENSE-APACHE` | new | licence text |
| `Makefile` | new | SPEC §3: build, test, test-integration, check, clean, help; tool guard, JUnit testcase count, TESTS→filterset |
| `src/lib.rs` | new | module declarations (§4); stub modules |
| `src/error.rs` | new | `Error`, 11 variants (§4) |
| `src/main.rs` | new | §20 main; DisabledFactory for now |
| `src/cli/mod.rs` | new | `run`, `ExitCode`, `Deps` (skeleton check) |
| `src/cli/args.rs` | new | `Cli`, `Command::Check` (skeleton) |
| `src/engine/mod.rs` | new | `RewriterFactory` trait and `DisabledFactory` only (§16), so main compiles |
| `tests/common/mod.rs` | new | `fixture`, `run_cli` |
| `tests/cli_exit.rs` | new | tests below |

## Acceptance Criteria

- [ ] `make test` exits 0 and writes `target/nextest/ci/junit.xml` holding the tests below (FR-1.3, FR-1.4).
- [ ] Deferred item 2 (DQ-2): `make test TESTS='sloperator::cli_exit#help_and_version_exit_zero'` writes JUnit XML with exactly one testcase, and `TESTS='no_such_test'` exits non-zero with `sloperator: no test ran` (FR-1.5; evidence recorded in the story's PR).
- [ ] `TESTS='cli_exit'` prints `sloperator: perf pass skipped` on stderr and writes no `target/nextest/ci-perf/junit.xml` (Gap 9).
- [ ] With `cargo` removed from `PATH`: `make test`, `make build` and `make check` exit non-zero with the rustup banner (FR-1.9, Gaps 1 and 15; evidence recorded).
- [ ] `make build` and `make check` exit 0, and `cargo deny check` passes inside `check` (FR-1.6, NFR-7).
- [ ] The Makefile defines build, test, test-integration, check, clean and help (FR-1.2).
- [ ] `sloperator check a.md` exits 0, and `sloperator check missing.md` exits 2 with a message naming `missing.md` (NFR-4).
- [ ] Everything needed to build and test the crate is inside the crate (NFR-1).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::cli_exit` | `help_and_version_exit_zero` | `--help` and `--version` exit 0 | FR-2.16 |
| 2 | `sloperator::cli_exit` | `exit_zero_clean` | `check` on a clean file exits 0 | FR-2.7 |
| 3 | `sloperator::cli_exit` | `exit_two_unknown_flag_names_it` | an unknown flag exits 2 and names the flag | FR-2.15 |

## Notes

Run the §23.2 Makefile checks by hand and paste their output into the PR. They are evidence, not test methods (SPEC §23.2). The JSON `exit_two_unknown_flag_names_it` relies on clap's error text; main maps every non-help/version clap error to 2.

Gate: `make test TESTS='sloperator::cli_exit#help_and_version_exit_zero sloperator::cli_exit#exit_zero_clean sloperator::cli_exit#exit_two_unknown_flag_names_it'`
