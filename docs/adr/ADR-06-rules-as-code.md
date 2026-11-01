# ADR-06: Rules are Rust code in a static registry, with fixture directories

**Status:** Accepted
**Date:** 2026-10-29
**Initiative:** [sloperator](../ARCHITECTURE.md) · **Serves:** FR-4.1-4.7, FR-2.12, NFR-3

## Context

The PRD keeps user-authored rule languages and style packs out of scope. v1 ships a built-in, versioned set of at
least 15 rules over nine families. Some are word lists (AI vocabulary), some are regex shapes ("it's not X, it's
Y"), some are structural (bullet spam, over-bolding, em-dash density per 1,000 words). Vale's YAML rule types are
the peer model (research `competitive-analysis.md` §1.1).

## Options

### Option 1: Each rule is a Rust type implementing `Rule`, with shared helpers for word-list and regex rules

**For:** structural and density rules need real code anyway. One mechanism covers all three kinds. Matching compiles
to `aho-corasick`/`RegexSet` once (NFR-3). Fixtures under `fixtures/rules/SLP###/` are enumerated by a test that
fails on a missing file (FR-4.6). **Against:** adding a rule needs a release. Users cannot write their own (out of
scope by PRD).

### Option 2: Rules as embedded data (TOML/YAML) interpreted by an engine, Vale-style

**For:** easy path to user packs later. **Against:** an interpreter and schema now, for a feature the PRD excludes.
Structural rules still need code, so there would be two mechanisms.

## Decision

Option 1. It meets every v1 rule kind with one mechanism and no speculative interpreter. If style packs are
requested later, a data-driven rule can be added as one more `Rule` implementation without changing the registry.

## Consequences

- **Easier:** testing (each rule unit-tested with its fixtures), performance, explanations and examples in `RuleMeta`.
- **Harder:** user extensibility is deferred.
- **Follow-ups:** the SPEC enumerates the initial rules, codes, families, default states and thresholds.
