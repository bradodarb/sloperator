# Story 02: `text::similarity`: content-word bags and weighted Jaccard

**Epic:** [epic-09-redundancy](../epic-09-redundancy.md)
**SPEC Refs:** §33 (`text::similarity`)
**PRD Refs:** FR-9.1, FR-9.5
**Component / ADR:** Architecture §4.7 Similarity
**Wave:** 2

## Scope

Create src/text/similarity.rs exactly as §33: `Bag` (stemmed content-word counts, stop words and letterless words removed), `Bag::size`, `bag(doc, seg, sentence)` and `jaccard(a, b)` (Σ min / Σ max over integer counts; 0 for two empty bags). Add `pub mod similarity;` to src/text/mod.rs. Nothing calls it yet.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/text/similarity.rs` | new | `Bag`, `bag`, `jaccard`, unit tests |
| `src/text/mod.rs` | module list | `pub mod similarity;` |

## Acceptance Criteria

- [ ] {a:2,b:1} vs {a:1,c:1} gives 0.25; empty vs empty gives 0 (FR-9.1)
- [ ] Stop words and words with no letter are not in a bag (FR-9.1)
- [ ] Inflected forms count as one key through `stem` (FR-9.1)
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::text::similarity::tests` | `jaccard_hand_values` | hand values and the empty case | FR-9.1 |
| 2 | `sloperator::text::similarity::tests` | `stop_words_removed` | stop and letterless words dropped | FR-9.1 |

## Notes

Uses `lexicon::is_stop_word` (epic-06). Weighted Jaccard needs no square root (DQ-14).

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='text::similarity::tests'`
