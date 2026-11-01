# ADR-10: The MCP server is a subcommand of the one binary

**Status:** Accepted
**Date:** 2026-11-01
**Initiative:** [sloperator](../ARCHITECTURE.md) · **Serves:** FR-2.1, FR-9.4, FR-9.6, NFR-5 · **See also:** ADR-02, ADR-09

## Context

Agents and editors want to lint and fix prose through MCP rather than by running `check` on files. They pass text,
not paths, and they decide what gets written. A loaded model is one heavy object whose llama.cpp types are not
`Send` (ADR-02), and a long-lived server should not hold its memory forever.

## Options

### Option 1: `sloperator mcp` in the same binary, behind a default-on cargo feature `mcp`

**For:** one install, one pipeline, one rules registry; the server is a thin adapter like `cli`.
`--no-default-features` still builds the bare linter. **Against:** the default binary gains `rmcp` and a tokio
runtime.

### Option 2: A separate crate or binary

**For:** the linter binary stays free of async code. **Against:** two artefacts to ship and version, and the
adapter would need the library's internals exported anyway.

## Decision

Option 1, with these rules:

- Tools `lint`, `fix`, `rules`, `explain_rule`; stdio transport only; stdout carries only the protocol.
- Text in, text out: `src/mcp/` may use `pipeline`, `report`, `rules` and the `engine` port, never `write`
  (`tests/layering.rs`). `path` only selects the format and the `sloperator.toml`. Only `src/mcp/mod.rs` names `rmcp`.
- One worker thread owns the rewriter factory and the model. It opens the model on the first model-backed fix,
  serves fixes in order over a channel (so model use is serialised) and drops the model after `--idle-unload`
  seconds (default 600; 0 = never). Lint and rules-only fixes run on tokio's blocking pool.
- `--rules-only` starts no worker. A missing model is a tool error naming `sloperator model pull`; nothing is
  downloaded.

## Consequences

- **Easier:** the non-`Send` model never crosses threads, so no unsafe `Send` wrappers are needed; tests drive the
  handler directly with a fake factory built on the worker thread.
- **Harder:** `main` must pass unlocked stdio handles, because a lock held for the whole run would block tokio's
  stdin reader.
- **Follow-ups:** a second transport (HTTP) only if a client needs it.
