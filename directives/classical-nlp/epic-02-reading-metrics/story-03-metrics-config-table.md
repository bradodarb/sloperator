# Story 03: `[metrics]` config table resolved into `Settings.metrics`

**Epic:** [epic-02-reading-metrics](../epic-02-reading-metrics.md)
**SPEC Refs:** OD-3 option (a) (Brad's Gap 5 decision); §15 for the error form
**PRD Refs:** FR-2.7, D-5, NFR-4
**Component / ADR:** Architecture §6 configuration (`Settings`), §4.13 Config bounds
**Wave:** 3

## Scope

Add a top-level `[metrics]` table to the config file schema (src/config/file.rs: `ConfigFile` at :15 and the key list near :180) with `min_sentences` and `min_words`. Both are integers ≥ 1; any other key, a non-integer or a value below 1 is `Error::Config` naming `metrics.<key>`. `Settings` (src/config/mod.rs:55) gains `pub metrics: text::metrics::Floors`, resolved from the file's config chain like the other top-level settings, defaulting to `Floors::default()`. `[metrics]` is not allowed in `[[per_file]]`. Add `Resolver::run_floors(&mut self, cwd: &Path) -> Result<Floors, Error>`: the `--config` chain's floors when one was given, else the nearest `sloperator.toml` up from `cwd`, else defaults. epic-02 story-05 uses it for the summary. No report change here.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/config/file.rs` | 15-31, ~170-215 | `metrics: Option<MetricsFile>`, key validation |
| `src/config/mod.rs` | 55-66, ~200-260 | `Settings.metrics`, resolution, `Resolver::run_floors` |
| `tests/config.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] With no config, `Settings.metrics == Floors { sentences: 5, words: 100 }` (D-5)
- [ ] `[metrics]\nmin_sentences = 10\nmin_words = 200` resolves to 10/200 for every file under that config (FR-2.7)
- [ ] `min_sentences = 0`, `min_words = -5`, `min_words = 1.5` and an unknown key `max = 3` are each rejected with a message naming `metrics.<key>` (NFR-4)
- [ ] `run_floors` returns the `--config` file's floors when given, else those of the nearest config above the working directory, else defaults
- [ ] Every existing test in tests/config.rs passes
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::config` | `metrics_floors_default_to_5_and_100` | defaults with no `[metrics]` | D-5 |
| 2 | `sloperator::config` | `metrics_floors_load_from_table` | `[metrics]` values reach `Settings.metrics` | FR-2.7, D-5 |
| 3 | `sloperator::config` | `metrics_floors_reject_invalid_values` | 0, negative, fractional and unknown keys give `Error::Config` naming the key | NFR-4 |
| 4 | `sloperator::config` | `metrics_floors_run_root` | `run_floors` uses `--config`, else the nearest config above cwd, else defaults | FR-2.7 |

## Notes

Follow `config_err(file, key, reason)` (src/config/file.rs:148) for messages. Settings is `serde::Serialize`, so `Floors` must derive Serialize (story-02).

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='metrics_floors'`
