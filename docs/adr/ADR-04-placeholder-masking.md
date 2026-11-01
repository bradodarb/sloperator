# ADR-04: Model windows are prose blocks with protected inline spans replaced by numbered placeholders

**Status:** Accepted
**Date:** 2026-10-29
**Initiative:** [sloperator](../ARCHITECTURE.md) · **Serves:** FR-8.1, FR-9.2, FR-9.3, FR-9.5, D-7

## Context

D-7 says engines see only prose spans. Prose in real Markdown is interleaved with inline code, link destinations and
URLs inside one sentence. For example: ``Use `cargo` to [build](https://x) it``. A rewrite needs the sentence as a
whole. Small models are unreliable with facts (research §2), so the protected text must not be in the model's
hands at all.

## Options

### Option 1: Mask protected inline spans with numbered placeholders (`⟦1⟧`), unmask after

The window is one prose block (paragraph, list item, cell, heading), or a sentence when `model.window = "sentence"`.
Each protected inline span becomes `⟦n⟧`, and link text stays prose with its destination masked. **For:** the model
can move a code span within the sentence but cannot alter it. The verifier checks the placeholder multiset
exactly, so a dropped or duplicated one is rejected. Protected bytes are restored from the source. **Against:**
small models may mangle unfamiliar tokens. Every rejection is safe but lowers fix yield (Risk R2).

### Option 2: Send only the maximal pure-prose runs between protected spans

**For:** no placeholders for the model to mishandle. **Against:** fragments like "Use" / "to" / "it" cannot be
rewritten meaningfully. Tricolons and negative parallelism span the protected tokens, so most model-class findings
would be untouchable.

### Option 3: Send the raw block, verify protected bytes after

**For:** most natural input for the model. **Against:** the model sees and may "fix" URLs and code, and recovering
the original bytes means diffing model output against protected spans. That contradicts D-7's "engines receive
only prose spans".

## Decision

Option 1, with Option 2 kept as the documented degraded mode if the spike shows a high placeholder rejection rate. It
is a SPEC-level switch chosen from data, never a silent runtime fallback. The prompt profile per model (in the
model registry, ADR-08) explains the placeholders and gives few-shot examples (D-12).

## Consequences

- **Easier:** protected text is safe by construction; verification of placeholders is exact.
- **Harder:** fix yield depends on the model honouring placeholders. The spike must measure this.
- **Follow-ups:** spike measures placeholder survival per model (Phase 1). SPEC fixes the placeholder glyphs and
  checks that they cannot occur in input (if they do, the window is not sent and a finding is reported).
