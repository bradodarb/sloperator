# Story 03: SPEC §21 describes the shipped runtime adapter

**Epic:** [epic-08-eval](../epic-08-eval.md)
**SPEC Refs:** §21, §24 (open question 4)
**PRD Refs:** FR-9.4, FR-9.8
**Component / ADR:** §4.10 engine::runtime (ADR-02)
**Wave:** 1

## Scope

Docs only. SPEC §21 (`docs/SPEC.md:2031-2110`) still shows `backend.void_logs()`, `Box::leak(Box::new(model))` and one `LlamaContext<'static>` reused with `clear_kv_cache`. The shipped `src/engine/runtime.rs` owns the model in `LlamaRewriter`, drops it before the backend (a live model at `process::exit` trips ggml's Metal teardown assert, exit 134), silences logs with `send_logs_to_tracing(LogOptions::default().with_logs_enabled(false))` before `LlamaBackend::init`, and builds a fresh context per window. Rewrite §21's API list, code block and prose to match `runtime.rs`, and answer open question 4 (SPEC line ~2549) with the reason the model is owned, not leaked.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `docs/SPEC.md` | §21 (~2031-2110), ~2549 | match the shipped adapter |
| `tests/layering.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] §21 no longer contains `Box::leak` or `void_logs`; it names `send_logs_to_tracing` and the owned model with a fresh context per window.
- [ ] §21's code block matches the signatures in `src/engine/runtime.rs` (`RuntimeFactory::new(verbose)`, `open`, `LlamaRewriter`, `N_CTX`).
- [ ] Open question 4 records the Metal teardown reason for owning the model.
- [ ] `runtime_crate_only_in_adapter` still passes (no code moved).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::layering` | `runtime_crate_only_in_adapter` | no code moved while the SPEC text changes | ADR-02 |

## Notes

Closes `docs/backlog/epic-07-spec21-leak-stale.md`. The index's Accepted Gap 13 row already states the owned-model behaviour.

Gate: `make test TESTS='sloperator::layering#runtime_crate_only_in_adapter' && ! grep -nE 'Box::leak|void_logs' docs/SPEC.md && grep -q 'send_logs_to_tracing' docs/SPEC.md`
