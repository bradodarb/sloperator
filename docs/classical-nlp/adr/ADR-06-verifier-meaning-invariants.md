# ADR-06: Meaning invariants live in `verify::edit`; the D-8 exemption is a counted allowance

**Status:** Accepted
**Date:** 2026-11-02
**Initiative:** [classical-nlp](../ARCHITECTURE-classical-nlp.md) · **Serves:** FR-6.1–FR-6.8, FR-8.5, D-8, NFR-6

## Context

`verify::edit(doc, p, frozen, protect)` compares token multisets of the replaced bytes and the replacement
(`src/verify/mod.rs:65-105`) and knows the proposal origin (`src/types.rs:49-62`). D-8 exempts edits that resolve
SLP003/SLP004 from the negation check: keyed by origin for rule edits; for model edits the window's findings decide,
and the SPEC defines how. A window knows its findings when it computes `weight_before` (`src/engine/window.rs:28-29`).
`verify` must not use `rules` (we add that to its layering row), so it cannot look up rule metadata.

## Options

### Option 1: Add `resolves: &[RuleCode]` to `verify::edit`; allowance = 1 if the origin is `Rule(3|4)`, else the number of SLP003/SLP004 codes in `resolves` (from `Window.codes`)

**For:** one verifier for both origins; a model rewrite of a window with one SLP003 may drop at most one negation; constants live in verify as `RuleCode`s (`types` is a leaf). **Against:** a signature change at every call site (pipeline `:118`, `:273`, tests); `Window` gains a field.

### Option 2: A boolean switch: skip the negation check when the window contains any SLP003/SLP004 finding

**For:** simpler. **Against:** a window with one SLP003 match could lose every other "not" in the paragraph unchecked.

### Option 3: Separate post-verify filter in the pipeline for meaning checks

**For:** no `verify` signature change. **Against:** a second gate outside `Verified<T>` (product ADR-05 forbids paths around the single choke point); rejections would not be `Check` variants.

## Decision

Option 1. The three checks run after the term check, as multisets: negation count (may fall by at most the
allowance, may rise), proper-noun proxy (capitalised, non-sentence-initial tokens: equal multisets), modal multiset
(equal). Sentence-initial status uses `text::segment::starts_sentence` on the edit's block text, located with
`BlockText::to_text` (ADR-03). `Window.codes` keeps one entry per overlapping non-system finding, duplicates included,
because the allowance counts SLP003/SLP004 findings and a deduplicated list could not count two (review round 1, Gap 6).

## Consequences

- **Easier:** templates and model rewrites face the same rules; each failure names its `Check`.
- **Harder:** every `verify::edit` caller passes `resolves`; adding another exempt rule means editing verify's constant list.
- **Follow-ups:** FR-6.6 regression gate over every Safe fixture in Phase 3; FR-6.7 doc sentence in three places.
  Which capitalised common words the proper-noun proxy skips (every origin, or only the proven Safe transformations)
  is open as review round 1 Gap 11 for Brad. This ADR does not decide it.

## Revision Log

- round 1: Gap 6 absorbed. The sentence-initial mechanism is `starts_sentence` + `BlockText::to_text`, and
  `Window.codes` keeps duplicates (Decision). Gap 11 is noted as open (Follow-ups).
