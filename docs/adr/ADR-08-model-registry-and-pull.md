# ADR-08: A built-in model registry with sha256 pins; blocking HTTPS download only on `model pull`

**Status:** Accepted
**Date:** 2026-10-29
**Initiative:** [sloperator](../ARCHITECTURE.md) · **Serves:** D-5, D-6, FR-2.14, FR-9.1, FR-9.6, FR-9.10

## Context

D-5 keeps weights external, fetched only by `sloperator model pull`, and fails with exit 2 when a needed model is
missing. The default model is picked after the spike (D-6, Question 4). Prompts differ by model family: Qwen3 needs
thinking disabled and has recommended sampling settings (research §2). `mistralrs` and the `hf-hub` crate can
download on first use. That conflicts with "never silently" and FR-9.1.

## Options

### Option 1: Static registry (`name, repo, file, sha256, bytes, licence, prompt_profile`) plus our own `ureq` download

**For:** download happens in exactly one code path and is integrity-checked. The runtime is only ever given a local
file path, so it cannot fetch. Prompt and sampling settings live next to the model they suit. `ureq` is blocking,
so no tokio in the lint build. **Against:** adding a model needs a release. A user's own GGUF needs a config escape
(`model.path`, a SPEC detail that still requires a registry `prompt_profile`).

### Option 2: Let the runtime/`hf-hub` resolve `repo:file` on first use, cached by the HF cache

**For:** less code; any HF model works. **Against:** network on `check --fix` breaks FR-9.1 and D-5 unless every
call sets offline mode. No pinned integrity. The async dependency chain lands in the base binary.

## Decision

Option 1. It is the only option where "no network during check" holds by construction and a swapped file is
detected.

## Consequences

- **Easier:** `model list/path`, missing-model error text, reproducibility (exact file known).
- **Harder:** registry upkeep; arbitrary models need a profile.
- **Follow-ups:** Phase 1 fills the registry entries and records licence text (Question 4). The SPEC defines
  `model.path` for local files.
