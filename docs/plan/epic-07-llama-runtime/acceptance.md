# Acceptance audit — epic-07-llama-runtime (sloperator)

**Auditor:** reviewer · **Base:** 00838070b00d21ec02f6c03a78f765dc51d52e43 · **Verdict (round 2): met**

## Round 2 (after story-02-fmt-check-green, HEAD 1009d99)

| Command | Result |
|---------|--------|
| `make check` | **PASS**, exit 0. `cargo fmt --check` passes and both clippy runs finish with `-D warnings`: the two `Finished dev` lines for `--no-default-features` and default features (Makefile `check` target). The Makefile has no change since base. |
| `make build` | PASS, exit 0 (release, default feature `model`) |
| `make test TESTS='sloperator::engine sloperator::cli_exit sloperator::layering#runtime_crate_only_in_adapter'` | PASS 38/38 |
| `make test-integration TESTS='…model_real#… (4 tests)'` | PASS 4/4 |

- Gap 1 (`check` failed at `cargo fmt --check`) is **closed**. All three epic criteria are now met.
- Story-02 criteria are met: fmt clean, `check` exits 0 with both clippy runs, the Makefile unchanged, reformatted binaries pass.
- Reachable as shipped: still holds. `src/main.rs:20` constructs `RuntimeFactory::new(cli.verbose > 0)`.
- Gaps 2 and 3 stay DEFERred. Their items exist in `docs/backlog/` (`epic-07-no-network-test-weak.md`, `epic-07-spec21-leak-stale.md`).

No blocking gap remains.

## Round 1 (original audit)

Diff since base: `src/engine/runtime.rs` (new, 156 lines), `src/engine/mod.rs` (`#[cfg(feature = "model")] pub mod runtime;`),
`src/main.rs` (factory chosen by feature), `tests/model_real.rs` (new), README and directive/context docs.

## Evidence run this session

| Command | Result |
|---------|--------|
| `make test TESTS='sloperator::layering#runtime_crate_only_in_adapter'` | PASS (1/1) |
| `make test-integration TESTS='…model_real#loads_once_for_many_windows …#same_seed_same_output …#real_rewrites_accepted_or_rejected_never_unverified …#no_network_during_check'` | PASS 4/4 against the cached Qwen3-4B model |
| `make build` | PASS (release build, default features = `model`, llama-cpp-2 0.1.160 compiled) |
| `make check` | **FAIL**: exit 2 at `cargo fmt --check`; the two clippy runs after it never ran |

## Epic acceptance criteria

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Integration tier green on the default model: single load, seed reproducible, 0 unverified writes | MET | the 4 `model_real` tests pass; `DEFAULT_MODEL = "qwen3-4b"` (`src/model/registry.rs:69`) matches the spike Decision (`docs/research/spike-results.md:134`) |
| `make build` builds the release binary with feature `model`, and `check` passes both clippy runs (ADR-02) | **NOT MET** | build passes; `check` fails at `cargo fmt --check` before clippy, with drift in `src/pipeline/mod.rs:9,15,147`, `tests/cli_exit.rs:268,282`, `tests/engine.rs:488,495,502,536,561,588,615` |
| Every story's Test Cases pass together on the merged epic | MET | layering + 4 integration tests above |

## Story-01 acceptance criteria

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Many windows load the model once (FR-9.4) | MET | `tests/model_real.rs:142-164` counts `open` calls through a real `RuntimeFactory` via the CLI over 8 one-window files; asserts `opens == 1` and `windows >= 2`. Fails if loading repeats. |
| Same seed → same output (FR-9.8) | MET | `runtime.rs:126-136` sampler chain ends in `LlamaSampler::dist(req.seed)`; `model_real.rs:166-187` rewrites twice with seed 1234 and asserts equal, non-empty output. Fresh context per window (`runtime.rs:110`) makes the two calls independent. |
| Every real rewrite verified-and-written or SLP901, never unverified (FR-9.5, D-4) | MET | `model_real.rs:189-249`: written files keep protected bytes exactly; `rejected == SLP901 count`; `windows == written + rejected`. |
| `HTTPS_PROXY` at a closed port, `check --fix` succeeds (FR-9.1) | MET (weak test, deferred) | `model_real.rs:251-279` runs the shipped binary with HTTP(S)/ALL proxies on a closed port, exit 0/1, no `sloperator:` error. |
| Model absent → integration tests fail with "run `sloperator model pull`" | MET | `model_real.rs:35-52` `require_model` panics naming the command; every test calls it first. |
| `llama_cpp_2` only in `src/engine/runtime.rs` | MET | grep: imports only at `runtime.rs:11-17`; `tests/layering.rs:199-200` enforces. |
| Every Test Case passes with the Gate | MET | see evidence table. |

## Reachable as shipped

`src/main.rs` constructs `RuntimeFactory::new(cli.verbose > 0)` under `#[cfg(feature = "model")]`; `model` is a default feature (`Cargo.toml:12`). `no_network_during_check` drives `CARGO_BIN_EXE_sloperator`, so the shipped binary rewrites with the real model. Logs are silenced unless `-v` (`runtime.rs:50-53`); the context-overflow error is at `runtime.rs:98-105`; GPU offload is chosen by `supports_gpu_offload` (`runtime.rs:55-59`), as the spike follow-up asked.

## Gaps

1. **`make check` fails — epic criterion 2 unmet.** Where: `cargo fmt --check` drift in `src/pipeline/mod.rs:9,15,147`, `tests/cli_exit.rs:268,282`, `tests/engine.rs:488-615`. The drift came from earlier epics (these files have not changed since base), but this epic's criterion says `check` passes both clippy runs, and on the merged tree neither clippy run is reached. **Severity: blocking. Disposition: FIX**: run `cargo fmt` in the crate, then confirm `make check` passes, which includes both clippy runs.
2. **`no_network_during_check` cannot catch a regression that skips proxies.** llama.cpp never reads proxy variables, so a non-proxied socket would still pass the test. The test meets the SPEC §23.19 wording, so this does not block. **Severity: minor. Disposition: DEFER** → `docs/backlog/epic-07-no-network-test-weak.md`.
3. **SPEC §21 still says the model is `Box::leak`ed (Accepted Gap 13); the code owns it instead.** The reason is the Metal teardown assert, which `runtime.rs:5-7` documents and the README records. Behaviour is correct, but the SPEC is now stale. **Severity: minor (docs). Disposition: DEFER** → `docs/backlog/epic-07-spec21-leak-stale.md`.
