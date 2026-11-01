# Epic 01: Walking skeleton: the crate runs and the contract runs its tests

**Goal:** the crate builds a `sloperator` binary whose `check` command runs end to end (help, version, exit 0 on a clean file, exit 2 on a missing path), and `make test|build|check` runs it through the crate's Makefile.
**PRD Refs:** FR-1.1–FR-1.9, FR-2.7 (skeleton), FR-2.16, NFR-1, NFR-2, NFR-4, NFR-7, D-1, D-2, D-11
**SPEC Refs:** §0 (Phase 0 toolchain gate), §2 (Crate packaging), §3 (Makefile), §4 (Crate root and error type), §5 (Layering and standalone tests), §23.1, §23.2
**Modules:** `sloperator` (new crate)
**Status:** Delivered. Both stories merged: the crate, `error`/`cli` skeleton, the Makefile and `tests/common` (story 01), then `tests/layering.rs` and `tests/standalone.rs` (story 02). Later epics fill the stub modules under `src/`.

## Waves

Copied from this epic's line in `index.md`. Stories in one wave run in parallel; each wave waits for the one before.

| Wave | Stories | Why this order |
|------|---------|----------------|
| 1 | [story-01-crate-and-contract-adapter](./epic-01-walking-skeleton/story-01-crate-and-contract-adapter.md) | The crate, the binary and the Makefile must exist before anything can be tested through make. |
| 2 | [story-02-layering-and-standalone](./epic-01-walking-skeleton/story-02-layering-and-standalone.md) | The layering and standalone scans need the module tree from story 01, and the integration-tier leg needs the Makefile recipe. |

## Acceptance Criteria

- [ ] `make test` exits 0, and `target/nextest/ci/junit.xml` lists the tests (FR-1.3, FR-1.4).
- [ ] `make build` and `make check` exit 0 (FR-1.6, NFR-7).
- [ ] The Makefile defines build, test, test-integration, check, clean and help (FR-1.2).
- [ ] The built binary runs: `sloperator --help` and `--version` exit 0, and `sloperator check clean.md` (a file with no findings) exits 0 (walking skeleton).
- [ ] Everything needed to build and test the crate is inside the crate (NFR-1).
- [ ] The tests named in every story's Test Cases pass together on the merged epic.

## Notes

Phase 0 is a precondition, not a story: `cargo` 1.99.1 via rustup, `cargo-nextest` 0.9.150, `cargo-deny` 0.20.2, `clang` and `cmake` must all answer `--version` before story 01 starts (SPEC §0, architecture R1). Brad's Q7 answer is applied as SPEC §3 writes it: every make target fails with a one-line banner when cargo is absent. Distribution binaries (OD-1) are out of scope by Brad's decision: no packaging target.
