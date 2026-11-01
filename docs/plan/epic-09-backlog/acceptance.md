# Acceptance audit: epic-09-backlog (sloperator)

**Base:** 835d15eeffd9c8ac1a3c16cf5832d4771beced07 · **Verdict:** met · **Reviewer:** reviewer

## What changed

`git diff --stat <base>`: `tests/cli_exit.rs` (+20/-9) and directive/context docs only. No change under `src/`.

## Epic acceptance criteria

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `make test-integration` exits 0 even when `$TMPDIR/.gitignore` ignores everything (CH-18) | met | I ran it with `TMPDIR=<mktemp -d>/` holding `*` in `.gitignore`: exit 0, 7/7 passed, including `sloperator::standalone builds_when_extracted` (19.2s) |
| Story Test Cases pass together on the merged epic | met | The same ignore-all TMPDIR with `TESTS='sloperator::cli_exit'` gave 15/15 passed, covering all six named tests. The `standalone#builds_when_extracted` result is in the row above. The workflow's tests_output is PASS. |

## Story 01 acceptance criteria

| Criterion | Status | Evidence |
|-----------|--------|----------|
| The six `cli_exit` tests pass under an ignore-all TMPDIR, both in-tree and extracted | met | `fixture_copy` at `tests/cli_exit.rs:14-21` copies the fixture into `common::cli_tempdir()` (`tests/common/mod.rs:139-143`, which writes `!*`). The six tests use it at `:35, :58, :75, :93, :133/:141, :154/:170`. In-tree: 15/15 pass. Extracted: `builds_when_extracted`, which runs the crate's tests from the copy under TMPDIR, passes. |
| `builds_when_extracted` passes with TMPDIR's `.gitignore` = `*` | met | Integration run above |
| No assertion removed or weakened; `src/`, `tests/standalone.rs`, `tests/common/mod.rs` unchanged | met | The diff changes only how each fixture path is obtained, and every assertion line is identical. The file names are kept, so `bad_directive.md` still appears in stderr and the concise line still starts with the path that was passed. `git diff --quiet <base> -- src tests/standalone.rs tests/common/mod.rs` exits 0. |
| Every Test Case passes with the Gate | met | See the runs above. The workflow's discriminate_output is PASS. |

## Would the tests fail if the requirement regressed?

Yes. The Gate's extra leg recreates the defect's condition on any host: `builds_when_extracted` runs under a TMPDIR whose `.gitignore` is `*`. Revert `fixture_copy` and the extracted copy's fixture paths sit under the ignore-all TMPDIR again. The CLI then reports `skipped (excluded)` and the six exit-code assertions fail inside the extracted run (the D1 failure mode in the epic-08 audit). The in-tree runs alone would not catch a regression, because in-tree fixtures are not under TMPDIR. The extra leg is the part that can fail.

## Reachable as shipped

This epic is test-only and adds no production surface, so there is nothing to wire into the composition root.

## PRD

FR-2.7/2.8/2.9 (the exit-code contract, `--exit-zero`, `--min-severity`) are still asserted exactly as before. FR-1.7 (standalone build) is proven under the hostile TMPDIR.

## Gaps

None blocking.

- Housekeeping (not a gap in the built epic): `docs/backlog/cli-exit-tmpdir-gitignore.md` still exists, though story 01's Notes say it is resolved. Closing the backlog file is left to the plan/backlog owner. No new deferred item was filed, because the fix itself is complete.
