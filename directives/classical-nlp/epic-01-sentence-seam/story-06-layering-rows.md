# Story 06: Layering rows and once-per-pass structural tests

**Epic:** [epic-01-sentence-seam](../epic-01-sentence-seam.md)
**SPEC Refs:** §6 (`text` module root and layering rows)
**PRD Refs:** NFR-2, NFR-5, NFR-7, FR-1.4
**Component / ADR:** Architecture §5 Module rules; ADR-01
**Wave:** 4

## Scope

In tests/layering.rs: add the `src/text/` row to `FORBIDDEN_USES`, add `"crate::rules"` to the `src/verify/` row (lines 40-43), and add `"src/text/"` to `PURE_DIRS`. Add `only_in_files`, reimplement `only_in` through it, and add `prose_built_only_per_pass`, `segmentation_runs_only_inside_prose` and `sentence_scan_only_in_segmenter`. `tagger_runs_only_inside_prose` lands in epic-07.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `tests/layering.rs` | 9-95, 96-103, 189-209, after 227 | rows, `PURE_DIRS`, `only_in_files`, three structural tests |

## Acceptance Criteria

- [ ] `src/text/` cannot use rules, verify, engine, pipeline, report, config, input, write, edit, score, directives, model, mcp or cli (NFR-7)
- [ ] `src/text/` is scanned by `pure_modules_have_no_io` (no `std::net`, `std::fs`, `std::process`, `std::env`) (NFR-5)
- [ ] `Prose::new(` appears only in the four allowed files; `segment::segment(` only in src/text/mod.rs; `split_sentence_bound_indices` only in src/text/segment.rs (NFR-2, FR-1.4)
- [ ] Every existing layering test still passes
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::layering` | `forbidden_uses` | the new text row and the widened verify row | NFR-7 |
| 2 | `sloperator::layering` | `pure_modules_have_no_io` | src/text/ has no I/O | NFR-5 |
| 3 | `sloperator::layering` | `prose_built_only_per_pass` | `Prose::new(` only where a pass parses, plus in-crate tests | NFR-2 |
| 4 | `sloperator::layering` | `segmentation_runs_only_inside_prose` | `segment::segment(` only in src/text/mod.rs | NFR-2 |
| 5 | `sloperator::layering` | `sentence_scan_only_in_segmenter` | one sentence definition | FR-1.4 |

## Notes

A substring scan cannot see `#[cfg(test)]`, so review of src/engine/window.rs and src/rules/mod.rs is what keeps `Prose::new(` inside their test modules.

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='sloperator::layering'`
