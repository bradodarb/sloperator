# ADR-04: Metric counts are sufficient statistics plus a type set; overall is a merge

**Status:** Accepted
**Date:** 2026-11-02
**Initiative:** [classical-nlp](../ARCHITECTURE-classical-nlp.md) · **Serves:** FR-2.1–FR-2.9, D-6, NFR-3

## Context

D-6 requires per-file and overall metrics from summed integers in one fixed order, never a mean of per-file values,
like `score::overall` (`src/score/mod.rs:39-48`). FR-2.1 lists per-sentence lengths and distinct types. Two facts:
the population SD needs only n, Σx and Σx², all additive; distinct types are **not** additive across files.

## Options

### Option 1: `Counts { words, sentences, syllables, sentence_len_sq: u64, types: BTreeSet<Box<str>> }`; merge = add + union

**For:** exact SD from integers; exact overall TTR; deterministic (integer sums, sorted set); no per-sentence list kept. **Against:** holds a type set per file until the summary is rendered; departs from FR-2.1's literal "list".

### Option 2: Keep the per-sentence length list and a distinct-type count; sum counts across files

**For:** literal FR-2.1/D-6 wording. **Against:** overall TTR wrong (shared words double-counted); the list is O(sentences) per file for no gain.

### Option 3: Moving-average TTR over fixed windows (summable integers)

**For:** length-robust (PRD Risk 3) and additive. **Against:** changes FR-2.5's formula, which the PRD fixes; needs Brad.

## Decision

Option 1, recorded as PRD Deviation Note 1. Derived values are computed only in `Counts::derive` and rounded to two
decimals with the score's rounding rule; files merge in the report's path order.

## Consequences

- **Easier:** overall metrics are exact and byte-stable; adding a metric means adding a counter.
- **Harder:** `FileResult` carries a set, not a few numbers; Option 3 remains a later, explicit change.
- **Follow-ups:** REFERENCE documents TTR length dependence.
