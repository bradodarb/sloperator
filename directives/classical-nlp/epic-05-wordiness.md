# Epic 05: SLP001 wordiness rows and SLP022 weak-intensifier

**Goal:** No-model `--fix` rewrites padded verbs, nominalisations and padded phrases through default-on SLP001 (guarded so every fix is grammatical), and default-off SLP022 flags intensifiers without a fix (Brad's Gap 4 decision (a)).
**PRD Refs:** FR-5.1–FR-5.5, FR-10.2, FR-10.3, FR-11.2, D-9, D-10
**SPEC Refs:** §20 (SLP001 wordiness and nominalisation rows), §21 (SLP022 weak-intensifier), §18 (catalogue), §35 (perf, Phase 2), §2 (phase row), §36.5
**Modules:** `sloperator`

## Waves

Copied from this epic's line in `index.md`. Stories in one wave run in parallel; each wave waits for the one before.

| Wave | Stories | Why this order |
|------|---------|----------------|
| 1 | [story-01-slp001-wordiness-rows](./epic-05-wordiness/story-01-slp001-wordiness-rows.md) | SLP001 rows and guard first; they touch tests/rules_fixtures.rs |
| 2 | [story-02-slp022-weak-intensifier](./epic-05-wordiness/story-02-slp022-weak-intensifier.md) | SLP022 also edits tests/rules_fixtures.rs (macro) and builtin/mod.rs, so it waits for story-01 |
| 3 | [story-03-wordiness-catalogue](./epic-05-wordiness/story-03-wordiness-catalogue.md) | catalogue, perf config and the phase-2 model-reliance row need both rules |

## Acceptance Criteria

- [ ] `sloperator fix` with default settings turns "They made a decision to leave." into "They decided to leave." and "We stayed due to the fact that it rained." into "We stayed because it rained." (FR-5.1, FR-5.2)
- [ ] Every one of the twenty `GUARD_COUNTER_CONTEXTS` gives no SLP001 finding and is left unchanged by a rules-only fix (D-10, FR-5.3, Gap 2)
- [ ] SLP022 is default-off, `FixClass::None` and never attaches a fix (FR-5.4, D-10)
- [ ] Fixing is idempotent over every fixture (FR-5.5)
- [ ] `eval/model-reliance.md` has a `phase 2` row, and the residual is ≤ baseline (FR-10.2, FR-10.3)
- [ ] The tests named in every story's Test Cases pass together on the merged epic.

## Notes

Gap 4 is decided (a): rows extend SLP001 (default-on, so default users gain these fixes and findings), and intensifiers go in a separate default-off detection-only SLP022. SLP001's name and family stay as they are. No row adds or removes a modal or negation, and none is a `None` row (DQ-17), so epic-06's FR-6.6 gate and the FR-10 residual hold. A failed guard drops the match: no finding and no fix (DQ-22). `a sufficient number of` and `in the event that` stay out (omitted rows).
