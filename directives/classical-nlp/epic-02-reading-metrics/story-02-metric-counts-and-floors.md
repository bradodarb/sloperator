# Story 02: `text::metrics` (`Counts`, `Metrics`, `Floors`) and `Prose::counts`

**Epic:** [epic-02-reading-metrics](../epic-02-reading-metrics.md)
**SPEC Refs:** §8 (`text::metrics`), §9 (`Prose::counts`), OD-3 option (a)
**PRD Refs:** FR-2.1, FR-2.2, FR-2.3, FR-2.4, FR-2.5, FR-2.7, FR-2.8, FR-2.9, FR-2.10, NFR-3, D-5, D-6
**Component / ADR:** Architecture §4.4 Metrics, §4.8 Prose; ADR-04
**Wave:** 2

## Scope

Create src/text/metrics.rs from SPEC §8, with Brad's Gap 5 change: `pub struct Floors { pub sentences: u64, pub words: u64 }` (Clone, Copy, Debug, PartialEq, Eq, serde::Serialize), whose `Default` is 5/100 (the §8 constants, kept as `DEFAULT_MIN_SENTENCES`/`DEFAULT_MIN_WORDS`). `Counts::derive(&self, floors: &Floors) -> Metrics` returns every value `None` below either floor. `counts(doc, seg)` counts paragraph sentences only. In src/text/mod.rs, `Prose` gains `counts: OnceLock<Counts>` and `Prose::counts(&self, doc)`. Nothing outside `text` calls them yet.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/text/metrics.rs` | new | `Floors`, `Counts` (merge, derive(floors)), `Metrics`, `counts`, unit tests |
| `src/text/mod.rs` | `Prose` struct and impl | `counts` field and accessor; `pub mod metrics;`, `pub use metrics::{Counts, Floors, Metrics};` |

## Acceptance Criteria

- [ ] Hand-computed counts (words 100, sentences 5, syllables 150, Σx² 2200, 52 types) derive avg 20.0, grade 9.91, SD 6.32, CV 0.32, TTR 0.52 with default floors (FR-2.2–FR-2.5)
- [ ] 4 sentences with default floors give every derived value `None`, serialised as JSON `null` (FR-2.7)
- [ ] The same counts with `Floors { sentences: 10, words: 100 }` give every value `None`, and with `Floors { sentences: 5, words: 100 }` give values (D-5, Gap 5 (a))
- [ ] Merging a copy with itself keeps 52 types and halves TTR to 0.26 (FR-2.8)
- [ ] Headings, list items and table cells add no count (FR-2.10)
- [ ] Floats appear only inside `derive`; the SD comes from an exact integer numerator (NFR-3)
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::text::metrics::tests` | `hand_computed_values` | FR-2.2–2.5 values from integer counts | FR-2.2, FR-2.3, FR-2.4, FR-2.5 |
| 2 | `sloperator::text::metrics::tests` | `below_floor_is_unavailable` | 4 sentences give `None` everywhere and JSON `null` | FR-2.7 |
| 3 | `sloperator::text::metrics::tests` | `configured_floor_is_honoured` | raised floors make available counts unavailable | FR-2.7, D-5 |
| 4 | `sloperator::text::metrics::tests` | `merge_unions_types` | set union, not summed distinct counts | FR-2.8 |
| 5 | `sloperator::text::metrics::tests` | `paragraphs_only` | heading, list item and table cell add no count | FR-2.10 |

## Notes

Accepted Gap 12: sufficient statistics stand in for the PRD's per-sentence list. `round2` is private to `text` (it may not use `score`).

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='text::metrics::tests'`
