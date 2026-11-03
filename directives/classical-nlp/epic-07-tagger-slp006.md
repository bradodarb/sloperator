# Epic 07: Brill-style tagger and tag-gated SLP006 subjects

**Goal:** A static-table tagger, run at most once per document pass and only when a rule asks, lets SLP006 fix "Det N serves as" and "Det Adj N serves as" when every subject word is certain. A phase-start spike proves the tagger can do it first.
**PRD Refs:** FR-7.1–FR-7.8, NFR-2, NFR-4, NFR-10, D-7
**SPEC Refs:** §26 (text::tagger), §27 (Prose::tags, lazy, once per pass; the Gap 9 pipeline test), §28 (SLP006 tag-gated subjects; Phase 4 spike gate), §6 (`tagger_runs_only_inside_prose`), §35 (perf), §36.8, §36.7
**Modules:** `sloperator`

## Waves

Copied from this epic's line in `index.md`. Stories in one wave run in parallel; each wave waits for the one before.

| Wave | Stories | Why this order |
|------|---------|----------------|
| 1 | [story-01-tagger-and-spike-gate](./epic-07-tagger-slp006/story-01-tagger-and-spike-gate.md) | spike gate first (architecture §12 phase 4): the tagger and its spike test; nothing consumes tags yet |
| 2 | [story-02-prose-tags-and-slp006](./epic-07-tagger-slp006/story-02-prose-tags-and-slp006.md) | `Prose::tags` and the SLP006 branch need the tagger; they share src/text/mod.rs, so they wait for story-01 |
| 3 | [story-03-multi-window-pass-test](./epic-07-tagger-slp006/story-03-multi-window-pass-test.md) | the multi-window pass test needs SLP006 to tag the original pass; the docs and phase row need the finished rule |

## Acceptance Criteria

- [ ] Spike gate: `text_tagger::spike_slp006_classes` passes, and `docs/classical-nlp/spikes/G4-slp006-tagger-spike.md` records the classes tried, the fixtures, and verdict PASS (FR-7.5, FR-7.8)
- [ ] `sloperator fix` turns "The library serves as a guide." into "The library is a guide." and "The main library serves as a guide." into "The main library is a guide.", and leaves "The tool serves as a bridge." unfixed (FR-7.5, FR-7.7)
- [ ] One document pass tags at most once, and a document with no candidate is never tagged (FR-7.3, FR-7.4, NFR-2)
- [ ] A three-window sentence-mode fix runs five passes, each scanning each block once and tagging at most once, and window planning scans nothing (NFR-2, Gap 9)
- [ ] `eval/model-reliance.md` has a `phase 4` row ≤ baseline (FR-10.2, FR-10.3)
- [ ] The tests named in every story's Test Cases pass together on the merged epic.

## Notes

Spike gate (SPEC §28, architecture §12, review Gap 7): story-01 must fail loudly when no subject class is certain on the SLP006 positives or the tagger fails to decline a negative. Then the story records the classes tried and the failing fixtures in the spike file with verdict FAIL, and the epic stops. It continues only when the tables or rules are corrected and the spike passes, or when Brad records a scope decision against FR-7.5–FR-7.8. The widening is never dropped automatically, and epic-08 needs the spike file (needs block). Lexicon policy: `tool`, `service`, `guide` are `[Noun, Verb]`, `fast` is `[Adj, Adv]`, `these`/`this`/`that` include Det and Pron, and there are no ProperNoun entries (§26). The Gap 9 test's exact pass count (5) depends on `tool` staying ambiguous.
