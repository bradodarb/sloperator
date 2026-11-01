# Epic 08: Eval harness and report

**Status:** delivered

**Goal:** `tests/eval.rs` produces the FR-10.2 report per cached model on the 20 eval paragraphs and enforces 20/20 integrity and the spike's latency budget, and both tiers run green through the contract.
**PRD Refs:** FR-10.1, FR-10.2, NFR-3 (model budget), FR-9.1, D-12
**SPEC Refs:** §22, §23.19, §21
**Modules:** `sloperator`

## Waves

Copied from this epic's line in `index.md`. Stories in one wave run in parallel; each wave waits for the one before.

| Wave | Stories | Why this order |
|------|---------|----------------|
| 1 | [story-01-eval-harness](./epic-08-eval/story-01-eval-harness.md), [story-02-no-network-sandbox](./epic-08-eval/story-02-no-network-sandbox.md), [story-03-spec21-runtime-text](./epic-08-eval/story-03-spec21-runtime-text.md) | Three independent pieces with disjoint files: the eval harness (`tests/eval.rs`, `eval/README.md`), the sandboxed no-network test (`tests/model_real.rs`), and the SPEC §21 text (`docs/SPEC.md`). Stories 02 and 03 are epic-07 acceptance-audit follow-ups (changes.md CH-16, CH-17). |
| 2 | [story-04-readme-standalone](./epic-08-eval/story-04-readme-standalone.md) | Acceptance-audit fix (Gap 1): story 01's `eval/README.md` named the host project, so `standalone#no_host_references` failed. This story rewrites the run instruction with cargo and touches only the README, so it runs after wave 1. |

## Acceptance Criteria

- [ ] `target/eval/report.json` and the printed table exist for the default model (FR-10.2).
- [ ] `make test` and `make test-integration` both exit 0 (architecture Phase 7 gate).
- [ ] FR-9.1 is proven by a test that denies the process network access, not only by proxy variables (CH-16).
- [ ] SPEC §21 describes the shipped adapter: owned model, per-window context, `send_logs_to_tracing` (CH-17).
- [ ] The tests named in every story's Test Cases pass together on the merged epic.

## Notes

Fine-tuning stays deferred (D-12). The report is the input to that later decision. Stories 02 and 03 come from the epic-07 acceptance audit's deferred Gaps 2 and 3 (`docs/backlog/epic-07-no-network-test-weak.md`, `sloperator-epic-07-spec21-leak-stale.md`); nothing depends on them, so they are appended here rather than blocking.
