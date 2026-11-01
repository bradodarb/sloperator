# Story 01: Windows, placeholder masking, prompt and output contract

**Epic:** [epic-06-model-fix-path](../epic-06-model-fix-path.md)
**SPEC Refs:** §16 (window.rs, mask.rs, prompt.rs), §23.12 (window and mask tests), DQ-5, DQ-7, DQ-14
**PRD Refs:** FR-9.2, FR-9.3, FR-9.7, FR-8.2, D-12
**Component / ADR:** §4.10 engine; ADR-04
**Wave:** 1

## Scope

Implement `src/engine/window.rs` (Window, Wrap, WindowPlan, `windows`), `src/engine/mask.rs` (`mask`, `propose`: restores placeholders, re-wraps with prefixes) and `src/engine/prompt.rs` (system prompt, few-shot examples, `max_tokens`, the output contract: no think block, preamble, quotes or blank line; strip an empty think).

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/engine/window.rs` | new | window selection and Skip |
| `src/engine/mask.rs` | new | ⟦n⟧ masking and propose |
| `src/engine/prompt.rs` | new | prompt, few-shot, output contract, max_tokens |
| `tests/engine.rs` | new | tests below |

## Acceptance Criteria

- [ ] Windows are chosen only for model-class findings, and Safe findings with no fix, at or above min severity; a `none`-class finding selects no window (FR-9.2).
- [ ] Protected runs become `⟦n⟧` numbered from 1 in source order (DQ-7).
- [ ] Output with a think block, a preamble, quotes or a blank line is rejected; an empty think block is stripped (FR-9.3, DQ-14).
- [ ] A dropped or duplicated placeholder is rejected (ADR-04).
- [ ] Input already holding ⟦/⟧, an over-long window (`max_window_chars`) and a window with inconsistent prefixes become `Skip` (SLP902) (FR-9.7, DQ-5, Gap 7).
- [ ] Hard-wrapped windows, including a `> ` quote and a list item with continuation indent, are re-wrapped with every prefix kept (FR-8.2, Gap 7).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::engine` | `windows_only_for_findings_at_or_above_min_severity` | window selection | FR-9.2 |
| 2 | `sloperator::engine` | `none_class_selects_no_window` | none class | FR-9.2 |
| 3 | `sloperator::engine` | `mask_replaces_protected_runs_with_numbered_placeholders` | masking | ADR-04 |
| 4 | `sloperator::engine` | `contract_rejects_think_block_preamble_quotes_blank_line` | output contract | FR-9.3 |
| 5 | `sloperator::engine` | `strips_empty_think` | empty think stripped | FR-9.3 |
| 6 | `sloperator::engine` | `placeholder_dropped_or_duplicated_rejected` | placeholder integrity | ADR-04 |
| 7 | `sloperator::engine` | `input_with_placeholder_glyph_skipped_slp902` | glyph input skipped | FR-9.7, DQ-5 |
| 8 | `sloperator::engine` | `over_long_window_skipped_slp902` | long window skipped | FR-9.7, DQ-5 |
| 9 | `sloperator::engine` | `hard_wrapped_window_rewrapped` | rewrap | §16 |
| 10 | `sloperator::engine` | `wrapped_blockquote_and_list_item_rewrapped_with_prefixes` | prefixes kept | FR-8.2, FR-9.5 (Gap 7) |
| 11 | `sloperator::engine` | `inconsistent_prefix_window_skipped_slp902` | inconsistent prefix skipped | FR-9.7 (Gap 7) |

## Notes

Accepted Gap 13: `max_window_chars = 4000` stays; the spike may retune it. If the spike chose ADR-04 Option 2 (degraded masking), epic 07 switches the mode; this story builds Option 1.

Gate: `make test TESTS='sloperator::engine#windows_only_for_findings_at_or_above_min_severity sloperator::engine#none_class_selects_no_window sloperator::engine#mask_replaces_protected_runs_with_numbered_placeholders sloperator::engine#contract_rejects_think_block_preamble_quotes_blank_line sloperator::engine#strips_empty_think sloperator::engine#placeholder_dropped_or_duplicated_rejected sloperator::engine#input_with_placeholder_glyph_skipped_slp902 sloperator::engine#over_long_window_skipped_slp902 sloperator::engine#hard_wrapped_window_rewrapped sloperator::engine#wrapped_blockquote_and_list_item_rewrapped_with_prefixes sloperator::engine#inconsistent_prefix_window_skipped_slp902'`
