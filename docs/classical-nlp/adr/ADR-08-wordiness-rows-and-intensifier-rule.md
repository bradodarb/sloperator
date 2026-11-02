# ADR-08: Wordiness rows extend SLP001; detection-only intensifiers get a new `FixClass::None` rule

**Status:** Proposed (needs Brad: it reads a PRD phrase the code contradicts)
**Date:** 2026-11-02
**Initiative:** [classical-nlp](../ARCHITECTURE-classical-nlp.md) · **Serves:** FR-5.1–FR-5.5, FR-10.3, FR-11.2, D-10

## Context

FR-11.2 and D-11 refer to "the existing wordiness rule". No such rule exists: the Safe phrase table is SLP001
`ai-vocabulary-replaceable` (`src/rules/builtin/slp001_ai_vocabulary_replaceable.rs:1-27`), which PRD root cause 3
cites. FR-5.4 requires intensifier and weak-adverb entries to be detection-only. In a `WordList`, a row with no
replacement in a Safe rule produces a fix-less Safe finding, and a fix-less Safe finding **selects a model window**
(`src/engine/window.rs:116`). The model would then delete or rewrite "very", which D-10 rules out as a Safe edit and
which raises model reliance against FR-10.3.

## Options

### Option 1: Nominalisation and phrase-reduction rows go into SLP001; intensifiers and weak adverbs go into a new rule SLP022 `weak-intensifier`, FixClass::None, default-off, Suggestion

**For:** matches the PRD's "rows keep the existing rule's default" for the Safe rows; intensifiers never reach the model; the new rule follows parent FR-4.5 (starts off). **Against:** SLP001's name and `ai-vocabulary` family now cover generic wordiness; default users see new SLP001 findings on padded phrasing.

### Option 2: A new Safe rule `wordiness` (default-off) for all new rows, with intensifiers as `None` rows in it

**For:** clean naming. **Against:** default users get none of the new fixes; `None` rows still select model windows (same flaw as above); contradicts FR-11.2's carve-out.

### Option 3: Everything into SLP001, intensifiers as `None` rows

**For:** smallest change. **Against:** intensifiers become default-on findings that send windows to the model (D-10, FR-10.3).

## Decision

Option 1, proposed. Uncertain wordiness rows (FR-5.3) stay `None` rows in SLP001 and fall through to the model by
design, as SLP001 rows already do.

## Consequences

- **Easier:** no-model `--fix` gains the new Safe rows by default; intensifier detection costs no model time.
- **Harder:** SLP001's `explain` and family description must be widened; a later split into a dedicated wordiness rule would move rows between codes (a breaking change for directives that suppress SLP001).
- **Follow-ups:** Brad confirms Option 1 or picks Option 2 at the architecture gate.
