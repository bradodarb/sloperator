# Epic 06: The model fix path, proven with FakeRewriter

**Goal:** `sloperator check --fix` sends findings' windows to the Rewriter port, masks protected runs, verifies every rewrite (strictly fewer weighted findings), writes only verified rewrites, reports rejections as SLP901/902, and `sloperator model pull|list|path` manages the cache. All of it is tested with FakeRewriter in the unit tier.
**PRD Refs:** FR-2.14, FR-6.6, FR-8.7, FR-9.2–FR-9.8, D-3, D-4, D-5, Brad Q2 (a), Brad Q3 (a)
**SPEC Refs:** §15 (pull, CLI), §16 (windows, mask, prompt), §17 (step 2), §23.11, §23.12, §23.13, DQ-3, DQ-4, DQ-5, DQ-7, DQ-14
**Modules:** `sloperator`
**Status:** Delivered. All three stories merged: `engine::{window,mask,prompt}` (windows, protected-run masking, prompt), `model::pull` with the `model pull|list|path` CLI, and the model step of `pipeline::fix` (verify, then write; rejections reported as SLP901/SLP902), all tested with FakeRewriter. The real llama.cpp adapter behind feature `model` stays for epic 07.

## Waves

Copied from this epic's line in `index.md`. Stories in one wave run in parallel; each wave waits for the one before.

| Wave | Stories | Why this order |
|------|---------|----------------|
| 1 | [story-01-windows-mask-prompt](./epic-06-model-fix-path/story-01-windows-mask-prompt.md), [story-02-model-pull-commands](./epic-06-model-fix-path/story-02-model-pull-commands.md) | Parallel: windowing/masking/prompt (src/engine/{window,mask,prompt}.rs) and model pull with its CLI (src/model/pull.rs, cli) touch disjoint files. |
| 2 | [story-03-model-window-pipeline](./epic-06-model-fix-path/story-03-model-window-pipeline.md) | The pipeline's model step uses windows, masks and the prompt from wave 1. |

## Acceptance Criteria

- [ ] Under FakeRewriter, a good rewrite is written and a rewrite that drops a number is rejected with the original kept (D-4).
- [ ] No window means no model load; many windows load once (FR-9.4).
- [ ] `sloperator model pull` against a local server verifies sha256 and removes a corrupt `.part` (FR-2.14).
- [ ] The tests named in every story's Test Cases pass together on the merged epic.

## Notes

This is the spike-independent part of Phase 6 (SPEC Architecture Deviation Notes §12). Brad Q3 (a): verified model rewrites are written by `--fix`; `--diff` previews them. Gap 7 (container prefixes) applies to story 01.
