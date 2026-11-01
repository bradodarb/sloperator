# ADR-01: One crate with layering enforced by tests

**Status:** Accepted
**Date:** 2026-10-29
**Initiative:** [sloperator](../ARCHITECTURE.md) · **Serves:** FR-1.1, FR-1.8, NFR-2, D-2

## Context

sloperator has about fourteen components (architecture §4). It is a standalone repository with the crate at the
root (D-2), and a copy of the crate must build on its own (NFR-2). The repository's `Makefile` drives it with
cargo and nextest, run from the repository root (D-11). The design needs the pure
core (`doc`, `rules`, `verify`, `edit`, `score`) kept free of I/O and of the model runtime.

## Options

### Option 1: One crate (lib + bin), module layering checked by tests

**For:** one `Cargo.toml`, one lockfile, one nextest invocation, one JUnit file set. Simplest `Makefile`. Copying the
crate means copying one directory. Refactors across layers need no version bumps. **Against:** Rust modules inside a crate
can `use crate::anything`. Layering is enforced by text-scan tests (`tests/layering.rs`), which a re-export could
dodge.

### Option 2: A cargo workspace with crate-per-layer (`sloperator-core`, `-rules`, `-engine`, `-cli`)

**For:** the compiler enforces the dependency graph; the runtime crate is confined to `-engine` by construction.
**Against:** four manifests, cross-crate visibility churn while the design is young. More build units for
nextest and deny. The `Verified<T>` constructor would have to be public across crates, so ADR-05's
unskippability gets weaker, not stronger.

## Decision

Option 1. The two boundaries that matter most are also enforced by the build. The runtime crate is an optional
dependency behind feature `model` (ADR-02), and `Verified::new` is `pub(in crate::verify)` (ADR-05). The rest is
checked by `tests/layering.rs`. A workspace can be introduced later without changing any public CLI surface.

## Consequences

- **Easier:** the `Makefile`, standalone builds, refactoring, one coherent version number.
- **Harder:** layer violations are caught by a test, not by `rustc`. The test must be kept in step with §4's table.
- **Follow-ups:** the SPEC lists the exact forbidden-prefix table that `tests/layering.rs` asserts.
