# Story 05: `report`: text, concise, json, github

**Epic:** [epic-05-linter-surface](../epic-05-linter-surface.md)
**SPEC Refs:** §18, §23.14
**PRD Refs:** FR-3.1, FR-3.2, FR-3.3, FR-3.4, FR-3.5, FR-3.6, FR-5.2, NFR-9
**Component / ADR:** §4.12 report
**Wave:** 4

## Scope

Implement `src/report/{mod,text,concise,json,github}.rs`: OutputFormat, Summary, `render` and `gh_escape`, as §18 specifies. The JSON carries `schemaVersion: 1` and a `path` on every finding (Gap 8).

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/report/mod.rs` | new | OutputFormat, Summary, render, gh_escape |
| `src/report/text.rs` | new | text with source line and caret |
| `src/report/concise.rs` | new | one line per finding |
| `src/report/json.rs` | new | camelCase schema v1 |
| `src/report/github.rs` | new | workflow commands |
| `tests/report.rs` | new | tests below |

## Acceptance Criteria

- [ ] Text shows path:line:col, code, severity, message and the source line; concise is one line per finding (FR-3.2).
- [ ] JSON matches the §18 shape, including `schemaVersion`, `scoreBefore` and per-finding `path` (FR-3.3, FR-5.2, NFR-9).
- [ ] GitHub annotations map suggestion→notice, warning→warning, error→error and escape data and properties (FR-3.4).
- [ ] No ANSI colour when `color` is false (FR-3.5).
- [ ] Output order is files by Origin, then findings in source order (FR-3.6).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::report` | `text_shows_path_line_col_code_severity_message_source_line` | text format | FR-3.2 |
| 2 | `sloperator::report` | `concise_one_line_per_finding` | concise format | FR-3.2 |
| 3 | `sloperator::report` | `json_shape_matches_schema` | JSON schema | FR-3.3, FR-5.2, NFR-9 |
| 4 | `sloperator::report` | `github_annotations_and_escaping` | github format | FR-3.4 |
| 5 | `sloperator::report` | `no_color_when_not_terminal` | no ANSI | FR-3.5 |
| 6 | `sloperator::report` | `stable_order` | ordering | FR-3.6 |

## Notes

`unknown_output_format_exits_two` needs clap and moves to story 07. DQ-16: no `anstream`.

Gate: `make test TESTS='sloperator::report#text_shows_path_line_col_code_severity_message_source_line sloperator::report#concise_one_line_per_finding sloperator::report#json_shape_matches_schema sloperator::report#github_annotations_and_escaping sloperator::report#no_color_when_not_terminal sloperator::report#stable_order'`
