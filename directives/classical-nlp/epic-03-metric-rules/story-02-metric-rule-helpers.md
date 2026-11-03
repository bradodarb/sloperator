# Story 02: `Family` additions, metric-rule helpers, floor-bounded minimums and SLP018

**Epic:** [epic-03-metric-rules](../epic-03-metric-rules.md)
**SPEC Refs:** §16 (`Family` additions and metric-rule helpers), §17 (SLP018 in full), §18 (`RULE_SET_VERSION`), OD-3 option (a)
**PRD Refs:** FR-4.1, FR-4.2, FR-4.3, FR-4.5, FR-11.1, D-5
**Component / ADR:** Architecture §4.9 Rule integration, §4.13 Config bounds; ADR-09
**Wave:** 2

## Scope

`Family` (src/rules/mod.rs:48) gains `Readability`, `Wordiness` and `Redundancy` (kebab-case). `RULE_SET_VERSION` (:146) becomes "0.2.0". src/rules/helpers.rs gains `MIN_SENTENCES_PARAM`/`MIN_WORDS_PARAM` (defaults 5/100, static `min: 1.0`), `document_metrics(ctx)` (derives with `Floors { sentences: param, words: param }`), and `metric_anchor(doc)`. Config resolution (src/config/mod.rs) applies the report floors to every enabled `Family::Readability` rule after all layers merge: an unset `min_sentences`/`min_words` takes `Settings.metrics`, and an explicit value below it is `Error::Config` naming `rules.SLPxxx.min_sentences` and the floor. Add SLP018 `grade-level` exactly as §17 (new file, builtin/mod.rs entry, `fixtures/rules/SLP018/positive.md` and `negative.md`, macro entry under `model(…)`).

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/rules/mod.rs` | 46-58, 146 | `Family` variants; `RULE_SET_VERSION = "0.2.0"` |
| `src/rules/helpers.rs` | after 211 | metric params, `document_metrics`, `metric_anchor` |
| `src/config/mod.rs` | Settings resolution (~200-300) | floor-bounded metric-rule minimums |
| `src/rules/builtin/slp018_grade_level.rs` | new | SLP018 |
| `src/rules/builtin/mod.rs` | 5-21, 23-43 | `mod slp018_grade_level;` and its `rule()` |
| `fixtures/rules/SLP018/positive.md` | new | a paragraph run over the floors with grade > 12 |
| `fixtures/rules/SLP018/negative.md` | new | a plain paragraph run plus a fenced block, inline code and a link |
| `tests/rules_fixtures.rs` | 135-198 | `model(18, slp018_positive_fires, slp018_negative_silent)` |
| `tests/config.rs` | existing | tests below (methods added) |
| `tests/rules_registry.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] SLP018 is `Readability`, `Suggestion`, `default_on: false`, `FixClass::None`, `since: "0.2.0"`, and its `explain` contains "provisional" (FR-4.1, FR-4.5)
- [ ] SLP018 fires on positive.md and is silent on negative.md (FR-4.7)
- [ ] A 3-sentence document with a high grade gives no SLP018 finding (FR-4.3)
- [ ] `[rules.SLP018]\nmin_sentences = 0` is rejected naming `rules.SLP018.min_sentences` (NFR-4)
- [ ] With `[metrics] min_sentences = 10`, SLP018's resolved `min_sentences` is 10, and `[rules.SLP018] min_sentences = 6` in the same file is rejected naming the floor (Gap 5 (a), D-5)
- [ ] `max_grade = -1` and `max_grade = 31` are rejected (NFR-4)
- [ ] `default_set_is_explicit` and `new_rules_start_off` pass unchanged (FR-11.1)
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::rules_fixtures` | `slp018_positive_fires` | fires on positive.md | FR-4.7 |
| 2 | `sloperator::rules_fixtures` | `slp018_negative_silent` | silent on negative.md | FR-4.7 |
| 3 | `sloperator::rules_fixtures` | `metric_rule_silent_when_unavailable` | 3 sentences, high grade, no finding | FR-4.3 |
| 4 | `sloperator::config` | `rejects_param_below_minimum` | `min_sentences = 0` rejected with key and code named | NFR-4 |
| 5 | `sloperator::config` | `rejects_param_above_maximum` | `max_grade = 31` rejected | NFR-4 |
| 6 | `sloperator::config` | `metric_rule_minimums_follow_report_floors` | unset minimum takes `[metrics]`; an explicit one below it is rejected | D-5, FR-4.3 |
| 7 | `sloperator::rules_registry` | `(all existing, incl. new_rules_start_off, every_rule_has_fixtures)` | registry invariants hold with SLP018 | FR-11.1, FR-11.3 |

## Notes

`check_negative` needs a fenced block, inline code and a link in negative.md (tests/rules_fixtures.rs:86-98). The `model(…)` arm asserts the class is not Safe (:123-132). Comparisons use rounded values (NFR-3).

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='slp018_ metric_rule_silent_when_unavailable rejects_param_below_minimum rejects_param_above_maximum metric_rule_minimums_follow_report_floors sloperator::rules_registry'`
