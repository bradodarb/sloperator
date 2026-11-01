# Story 08: NFR-3 rules-only performance test (release profile)

**Epic:** [epic-05-linter-surface](../epic-05-linter-surface.md)
**SPEC Refs:** §23.18, §3 (perf pass)
**PRD Refs:** NFR-3
**Component / ADR:** architecture §8 performance budgets
**Wave:** 6

## Scope

Add `tests/perf.rs` with `rules_only_100kb_under_1s`. It generates a 100 KB Markdown file from the protect and rule fixtures, then runs the built binary with `check --rules-only` and a `sloperator.toml` whose `model.cache_dir` points at a `TempDir` with mode 000. No production code changes, unless the budget fails.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `tests/perf.rs` | new | tests below |

## Acceptance Criteria

- [ ] Deferred item 5 (Gap 12): the test sets `[model] cache_dir` to a directory with mode 000, and `check --rules-only` on 100 KB still exits by findings (0 or 1, never 2) in under 1 s wall time, which proves the cache is never read (NFR-3).
- [ ] The test runs only in the `ci-perf` release pass (`TESTS` naming `perf`), and its JUnit lands in `target/nextest/ci-perf/junit.xml` (§3).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::perf` | `rules_only_100kb_under_1s` | budget and no cache read | NFR-3 |

## Notes

Restore the directory mode in a drop guard so `TempDir` can clean up. Root runs ignore mode 000; the test skips with a printed reason when it runs as uid 0, and never passes silently.

Gate: `make test TESTS='sloperator::perf#rules_only_100kb_under_1s'`
