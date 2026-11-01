# ADR-02: A synchronous `Rewriter` port, with the runtime behind cargo feature `model`

**Status:** Accepted
**Date:** 2026-10-29
**Initiative:** [sloperator](../ARCHITECTURE.md) · **Serves:** D-3, D-6, FR-1.7, FR-9.1, FR-9.4, FR-9.7, NFR-3

## Context

D-6 puts the model behind one engine boundary and leaves the runtime crate (`llama-cpp-2` or `mistralrs`) to a
spike. The two differ in shape. `llama-cpp-2` is synchronous and needs `clang` plus a C++ build. `mistralrs` is
async (`tokio`, `build().await`) (research `inference-and-models.md` §1). The unit tier must not download or load a
model (FR-1.7). No Rust toolchain exists on the dev machine today (checked: `command -v cargo` empty), so a heavy C++
build in every test run is a real cost.

## Options

### Option 1: Synchronous trait; runtime is an optional dependency behind feature `model`, used only in `engine::runtime`

**For:** the pipeline stays sync, so there is no async colouring. Unit tests build with `--no-default-features`, so
they skip the C++ build and inject `FakeRewriter`. Replacing the runtime touches one file. A build without the
feature returns `Error::ModelUnsupported` (exit 2), never a silent skip. **Against:** the release binary's real model
path is exercised only in the integration tier. An async runtime needs a private `block_on` inside the adapter.

### Option 2: Runtime always compiled; an async engine trait across the pipeline

**For:** one build configuration. Tests cover the code exactly as shipped. **Against:** every unit-test run pays the
runtime's compile and C++ toolchain requirement. Async spreads into `pipeline`, `cli` and tests for no benefit,
because windows are rewritten one by one anyway (ADR-09). Swapping runtimes would touch more code.

### Option 3: Out-of-process server client (`mistralrs serve`, `ollama`)

**For:** no native build at all. **Against:** contradicts FR-9.1 (in-process) and PRD Question 11's assumption.
"Both engines on by default" would need a running server.

## Decision

Option 1. It keeps the unit tier fast and toolchain-light. It meets D-6's "replaceable" literally, and keeps lint
mode physically unable to touch the model. Default features include `model`, so `cargo install` and
`make build` ship the full binary.

## Consequences

- **Easier:** spike result plugs into one adapter; unit tests drive every model branch with fakes.
- **Harder:** two build configurations to keep green (`check` runs clippy on both). Real-model behaviour is caught
  only by `TIER=integration`.
- **Follow-ups:** the SPEC names the adapter for the chosen crate, its tokio handling (if `mistralrs`) and the
  feature names (`metal`/`cuda` passthrough).
