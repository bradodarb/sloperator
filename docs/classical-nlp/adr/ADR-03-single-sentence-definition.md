# ADR-03: Sentence boundaries move to `text::segment`; the continuation-prefix helper moves to `doc`

**Status:** Accepted (revised round 1: scan once, cached bounds)
**Date:** 2026-11-02
**Initiative:** [classical-nlp](../ARCHITECTURE-classical-nlp.md) · **Serves:** FR-1.4, FR-6.2, NFR-7

## Context

The only sentence splitter is private to the window code (`src/engine/window.rs:237-278`). It calls
`engine::mask::prefix_len` (`src/engine/mask.rs:107-117`) to blank continuation prefixes. FR-1.4 demands one
definition. `text` may not depend on `engine` (ADR-01); `prefix_len` itself reads only `Document`. A second helper,
`starts_sentence` (`src/rules/helpers.rs:138-157`), decides sentence-initial position for SLP008/SLP016 and is needed
by the verifier's proper-noun check, which may not use `rules`. Today the window rescans a block per selecting
finding (`src/engine/window.rs:81-88`), so moving the algorithm alone would give one definition with a repeated
computation (review round 1, Gap 1).

## Options

### Option 1: Move the boundary scan to `text::segment::sentence_starts`; move `prefix_len` to `Document::continuation_prefix_len`; move `starts_sentence` to `text::segment`

**For:** one definition used by windows, metrics and verify; dependency arrows point downward only. **Against:** touches `engine/mask.rs`, `engine/window.rs` and `rules/helpers.rs`; window tests must prove no behaviour change.

### Option 2: Let `text` call `engine::mask::prefix_len`

**For:** smaller diff. **Against:** a pure layer depending on the model-engine module; violates ADR-01's row.

### Option 3: Duplicate the splitter in `text`, leave the window's

**For:** zero risk to windows. **Against:** two definitions, violating FR-1.4 directly.

## Decision

Option 1, with the scan's result cached. `text::segment::segment` is the only function that scans blocks (through the
private `blanked` and `scan_starts`). It stores each block's raw bounds in `Segments` beside the trimmed sentences.
`Segments::sentence_starts(block)` returns that slice, and `Segments::sentence_span(doc, block, r)` carries today's
`sentence_range` arithmetic and trim over the cache, byte for byte. The window's `sentence_range` is deleted. The old
functions are deleted, not aliased (NFR-9); `helpers` re-exports `text::segment::starts_sentence` under its existing name
so rule call sites are untouched. The verifier decides sentence-initial position with `starts_sentence` on the block
text, mapping the edit offset with the new `BlockText::to_text`. It does not use raw starts, because a block range
includes line-start markup such as `# ` (`tests/rules_fixtures.rs:266-269`).

## Consequences

- **Easier:** a sentence means the same thing in windows, metrics, templates and verify.
- **Harder:** any future change to sentence splitting changes model windows too, so window tests guard metrics changes.
- **Follow-ups:** Phase 1b gate: existing `window`/`mask` tests pass with only the added `&prose` argument;
  `sentence_span` equals the old `sentence_range` on every fixture finding; window planning performs zero scans.

## Revision Log

- round 1: Gap 1 fixed (Context, Decision: cached bounds, `sentence_span`, private scan helpers; Follow-ups gate).
  Gap 6 absorbed: the private helpers are named, and the verifier uses `starts_sentence` + `BlockText::to_text`.
