# Story 03: SLP019, SLP020, SLP021 metric rules and fixtures

**Epic:** [epic-03-metric-rules](../epic-03-metric-rules.md)
**SPEC Refs:** §17 (SLP018–SLP021 table rows)
**PRD Refs:** FR-4.1, FR-4.2, FR-4.5, FR-4.7, FR-11.2, D-3
**Component / ADR:** Architecture §4.9 Rule integration (rows SLP019–SLP021)
**Wave:** 3

## Scope

Add SLP019 `sentence-length-variation` (`min_cv` 0.35, 0, 5; fires when cv < min), SLP020 `long-sentences` (`max_avg_words` 25, 1, 1000; fires when avg > max) and SLP021 `lexical-diversity` (`min_ttr` 0.4, 0, 1; fires when TTR < min), each shaped like SLP018 with the §17 messages. Each gets a builtin/mod.rs entry, positive/negative fixtures and a macro entry under `model(…)`. SLP019's `explain` says it measures sentence-length variation, not perplexity-based burstiness (D-3).

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/rules/builtin/slp019_sentence_length_variation.rs` | new | SLP019 |
| `src/rules/builtin/slp020_long_sentences.rs` | new | SLP020 |
| `src/rules/builtin/slp021_lexical_diversity.rs` | new | SLP021 |
| `src/rules/builtin/mod.rs` | module list and `rule()` list | three entries in code order |
| `fixtures/rules/SLP019/positive.md, negative.md` | new | fixtures |
| `fixtures/rules/SLP020/positive.md, negative.md` | new | fixtures |
| `fixtures/rules/SLP021/positive.md, negative.md` | new | fixtures |
| `tests/rules_fixtures.rs` | 135-198 | three `model(…)` entries |
| `tests/rules_registry.rs` | existing | tests below (methods added) |
| `tests/config.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] Each of SLP019–SLP021 fires on its positive.md and is silent on its negative.md (FR-4.7)
- [ ] All three are `Readability`, `Suggestion`, default-off, `FixClass::None`, `since: "0.2.0"`, declare `MIN_SENTENCES_PARAM` and `MIN_WORDS_PARAM` after their threshold, and their `explain` contains "provisional" (FR-4.2, FR-4.5)
- [ ] `[rules.SLP021]\nmin_ttr = 2` is rejected (NFR-4)
- [ ] SLP019's `explain` names sentence-length variation and says it is not perplexity-based burstiness (D-3)
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::rules_fixtures` | `slp019_positive_fires / slp019_negative_silent` | SLP019 fixtures | FR-4.7 |
| 2 | `sloperator::rules_fixtures` | `slp020_positive_fires / slp020_negative_silent` | SLP020 fixtures | FR-4.7 |
| 3 | `sloperator::rules_fixtures` | `slp021_positive_fires / slp021_negative_silent` | SLP021 fixtures | FR-4.7 |
| 4 | `sloperator::rules_registry` | `metric_rules_explain_provisional` | SLP018–SLP021 `explain` contains "provisional" | FR-4.5 |
| 5 | `sloperator::config` | `rejects_param_above_maximum` | extended with `min_ttr = 2` for SLP021 | NFR-4 |

## Notes

epic-09 adds SLP023 to `metric_rules_explain_provisional`'s list.

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='slp019_ slp020_ slp021_ metric_rules_explain_provisional rejects_param_above_maximum'`
