# Story 02: `model pull`, `model list`, `model path`

**Epic:** [epic-06-model-fix-path](../epic-06-model-fix-path.md)
**SPEC Refs:** §15 (pull.rs, CLI), §23.11 (pull, list, path)
**PRD Refs:** FR-2.14, FR-9.1, D-5, Brad Q2 (a)
**Component / ADR:** §4.14 model; ADR-08
**Wave:** 1

## Scope

Implement `src/model/pull.rs`, the only network code: streamed download to `.part`, sha256, rename, and a `.sha256` sidecar. Add `Command::Model(ModelCmd)` to `src/cli/{args,mod}.rs`: `pull` ("up to date" when already present), `list` and `path`.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/model/pull.rs` | new | pull(spec, dir, base, progress) |
| `src/model/mod.rs` | from epic-05 story-03 | `pub mod pull` |
| `src/cli/args.rs` | from epic-05 story-07 | ModelCmd |
| `src/cli/mod.rs` | from epic-05 story-07 | model subcommands |
| `tests/model_cache.rs` | existing | tests below (methods added) |
| `tests/layering.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] A pull from a local `TcpListener` server writes the file and its `.sha256` (FR-2.14).
- [ ] A corrupt download is `Error::Download` naming the URL, and no `.part` remains (ADR-08).
- [ ] `model list` shows name, file, size, licence and present/absent; `model path` exits 0 when present and 2 with `ModelMissing` when absent (FR-2.14).
- [ ] `ureq` is still used only in `src/model/pull.rs` (layering `network_only_in_pull`, FR-9.1).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::model_cache` | `pull_from_local_server_verifies_sha256` | pull happy path | FR-2.14, ADR-08 |
| 2 | `sloperator::model_cache` | `pull_rejects_corrupt_file_and_removes_part` | corrupt download | FR-2.14, ADR-08 |
| 3 | `sloperator::model_cache` | `list_and_path_report_presence` | list/path | FR-2.14 |
| 4 | `sloperator::layering` | `network_only_in_pull` | network confined | FR-9.1 |

## Notes

Tests pass a local `base`; the CLI always passes `HF_BASE`. No test reaches the network.

Gate: `make test TESTS='sloperator::model_cache#pull_from_local_server_verifies_sha256 sloperator::model_cache#pull_rejects_corrupt_file_and_removes_part sloperator::model_cache#list_and_path_report_presence sloperator::layering#network_only_in_pull'`
