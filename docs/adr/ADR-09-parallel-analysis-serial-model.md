# ADR-09: Parallel per-file analysis; a serial fix phase owning the single `Rewriter`

**Status:** Accepted
**Date:** 2026-10-29
**Initiative:** [sloperator](../ARCHITECTURE.md) · **Serves:** NFR-3, NFR-6, FR-3.6, FR-9.4, FR-9.8, NFR-5

## Context

Lint over a repository should be fast (NFR-3) and byte-deterministic (NFR-6). The model is loaded at most once per
invocation (FR-9.4). A loaded 2.5 GB model with its KV cache is one heavy, stateful object, and seeded sampling must
be reproducible (FR-9.8). NFR-5 forbids writing a file when any of its processing failed.

## Options

### Option 1: rayon for pure analysis; sort results; fix and write serially on the main thread

**For:** analysis is a pure function, so parallelism cannot change output once results are sorted by path. One
`&mut Rewriter`, no locks. Windows run in a fixed order with a cleared KV cache, so seeds reproduce. Writes happen
after every file is processed. **Against:** model rewriting does not use multiple cores at the process level. The
runtime parallelises inside one generation anyway.

### Option 2: Fully serial

**For:** simplest. **Against:** gives up an easy multi-core speed-up for lint over many files, with no gain in
determinism.

### Option 3: Parallel model rewriting (model behind a mutex or a pool of contexts)

**For:** possible throughput gains on large GPUs. **Against:** several contexts multiply memory on laptops.
A mutex serialises anyway. Ordering of seeded generations becomes harder to reproduce.

## Decision

Option 1.

## Consequences

- **Easier:** determinism tests (shuffled walk order → identical JSON bytes), no shared mutable state except the
  read-only `Arc<Settings>` cache filled before analysis.
- **Harder:** large `--fix` runs are bounded by sequential generation; revisit only with spike data.
- **Follow-ups:** none.
