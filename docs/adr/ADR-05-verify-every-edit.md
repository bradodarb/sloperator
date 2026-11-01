# ADR-05: Every edit from either engine passes the verifier, enforced by a `Verified<T>` type

**Status:** Accepted
**Date:** 2026-10-29
**Initiative:** [sloperator](../ARCHITECTURE.md) · **Serves:** D-4, D-7, FR-8.5-8.7, FR-9.5, NFR-4

## Context

D-4 writes model rewrites only after verification. Safe rule fixes are deterministic, so a reviewer could argue
they need no check. But a bad regex in a safe rule can also eat a number or a URL. The project's fail-loudly rule
forbids any path that writes unchecked output.

## Options

### Option 1: One verifier for all edits; `edit::apply` accepts only `Verified<Proposal>`

**For:** a single choke point, impossible to skip without changing `verify`'s visibility. Rule bugs are caught the
same way model errors are. The whole-document check after all stages is cheap defence in depth. **Against:** small
cost per safe edit (a token scan of a short window). Rule fixes that legitimately change a number (none exist in the
FR-4.2 families) would be impossible.

### Option 2: Verify model edits only; trust rule fixes via their fixtures

**For:** marginally faster. Simpler rules path. **Against:** fixtures cover the cases authors thought of. A
verifier gap then becomes a silent corruption in the default `--fix` path.

## Decision

Option 1. The checks are: number, URL, term and placeholder multisets equal; same block type; no new block marker;
for model output, the output contract (no preamble, no thinking block) and no increase in weighted findings for the
window. A failure keeps the original text and emits finding SLP901 `rewrite-rejected` naming the window and the
failed check. SLP901 counts toward exit 1 and cannot be ignored.

## Consequences

- **Easier:** reasoning about safety: one module, one test suite (driven by `FakeRewriter` and forged proposals).
- **Harder:** the verifier must tokenise numbers consistently (`1,000` vs `1000`, `3.5%`); a strict tokenizer may
  reject harmless rewrites, which shows up as SLP901, never as corruption.
- **Follow-ups:** the SPEC defines the number/URL tokenizer grammar and its fixtures.
