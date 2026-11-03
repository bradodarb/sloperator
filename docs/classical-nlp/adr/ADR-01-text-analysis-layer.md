# ADR-01: One pure `src/text/` module holds all classical analysis

**Status:** Accepted
**Date:** 2026-11-02
**Initiative:** [classical-nlp](../ARCHITECTURE-classical-nlp.md) · **Serves:** FR-1.5, FR-2, FR-7.2, FR-9, NFR-7, NFR-8

## Context

The PRD adds segmentation, metrics, syllables, a tagger, a stemmer and similarity, all pure (NFR-7), used by rules,
verify, the window code and the pipeline. The crate is one crate of layered modules with text-scan layering tests
(product ADR-01; `tests/layering.rs:9-103`). `doc` must use no crate module (`tests/layering.rs:10`). `rules` may not
use `verify`, and `verify` must be able to use the segmenter and the closed-class lists, so the analysis cannot live
in `rules`.

## Options

### Option 1: One new module `src/text/` with submodules

**For:** one layering row and one `PURE_DIRS` entry cover all of it; submodules share `Segments` and the lexicon
without cross-module imports; verify, rules, engine and pipeline can all depend on it. **Against:** a larger module;
internal ordering (tagger before similarity) is by convention inside it, not enforced by the scan.

### Option 2: Several top-level modules (`segment`, `metrics`, `tagger`, `similarity`)

**For:** finer layering rows (e.g. metrics may not use tagger). **Against:** four or more rows and `lib.rs` entries;
they all depend on `segment`, so the rows add little; more public surface.

### Option 3: Put segmentation in `doc` and the rest in `rules::helpers`

**For:** no new module. **Against:** `verify` would need `rules` for the lexicon (forbidden direction we want to
keep); `doc` grows NLP concerns; `rules::helpers` is already the rule detectors (`src/rules/helpers.rs:1`).

## Decision

Option 1. `src/text/` depends only on `crate::doc`, `crate::types`, std and `unicode-segmentation` (plus the Phase 6
stemmer). A new `FORBIDDEN_USES` row and a `PURE_DIRS` entry enforce it.

## Consequences

- **Easier:** every consumer imports one module; purity and thread safety are checked in one place.
- **Harder:** intra-`text` dependency order is not scanned; a reviewer must keep `tagger` and `similarity` from depending on each other.
- **Follow-ups:** layering rows in Phase 1b.
