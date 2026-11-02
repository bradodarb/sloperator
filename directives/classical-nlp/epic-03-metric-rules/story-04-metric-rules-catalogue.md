# Story 04: Metric-rule catalogue, CHANGELOG, perf config and cross-module tests

**Epic:** [epic-03-metric-rules](../epic-03-metric-rules.md)
**SPEC Refs:** §17 (design notes), §18 (CHANGELOG, catalogue), §35 (perf extension, Phase 1d), §36.4
**PRD Refs:** FR-4.4, FR-4.6, FR-11.1, FR-11.2, FR-11.3, NFR-2
**Component / ADR:** Architecture §4.9 Rule integration; §8 Performance budgets
**Wave:** 4

## Scope

CHANGELOG.md's `0.2.0 (unreleased)` section adds SLP018–SLP021 (code, name, severity, class, default), the `ParamSpec` bounds, and the metric-rule minimums bound by `[metrics]`. docs/REFERENCE.md's catalogue paragraph (~174-219) and docs/SPEC.md §10 table (~1013-1041) gain a row per rule with params. README.md's rule count changes if it states one. tests/perf.rs (:105-112) writes `extend_select = ["SLP018", "SLP019", "SLP020", "SLP021"]`. Add `metric_finding_never_selects_window` (tests/engine.rs) and `enabling_metric_rule_adds_suggestion_weight` (tests/pipeline.rs).

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `CHANGELOG.md` | 0.2.0 section | metric rules and bounds |
| `docs/REFERENCE.md` | 174-219 | catalogue rows SLP018–SLP021 |
| `docs/SPEC.md` | 1013-1041 | §10 rows |
| `README.md` | rule count, if stated | count updated |
| `tests/perf.rs` | 105-112 | `extend_select` with SLP018–SLP021 |
| `tests/engine.rs` | existing | tests below (methods added) |
| `tests/pipeline.rs` | existing | tests below (methods added) |
| `tests/standalone.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] An SLP018 finding produces no window plan (FR-4.4)
- [ ] With SLP018 enabled on a document that crosses its threshold, `score.weighted` is exactly 1 more than without it (FR-4.6)
- [ ] The perf test passes in its 1-second budget with the four rules on (NFR-2)
- [ ] CHANGELOG, REFERENCE and docs/SPEC.md §10 list the four rules with params and defaults (FR-11.2, FR-11.3)
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::engine` | `metric_finding_never_selects_window` | FixClass::None finding gives no window | FR-4.4 |
| 2 | `sloperator::pipeline` | `enabling_metric_rule_adds_suggestion_weight` | weighted grows by exactly 1 | FR-4.6 |
| 3 | `sloperator::perf` | `rules_only_100kb_under_1s` | budget holds with metric rules on | NFR-2 |
| 4 | `sloperator::standalone` | `no_host_references` | doc edits use the repo's own form | NFR-1 |

## Notes

The perf selector triggers the release-profile pass (Makefile `RUN_PERF`). `extend_select` is a top-level key (src/config/file.rs:18).

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='metric_finding_never_selects_window enabling_metric_rule_adds_suggestion_weight sloperator::perf sloperator::standalone#no_host_references'`
