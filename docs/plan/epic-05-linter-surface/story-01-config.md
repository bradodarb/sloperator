# Story 01: `config`: discovery, extend, per-file overrides, resolution

**Epic:** [epic-05-linter-surface](../epic-05-linter-surface.md)
**SPEC Refs:** §13, §23.9
**PRD Refs:** FR-7.1, FR-7.2, FR-7.3, FR-7.4, FR-7.5, FR-7.6, FR-2.11, D-8, D-9
**Component / ADR:** §4.2 config
**Wave:** 1

## Scope

Implement `src/config/{mod,file}.rs`: `ConfigFile`, `PerFile`, `ProtectFile`, `ModelFile` with `deny_unknown_fields`, `WindowMode`, `RuleConf`, `ModelSettings`, `Settings` (with `enabled(registry)`), `CliOverrides` and `Resolver` (`new`, `for_file`). Re-export `Format`/`ProtectSettings` from `doc`. The `config` CLI command is in story 07.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/config/mod.rs` | new | Settings, CliOverrides, Resolver |
| `src/config/file.rs` | new | serde file structs, extend chain |
| `fixtures/config/` | new | the §13 example and error cases |
| `tests/config.rs` | new | tests below |

## Acceptance Criteria

- [ ] The nearest `sloperator.toml` wins with no parent merge, `extend` chains, and a cycle is an error naming the files (FR-7.1).
- [ ] `--config` overrides discovery (FR-7.2).
- [ ] Every key in the §13 example parses (FR-7.3, FR-7.4).
- [ ] CLI beats file beats default (FR-7.5, FR-2.11).
- [ ] `per_file` globs override rule selection (FR-7.3).
- [ ] An unknown key, wrong type, unknown code, unknown param or bad regex is `Error::Config`/`UnknownRule` naming the file and key (FR-7.6, NFR-4).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::config` | `nearest_file_wins_no_parent_merge` | discovery | FR-7.1 |
| 2 | `sloperator::config` | `extend_chain_and_cycle_error` | extend | FR-7.1 |
| 3 | `sloperator::config` | `explicit_config_overrides_discovery` | --config | FR-7.2 |
| 4 | `sloperator::config` | `all_documented_keys_parse` | §13 example parses | FR-7.3, FR-7.4 |
| 5 | `sloperator::config` | `cli_beats_file_beats_default` | precedence | FR-7.5, FR-2.11 |
| 6 | `sloperator::config` | `per_file_glob_overrides` | per-file | FR-7.3 |
| 7 | `sloperator::config` | `unknown_key_wrong_type_unknown_code_unknown_param_bad_regex_exit_two` | config errors name file and key | FR-7.6, NFR-4 |

## Notes

`config` must not use pipeline, engine, report or write (layering).

Gate: `make test TESTS='sloperator::config#nearest_file_wins_no_parent_merge sloperator::config#extend_chain_and_cycle_error sloperator::config#explicit_config_overrides_discovery sloperator::config#all_documented_keys_parse sloperator::config#cli_beats_file_beats_default sloperator::config#per_file_glob_overrides sloperator::config#unknown_key_wrong_type_unknown_code_unknown_param_bad_regex_exit_two'`
