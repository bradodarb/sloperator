# ADR-03: Edits are byte ranges over the original source; Markdown is never re-rendered

**Status:** Accepted
**Date:** 2026-10-29
**Initiative:** [sloperator](../ARCHITECTURE.md) · **Serves:** FR-8.1-8.4, FR-8.9, D-7

## Context

FR-8.4 requires the original bytes of every protected span, line endings and the trailing newline to survive
`--fix`. FR-8.2 forbids changing heading levels, list markers, table shape or block order. `pulldown-cmark`'s
`OffsetIter` yields events with source ranges (research `inference-and-models.md` §4).

## Options

### Option 1: Range edits over the source text

Parse once for spans. Engines propose `(range, replacement)` inside prose spans, and the applier splices accepted
proposals into the original string. **For:** every byte outside an accepted edit is the input byte by
construction. CRLF, odd list markers, table alignment and HTML survive without special code. Idempotence (FR-8.9)
holds when no rule matches its own output. **Against:** overlapping proposals need a deterministic conflict policy.
Each stage re-parses to see the new text.

### Option 2: Parse to an AST, edit text nodes, re-render (e.g. `comrak` round-trip)

**For:** structural edits are natural. **Against:** renderers normalise markup: list markers, emphasis characters,
escaping, table padding and line wrapping. That breaks FR-8.4 on ordinary documents. Every normalisation would need
to be found and suppressed.

## Decision

Option 1. Byte preservation is the requirement, and range editing gives it structurally rather than by testing
every renderer quirk. Conflicts are resolved by `edit::plan`: sort by start offset, then rule code, and drop any
proposal overlapping an earlier one or a frozen (suppressed) range. A dropped proposal stays a finding, so the next
run can fix it.

## Consequences

- **Easier:** protected-span guarantees, CRLF handling, diffs, whole-document verification (compare protected bytes).
- **Harder:** rules cannot restructure a list (e.g. collapse bullet spam into prose) except by replacing the
  list's text inside its prose spans. Structural rewrites are therefore model-class, and the verifier forbids new
  or removed block markers.
- **Follow-ups:** none.
