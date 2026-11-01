# Epic 05: The linter surface: config, input, pipeline, report, write, CLI

**Goal:** `sloperator check` takes the full CLI surface: files, dirs, globs and stdin, `sloperator.toml`, four output formats, `--fix`/`--diff` with safe rule fixes written atomically, `rules`/`config` commands, exit codes 0/1/2, and the NFR-3 rules-only budget.
**PRD Refs:** FR-2.1–FR-2.13, FR-2.15, FR-3.1–FR-3.6, FR-4.7, FR-5.2, FR-7.1–FR-7.6, FR-8.8, FR-8.9, FR-9.6, FR-9.10, NFR-3, NFR-4, NFR-5, NFR-6, NFR-8, NFR-9, D-3, D-8, D-9
**SPEC Refs:** §13 (config), §14 (input), §15 (registry and resolve), §16 (port and LazyRewriter only), §17 (pipeline, rules engine), §18 (report), §19 (write), §20 (cli), §23.9–§23.18
**Modules:** `sloperator`
**Status:** Delivered. All nine stories merged: `config` (`sloperator.toml` chain, `Resolver`, `Settings`), `input` (`resolve`/`read`, stdin, globs), `model` registry plus the `Rewriter` port and `LazyRewriter` in `engine`, `pipeline` (`analyse`/`lint`/`fix`, `FileResult`), `report` (text, concise, json, github), `write` (atomic `commit`, `Fs` trait), the `check`/`rules`/`config` CLI with exit codes 0/1/2, the NFR-3 perf budget and the registry metadata assert. Tests: `tests/config.rs`, `input.rs`, `model_cache.rs`, `pipeline.rs`, `report.rs`, `write.rs`, `e2e.rs`, `perf.rs`, `rules_registry.rs`. The model step of `pipeline::fix` and the `model` subcommand stay for epics 06 and 07.

## Waves

Copied from this epic's line in `index.md`. Stories in one wave run in parallel; each wave waits for the one before.

| Wave | Stories | Why this order |
|------|---------|----------------|
| 1 | [story-01-config](./epic-05-linter-surface/story-01-config.md) | Config is read by input, pipeline and cli, so it comes first. |
| 2 | [story-02-input](./epic-05-linter-surface/story-02-input.md), [story-03-model-registry-and-port](./epic-05-linter-surface/story-03-model-registry-and-port.md) | Parallel: input (src/input) and the model registry plus the Rewriter port (src/model, src/engine/mod.rs) touch disjoint files and need only config. |
| 3 | [story-04-pipeline](./epic-05-linter-surface/story-04-pipeline.md) | The pipeline needs Source from input and the LazyRewriter type from the port. |
| 4 | [story-05-report](./epic-05-linter-surface/story-05-report.md), [story-06-write](./epic-05-linter-surface/story-06-write.md) | Parallel: report and write each consume FileResult and touch disjoint files. |
| 5 | [story-07-cli-check](./epic-05-linter-surface/story-07-cli-check.md) | The CLI composes everything and replaces the skeleton check. |
| 6 | [story-08-perf-budget](./epic-05-linter-surface/story-08-perf-budget.md), [story-09-registry-metadata-assert](./epic-05-linter-surface/story-09-registry-metadata-assert.md) | The release-profile perf test runs against the finished CLI path. Story 09, added at re-plan (CH-9), is test-only, touches only `tests/rules_registry.rs`, and runs in parallel with it. |

## Acceptance Criteria

- [ ] e2e tests cover every FR-2, FR-3 and FR-7 behaviour (architecture Phase 5 gate).
- [ ] `rules_only_100kb_under_1s` passes in the release profile (NFR-3).
- [ ] JSON output is byte-identical across two runs and across a shuffled walk order (NFR-6).
- [ ] `sloperator check --fix` writes safe rule fixes in place, and `--diff` prints them and writes nothing (FR-2.5, FR-2.6).
- [ ] The tests named in every story's Test Cases pass together on the merged epic.

## Notes

Brad Q5: `sloperator.toml` with snake_case keys (D-9); report JSON stays camelCase (DQ-9). Brad Q2: no auto-download. A missing model makes `--fix`/`--diff` with the model engine exit 2 and name `model pull` and `--rules-only` (FR-9.6). In this epic the model step of `pipeline::fix` is not built yet: the model engine has no windows (epic 06), but `LazyRewriter::new` already runs in cli step 4. DQ-10: nothing is written when any file hit an exit-2 error.
