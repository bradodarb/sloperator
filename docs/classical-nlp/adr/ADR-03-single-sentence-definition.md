# ADR-03: Sentence boundaries move to `text::segment`; the continuation-prefix helper moves to `doc`

**Status:** Accepted
**Date:** 2026-11-02
**Initiative:** [classical-nlp](../ARCHITECTURE-classical-nlp.md) · **Serves:** FR-1.4, FR-6.2, NFR-7

## Context

The only sentence splitter is private to the window code (`src/engine/window.rs:237-278`). It calls
`engine::mask::prefix_len` (`src/engine/mask.rs:107-117`) to blank continuation prefixes. FR-1.4 demands one
definition. `text` may not depend on `engine` (ADR-01); `prefix_len` itself reads only `Document`. A second helper,
`starts_sentence` (`src/rules/helpers.rs:138-157`), decides sentence-initial position for SLP008/SLP016 and is needed
by the verifier's proper-noun check, which may not use `rules`.

## Options

### Option 1: Move the boundary scan to `text::segment::sentence_starts`; move `prefix_len` to `Document::continuation_prefix_len`; move `starts_sentence` to `text::segment`

**For:** one definition used by windows, metrics and verify; dependency arrows point downward only. **Against:** touches `engine/mask.rs`, `engine/window.rs` and `rules/helpers.rs`; window tests must prove no behaviour change.

### Option 2: Let `text` call `engine::mask::prefix_len`

**For:** smaller diff. **Against:** a pure layer depending on the model-engine module; violates ADR-01's row.

### Option 3: Duplicate the splitter in `text`, leave the window's

**For:** zero risk to windows. **Against:** two definitions, violating FR-1.4 directly.

## Decision

Option 1. The window's `sentence_range` keeps only its range arithmetic over `sentence_starts`. The old functions are
deleted, not aliased (NFR-9); `helpers` re-exports `text::segment::starts_sentence` under its existing name so rule
call sites are untouched.

## Consequences

- **Easier:** a sentence means the same thing in windows, metrics, templates and verify.
- **Harder:** any future change to sentence splitting changes model windows too, so window tests guard metrics changes.
- **Follow-ups:** Phase 1b gate: existing `window`/`mask` tests pass unmodified.
