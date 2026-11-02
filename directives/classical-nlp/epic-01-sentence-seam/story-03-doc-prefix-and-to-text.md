# Story 03: Move `prefix_len` to `Document::continuation_prefix_len`; add `BlockText::to_text`

**Epic:** [epic-01-sentence-seam](../epic-01-sentence-seam.md)
**SPEC Refs:** §3 (`doc`: `continuation_prefix_len` and `BlockText::to_text`)
**PRD Refs:** FR-1.4, FR-6.2, NFR-7, NFR-9
**Component / ADR:** Architecture §4.1 Segmenter (ADR-03 move)
**Wave:** 1

## Scope

Add `Document::continuation_prefix_len` (the body of `engine::mask::prefix_len`, unchanged) and `BlockText::to_text` (the inverse of `to_src`) to src/doc/mod.rs. Replace every `mask::prefix_len(doc, a, b)` call with `doc.continuation_prefix_len(a, b)`, and delete the old function. `sentence_range` (window.rs:241, which still calls it at :250) switches to the new method here and is deleted in story-05.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/doc/mod.rs` | after `is_protected` (~274); after `BlockText::to_src` (~324-329) | `continuation_prefix_len`, `to_text`, and unit tests in the existing `#[cfg(test)]` module or a new one |
| `src/engine/mask.rs` | 80, 105-117 | call site switched; `prefix_len` deleted |
| `src/engine/window.rs` | 162, 220, 230, 250 | call sites switched to `doc.continuation_prefix_len` |
| `tests/engine.rs` | existing | tests below (methods added) |
| `tests/layering.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] `prefix_len` no longer exists in src/engine/mask.rs, and nothing re-exports it (NFR-9)
- [ ] `src/doc/` still uses no other crate module (`layering::forbidden_uses` passes) (NFR-7)
- [ ] `BlockText::to_text(to_src(t)) == Some(t)` for a text offset inside a prose run, and `None` inside a protected run
- [ ] All existing mask and window tests in tests/engine.rs pass unchanged (FR-1.4)
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::doc::tests` | `to_text_inverts_to_src` | a round trip through `to_src`/`to_text` inside prose; `None` for an offset inside inline code | FR-6.2 |
| 2 | `sloperator::doc::tests` | `continuation_prefix_len_skips_quote_markup` | `> ` on a block-quote continuation line has prefix length 2; plain text gives 0 | FR-1.4 |
| 3 | `sloperator::engine` | `(all existing window and mask tests)` | masking and window ranges are unchanged by the move | FR-1.4, NFR-9 |
| 4 | `sloperator::layering` | `forbidden_uses` | doc still depends on nothing in the crate | NFR-7 |

## Notes

If src/doc/mod.rs has no `#[cfg(test)] mod tests`, add one at the end of the file.

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='doc::tests::to_text doc::tests::continuation_prefix sloperator::engine sloperator::layering#forbidden_uses'`
