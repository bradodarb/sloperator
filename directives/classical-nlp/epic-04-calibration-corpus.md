# Epic 04: Calibration corpus and recorded metric values

**Goal:** Check in a public-domain human-prose corpus and record each metric's values per file and overall beside the provisional thresholds, so later threshold changes have evidence (D-4 follow-up).
**PRD Refs:** D-4 follow-up (PRD §7 Phase 1 step 6), FR-4.5
**SPEC Refs:** §19 (Calibration corpus), §36.11
**Modules:** `sloperator`

## Waves

Copied from this epic's line in `index.md`. Stories in one wave run in parallel; each wave waits for the one before.

| Wave | Stories | Why this order |
|------|---------|----------------|
| 1 | [story-01-calibration-corpus](./epic-04-calibration-corpus/story-01-calibration-corpus.md) | one story |

## Acceptance Criteria

- [ ] `eval/calibration/` holds at least 12 Markdown files of public-domain English prose, 1–3 MB in total, paragraphs only (D-4)
- [ ] `eval/calibration/README.md` lists every file with source URL, licence `public domain` and word count, plus a table of each metric per file and overall beside the provisional thresholds of SLP018–SLP021 (FR-4.5)
- [ ] No rule default changes in this epic (phase-1e gate)
- [ ] The tests named in every story's Test Cases pass together on the merged epic.

## Notes

Runs beside epic-03: it touches only `eval/calibration/` and `tests/calibration.rs`. The thresholds it records beside are SPEC §17's (12, 0.35, 25, 0.4); it does not read the rule files. Collecting the corpus needs network access to public-domain sources (US federal works under 17 U.S.C. §105, texts first published before 1929). No CC BY-SA or other attribution/share-alike text (DQ-16). Strip every distributor licence header. The files are repo data, not part of the binary.
