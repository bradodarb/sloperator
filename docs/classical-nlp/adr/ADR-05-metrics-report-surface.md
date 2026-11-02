# ADR-05: Metrics appear as `metrics`/`metricsBefore` siblings of the score, with no schema bump

**Status:** Accepted
**Date:** 2026-11-02
**Initiative:** [classical-nlp](../ARCHITECTURE-classical-nlp.md) · **Serves:** FR-3.1–FR-3.5, D-2, D-11

## Context

The JSON file object is `{ path, score, scoreBefore, findings }` (`src/report/json.rs:25-32`); MCP fix returns
`scoreBefore`/`scoreAfter` (`src/mcp/mod.rs:218-219`); the `Score` JSON is pinned (`tests/score.rs:96-98`). D-11
resolves that an additive object does not bump `SCHEMA_VERSION` (`src/report/json.rs:13-14`). The open choice is
naming and nesting.

## Options

### Option 1: Mirror the score: `metrics` + `metricsBefore` per file, `summary.metrics`; MCP lint `metrics`, fix `metricsBefore`/`metricsAfter`

**For:** consumers already know the before/after pattern of each surface; `Score` untouched. **Against:** JSON file and MCP use different "after" names, as they already do for score.

### Option 2: One nested object `analysis: { metrics, metricsBefore }`

**For:** room for future analysis output. **Against:** a namespace no requirement asks for; breaks the existing pattern.

### Option 3: Fields inside `Score`

**For:** one object. **Against:** changes the pinned `Score` shape — forbidden by D-2 and FR-3.2.

## Decision

Option 1. Unavailable values serialise as `null`; `words` and `sentences` are always present so a reader can see why.

## Consequences

- **Easier:** one pattern per surface; additive for existing readers.
- **Harder:** a reader validating with a closed schema must update; documented in REFERENCE and CHANGELOG.
- **Follow-ups:** update the pinned report and MCP JSON tests in Phase 1c.
