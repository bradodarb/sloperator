# Epic 07: The llama.cpp runtime adapter and the integration tier

**Status:** delivered

**Goal:** A default-features `sloperator` binary rewrites with a real local model through `engine::runtime`, and `make test-integration` proves single load, seed reproducibility, verified-only writes and no network during `check`.
**PRD Refs:** FR-9.1, FR-9.3, FR-9.4, FR-9.5, FR-9.7, FR-9.8, FR-1.7, NFR-7, D-1, D-6
**SPEC Refs:** §21 (`engine::runtime`), §1 (Decision), §23.19
**Modules:** `sloperator`

## Waves

Copied from this epic's line in `index.md`. Stories in one wave run in parallel; each wave waits for the one before.

| Wave | Stories | Why this order |
|------|---------|----------------|
| 1 | [story-01-runtime-adapter](./epic-07-llama-runtime/story-01-runtime-adapter.md) | One adapter file plus its wiring and the integration tests. |
| 2 | [story-02-fmt-check-green](./epic-07-llama-runtime/story-02-fmt-check-green.md) | Acceptance-audit fix: rustfmt drift blocked `check`, so both clippy runs never ran (criterion 2). |

## Acceptance Criteria

- [ ] The integration tier is green on the default model: single load, seed reproducible, 0 unverified writes (architecture Phase 6 gate).
- [ ] `make build` builds the release binary with feature `model`, and `check` passes both clippy runs (ADR-02).
- [ ] The tests named in every story's Test Cases pass together on the merged epic.

## Notes

It needs `docs/research/spike-results.md` with a PASS verdict (index needs block). If the spike picked `mistralrs`, only §21 and the `model` feature line in Cargo.toml change (SPEC §2 note). Brad's OD-1 answer: distribution builds are out of scope; `make build` produces the host binary only.
