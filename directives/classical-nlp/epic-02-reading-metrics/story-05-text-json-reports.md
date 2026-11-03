# Story 05: Text and JSON report metrics, REFERENCE and CHANGELOG

**Epic:** [epic-02-reading-metrics](../epic-02-reading-metrics.md)
**SPEC Refs:** §12 (Text and JSON reports), §14 (REFERENCE: metrics), §18 (CHANGELOG binary size, Phase 1c)
**PRD Refs:** FR-2.8, FR-3.1, FR-3.2, FR-3.4, FR-3.5, FR-3.6, NFR-3, NFR-5, D-11
**Component / ADR:** Architecture §4.12 Report and MCP surfaces; ADR-05
**Wave:** 5

## Scope

`Summary` (src/report/mod.rs:35) gains `metrics: Metrics`, declared after `score` and before `errors`. `Summary::of(results, errors, floors: &Floors)` merges every file's counts and derives once with the run floors. The CLI (src/cli/mod.rs:229) passes `resolver.run_floors(&deps.cwd)?`. src/report/text.rs prints `metrics before` (Fix/Diff) and `metrics` lines after the score line, and `overall metrics` after the summary line. The 'unavailable' text names the floors used (`f.result.floors`; the run floors for the overall line). src/report/json.rs `File` gains `metrics` and `metricsBefore`, derived with `f.result.floors`. `concise` and `github` are untouched. docs/REFERENCE.md gains the 'Text metrics' subsection (§14, plus `[metrics]`). CHANGELOG.md gains `## 0.2.0 (unreleased)` with the metrics keys, `[metrics]` and the release-binary size before and after.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/report/mod.rs` | 14, 33-67 | `Summary.metrics`; `of(results, errors, floors)` |
| `src/report/text.rs` | 6-8, 32-35, new `metrics_text` | metrics lines |
| `src/report/json.rs` | 11, 25-32, 100-105 | `metrics`, `metricsBefore` |
| `src/cli/mod.rs` | 229 | `Summary::of(&results, errors, &floors)` with `resolver.run_floors(&deps.cwd)?` |
| `docs/REFERENCE.md` | after 'Score formula' (~421); 'Report' (~380) | Text metrics subsection and `[metrics]` config |
| `CHANGELOG.md` | top | `## 0.2.0 (unreleased)` with metrics bullets and binary size |
| `tests/report.rs` | existing | tests below (methods added) |
| `tests/text_metrics.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] The text report shows `<path>: metrics …` after each score line and `overall metrics …` after the summary line, and `summary_line` is byte-identical to before (FR-3.1)
- [ ] With available counts the line reads `avg sentence … words, grade …, sentence-length variation … (cv …), type/token …` (FR-3.1)
- [ ] A fix prints a `metrics before` line (FR-3.4)
- [ ] JSON `files[0].metrics` and `metricsBefore` are present, `score` is unchanged and `schemaVersion` is 1 (FR-3.2, D-11)
- [ ] `summary.metrics` is the merge of file counts, never a mean (FR-2.8)
- [ ] Two CLI runs of `check --output-format json` give identical bytes (NFR-3)
- [ ] With `[metrics] min_sentences = 10`, a 6-sentence file's `metrics` values are `null` in JSON, and its text line says `needs 10 sentences` (Gap 5 (a))
- [ ] `concise` and `github` contain no metrics text (FR-3.5)
- [ ] REFERENCE states the formulas, paragraph-only counting, the floors and `[metrics]`, the approximate syllables, the TTR length caveat, 'Metrics are style indicators, not a measure of quality or provenance.' and the keys (FR-3.6, FR-2.6)
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::report` | `text_shows_path_line_col_code_severity_message_source_line` | updated: metrics line after the score line, overall line after the summary (count 8) | FR-3.1 |
| 2 | `sloperator::report` | `text_metrics_line_lists_values` | available values formatting | FR-3.1 |
| 3 | `sloperator::report` | `text_fix_prints_metrics_before` | `metrics before` line | FR-3.4 |
| 4 | `sloperator::report` | `json_file_has_metrics_and_metrics_before` | per-file JSON keys; score shape unchanged | FR-3.2 |
| 5 | `sloperator::report` | `unavailable_text_names_configured_floors` | raised floors appear in the unavailable text and give JSON nulls | FR-2.7, D-5 |
| 6 | `sloperator::report` | `concise_and_github_print_no_metrics` | no metrics in either format | FR-3.5 |
| 7 | `sloperator::text_metrics` | `json_identical_across_runs` | two CLI runs give identical JSON bytes | NFR-3 |
| 8 | `sloperator::text_metrics` | `summary_uses_run_root_floors` | a CLI run in a dir whose `sloperator.toml` sets `[metrics] min_sentences = 10` gives `summary.metrics` values null for a 6-sentence file | FR-2.7, D-5 |

## Notes

The summary JSON assertion (tests/report.rs:166-168) gains a `metrics` object with every value `null`. Measure the binary size with `make build` before and after, and write both sizes into the CHANGELOG. The CLI tests in tests/text_metrics.rs follow tests/score.rs:82-99.

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='sloperator::report sloperator::text_metrics#json_identical_across_runs sloperator::text_metrics#summary_uses_run_root_floors'`
