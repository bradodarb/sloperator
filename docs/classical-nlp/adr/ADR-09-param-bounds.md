# ADR-09: `ParamSpec` gains `min`/`max`, checked at config load

**Status:** Accepted
**Date:** 2026-11-02
**Initiative:** [classical-nlp](../ARCHITECTURE-classical-nlp.md) · **Serves:** NFR-4, FR-4.2, FR-4.3, D-5

## Context

NFR-4 requires config to reject out-of-range values (a negative threshold, zero minimum sentences) naming rule and
param. `ParamSpec` has `name`, `default`, `help` (`src/rules/mod.rs:60-66`); config rejects unknown names
(`src/config/mod.rs:509-511`) and accepts any number (`:512-523`). An unknown param read at runtime is NaN and never
fires (`src/rules/mod.rs:100-103`), a silent behaviour. FR-4.3 requires metric rules to stay silent when the report
shows a metric as unavailable.

## Options

### Option 1: Add `min: f64, max: f64` to `ParamSpec`; config checks `min <= v <= max`

**For:** fails at load with the rule and param named; bounds documented in `explain` and REFERENCE from one source; a metric rule's minimum params take `min = MIN_SENTENCES/MIN_WORDS`, so FR-4.3 holds by construction. **Against:** every existing `ParamSpec` literal (e.g. SLP005, SLP014) gains two fields.

### Option 2: Each rule validates its params in `check`

**For:** no struct change. **Against:** errors surface mid-run per file, or get swallowed into "no finding" — the silent path NFR-4 forbids.

### Option 3: A separate bounds table in `config`

**For:** `rules` untouched. **Against:** two sources of truth for one param; drift.

## Decision

Option 1.

## Consequences

- **Easier:** all present and future params are range-checked loudly.
- **Harder:** existing params need deliberately chosen bounds; a too-tight bound would reject a config that works today (the SPEC sets existing bounds to the widest sane range, `0..=f64::MAX` for counts and rates).
- **Follow-ups:** config tests for a negative threshold and a zero minimum.
