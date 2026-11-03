# Story 06: MCP `metrics`, `metricsBefore`, `metricsAfter`

**Epic:** [epic-02-reading-metrics](../epic-02-reading-metrics.md)
**SPEC Refs:** §13 (MCP results)
**PRD Refs:** FR-3.3
**Component / ADR:** Architecture §4.12 Report and MCP surfaces; ADR-05
**Wave:** 5

## Scope

MCP `lint` (src/mcp/mod.rs:109-116) adds `"metrics": r.metrics.derive(&r.floors)`. `fix_json` (:207-220) adds `"metricsBefore"` and `"metricsAfter"` after `scoreAfter`. docs/MCP.md lists the keys in the lint and fix rows and in its example. No other tool output changes.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/mcp/mod.rs` | 109-116, 207-220 | metrics keys |
| `docs/MCP.md` | 16, 40-50 | keys in the tables and the example |
| `tests/mcp.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] MCP `lint` returns a `metrics` object with the camelCase keys of `Metrics` (FR-3.3)
- [ ] MCP `fix` returns `metricsBefore` and `metricsAfter` (FR-3.3)
- [ ] Every existing MCP test passes
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::mcp` | `lint_has_metrics` | the lint result carries metrics | FR-3.3 |
| 2 | `sloperator::mcp` | `fix_has_metrics_before_and_after` | the fix result carries both | FR-3.3 |

## Notes

MCP resolves settings per source (src/mcp/mod.rs:164), so each result uses its own file's floors.

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='sloperator::mcp'`
