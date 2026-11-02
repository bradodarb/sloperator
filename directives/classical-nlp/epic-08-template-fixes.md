# Epic 08: SLP004 and SLP003 template fixes (class → Safe)

**Goal:** Two tells that only the model could fix today get deterministic template fixes when every guard passes. They pass the verifier only through the D-8 negation allowance, and anything unproven stays fix-less for the model.
**PRD Refs:** FR-8.1–FR-8.7, FR-10.2, FR-10.3, NFR-6, NFR-9, D-8, D-10
**SPEC Refs:** §29 (SLP004 template `X and Y`), §30 (SLP003 template `it's Y`), §31 (Class change fallout), §36.9, §36.7 (`failing_template_fix_is_slp901`)
**Modules:** `sloperator`

## Waves

Copied from this epic's line in `index.md`. Stories in one wave run in parallel; each wave waits for the one before.

| Wave | Stories | Why this order |
|------|---------|----------------|
| 1 | [story-01-slp004-template](./epic-08-template-fixes/story-01-slp004-template.md) | SLP004 first (reads tags from epic-07) |
| 2 | [story-02-slp003-template](./epic-08-template-fixes/story-02-slp003-template.md) | SLP003 also edits tests/rules_fixtures.rs (macro arm, new tests), so it waits for story-01 |
| 3 | [story-03-class-change-fallout](./epic-08-template-fixes/story-03-class-change-fallout.md) | fallout docs, the registry/pipeline tests and the phase-5 row need both class changes |

## Acceptance Criteria

- [ ] `sloperator fix` turns "It is not only cheap but also quick." into "It is cheap and quick." and "It's not a bug, it's a feature." into "It's a feature." (FR-8.1, FR-8.2)
- [ ] "The tool is not just fast, but also simple.", "It's not only cheap, it's quick." and "It's not a bug that we can fix in an hour, it's a feature." keep their findings with no fix (FR-8.3, Gap 3)
- [ ] SLP003 and SLP004 are `FixClass::Safe`, and SLP007 and SLP015 are unchanged (FR-8.4, FR-8.6)
- [ ] A forged template fix that drops Y's capitalised word becomes SLP901 (NFR-6, FR-8.5)
- [ ] Fixing is idempotent over every fixture, and the `phase 5` model-reliance row is ≤ baseline (FR-8.7, FR-10.3)
- [ ] The tests named in every story's Test Cases pass together on the merged epic.

## Notes

Needs epic-07 and its spike record (`docs/classical-nlp/spikes/G4-slp006-tagger-spike.md`, verdict PASS). Architecture §12 forbids starting phase 5 on a tagger-only phase 4 without Brad's scope decision, so a FAIL record also stops this epic. JSON `fix.available` for SLP003/SLP004 becomes "a template fix was attached": `false` on an unfixed match, even though the model may still rewrite it (architecture §14 risk). The code does not change; the comment, REFERENCE and CHANGELOG say so.
