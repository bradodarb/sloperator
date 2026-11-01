# Story 07: The full CLI: check flags, rules, config, exit codes, e2e

**Epic:** [epic-05-linter-surface](../epic-05-linter-surface.md)
**SPEC Refs:** §20, §13 (config command), §23.16, §23.17
**PRD Refs:** FR-2.1, FR-2.5, FR-2.7, FR-2.8, FR-2.9, FR-2.10, FR-2.12, FR-2.13, FR-2.15, FR-3.1, FR-3.5, FR-9.6, NFR-4, NFR-6, NFR-8, D-3, D-8, Brad Q6
**Component / ADR:** §4.1 cli; ADR-09
**Wave:** 5

## Scope

Replace the skeleton `check` in `src/cli/{mod,args}.rs` with the §20 version: CheckArgs, steps 1-10 with parallel analyse and serial fix, Diff/Fix/stdin routing (DQ-11), and the exit-code rule. Add `rules [CODE]` and `config [PATH]`. Update `src/main.rs` with the colour detection. Add `tests/e2e.rs` (assert_cmd).

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/cli/mod.rs` | from epic-04 story-08 | full run(Check), rules, config |
| `src/cli/args.rs` | from epic-01 story-01 | full Cli/Command/CheckArgs |
| `src/main.rs` | from epic-01 story-01 | colour detection, Deps |
| `tests/cli_exit.rs` | from epic-04 story-08 | new exit tests |
| `tests/report.rs` | from story-05 | unknown_output_format_exits_two |
| `tests/config.rs` | from story-01 | config_command_prints_resolved_json |
| `README.md` | from epic-04 story-04 | Usage section: commands, flags, exit codes, formats, directives, config reference |
| `tests/e2e.rs` | new | tests below |

## Acceptance Criteria

- [ ] Exit 2 on a bad config, an unknown rule flag or an unknown flag, with the offending name (FR-2.7, FR-2.15).
- [ ] `--exit-zero` forces 0 on findings but not on errors (FR-2.8).
- [ ] `--min-severity` filters both the report and the exit code (FR-2.9).
- [ ] `--rules-only` with `--model-only`, and `--fix` with `--diff`, exit 2 (FR-2.10, DQ-11).
- [ ] `rules` lists every rule and `rules SLP001` explains it (FR-2.12).
- [ ] `config PATH` prints the resolved settings as snake_case JSON (FR-2.13, DQ-9).
- [ ] e2e: protected bytes of `mixed.md` are unchanged after `--fix --rules-only`; `--diff` and plain `check` write nothing; JSON is byte-identical across runs and shuffled directories; `--fix` without a model exits 2 (NFR-8, FR-2.5, FR-2.1, NFR-6, FR-9.6).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::cli_exit` | `exit_two_on_bad_config` | bad config → 2 | FR-2.7 |
| 2 | `sloperator::cli_exit` | `exit_two_unknown_rule_flag` | unknown --select code → 2 | FR-2.15 |
| 3 | `sloperator::cli_exit` | `exit_zero_flag_masks_findings_not_errors` | --exit-zero | FR-2.8 |
| 4 | `sloperator::cli_exit` | `min_severity_filters_report_and_exit` | --min-severity | FR-2.9 |
| 5 | `sloperator::cli_exit` | `rules_only_and_model_only_together_exit_two` | engine flag conflict | FR-2.10 |
| 6 | `sloperator::cli_exit` | `rules_list_and_explain` | rules command | FR-2.12 |
| 7 | `sloperator::cli_exit` | `fix_and_diff_together_exit_two` | --fix --diff → 2 | DQ-11 |
| 8 | `sloperator::report` | `unknown_output_format_exits_two` | bad --output-format → 2 | FR-3.1 |
| 9 | `sloperator::config` | `config_command_prints_resolved_json` | config command | FR-2.13 |
| 10 | `sloperator::e2e` | `mixed_fixture_protected_bytes_unchanged_after_rules_fix` | protection end to end | NFR-8, FR-8.1, FR-8.4 |
| 11 | `sloperator::e2e` | `diff_writes_nothing` | --diff writes nothing | FR-2.5 |
| 12 | `sloperator::e2e` | `check_modifies_nothing` | check is read-only | FR-2.1 |
| 13 | `sloperator::e2e` | `json_identical_across_runs_and_shuffled_dirs` | determinism | NFR-6 |
| 14 | `sloperator::e2e` | `fix_without_model_exits_two` | missing model → 2 before any write | FR-9.6, NFR-4 |

## Notes

`-v`/`-q` behave as §20's design notes say. The unit tier builds without feature `model`, so the e2e binary uses DisabledFactory, and `fix_without_model_exits_two` exits 2 whichever check fails first.

Gate: `make test TESTS='sloperator::cli_exit#exit_two_on_bad_config sloperator::cli_exit#exit_two_unknown_rule_flag sloperator::cli_exit#exit_zero_flag_masks_findings_not_errors sloperator::cli_exit#min_severity_filters_report_and_exit sloperator::cli_exit#rules_only_and_model_only_together_exit_two sloperator::cli_exit#rules_list_and_explain sloperator::cli_exit#fix_and_diff_together_exit_two sloperator::report#unknown_output_format_exits_two sloperator::config#config_command_prints_resolved_json sloperator::e2e#mixed_fixture_protected_bytes_unchanged_after_rules_fix sloperator::e2e#diff_writes_nothing sloperator::e2e#check_modifies_nothing sloperator::e2e#json_identical_across_runs_and_shuffled_dirs sloperator::e2e#fix_without_model_exits_two'`
