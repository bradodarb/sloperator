# Story 01: `ParamSpec` min/max, config validation and `explain` bounds

**Epic:** [epic-03-metric-rules](../epic-03-metric-rules.md)
**SPEC Refs:** §15 (`ParamSpec` bounds, config check, `explain`)
**PRD Refs:** NFR-4, FR-4.2, FR-11.3
**Component / ADR:** Architecture §4.13 Config bounds; ADR-09
**Wave:** 1

## Scope

`ParamSpec` (src/rules/mod.rs:62) gains `min` and `max` (inclusive; `f64::MAX` = unbounded). The five existing literals (slp005:41, slp010:43, 48, slp011:39, slp014:38) gain `min: 0.0, max: f64::MAX`. `rule_tables` (src/config/mod.rs:484, before `o.params.insert` at :523) rejects NaN, below-min and above-max values through `config_err`. CLI `rules CODE` (src/cli/mod.rs:317-321) prints `(min …)` or `(min …, max …)` after the help text, and MCP `explain_rule` (src/mcp/mod.rs:257-259) adds `min` and `max` (`null` when unbounded). No new rule.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/rules/mod.rs` | 60-66 | `min`, `max` fields |
| `src/rules/builtin/slp005_rule_of_three.rs` | 41-45 | bounds on the literal |
| `src/rules/builtin/slp010_bullet_spam.rs` | 43-52 | bounds on both literals |
| `src/rules/builtin/slp011_over_bolding.rs` | 39-43 | bounds |
| `src/rules/builtin/slp014_em_dash_density.rs` | 38-42 | bounds |
| `src/config/mod.rs` | 512-523 | NaN/min/max checks |
| `src/cli/mod.rs` | 317-321 | bounds in `explain` output |
| `src/mcp/mod.rs` | 257-259 | `min`/`max` in `explain_rule` |
| `tests/config.rs` | existing | tests below (methods added) |
| `tests/rules_registry.rs` | existing | tests below (methods added) |
| `tests/cli_exit.rs` | existing | tests below (methods added) |
| `tests/mcp.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] `[rules.SLP014]\nmax_per_1000_words = -1` is `Error::Config` naming `rules.SLP014.max_per_1000_words` and the minimum (NFR-4)
- [ ] `max_per_1000_words = 1.0` (tests/config.rs:260) still loads (ADR-09)
- [ ] Every built-in `ParamSpec` has `min <= default <= max` (ADR-09)
- [ ] `sloperator rules SLP014` prints `(min 0)` after the param help, and MCP `explain_rule` returns `"min": 0.0, "max": null` (FR-11.3)
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::config` | `rejects_negative_threshold` | a below-min value is rejected with the key named | NFR-4 |
| 2 | `sloperator::config` | `existing_params_still_load` | today's configs still load | NFR-4 |
| 3 | `sloperator::rules_registry` | `param_defaults_within_bounds` | every ParamSpec has min <= default <= max | NFR-4 |
| 4 | `sloperator::cli_exit` | `rules_explain_shows_param_bounds` | `rules SLP014` prints the bounds | FR-11.3 |
| 5 | `sloperator::mcp` | `explain_rule_has_param_bounds` | `min`/`max` keys, null when unbounded | FR-11.3 |

## Notes

`rules_list_and_explain` (tests/cli_exit.rs:216) and `rules_and_explain_rule` (tests/mcp.rs:287) must still pass. `config_err(file, key, reason)` is at src/config/file.rs:148.

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='sloperator::config sloperator::rules_registry rules_explain_shows_param_bounds explain_rule_has_param_bounds'`
