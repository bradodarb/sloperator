# Story 06: `write`: atomic commit and stdin output

**Epic:** [epic-05-linter-surface](../epic-05-linter-surface.md)
**SPEC Refs:** §19, §23.15, DQ-10
**PRD Refs:** FR-2.6, NFR-5
**Component / ADR:** §4.13 write
**Wave:** 4

## Scope

Implement `src/write/mod.rs`: the `Fs` trait, `RealFs` (tempfile, sync, copy permissions, persist) and `commit`. Add `FailingFs` to `tests/common/mod.rs`.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/write/mod.rs` | new | Fs, RealFs, commit |
| `tests/common/mod.rs` | from story-03 | FailingFs (fails on Nth write) |
| `tests/write.rs` | new | tests below |

## Acceptance Criteria

- [ ] An unchanged file is not written, and a written file keeps its permissions (NFR-5).
- [ ] A write failure on the second file is `Error::Io` naming that path, and the first file is byte-identical to before, or fully replaced, never partial (NFR-5).
- [ ] With stdin, the fixed text goes to stdout (FR-2.6).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::write` | `unchanged_file_not_written` | no-op write skipped | NFR-5 |
| 2 | `sloperator::write` | `atomic_write_keeps_permissions` | permissions kept | NFR-5 |
| 3 | `sloperator::write` | `failure_on_second_file_leaves_files_byte_identical` | failure safety | NFR-5 |
| 4 | `sloperator::write` | `stdin_fix_prints_fixed_text` | stdin --fix | FR-2.6 |

## Notes

Story 03 and this story both touch `tests/common/mod.rs`, so they are in different waves (2 and 4).

Gate: `make test TESTS='sloperator::write#unchanged_file_not_written sloperator::write#atomic_write_keeps_permissions sloperator::write#failure_on_second_file_leaves_files_byte_identical sloperator::write#stdin_fix_prints_fixed_text'`
