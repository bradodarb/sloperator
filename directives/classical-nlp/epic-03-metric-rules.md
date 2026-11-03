# Epic 03: Default-off metric rules SLP018–SLP021 with bounded params

**Goal:** Metrics can feed the score through four default-off Suggestion rules, whose thresholds and minimums are validated against declared bounds and whose minimums never fall below the configured report floors.
**PRD Refs:** FR-4.1–FR-4.7, FR-11.1–FR-11.3, NFR-4, D-4, D-5
**SPEC Refs:** §15 (ParamSpec bounds, config check, explain), §16 (Family additions and metric-rule helpers), §17 (SLP018–SLP021), §18 (Rule-set version, CHANGELOG, catalogue), §35 (perf extension, Phase 1d), OD-3 option (a), §36.4
**Modules:** `sloperator`

## Waves

Copied from this epic's line in `index.md`. Stories in one wave run in parallel; each wave waits for the one before.

| Wave | Stories | Why this order |
|------|---------|----------------|
| 1 | [story-01-param-bounds](./epic-03-metric-rules/story-01-param-bounds.md) | bounds come first: every later ParamSpec literal needs `min`/`max` |
| 2 | [story-02-metric-rule-helpers](./epic-03-metric-rules/story-02-metric-rule-helpers.md) | helpers and SLP018 use `ParamSpec` bounds and touch src/rules/mod.rs and src/config/mod.rs after story-01 |
| 3 | [story-03-slp018-021-rules](./epic-03-metric-rules/story-03-slp018-021-rules.md) | SLP019–SLP021 reuse story-02's helpers and the macro arm; they touch builtin/mod.rs and rules_fixtures.rs after story-02 |
| 4 | [story-04-metric-rules-catalogue](./epic-03-metric-rules/story-04-metric-rules-catalogue.md) | the catalogue, perf config and cross-module tests need all four rules |

## Acceptance Criteria

- [ ] `sloperator check --extend-select SLP018` on a long, polysyllabic document reports one SLP018 Suggestion on the first prose run of the first paragraph; with defaults, no new finding appears (FR-4.1, D-4)
- [ ] Under the floors or the rule's own minimums, no metric rule fires (FR-4.3)
- [ ] A metric finding never selects a model window, and enabling SLP018 adds exactly 1 to `weighted` (FR-4.4, FR-4.6)
- [ ] A param outside `[min, max]`, or NaN, is a config error naming `rules.SLPxxx.<param>` (NFR-4)
- [ ] `[metrics] min_sentences = 10` makes SLP018's effective `min_sentences` 10, and an explicit `rules.SLP018.min_sentences = 6` under it is rejected (Gap 5 (a))
- [ ] Scores in every existing test are unchanged with default settings (phase-1d gate)
- [ ] `RULE_SET_VERSION` is "0.2.0", and CHANGELOG, REFERENCE and docs/SPEC.md §10 list SLP018–SLP021 with params (FR-11.1–FR-11.3)
- [ ] The tests named in every story's Test Cases pass together on the merged epic.

## Notes

Gap 5 (a) changes §16: `MIN_SENTENCES_PARAM`/`MIN_WORDS_PARAM` keep defaults 5/100 but get static `min: 1.0`. The lower bound that matters, the configured report floor, is applied when config resolves a file's settings (story-02). An explicit value below `Settings.metrics` is rejected, and an unset one takes the floor. So `document_metrics` derives with `Floors { sentences: param, words: param }`, which is always ≥ the report floor, and a rule never fires on a metric the report shows as unavailable (FR-4.3). Metric rules are recognised by `Family::Readability`, not by param name, because SLP023 (epic-09) has its own `min_sentences` with another meaning. Thresholds are provisional (D-4); epic-04 records calibration values but changes no default.
