# Epic 06: Verifier meaning invariants: negations, names, modals

**Goal:** Every edit that reaches the source now also keeps its negations (apart from the D-8 allowance), its capitalised non-initial words and its modal verbs; a failing edit becomes a loud SLP901, and no existing Safe fixture fix is newly rejected.
**PRD Refs:** FR-6.1–FR-6.8, NFR-6, NFR-9, D-8
**SPEC Refs:** §22 (text::lexicon), §23 (verify::meaning, Check variants, verify::edit(resolves)), §24 (Window.codes, call sites, check names), §25 (verification-limits sentence), §36.6, §36.7
**Modules:** `sloperator`

## Waves

Copied from this epic's line in `index.md`. Stories in one wave run in parallel; each wave waits for the one before.

| Wave | Stories | Why this order |
|------|---------|----------------|
| 1 | [story-01-closed-class-lexicon](./epic-06-verifier-invariants/story-01-closed-class-lexicon.md), [story-02-window-codes](./epic-06-verifier-invariants/story-02-window-codes.md) | independent: the lexicon (src/text/, slp006) and `Window.codes` (src/engine/window.rs, tests/engine.rs) touch disjoint files |
| 2 | [story-03-meaning-checks](./epic-06-verifier-invariants/story-03-meaning-checks.md) | `verify::meaning` uses the lexicon, and the pipeline passes `&w.codes` |
| 3 | [story-04-verification-limits-sentence](./epic-06-verifier-invariants/story-04-verification-limits-sentence.md) | the module header sentence lives in src/verify/mod.rs, which story-03 edits |

## Acceptance Criteria

- [ ] A model edit that drops a `not` is rejected as SLP901 `rewrite rejected (negation): negations 1 -> 0 (allowance 0)`, and the original bytes stay (FR-6.1, FR-6.4, FR-6.5)
- [ ] A model edit that removes the non-initial `It` in "We watched It and a robust sequel." is rejected with `Check::ProperNouns`, and the rule-origin SLP009 fix "It May Potentially fail." → "It May fail." still passes (FR-6.2, Gap 11 (a))
- [ ] "It must ship." → "It should ship." is rejected with `Check::Modals`; "It will not ship." → "It won't ship." passes (FR-6.3)
- [ ] FR-6.6 gate: zero Safe fixture fixes are newly rejected (fail = any) (FR-6.6)
- [ ] "not names or claim polarity" appears nowhere in the repo (FR-6.7, NFR-9)
- [ ] The tests named in every story's Test Cases pass together on the merged epic.

## Notes

Gap 11 is decided (a): `meaning(text, initial_at_start, skip_common)` skips capitalised common words (`NOT_NAMES`, stop words, modals, hedges) for `ProposalOrigin::Rule` only, and model rewrites get FR-6.2's literal proxy. That matches what SPEC §22–§23 already write, so the Architecture Deviation Note on §4.10's `meaning` signature stands as the chosen form. DQ-10: a mid-sentence all-caps SLP001 replacement or a title-case heading re-case becomes SLP901 (loud, never silent), and no fixture has one. `verify` may now use `crate::text` but not `crate::rules` (epic-01 story-06 layering row).
