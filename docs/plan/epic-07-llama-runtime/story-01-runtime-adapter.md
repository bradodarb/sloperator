# Story 01: `engine::runtime` RuntimeFactory and real-model tests

**Epic:** [epic-07-llama-runtime](../epic-07-llama-runtime.md)
**SPEC Refs:** §21, §20 (main factory selection), §15 (DEFAULT_MODEL from the spike), §23.19
**PRD Refs:** FR-9.1, FR-9.3, FR-9.4, FR-9.5, FR-9.7, FR-9.8, D-6
**Component / ADR:** §4.10 engine::runtime; ADR-02
**Wave:** 1

## Scope

Implement `src/engine/runtime.rs` (compiled only with feature `model`): RuntimeFactory with `preflight` and `open`, sampler set from the PromptProfile and the seed, llama.cpp logs silenced unless `-v`, and a context-overflow error. Switch `src/main.rs` to RuntimeFactory under `cfg(feature = "model")`. Apply the spike's Decision: DEFAULT_MODEL, registry entries and masking mode. Add `tests/model_real.rs` (`#![cfg(feature = "model")]`, every test `#[ignore = "integration tier"]`).

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/engine/runtime.rs` | new | RuntimeFactory, llama-cpp-2 adapter |
| `src/main.rs` | from epic-05 story-07 | factory chosen by feature |
| `src/model/registry.rs` | from epic-05 story-03 | DEFAULT_MODEL/entries per the spike Decision |
| `Cargo.toml` | from epic-01 story-01 | only if the spike chose mistralrs |
| `tests/model_real.rs` | new | tests below |
| `tests/layering.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] Many windows load the model once (FR-9.4).
- [ ] The same seed gives the same output twice (FR-9.8).
- [ ] Every real rewrite is either verified and written or rejected as SLP901; no unverified byte is written (FR-9.5, D-4).
- [ ] With `HTTPS_PROXY` pointing at a closed port, `check --fix` succeeds: no network use (FR-9.1).
- [ ] When the model is absent, the integration tests fail with "run `sloperator model pull`" (architecture §11).
- [ ] `llama_cpp_2` appears only in `src/engine/runtime.rs` (layering).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::model_real` | `loads_once_for_many_windows` | single load | FR-9.4 |
| 2 | `sloperator::model_real` | `same_seed_same_output` | reproducible | FR-9.8 |
| 3 | `sloperator::model_real` | `real_rewrites_accepted_or_rejected_never_unverified` | verified-only writes | FR-9.5, D-4 |
| 4 | `sloperator::model_real` | `no_network_during_check` | no network | FR-9.1 |
| 5 | `sloperator::layering` | `runtime_crate_only_in_adapter` | runtime crate confined | ADR-02 |

## Notes

Accepted Gap 13: `Box::leak` of the backend, one model per process. Pull the default model before running the Gate (`sloperator model pull`). `FEATURES.sloperator=metal` selects Metal on macOS.

Gate: `make test TESTS='sloperator::layering#runtime_crate_only_in_adapter' && make test-integration TESTS='sloperator::model_real#loads_once_for_many_windows sloperator::model_real#same_seed_same_output sloperator::model_real#real_rewrites_accepted_or_rejected_never_unverified sloperator::model_real#no_network_during_check'`
