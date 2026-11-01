# Story 03: Directives: extract, regions, ignore-file, plain text

**Epic:** [epic-04-rules-score-directives](../epic-04-rules-score-directives.md)
**SPEC Refs:** §12, §23.8
**PRD Refs:** FR-6.1, FR-6.2, FR-6.3, FR-6.4, FR-6.5, FR-6.7, NFR-4
**Component / ADR:** §4.5 directives
**Wave:** 2

## Scope

Implement `directives::extract` and `MD_DIRECTIVE`/`TXT_DIRECTIVE` in `src/directives/mod.rs`. Story 01 added the `Suppressions` struct; this story fills the module. A Markdown directive counts only when one HTML span covers the whole line (Gap 6). Directive lines are re-tagged `Protected(Directive)`.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/directives/mod.rs` | from story-01 | extract, regexes, semantics |
| `tests/directives.rs` | new | tests below |

## Acceptance Criteria

- [ ] `ignore` covers the next line that holds prose, for the named codes or all (FR-6.1).
- [ ] `disable`/`enable` regions suppress in between; an unclosed `disable` is `Error::Directive` with the line (FR-6.2, FR-6.3).
- [ ] `ignore-file` suppresses everything and freezes the whole document (FR-6.4).
- [ ] An unknown or system code is `Error::UnknownRule` with `path:line` (FR-6.5).
- [ ] A directive-shaped line inside a fenced block, an indented block or inline code has no effect (Gap 6).
- [ ] `[sloperator-ignore]` lines work in plain text (FR-6.7).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::directives` | `ignore_next_line_named_and_all` | ignore directive | FR-6.1 |
| 2 | `sloperator::directives` | `disable_enable_region` | regions | FR-6.2 |
| 3 | `sloperator::directives` | `unclosed_disable_exits_two_with_line` | unclosed region error names the line | FR-6.3, NFR-4 |
| 4 | `sloperator::directives` | `ignore_file` | ignore-file | FR-6.4 |
| 5 | `sloperator::directives` | `unknown_code_exits_two_with_line` | unknown code error | FR-6.5 |
| 6 | `sloperator::directives` | `system_code_rejected` | SLP901 in a directive is unknown | FR-6.5 |
| 7 | `sloperator::directives` | `plain_text_bracket_directives` | plain-text form | FR-6.7 |
| 8 | `sloperator::directives` | `directive_in_code_fence_ignored` | fenced/indented directive has no effect | FR-6.3, FR-8.1 (Gap 6) |
| 9 | `sloperator::directives` | `directive_in_inline_code_ignored` | inline-code directive has no effect | FR-6.1, FR-8.1 (Gap 6) |

## Notes

These tests call `extract` directly with an empty or test registry built from the core. Where the SPEC's test asserts "exits two", assert on the `Error` variant here; epic 05 story-07 checks the exit code through the CLI. `suppressed_region_not_edited_by_either_engine` needs the model path and lands in epic 06 story-03.

Gate: `make test TESTS='sloperator::directives#ignore_next_line_named_and_all sloperator::directives#disable_enable_region sloperator::directives#unclosed_disable_exits_two_with_line sloperator::directives#ignore_file sloperator::directives#unknown_code_exits_two_with_line sloperator::directives#system_code_rejected sloperator::directives#plain_text_bracket_directives sloperator::directives#directive_in_code_fence_ignored sloperator::directives#directive_in_inline_code_ignored'`
