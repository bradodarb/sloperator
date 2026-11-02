# ADR-07: Lexicon, contextual rules and word tables are compile-time Rust tables

**Status:** Accepted
**Date:** 2026-11-02
**Initiative:** [classical-nlp](../ARCHITECTURE-classical-nlp.md) · **Serves:** FR-7.1, FR-7.2, FR-2.6, NFR-4, NFR-5

## Context

D-7 (locked) makes the tagger in-tree with an owned lexicon. NFR-4 requires a corrupt or missing table to be a build
or startup failure, never a silent fallback. Existing rule data are `const` Rust slices (`WordList` entries,
`src/rules/helpers.rs:10-33`; SLP006 lists, `slp006_copula_avoidance.rs:20-35`). The decision is the data format.

## Options

### Option 1: `const`/`static` Rust tables (sorted `&[(&str, &[Tag])]`, ordered rule structs)

**For:** typos in tags fail compilation; no parse step at runtime, so no runtime failure mode; matches existing rule data; invariants (sorted, unique, non-empty) are unit tests. **Against:** large tables are verbose Rust; editing needs a rebuild (it always does for embedded data).

### Option 2: `include_str!` TSV parsed into a `LazyLock` on first use

**For:** compact, editable data files. **Against:** a parse error surfaces at the first tagged document, mid-run; needs a loader and its error path.

### Option 3: `build.rs` generating Rust from TSV

**For:** compact source data and compile-time checking. **Against:** a build script for a few hundred entries; slower builds; another moving part in `--no-default-features` builds.

## Decision

Option 1, for every embedded table (tagger lexicon and rules, syllable exceptions, negations, modals, stop words).

## Consequences

- **Easier:** a broken table cannot ship past `cargo build` plus the table tests.
- **Harder:** bulk lexicon edits are code edits; reviewers see them as Rust diffs.
- **Follow-ups:** a unit test per table asserting sorted, unique and non-empty.
