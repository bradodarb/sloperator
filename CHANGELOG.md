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
