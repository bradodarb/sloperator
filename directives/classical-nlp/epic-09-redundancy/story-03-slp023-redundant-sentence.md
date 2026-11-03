# Story 03: SLP023 `redundant-sentence`

**Epic:** [epic-09-redundancy](../epic-09-redundancy.md)
**SPEC Refs:** §34 (SLP023 `redundant-sentence`)
**PRD Refs:** FR-9.1, FR-9.2, FR-9.3, FR-9.4, FR-9.5, FR-9.6, FR-11.2
**Component / ADR:** Architecture §4.9 Rule integration, row SLP023
**Wave:** 3

## Scope

Add src/rules/builtin/slp023_redundant_sentence.rs: `Family::Redundancy`, `Severity::Suggestion`, `default_on: false`, `FixClass::None`, `since: "0.2.0"`, params `min_overlap` (0.6, 0.1, 1.0), `min_content_words` (4, 1, 100) and `min_sentences` (3, 2, f64::MAX), and `check` exactly as §34 (pairwise within each paragraph run, one finding per later sentence on its first prose run). `explain` says detection only and provisional. Add the builtin/mod.rs entry, fixtures and a `model(23, …)` macro entry, the two behaviour tests, and SLP023 in `metric_rules_explain_provisional`.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/rules/builtin/slp023_redundant_sentence.rs` | new | SLP023 |
| `src/rules/builtin/mod.rs` | module and `rule()` lists | SLP023 entry |
| `fixtures/rules/SLP023/positive.md` | new | a paragraph with a repeated sentence |
| `fixtures/rules/SLP023/negative.md` | new | distinct sentences plus a fenced block, inline code and a link |
| `tests/rules_fixtures.rs` | 135-198, new tests | macro entry; skip and paragraph tests |
| `tests/rules_registry.rs` | `metric_rules_explain_provisional` | SLP023 added |

## Acceptance Criteria

- [ ] SLP023 fires on positive.md with a message naming the earlier sentence's line, and is silent on negative.md (FR-9.2, FR-9.3)
- [ ] Sentences under `min_content_words` and documents under `min_sentences` paragraph sentences are skipped (FR-9.4)
- [ ] The same sentence in two different paragraphs is not reported (FR-9.5)
- [ ] No fix is ever attached, and the rule is default-off (FR-9.6, FR-11.2)
- [ ] Its `min_sentences` is not raised by `[metrics]` (it is not a Readability rule)
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::rules_fixtures` | `slp023_positive_fires / slp023_negative_silent` | SLP023 fixtures | FR-9.2, FR-9.3 |
| 2 | `sloperator::rules_fixtures` | `slp023_skips_short_sentences_and_documents` | FR-9.4 minimums | FR-9.4 |
| 3 | `sloperator::rules_fixtures` | `slp023_compares_within_paragraph_only` | no cross-paragraph report | FR-9.1, FR-9.5 |
| 4 | `sloperator::rules_registry` | `metric_rules_explain_provisional` | SLP023 explain says provisional | FR-4.5 |

## Notes

`chunk_by` on `seg.sentences` relies on block order (§4). The finding sits on the later sentence's first prose run, so `detect` never drops it for overlapping inline code (src/rules/mod.rs:172-174).

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='slp023_ sloperator::rules_registry'`
