# Story 03: Model registry, cache resolve, the Rewriter port and FakeFactory

**Epic:** [epic-05-linter-surface](../epic-05-linter-surface.md)
**SPEC Refs:** §15 (registry.rs, mod.rs; not pull), §16 (engine/mod.rs only), §23.11 (part)
**PRD Refs:** FR-9.6, FR-9.10, FR-9.4 (lazy load type), D-5, Brad Q2 (a), Brad Q4 (a)
**Component / ADR:** §4.14 model, §4.10 engine port; ADR-02, ADR-08
**Wave:** 2

## Scope

Implement `src/model/registry.rs` (PromptProfile, QWEN3, ModelSpec, REGISTRY, DEFAULT_MODEL) and `src/model/mod.rs` (ResolvedModel, cache_dir, cached_path, resolve, spec). Extend `src/engine/mod.rs` from the epic-01 stub to the full port: DEFAULT_SEED, RewriteRequest, Rewriter, RewriterFactory with preflight, DisabledFactory and LazyRewriter. Add `FakeFactory`/`FakeRewriter` and `TempModelCache` to `tests/common/mod.rs`. `pull` is epic 06.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/model/registry.rs` | new | registry with sha256 and sizes |
| `src/model/mod.rs` | new | resolve, cache_dir, cached_path, spec |
| `src/engine/mod.rs` | from epic-01 story-01 | full port, LazyRewriter |
| `tests/common/mod.rs` | from epic-01 story-01 | FakeFactory (counts open/preflight), TempModelCache |
| `tests/model_cache.rs` | new | tests below |

## Acceptance Criteria

- [ ] Every registry entry has a 64-hex sha256, a size, a licence and a profile (ADR-08).
- [ ] `resolve` with no cached file is `Error::ModelMissing`, whose message names `sloperator model pull <name>` and `--rules-only` (FR-9.6).
- [ ] `LazyRewriter::new` runs preflight and opens nothing; `get` opens once (FR-9.4).
- [ ] No `*.gguf` file and no file over 10 MB exists in the crate (FR-9.10).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::model_cache` | `registry_entries_have_sha256_and_profile` | registry shape | ADR-08 |
| 2 | `sloperator::model_cache` | `missing_model_error_names_pull_and_rules_only` | ModelMissing message | FR-9.6 |
| 3 | `sloperator::model_cache` | `no_weights_in_crate` | no weights committed | FR-9.10 |
| 4 | `sloperator::model_cache` | `lazy_rewriter_preflights_without_opening` | preflight once, open zero times until get, then once | FR-9.4, FR-9.6 |

## Notes

`lazy_rewriter_preflights_without_opening` is added to give the port its own gate; SPEC §23.12 checks the same behaviour end to end in epic 06. `model` must not use pipeline, rules or engine (layering). The spike (epic 02) may change DEFAULT_MODEL; that edit happens in epic 07.

Gate: `make test TESTS='sloperator::model_cache#registry_entries_have_sha256_and_profile sloperator::model_cache#missing_model_error_names_pull_and_rules_only sloperator::model_cache#no_weights_in_crate sloperator::model_cache#lazy_rewriter_preflights_without_opening'`
