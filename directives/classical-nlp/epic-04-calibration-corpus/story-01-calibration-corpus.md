# Story 01: Public-domain calibration corpus, README tables and guard test

**Epic:** [epic-04-calibration-corpus](../epic-04-calibration-corpus.md)
**SPEC Refs:** §19 (Calibration corpus)
**PRD Refs:** D-4 follow-up, FR-4.5
**Component / ADR:** Architecture §12 phase 1e
**Wave:** 1

## Scope

Add the corpus files under eval/calibration/ (≥ 12, different lengths, headings and lists removed), `eval/calibration/README.md` with the licence table and the metric table, and tests/calibration.rs exactly as SPEC §19. It asserts that every file is listed as public domain and passes the metric floors, and prints each file's metrics. Fill in the README metric table from the printed values. No `src/` change, and no rule default change.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `eval/calibration/*.md` | new | ≥ 12 public-domain prose files, 1–3 MB total |
| `eval/calibration/README.md` | new | `| file | source (URL) | licence | words |` table; metric values per file and overall beside the provisional thresholds |
| `tests/calibration.rs` | new | tests below |
| `tests/standalone.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] At least 12 corpus files exist, and the test asserts it (D-4)
- [ ] Every corpus file has a README row containing `| public domain |` (DQ-16)
- [ ] Every file passes the default metric floors (`avg_sentence_length` is Some) (FR-2.7)
- [ ] The README metric table holds the printed values for every file and the whole corpus, beside the provisional thresholds 12 / 0.35 / 25 / 0.4 (FR-4.5)
- [ ] No file under `src/` or `fixtures/` changes
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::calibration` | `calibration_corpus_is_licensed_and_measurable` | ≥ 12 files, each listed public domain and measurable | D-4, FR-4.5 |
| 2 | `sloperator::standalone` | `no_host_references` | the corpus and README name no host build contract | NFR-1 |

## Notes

Uses `Prose::new(&d).counts(&d).derive(&Floors::default())` (epic-02 changed `derive` to take floors). `tests/standalone.rs` scans every file, so a corpus text that happens to contain a host marker would fail it; pick or trim texts accordingly.

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='sloperator::calibration sloperator::standalone#no_host_references'`
