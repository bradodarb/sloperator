# Acceptance audit: sloperator / epic-01-walking-skeleton

Base: `14704b8d4af016d022c607d584983bc6816368a4` → HEAD (`bb0b6f1`). Reviewer: reviewer.

**Verdict: met.** There are no blocking gaps.

## Epic acceptance criteria

| Criterion | Evidence (run in this session) | Result |
|---|---|---|
| `make test` exits 0 and the JUnit XML lists the tests (FR-1.3, FR-1.4) | rc=0. the unit-tier JUnit XML holds 12 testcases: 4 from cli_exit, 5 from layering, 3 from standalone | met |
| `build` and `check` exit 0 (FR-1.6, NFR-7) | build=0, check=0. `cargo deny check` printed "advisories ok, bans ok, licenses ok, sources ok" | met |
| FR-1.2 build-target check | Withdrawn: applied only to the build setup the crate was developed under; the crate's own Makefile replaces it | n/a |
| The binary runs: `--help`/`--version` exit 0, and `check clean.md` exits 0 | `target/release/sloperator`: help=0, `--version` → "sloperator 0.1.0" with exit 0, `check fixtures/clean.md` → 0. `check missing.md` → 2 and the message names the path (`src/cli/mod.rs:49-62`, `src/error.rs:6-11`) | met |
| The diff touches only the crate (NFR-1) | neither story merge changed anything outside the crate | met (see note 1) |
| Every story's Test Cases pass together | Unit tier: 12/12 passed. Integration tier: `TESTS='sloperator::standalone#builds_when_extracted'` rc=0, and the integration-tier JUnit XML holds `builds_when_extracted` | met |

## Reachable as shipped

`src/main.rs:8-41` parses the `Cli`, builds the `Deps` with `engine::DisabledFactory`, and calls `cli::run`. `std::process::exit` is the single exit point. The stub modules (`config`, `doc`, `rules`, …) are declared empty by design. The epic's Status line and the `index.md` plan assign them to later epics. This epic delivers no unreachable code.

## Would the tests fail on regression?

- `cli_exit` asserts both the exact `ExitCode` and the text on stdout/stderr (`tests/cli_exit.rs:9-38`). If exit-code mapping or path reporting regressed, these tests would fail.
- `layering::forbidden_uses` and `pure_modules_have_no_io` also assert that every row matches at least one file (`tests/layering.rs:146,169`), so a renamed directory cannot let them pass with nothing checked. `single_exit_point` would fail on any second `std::process::exit`.
- `standalone::planted_reference_is_caught` plants a marker and asserts it is found exactly once (`tests/standalone.rs:78-102`).
- `builds_when_extracted` really builds the crate offline in a temp copy (`tests/standalone.rs:145-161`).
- The workflow's discriminate step reported PASS.

## Notes (non-blocking, ACCEPT)

1. **NFR-1 diff scope.** The only files changed outside the crate and its docs are three context docs. The workflow's record commit made those changes, not the stories. They are the repo's required context-map upkeep and add no code. ACCEPT: NFR-1 governs the crate's own footprint, and both story merges stayed inside it.
2. **Model-less build.** `RewriterFactory::preflight` and `DisabledFactory` are not exercised by any `check` path yet. That is expected: the skeleton's `check` does no rewriting, and epic 07 wires the runtime, as `src/main.rs:17` states. ACCEPT.

No items were deferred.
