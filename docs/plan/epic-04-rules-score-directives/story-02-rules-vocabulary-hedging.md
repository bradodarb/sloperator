# Story 02: Rules SLP001, SLP002, SLP008, SLP009, SLP016

**Epic:** [epic-04-rules-score-directives](../epic-04-rules-score-directives.md)
**SPEC Refs:** §10
**PRD Refs:** FR-4.2, FR-4.6, FR-4.7, D-4
**Component / ADR:** §4.6 rules; ADR-06
**Wave:** 2

## Scope

Add the five word-list and phrase rules as `src/rules/builtin/slpNNN_<name>.rs`, with fixtures `fixtures/rules/SLPNNN/{positive,negative,fixed}.md` (SLP002 has no `fixed.md`). Register them in `all()`. Create `tests/rules_fixtures.rs` with the `rule_fixtures!` macro called for 1, 2, 8, 9 and 16.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/rules/builtin/slp001_ai_vocabulary_replaceable.rs` | new | inflection-matched WordList; bare `leverage` gets no fix |
| `src/rules/builtin/slp002_ai_vocabulary.rs` | new | model-class word list |
| `src/rules/builtin/slp008_hedging_preamble.rs` | new | delete and capitalise |
| `src/rules/builtin/slp009_stacked_hedge.rs` | new | keep the first word |
| `src/rules/builtin/slp016_summary_signpost.rs` | new | delete and capitalise |
| `src/rules/builtin/mod.rs` | from story-01 | register the five rules |
| `fixtures/rules/SLP001/ … SLP016/` | new | positive/negative[/fixed] for 001, 002, 008, 009, 016 |
| `tests/rules_fixtures.rs` | new | tests below |

## Acceptance Criteria

- [ ] Each rule fires on `positive.md` and is silent on `negative.md`, which also holds the trigger inside inline code, a fence and a link destination (FR-4.4, FR-4.6).
- [ ] SLP001: "we have leverage over suppliers" fires with no fix, and "she leveraged the API" becomes "she used the API" (Gap 2).
- [ ] Applying each safe rule's fixes to `positive.md` gives `fixed.md` byte for byte (FR-4.7).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::rules_fixtures` | `slp001_positive_fires / slp001_negative_silent / slp001_fix_matches` | SLP001 fixtures | FR-4.2, FR-4.6 |
| 2 | `sloperator::rules_fixtures` | `slp002_positive_fires / slp002_negative_silent` | SLP002 fixtures | FR-4.2, FR-4.6 |
| 3 | `sloperator::rules_fixtures` | `slp008_positive_fires / slp008_negative_silent / slp008_fix_matches` | SLP008 fixtures | FR-4.2, FR-4.6 |
| 4 | `sloperator::rules_fixtures` | `slp009_positive_fires / slp009_negative_silent / slp009_fix_matches` | SLP009 fixtures | FR-4.2, FR-4.6 |
| 5 | `sloperator::rules_fixtures` | `slp016_positive_fires / slp016_negative_silent / slp016_fix_matches` | SLP016 fixtures | FR-4.2, FR-4.6 |

## Notes

The Gate selectors are bare substrings (SPEC §3: a bare word matches a substring of the binary id or test name). Each `slpNNN_` prefix selects that rule's three macro-generated tests. A `binary#test` form would be an exact match and select nothing.

Gate: `make test TESTS='slp001_ slp002_ slp008_ slp009_ slp016_'`
