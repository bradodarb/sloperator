# Story 01: `cli_exit` tests do not depend on the host's temp-dir ignore files

**Epic:** [epic-09-backlog](../epic-09-backlog.md)
**SPEC Refs:** §5, §23.16
**PRD Refs:** FR-2.7, FR-2.8, FR-2.9
**Wave:** 1

## Scope

Fix for epic-08 acceptance audit D1 (changes.md CH-18). `standalone#builds_when_extracted` (`tests/standalone.rs:145-161`) copies the crate under `$TMPDIR` and runs `cargo test` there. When `$TMPDIR/.gitignore` contains `*`, the CLI's gitignore handling (DQ-13, `src/input/mod.rs:199`) reports `sloperator: skipped (excluded)` for the fixture paths that six `cli_exit` tests pass. Those paths resolve under the extracted copy, inside `$TMPDIR`. The six tests are `exit_zero_clean` (`tests/cli_exit.rs:21`), `exit_one_on_warning` (`:44`), `exit_zero_on_suggestion_only` (`:61`), `exit_two_on_directive_error` (`:79`), `exit_zero_flag_masks_findings_not_errors` (`:119`) and `min_severity_filters_report_and_exit` (`:140`). Add a helper in `tests/cli_exit.rs` that copies a fixture, keeping its file name, into a `common::cli_tempdir()` (`tests/common/mod.rs:139-143`, which writes `!*` into `.gitignore`). These six tests then check the copy instead of `fixture(...)` directly. Keep every assertion as it is: `exit_two_on_directive_error` still expects `bad_directive.md` in stderr, and `exit_one_on_warning` still expects the line to start with the path it passed. Do not change `src/`, `tests/standalone.rs` or `tests/common/mod.rs`.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `tests/cli_exit.rs` | ~1-180 | copy-fixture-into-cli_tempdir helper; the six tests above use it |
| `tests/standalone.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] The six `cli_exit` tests listed in Scope pass under a `TMPDIR` whose `.gitignore` is `*`, both in-tree and inside the extracted copy.
- [ ] `sloperator::standalone#builds_when_extracted` passes with `TMPDIR` pointing at a directory whose `.gitignore` is `*` (the extra Gate leg).
- [ ] No assertion in `tests/cli_exit.rs` is removed or weakened. `git diff` shows no change under `src/`, nor to `tests/standalone.rs` or `tests/common/mod.rs`.
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::cli_exit` | `exit_zero_clean` | clean input exits 0 when the fixture is checked from a cli_tempdir | FR-2.7 |
| 2 | `sloperator::cli_exit` | `exit_one_on_warning` | a warning exits 1 with the concise line for the path passed | FR-2.7 |
| 3 | `sloperator::cli_exit` | `exit_zero_on_suggestion_only` | suggestion-only input exits 0 and prints nothing | FR-2.7 |
| 4 | `sloperator::cli_exit` | `exit_two_on_directive_error` | a directive error exits 2 and names bad_directive.md | FR-6.5 |
| 5 | `sloperator::cli_exit` | `exit_zero_flag_masks_findings_not_errors` | --exit-zero masks findings, not errors | FR-2.8 |
| 6 | `sloperator::cli_exit` | `min_severity_filters_report_and_exit` | --min-severity filters both the report and exit 1 | FR-2.9 |
| 7 | `sloperator::standalone` | `builds_when_extracted` | the extracted crate's tests pass (run again by the extra leg under an ignore-all TMPDIR) | FR-1.7, NFR-7 |

## Notes

Test-only. The backlog file `docs/backlog/cli-exit-tmpdir-gitignore.md` is resolved by this story. On the dev host the default `TMPDIR` already carries the ignore-all file. The extra leg recreates that condition on any host, so the Gate proves the fix rather than depending on the environment.

Gate: `make test TESTS='sloperator::cli_exit#exit_zero_clean sloperator::cli_exit#exit_one_on_warning sloperator::cli_exit#exit_zero_on_suggestion_only sloperator::cli_exit#exit_two_on_directive_error sloperator::cli_exit#exit_zero_flag_masks_findings_not_errors sloperator::cli_exit#min_severity_filters_report_and_exit' && make test-integration TESTS='sloperator::standalone#builds_when_extracted' && d=$(mktemp -d) && printf '*\n' > "$d/.gitignore" && TMPDIR="$d/" make test-integration TESTS='sloperator::standalone#builds_when_extracted' && git diff --quiet -- src tests/standalone.rs tests/common/mod.rs`
