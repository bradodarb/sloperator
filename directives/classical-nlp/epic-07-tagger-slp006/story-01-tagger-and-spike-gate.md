# Story 01: `text::tagger` tables and the Phase 4 spike gate

**Epic:** [epic-07-tagger-slp006](../epic-07-tagger-slp006.md)
**SPEC Refs:** §26 (`text::tagger`), §28 (Phase 4 spike gate), §36.8
**PRD Refs:** FR-7.1, FR-7.2, FR-7.5, FR-7.8, NFR-4, NFR-10, D-7
**Component / ADR:** Architecture §4.5 Tagger; ADR-07
**Wave:** 1

## Scope

Create src/text/tagger/mod.rs (`Tag`, `Tagged`, `Context`, `ContextRule`, `lookup`, `guess`, `tag(doc, seg)`) exactly as §26, plus src/text/tagger/lexicon_data.rs (sorted, lower-case; closed classes and frequent words, ≤ 3,000 entries, with §26's required entries) and src/text/tagger/rules_data.rs (≤ a few hundred rules). Add `pub mod tagger; pub use tagger::{Tag, Tagged};` to src/text/mod.rs. Add tests/text_tagger.rs, including `spike_slp006_classes`: on every SLP006 class A/B positive ("The library serves as a guide.", "The main library serves as a guide.") the subject words are certain with tags Det N / Det Adj N, and on every SLP006 negative and `unchanged` input ("The tool serves as a bridge.", "The fast library serves as a guide.", …) at least one subject word is not certain or the shape fails. Write `docs/classical-nlp/spikes/G4-slp006-tagger-spike.md` with the classes tried, the inputs and the verdict. No rule changes yet.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/text/tagger/mod.rs` | new | tagger engine and unit tests |
| `src/text/tagger/lexicon_data.rs` | new | `LEXICON` table |
| `src/text/tagger/rules_data.rs` | new | `RULES` table |
| `src/text/mod.rs` | module list | `pub mod tagger;` and re-exports |
| `docs/classical-nlp/spikes/G4-slp006-tagger-spike.md` | new | spike record: classes, inputs, verdict PASS/FAIL |
| `tests/text_tagger.rs` | new | tests below |

## Acceptance Criteria

- [ ] The lexicon is sorted, unique, lower-case and non-empty per entry, with ≤ 3,000 entries (NFR-4, D-7)
- [ ] `tool`, `service`, `guide` are `[Noun, Verb]`; `fast` is `[Adj, Adv]`; `library` is `[Noun]`; `main`, `careful`, `ready`, `cheap`, `quick`, `simple` are `[Adj]`; `these`, `this` include Det and Pron (§26)
- [ ] A known single-tag word is certain; "zorbly" is `Adv`, unknown and not certain; "tool" is not certain; in "the guide" `guide` is `Noun` and not certain (FR-7.1)
- [ ] One `Tagged` per entry of `seg.words`, in order (FR-7.2)
- [ ] `spike_slp006_classes` passes: at least one class is certain on every positive of that class, and every negative declines; otherwise it fails the build (FR-7.5, FR-7.8)
- [ ] The spike file exists, and its verdict matches the test result
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::text::tagger::tests` | `lexicon_sorted_unique_lowercase_nonempty` | table invariants | NFR-4 |
| 2 | `sloperator::text::tagger::tests` | `required_entries` | the pinned §26 entries | FR-7.7 |
| 3 | `sloperator::text_tagger` | `known_word_is_certain` | single-tag lexicon word is certain | FR-7.1, NFR-10 |
| 4 | `sloperator::text_tagger` | `unknown_word_flagged_and_guessed` | zorbly → Adv, unknown, not certain | FR-7.1 |
| 5 | `sloperator::text_tagger` | `ambiguous_word_not_certain` | tool not certain | FR-7.1 |
| 6 | `sloperator::text_tagger` | `rule_changes_tag_but_never_certainty` | the guide: Noun, not certain | FR-7.1 |
| 7 | `sloperator::text_tagger` | `spike_slp006_classes` | Phase 4 spike gate | FR-7.5, FR-7.8 |

## Notes

No third-party tagger or lexicon (D-7). The tables are `static` Rust, so a tag typo fails compilation. Rules never make a word certain. If the spike fails, do not weaken it: record FAIL with the classes and fixtures, and stop (the gate fails).

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='text::tagger::tests text_tagger'`
