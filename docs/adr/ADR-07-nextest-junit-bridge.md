# ADR-07: nextest for test runs and JUnit reports

**Status:** Accepted
**Date:** 2026-10-29
**Initiative:** [sloperator](../ARCHITECTURE.md) · **Serves:** D-11, FR-1.3-1.7, FR-1.9

## Context

The repository's `Makefile` is the one entry point for building, testing and checking the crate (D-11): targets
`build`, `test`, `test-integration`, `check`, `clean` and `help` (FR-1.2). Test runs must produce machine-readable
results that CI and reviewers can read (FR-1.4), a selector must narrow a run to named tests, and a selector that
matches nothing must fail rather than pass with zero tests (FR-1.5). A missing tool must fail loudly with a one-line
hint, never silently (FR-1.9).

## Options

### Option 1: cargo-nextest with JUnit profiles, wrapped by the Makefile

`.config/nextest.toml` defines three profiles: `ci` (unit tier), `ci-perf` (the NFR-3 benchmark, run with
`--cargo-profile release` in its own `--target-dir target/perf`) and `it` (integration tier, one test at a time).
Each writes JUnit XML to `target/nextest/<profile>/junit.xml`. `make test TESTS='<selector> ...'` turns the selector
words into one nextest filterset: a `binary_id#test` word becomes an exact `binary_id(=…)&test(=…)`, a bare word a
substring of the test name (and of the binary id when it occurs in one), joined with `or` (SPEC §3). **For:** real
JUnit from a maintained tool; nextest's `binary_id#test` ids round-trip into selectors; exit codes pass through.
**Against:** one extra installed tool (`cargo-nextest`).

### Option 2: Plain `cargo test` plus a converter (libtest JSON or text → JUnit)

**For:** no extra tool. **Against:** libtest's JSON output is nightly-only (`-Z unstable-options`). Parsing text
output is fragile and would be a hand-written converter to maintain.

### Option 3: Plain `cargo test` with no JUnit

**For:** nothing to install. **Against:** no machine-readable results, and no reliable way to tell that a filter
matched nothing, which FR-1.5 requires to fail.

## Missing-tool guard and empty selectors

Each recipe first checks that its tools (`cargo`; `cargo-nextest` for the test targets; `cargo-deny` for `check`)
are on `PATH`. A missing tool prints one line, `sloperator: '<tool>' not found on PATH — <install hint>`, and the
target exits 1 (FR-1.9). After the nextest runs, the recipe counts `<testcase` entries in the JUnit files it just
wrote; zero means the selector matched nothing, and the target fails with `sloperator: no test ran` (FR-1.5).

## Decision

Option 1. nextest runs every test tier, writes JUnit XML to `target/nextest/<profile>/junit.xml`, and the `Makefile`
wraps it with the selector mapping, the missing-tool guard and the empty-run check.

## Consequences

- **Easier:** CI (`.github/workflows/ci.yml`: `make check`, `make build`, `make test`) and local runs use the same
  targets; the JUnit files sit under `target/`, which `.gitignore` already excludes; selectors work.
- **Harder:** contributors install `cargo-nextest` (named by the banner). JUnit `classname` is the nextest binary
  id, so story gates name tests as `<binary_id>#<test name>` or as a bare substring.
- **Follow-ups:** none.
