# ADR-02: A per-document `Prose` context, eager segments, lazy tags and counts, shared through `RuleCtx`

**Status:** Accepted
**Date:** 2026-11-02
**Initiative:** [classical-nlp](../ARCHITECTURE-classical-nlp.md) · **Serves:** FR-7.3, FR-7.4, FR-3.4, NFR-2, NFR-8

## Context

`detect` builds a fresh `RuleCtx { doc, params, severity }` per rule (`src/rules/mod.rs:84-88`, `:164-171`), so
nothing is shared between rules. Tagging must run at most once per document pass and only on a candidate hit
(FR-7.3, FR-7.4). Metrics must be reported for every file and also read by metric rules. Each accepted model
window reruns detection on a whole candidate (`src/pipeline/mod.rs:283`). `detect_in` returns the `Document` by value
(`src/pipeline/mod.rs:295-309`), so a context that borrows the document cannot be returned beside it.

## Options

### Option 1: Owned `Prose` (segments eager, `OnceLock` tags and counts), built in `detect_in`, passed to `detect` and kept in `Analysis`

**For:** one segmentation per pass shared by rules and reports; lazy tags; owned so it travels with `Document`;
`OnceLock` is `Send + Sync`. **Against:** `RuleCtx` and `detect` signatures change (every test that calls `detect`
updates); the caller must pair a `Prose` with its own `Document`.

### Option 2: Each rule computes what it needs

**For:** no signature change. **Against:** violates FR-7.3; four metric rules would segment four times; NFR-2 risk.

### Option 3: Precompute everything (segments, tags, counts, similarity) in the pipeline

**For:** simplest data flow. **Against:** violates FR-7.4 (tags every document); similarity work when the rule is off.

## Decision

Option 1. `Prose::new(&doc)` segments eagerly, because metrics are always reported; tags and counts fill on first use.

## Consequences

- **Easier:** the NFR-2 "one tagging per pass" test reads a counter on `Prose`; new analyses add a `OnceLock` field.
- **Harder:** callers of `rules::detect` (pipeline, tests) must build a `Prose`; a mismatched `doc`/`Prose` pair is a programming error the type system does not stop (a debug assertion on source length mitigates).
- **Follow-ups:** none.
