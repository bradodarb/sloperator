# Story 03: Gap 9 multi-window pass test, tagger layering test, docs and phase-4 row

**Epic:** [epic-07-tagger-slp006](../epic-07-tagger-slp006.md)
**SPEC Refs:** §27 (pipeline test-only records; `multi_window_fix_analyses_each_pass_once`), §6 (`tagger_runs_only_inside_prose`), §18 (catalogue), §35 (perf), §2 (phase row)
**PRD Refs:** NFR-2, FR-7.3, FR-7.4, FR-10.2, FR-10.3, FR-11.3
**Component / ADR:** Architecture §4.11 Pipeline (behavioral NFR-2 test), §5 Module rules
**Wave:** 3

## Scope

Deferred Gap 9: in src/pipeline/mod.rs add the `#[cfg(test)]` `Pass` record and the `PASSES`/`PLAN_SCANS` thread-locals, record them in `detect_in` and around `windows` in `model_windows`, and add the in-crate `mod tests` with `Plain`/`PlainFactory` and `multi_window_fix_analyses_each_pass_once` exactly as §27. Add `tagger_runs_only_inside_prose` to tests/layering.rs. Update the SLP006 row in CHANGELOG.md, docs/REFERENCE.md and docs/SPEC.md §10. Append a `phase 4` row to eval/model-reliance.md, and confirm the perf test still passes.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/pipeline/mod.rs` | 295-309 (`detect_in`), 212-227 (`model_windows`), end | test-only records; `#[cfg(test)] mod tests` |
| `tests/layering.rs` | structural tests | `tagger_runs_only_inside_prose` |
| `CHANGELOG.md` | 0.2.0 section | SLP006 tag-gated subjects; tagger |
| `docs/REFERENCE.md` | SLP006 entry | widened description |
| `docs/SPEC.md` | §10 SLP006 row | widened description |
| `eval/model-reliance.md` | table | `phase 4` row |
| `tests/model_reliance.rs` | existing | tests below (methods added) |
| `tests/perf.rs` | existing | tests below (methods added) |
| `tests/standalone.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] Through the fake rewriter seam, a sentence-mode fix of the three-sentence paragraph runs exactly 5 passes: the original, three candidates and the final text (Gap 9)
- [ ] Every pass scans each block exactly once (`scans == blocks`) and tags at most once; a second `Prose` or a second tagging within a pass fails the test, while separate candidate passes are allowed (NFR-2, Gap 9)
- [ ] The original pass tags once (its SLP006 hit) and the final pass never tags (FR-7.4)
- [ ] `PLAN_SCANS == [0]`: window planning scans nothing (NFR-2, Gap 1)
- [ ] `tagger::tag(` appears only in src/text/mod.rs (NFR-2)
- [ ] The `phase 4` row is recorded and ≤ baseline, and the perf budget holds (FR-10.2, FR-10.3, NFR-2)
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::pipeline::tests` | `multi_window_fix_analyses_each_pass_once` | per-pass scans and tag runs across a multi-window fix | NFR-2, FR-7.3, FR-7.4 |
| 2 | `sloperator::layering` | `tagger_runs_only_inside_prose` | tags computed only by `Prose::tags` | NFR-2 |
| 3 | `sloperator::model_reliance` | `residual_does_not_exceed_baseline` | phase-4 value ≤ baseline | FR-10.2, FR-10.3 |
| 4 | `sloperator::perf` | `rules_only_100kb_under_1s` | budget holds with tag paths | NFR-2 |
| 5 | `sloperator::standalone` | `no_host_references` | doc edits use the repo's own form | NFR-1 |

## Notes

The model file is a placeholder (`model::resolve` checks only that the path is a file and the profile is known, src/model/mod.rs:59-83); `Plain` never loads it. `tempfile` is a normal dependency. If the tagger ever fixes "The tool serves as a bridge." in step 1, the pass count changes, and the assertion message lists the passes.

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='pipeline::tests::multi_window_fix_analyses_each_pass_once sloperator::layering model_reliance sloperator::perf sloperator::standalone#no_host_references'`
