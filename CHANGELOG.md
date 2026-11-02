# Changelog

## 0.1.0 (2026-11-01)

First release, delivering epics 01-09. The plan and acceptance audits are in `docs/plan/`.

- **01 Walking skeleton:** one Rust crate (library and binary), layering and standalone guards, nextest JUnit reports.
- **02 Inference spike:** `llama-cpp-2` with Qwen3 GGUF models chosen; results in `docs/research/spike-results.md`.
- **03 Protection core:** document span map, the verifier that gates every edit, source-range edits.
- **04 Rules, score and directives:** the SLP001-SLP016 rule catalogue, the style score and inline directives.
- **05 Linter surface:** `sloperator.toml` config, input discovery, pipeline, text/JSON/GitHub reports, atomic writes,
  the `check` command and its exit codes.
- **06 Model fix path:** windowing, placeholder masking and prompts, `model pull|list|path`.
- **07 llama.cpp runtime:** the runtime adapter behind the default `model` feature and the integration tier.
- **08 Eval:** the 20-paragraph eval corpus and per-model report, network-free tests.
- **09 Backlog:** delivery follow-ups from the acceptance audits.
- **SLP017 `decorative-emoji`** (warning, safe fix, on by default): flags pictographic emoji in prose, one finding
  per visible emoji (ZWJ sequences, skin tones, flags and keycaps count once), and deletes each with one adjacent
  space. Plain ©, ® and ™ are not flagged; an emoji that is a bare list item or heading is flagged with no fix. With
  it the catalogue is 17 rules (SLP001-SLP017).
- **MCP server:** `sloperator mcp` serves `lint`, `fix`, `rules` and `explain_rule` over stdio, behind the default
  `mcp` feature (ADR-10, docs/MCP.md). Text in, text out: it never writes files. The model loads lazily, is used
  serially and unloads after `--idle-unload` seconds (default 600; 0 = never). `--rules-only` turns it off.
- `main` passes unlocked stdio handles to `cli::run`.
- `make test` builds with `--features mcp`, so the unit tier covers the server.
- **Releases:** prebuilt binaries for macOS (Apple silicon) and Linux (x86_64, aarch64), `.deb` packages for
  Linux, and a Homebrew formula in this repository (`docs/RELEASING.md`).
