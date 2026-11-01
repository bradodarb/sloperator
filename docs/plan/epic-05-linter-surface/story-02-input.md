# Story 02: `input`: paths, globs, stdin, ignore rules, UTF-8

**Epic:** [epic-05-linter-surface](../epic-05-linter-surface.md)
**SPEC Refs:** §14, §23.10
**PRD Refs:** FR-2.2, FR-2.3, FR-2.4, FR-8.8, FR-3.6, NFR-4, NFR-6
**Component / ADR:** §4.3 input
**Wave:** 2

## Scope

Implement `src/input/mod.rs`: `Origin`, `SourceRef`, `Source`, `resolve`, `read`, `format_for`, with the `ignore` walker defaults pinned as §14 specifies (Gap 5). Output is sorted by `Origin`.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/input/mod.rs` | new | resolve, read, format_for |
| `tests/input.rs` | new | tests below |

## Acceptance Criteria

- [ ] Files, directories, globs and `-` resolve; no arguments means the cwd (FR-2.2).
- [ ] `--stdin-filename` selects format and config; stdin defaults to Markdown (FR-2.3).
- [ ] `.gitignore` and `exclude` are honoured, and `respect_gitignore = false` turns that off (FR-2.4).
- [ ] Hidden directories are walked, and `.ignore` and the global excludes file are not read; `.gitignore` applies outside a git repo (Gap 5).
- [ ] A missing path or empty glob is exit-2 `Error::Io`; invalid UTF-8 is `Error::NotUtf8` naming the byte offset, and the file is untouched (NFR-4, FR-8.8).
- [ ] The order is independent of walk order (FR-3.6, NFR-6).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::input` | `files_dirs_globs_and_stdin` | input forms | FR-2.2 |
| 2 | `sloperator::input` | `no_args_checks_cwd` | default cwd | FR-2.2 |
| 3 | `sloperator::input` | `stdin_filename_selects_format_and_config` | stdin filename | FR-2.3 |
| 4 | `sloperator::input` | `stdin_defaults_markdown` | stdin default | FR-2.3 |
| 5 | `sloperator::input` | `gitignore_and_exclude_skipped_respect_gitignore_false` | ignore rules | FR-2.4 |
| 6 | `sloperator::input` | `hidden_dir_walked_dot_ignore_and_global_ignore_not_read` | pinned walker defaults | FR-2.2, FR-2.4 (Gap 5) |
| 7 | `sloperator::input` | `gitignore_applies_outside_git_repo` | .gitignore without .git | FR-2.4 (Gap 5) |
| 8 | `sloperator::input` | `missing_path_exits_two` | missing path error | NFR-4 |
| 9 | `sloperator::input` | `non_utf8_exits_two_unmodified` | UTF-8 error, file unchanged | FR-8.8 |
| 10 | `sloperator::input` | `order_independent_of_walk_order` | stable order | FR-3.6, NFR-6 |

## Notes

DQ-13: an explicit file matching `exclude`/`.gitignore` is skipped with a note on stderr.

Gate: `make test TESTS='sloperator::input#files_dirs_globs_and_stdin sloperator::input#no_args_checks_cwd sloperator::input#stdin_filename_selects_format_and_config sloperator::input#stdin_defaults_markdown sloperator::input#gitignore_and_exclude_skipped_respect_gitignore_false sloperator::input#hidden_dir_walked_dot_ignore_and_global_ignore_not_read sloperator::input#gitignore_applies_outside_git_repo sloperator::input#missing_path_exits_two sloperator::input#non_utf8_exits_two_unmodified sloperator::input#order_independent_of_walk_order'`
