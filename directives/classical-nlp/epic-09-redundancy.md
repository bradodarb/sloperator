# Epic 09: Stemmer decision, similarity and SLP023 redundant-sentence

**Goal:** Default-off SLP023 reports a sentence that repeats most of the content words of an earlier sentence in the same paragraph, using a stemmer chosen through `cargo deny` and weighted Jaccard over integer counts. The final model-reliance value is still within the baseline.
**PRD Refs:** FR-9.1–FR-9.6, FR-10.2, FR-10.3, FR-11.2, NFR-2, NFR-5, D-9
**SPEC Refs:** §32 (text::stem), §33 (text::similarity), §34 (SLP023), §35 (Performance test extension), §18 (catalogue, binary size), §2 (phase row), §36.10, §36.11
**Modules:** `sloperator`

## Waves

Copied from this epic's line in `index.md`. Stories in one wave run in parallel; each wave waits for the one before.

| Wave | Stories | Why this order |
|------|---------|----------------|
| 1 | [story-01-stemmer-decision](./epic-09-redundancy/story-01-stemmer-decision.md) | the stemmer decision comes first: similarity calls `stem` |
| 2 | [story-02-similarity](./epic-09-redundancy/story-02-similarity.md) | similarity needs `stem` and edits src/text/mod.rs after story-01 |
| 3 | [story-03-slp023-redundant-sentence](./epic-09-redundancy/story-03-slp023-redundant-sentence.md) | SLP023 needs `similarity::bag`/`jaccard` |
| 4 | [story-04-redundancy-catalogue-and-perf](./epic-09-redundancy/story-04-redundancy-catalogue-and-perf.md) | perf config lists SLP023 (it must exist), and the final docs and phase row need the shipped rule |

## Acceptance Criteria

- [ ] The `cargo deny check` result for `rust-stemmers = "=1.2.0"` is recorded in CHANGELOG.md, and exactly one `stem` body ships (D-9)
- [ ] make/makes/making share one stem, and caches/cached/caching share one stem, under the backend that ships (FR-9.1, Gap 8)
- [ ] `sloperator check --extend-select SLP023` reports a paragraph's repeated sentence on its first prose run and names the earlier sentence's line; the same sentence in two paragraphs is not reported (FR-9.2, FR-9.3, FR-9.5)
- [ ] SLP023 is default-off, `FixClass::None`, and its `explain` says detection only and provisional (FR-9.6, FR-4.5)
- [ ] The perf test passes with SLP018–SLP023 on; the release-binary size after is recorded; the `phase 6` model-reliance row is ≤ baseline (NFR-2, NFR-5, FR-10.3)
- [ ] The tests named in every story's Test Cases pass together on the merged epic.

## Notes

Deferred Gap 8 lands in story-01. Whichever backend ships, `inflections_meet` asserts the same equivalences unchanged. If the fallback (b) is chosen, its final-`e` floor is 3 (§32), so make/makes/making meet at "mak". If a backend fails the assertion, repair the backend; never weaken the test. Choosing (a) adds an exact pin and an `only_in` layering test that allows `rust_stemmers` only in src/text/stem.rs. `deny.toml` changes only if a licence entry is needed.
