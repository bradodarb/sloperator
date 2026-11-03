# Changelog

## 0.2.0 (unreleased)

- SLP001 `ai-vocabulary-replaceable` now also covers inflected nominalisations and padded phrases, including
  “make a decision” → “decide”, “conduct an analysis of” → “analyze”, and “at this point in time” → “now”.
  These rows remain default-on warnings with safe fixes, so default output gains findings, scores and fixes
  on padded phrasing. Noun-final and “the fact that” guards keep unproven grammatical contexts unchanged.
- Added SLP022 `weak-intensifier`: Wordiness, suggestion severity, fix class `None`, default-off, since 0.2.0,
  no params. It flags twelve intensifiers without deleting them because verification does not check strength.
- Phase-2 model reliance: 19.56 residual model-eligible findings per 1,000 words (baseline 19.56), measured
  2026-11-02 without a model or network; see `eval/model-reliance.md`.
- Added four opt-in Readability rules, all suggestion severity, fix class `None`, default-off, since 0.2.0:
  SLP018 `grade-level` (`max_grade = 12`, bounds 0–30);
  SLP019 `sentence-length-variation` (`min_cv = 0.35`, bounds 0–5);
  SLP020 `long-sentences` (`max_avg_words = 25`, bounds 1–1000);
  SLP021 `lexical-diversity` (`min_ttr = 0.4`, bounds 0–1). Thresholds are provisional.
- Every numeric `ParamSpec` now declares inclusive minimum/maximum bounds, validated by config and shown by
  CLI/MCP explanations. Existing parameters retain their nonnegative bounds.
- Metric rules share `min_sentences` / `min_words` (defaults 5/100, static minimum 1). Unset values inherit
  `[metrics]` report floors after resolution; explicit values below those floors are rejected. These rules
  add suggestion weight 1 when reported and never select model windows. SLP001–SLP017 defaults are unchanged.
- Text reports show paragraph reading metrics, original metrics for Fix/Diff, and overall metrics from merged counts.
- JSON adds `files[].metrics`, `files[].metricsBefore` and `summary.metrics`; `schemaVersion` remains 1 and score keys are unchanged.
- `[metrics] min_sentences` and `min_words` configure positive sample floors (defaults 5/100). Unavailable values are JSON `null`; text explains the configured floors.
- Release binary on macOS Apple silicon, same default-feature contract build: 20,160,544 bytes before report integration; 20,156,512 bytes after (4,032 bytes smaller). Measured with `make -f "$AEGIS_DIR/Makefile" build UNIT=sloperator` and `wc -c target/release/sloperator`.

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
