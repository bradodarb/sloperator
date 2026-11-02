# sloperator — Epics and Stories

**Source:** [PRD](../PRD.md) | [SPEC](../SPEC.md) | [Architecture](../ARCHITECTURE.md)
**Goal:** Deliver sloperator, a standalone Rust CLI that lints LLM-generated prose and rewrites it with safe rule fixes and verified local-model rewrites, built and tested through its own Makefile.
**Date:** 2026-10-30

## Delivery Plan

One line per epic, in delivery order. Stories separated by spaces run in parallel; `|` starts the next wave.

```slice
epic-01-walking-skeleton: story-01-crate-and-contract-adapter | story-02-layering-and-standalone
epic-02-inference-spike: story-01-spike-record
epic-03-protection-core: story-01-doc-span-map | story-02-verify-gate | story-03-edit-plan-apply | story-04-neutral-term-fixture
epic-04-rules-score-directives: story-01-rule-core | story-02-rules-vocabulary-hedging story-03-directives story-04-score | story-05-rule-copula | story-06-rules-patterns-residue | story-07-rules-density-markup | story-08-catalogue-and-check-wiring
epic-05-linter-surface: story-01-config | story-02-input story-03-model-registry-and-port | story-04-pipeline | story-05-report story-06-write | story-07-cli-check | story-08-perf-budget story-09-registry-metadata-assert
epic-06-model-fix-path: story-01-windows-mask-prompt story-02-model-pull-commands | story-03-model-window-pipeline
epic-07-llama-runtime: story-01-runtime-adapter | story-02-fmt-check-green
epic-08-eval: story-01-eval-harness story-02-no-network-sandbox story-03-spec21-runtime-text | story-04-readme-standalone
epic-09-backlog: story-01-cli-exit-tmpdir-independent
```

What each epic needs before it may start. The dependencies are complete, so the order is explicit. The spike
(epic 02) runs in parallel with epics 03-06 (architecture §12: "Phases 2-5 do not depend on the spike"). Epic 07,
the llama.cpp adapter, waits for the spike's results record. That is the Phase 1 gate (SPEC §1). If the spike
fails, the record says so and the run goes to Brad (D-1 fallback).

```needs
order: explicit
epic-02-inference-spike: epic-01-walking-skeleton
epic-03-protection-core: epic-01-walking-skeleton
epic-04-rules-score-directives: epic-03-protection-core
epic-05-linter-surface: epic-04-rules-score-directives
epic-06-model-fix-path: epic-05-linter-surface
epic-07-llama-runtime: epic-06-model-fix-path epic-02-inference-spike docs/research/spike-results.md
epic-08-eval: epic-07-llama-runtime
epic-09-backlog: epic-08-eval
```

## Epics

| Epic | Outcome | PRD Refs | Stories |
|------|---------|----------|---------|
| [epic-01-walking-skeleton](./epic-01-walking-skeleton.md) (**delivered**) | `sloperator check` builds and runs, and `make test` runs its tests through the contract (Phase 0 prerequisite, Phase 2). | FR-1.1–1.9, FR-2.16, NFR-1, NFR-2, NFR-7 | 2 |
| [epic-02-inference-spike](./epic-02-inference-spike.md) (**delivered**) | The spike record picks the runtime crate, the default model and the masking mode (Phase 1 gate). | FR-9.9, D-6 | 1 |
| [epic-03-protection-core](./epic-03-protection-core.md) (**delivered**) | Protected spans, the verifier and source-range edits keep every protected byte intact (Phase 3). | FR-8.1–8.6, FR-8.9, FR-9.5, FR-2.5 | 3 |
| [epic-04-rules-score-directives](./epic-04-rules-score-directives.md) (**delivered**) | 16 rules, the score and inline directives; `check` reports findings and exits 1 (Phase 4). | FR-4, FR-5, FR-6, FR-2.7 | 8 |
| [epic-05-linter-surface](./epic-05-linter-surface.md) (**delivered**) | The full linter: config, input, pipeline, report, write, every CLI flag, and the NFR-3 budget (Phase 5). | FR-2, FR-3, FR-4.1, FR-7, FR-8.8, FR-9.6, NFR-3–6, NFR-9 | 9 |
| [epic-06-model-fix-path](./epic-06-model-fix-path.md) (**delivered**) | Model windows, masking, `model pull` and the verified model-fix step, all proven with FakeRewriter (spike-independent part of Phase 6). | FR-2.14, FR-8.7, FR-9.2–9.8 | 3 |
| [epic-07-llama-runtime](./epic-07-llama-runtime.md) (**delivered**) | The llama.cpp adapter behind feature `model`; integration tier green on a real model (Phase 6). | FR-9.1, FR-9.4, FR-9.5, FR-9.8 | 1 |
| [epic-08-eval](./epic-08-eval.md) (**delivered**) | The eval report per model, plus the full unit and integration tiers green through the contract (Phase 7). | FR-10.1, FR-10.2, NFR-3, FR-9.1 | 4 |
| [epic-09-backlog](./epic-09-backlog.md) (**delivered**) | Non-blocking delivery follow-ups: the `cli_exit` tests pass when `$TMPDIR` has an ignore-all `.gitignore`, so the integration tier is green on any host (changes.md CH-18). | FR-2.7–2.9, FR-1.7 | 1 |

## Deferred items (from `deferred.md`)

| # | Item | Lands in |
|---|------|----------|
| 1 | The integration-tier JUnit report is `target/nextest/it/junit.xml` | AC in [epic-01 story-02](./epic-01-walking-skeleton/story-02-layering-and-standalone.md) |
| 2 | `TESTS` reaches cargo (through MAKEFLAGS, DQ-2): one selector gives exactly one testcase | AC in [epic-01 story-01](./epic-01-walking-skeleton/story-01-crate-and-contract-adapter.md) |
| 3 | FR-9.5 means strictly fewer weighted findings (DQ-3); a rewrite with an equal score is rejected | AC in [epic-03 story-02](./epic-03-protection-core/story-02-verify-gate.md) and [epic-06 story-03](./epic-06-model-fix-path/story-03-model-window-pipeline.md) |
| 4 | SLP901 is always `warning`, is never filtered by `--min-severity`, counts toward exit 1, and is still printed under `--exit-zero` (DQ-4) | AC in [epic-06 story-03](./epic-06-model-fix-path/story-03-model-window-pipeline.md) |
| 5 | Gap 12: `perf.rs` proves that the cache is never read by pointing `model.cache_dir` at an unreadable directory | AC in [epic-05 story-08](./epic-05-linter-surface/story-08-perf-budget.md) |

## Brad's decisions applied

- PRD gate Q1–Q8: all assumed options, built as the SPEC's DQ-0 records them. This means a self-contained crate,
  explicit `model pull` with exit 2 when the model is missing, verified model rewrites written by `--fix`, Qwen3-4B
  Q4_K_M as the default after the spike, `sloperator.toml` with snake_case keys, the CLI surface as written, the
  FR-1.9 banner, and the MIT licence.
- Q7 follow-up and SPEC gate Gap 11 / OD-1 (distribution binaries): Brad confirmed that Q7 governs only building the
  crate, and that a CI/CD process for distribution binaries is **out of scope for now**. No story builds
  release archives or a `sloperator.package` target. `make build` produces only the host binary (SPEC §3).

## Accepted Gaps

| Gap | Where it matters | Note |
|-----|------------------|------|
| Gap 13 (`slop_count`) | epic-01 story-01 | A selector that matches nothing fails the recipe. This is intended (fail loudly). |
| Gap 13 (`max_window_chars = 4000`) | epic-06 story-01 | Over-long windows become SLP902, never a silent drop. The spike may retune the value. |
| Gap 13 (`Box::leak`) | epic-07 story-01 | One model per process (FR-9.4). The adapter owns and drops the model before exit instead of leaking it: a live model at `process::exit` trips ggml's Metal teardown assert (`runtime.rs`, README). SPEC §21 wording is stale (deferred). |
| PRD ACCEPT: Rust tests not wired into `units BASE=` / `changed-tests` | all | Gates name sloperator selectors explicitly. |
