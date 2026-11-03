# Story 04: `text::segment`, the `text` module root and `Prose` (segments only)

**Epic:** [epic-01-sentence-seam](../epic-01-sentence-seam.md)
**SPEC Refs:** §4 (`text::segment`), §6 (module root), §9 (`Prose`, without `counts`), §36.1
**PRD Refs:** FR-1.1, FR-1.2, FR-1.3, FR-1.4, FR-1.5, NFR-2, NFR-7, NFR-8
**Component / ADR:** Architecture §4.1 Segmenter, §4.8 Per-document context; ADR-02, ADR-03
**Wave:** 2

## Scope

Create `src/text/segment.rs` exactly as SPEC §4, but without `starts_sentence` (it moves in story-05): `Sentence`, `Word`, `Segments` (cached per-block bounds, `word_before`, `word_at`, `sentence_of`, `sentence_starts`, `sentence_span`), private `blanked`/`scan_starts`, the `#[cfg(test)]` scan counter, and `segment`. Create `src/text/mod.rs` with `pub mod segment;`, the re-exports, `Prose { segments, src_len }`, `Prose::new`, `Prose::segments` and the Send+Sync assertion. Add `pub mod text;` to src/lib.rs. Add `tests/text_segment.rs` with the FR-1.3 count test, the random-input test and the FR-1.4 oracle test, which holds a verbatim copy of today's `sentence_range` (src/engine/window.rs:237-278). Nothing calls `Prose` from production code yet.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/text/segment.rs` | new | segmenter, scan counter and unit tests |
| `src/text/mod.rs` | new | module root, `Prose` (segments only), Send+Sync assertion |
| `src/lib.rs` | 19 | `pub mod text;` between `score` and `types` |
| `tests/text_segment.rs` | new | tests below |

## Acceptance Criteria

- [ ] "It is. We go." gives 2 sentences, 4 words, and `sentence_initial` on "It" and "We" (FR-1.1)
- [ ] A sentence bound inside inline code is dropped, and code spans give no word (FR-1.2)
- [ ] `segment(&d).words.len() == d.prose_words()` on every `.md` under `fixtures/` and `eval/paragraphs/`, and on 500 random inputs, with no panic (FR-1.3)
- [ ] `sentence_span` equals the test-local copy of `sentence_range` for every block and word range on every fixture (FR-1.4)
- [ ] `segment` runs exactly one scan per block (scan counter) (NFR-2)
- [ ] `Prose` is `Send + Sync` (a compile-time assertion) (NFR-8)
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::text::segment::tests` | `two_sentences_words_and_initials` | 2 sentences, 4 words, initials | FR-1.1 |
| 2 | `sloperator::text::segment::tests` | `protected_span_is_not_a_word_or_bound` | "Use `a. b` now." is one sentence; the code gives no word | FR-1.2 |
| 3 | `sloperator::text::segment::tests` | `word_lookup_helpers` | `word_before`, `word_at`, `sentence_of`, including an empty-word sentence | FR-1.1 |
| 4 | `sloperator::text::segment::tests` | `sentence_starts_is_the_cache` | "It is. We go." gives `[0, 7]`; an unknown block gives `[0]` | FR-1.4 |
| 5 | `sloperator::text::segment::tests` | `segment_scans_once_per_block` | a three-block document costs exactly three scans | NFR-2 |
| 6 | `sloperator::text_segment` | `word_count_equals_prose_words_on_every_fixture` | FR-1.3 equality on every fixture and eval paragraph | FR-1.3 |
| 7 | `sloperator::text_segment` | `random_input_never_panics` | 500 xorshift 200-char strings (Markdown punctuation, OBJ, é, newlines) never panic, and the word count still equals `prose_words` | FR-1.3, NFR-7 |
| 8 | `sloperator::text_segment` | `sentence_span_equals_old_sentence_range` | `Segments::sentence_span` equals a verbatim copy of today's `sentence_range` on every fixture | FR-1.4 |

## Notes

Write the oracle copy before story-05 deletes `sentence_range`. Use `doc.continuation_prefix_len` (story-03) for `mask::prefix_len`. The xorshift generator is inline, with no new dev-dependency. `src/text/` may use only `crate::doc` (ADR-01). Clippy runs with `-D warnings`; `unwrap` is allowed only in tests.

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='text::segment::tests text_segment'`
