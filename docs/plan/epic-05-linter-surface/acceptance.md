# Acceptance: epic-05-linter-surface (sloperator)

Base: `b75e08337bed39f8676693c4fd249739efa9cceb`. Reviewer: reviewer.
Workflow inputs: tests PASS, discriminate PASS, gates (conflict markers, gate-diff) PASS.

## Verdict: met (no blocking gap)

## Acceptance criteria

| Criterion | Evidence | Result |
|---|---|---|
| e2e tests cover FR-2, FR-3 and FR-7 | FR-2: `tests/e2e.rs:79,110,145,216`, `tests/cli_exit.rs:89,108,119,140,182,203,242`, `tests/input.rs:71-337`. FR-3: `tests/report.rs:85,110,123,179,221,240,278`. FR-7: `tests/config.rs:40,81,104,132,189,251,306,371` | met |
| `rules_only_100kb_under_1s` passes in the release profile (NFR-3) | `tests/perf.rs:87`. It runs in the release profile through the Makefile's perf pass (`--cargo-profile release --profile ci-perf`). The cache directory has mode 000, so the test also shows the model cache is never read | met |
| JSON is byte-identical across two runs and across a shuffled walk order (NFR-6) | `tests/e2e.rs:164`: the test compares two runs, then recreates the tree in reverse order and compares again | met |
| `--fix` writes safe fixes in place; `--diff` prints them and writes nothing | `tests/e2e.rs:79`: the file changes and protected bytes do not. `tests/e2e.rs:110`: a unified diff goes to stdout, the report goes to stderr, and a directory snapshot is unchanged | met |
| Every story's Test Cases pass together | 53 of the 54 named tests exist (grep `fn <name>` over `tests/`) and the targeted run is PASS. The 54th, `rejection_is_slp901_warning_counts_for_exit`, is assigned to epic 06 by the story itself (`story-04-pipeline.md:41`). That story exists: `docs/plan/epic-06-model-fix-path/story-03-model-window-pipeline.md` | met |

## Reachable as shipped

`src/cli/mod.rs` (the composition root, called from `main.rs:30`, which also injects `write::RealFs`) wires each part this epic built:
- `Resolver::new`: config (line ~119)
- `input::resolve` / `input::read`: input (124, 244)
- `LazyRewriter::new`: Fix/Diff only, before any write (155)
- `pipeline::lint` / `pipeline::fix`: pipeline (176, 179)
- `write::commit`: write (213)
- `report::render`: report (225, 227)
- `show_rules` / `show_config`: the `rules` and `config` commands (276, 330)

The model registry is reached through `model::resolve` from `LazyRewriter::new`. Step 2 of `pipeline::fix` (model windows) is a deliberate no-op. Epic 06 story-03 wires it, and that epic exists.

## Do the tests discriminate?

- `check_modifies_nothing`, `diff_writes_nothing` and `fix_without_model_exits_two` compare full directory snapshots, so any stray write fails them.
- `mixed_fixture_protected_bytes_unchanged_after_rules_fix` asserts that the file did change and that the protected bytes did not. A fix that writes nothing fails it, and so does a fix that touches protected bytes.
- The workflow's discriminate pass reported PASS.

## Gaps

None blocking. Two notes, neither a defect, so nothing is deferred:
- `perf.rs` skips with a loud stderr message when run as root, because mode 000 is not enforced for root. This is a documented limit, not a silent pass.
- FR-7 is mostly tested through the library (`Resolver`) in `tests/config.rs`, not through the binary. `config_command_prints_resolved_json` and `exit_two_on_bad_config` cover the CLI path. This is enough for the criterion.
