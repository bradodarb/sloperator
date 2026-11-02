# PRD: sloperator — a standalone CLI that makes LLM-generated text read as human

**Version:** 0.1
**Date:** 2026-10-29
**Status:** Draft
**Crate:** `sloperator` (repository root, see D-2)
**Request:** Build sloperator, a standalone linting CLI that takes LLM-generated text and makes it sound and feel more human. It combines a deterministic rule engine with a local small language model (SLM) rewriter, preserves code, links, numbers and Markdown structure, and is a self-contained repository that builds and tests with nothing outside it.
**Companion Research:**
- [Competitive Analysis](research/competitive-analysis.md) — Ruff and Vale conventions; the AI-writing signal catalogue
- [Inference and Models](research/inference-and-models.md) — Rust inference routes, candidate models, fine-tuning, language comparison
- [Feasibility Summary](research/feasibility-summary.md) — pillars, risks, effort, open questions OQ-1 to OQ-13

**Parent / Related PRDs:** none

---

## 1. Problem Statement

LLM-written prose carries recognisable tells: em-dash overuse, "delve", "it's not X, it's Y", tricolons, hedging, bullet spam. Writers and CI jobs have no tool that finds these, scores them, and fixes them without damaging code blocks, links, numbers or Markdown structure. Rule-only tools can detect the tells but cannot rewrite most of them safely. A local model can rewrite them, but a small model may alter or drop facts. No existing tool combines rule-based detection with safe, verified rewriting.

### Root Causes

1. **No prose-lint capability exists.** There was no prior code, no prose rules and no AI-writing detection to build on. Everything is new.
2. **Existing linters do not rewrite with a model.** Ruff and Vale are the closest peers; Ruff fixes deterministically with a safe/unsafe split and Vale is a rule framework that "doesn't offer any of its own advice" (`research/competitive-analysis.md` §1.1, §3). Neither combines rules with a local SLM.
3. **Rule-based fixes cannot cover the hard patterns.** "It's not X, it's Y", tricolons and bullet spam can be detected deterministically, but a safe fix is a rewrite (`research/competitive-analysis.md` §4; `research/inference-and-models.md` §4).
4. **Small models are unreliable with facts.** The Phi-4-mini and Gemma 3 cards both warn of factual errors or limited factual capacity (`research/inference-and-models.md` §2). Brad requires facts and numbers preserved, so the product needs verification, not trust.
5. **CI needs per-test results, and cargo emits none by default.** `cargo test` prints a human summary only; a CI job that reads JUnit XML would see zero tests. A test runner that writes JUnit (cargo-nextest) and make targets that wrap it are needed.
6. **Model choice, runtime and speed are unmeasured.** The research is documentation-level; nothing was run and Metal support in `llama-cpp-2` is unconfirmed (`research/inference-and-models.md` header, §1, §7).

### Impact

- **Manual de-slopping.** Writers edit LLM text by hand; there is no CI check that flags it or reports a score.
- **Unsafe automation if built naively.** A model-only rewriter would silently change numbers, URLs and code; the tool would be untrustworthy in `--fix` and pre-commit use.
- **No build or test entry point exists.** There is no crate and no `Makefile`, so `make test` has nothing to run.

---

## 2. Scope

### In Scope

- **Standalone Rust CLI `sloperator`** — `check` (lint, scores, never writes), `check --fix` (writes), file, directory, glob and stdin input, exit codes 0/1/2.
- **Machine-readable output** — JSON and GitHub annotation formats alongside human formats.
- **Inline and file-level ignore directives** — Markdown-appropriate comment directives.
- **Rule engine** — a built-in, versioned rule set (codes `SLP###`) covering the nine signal families in `research/competitive-analysis.md` §4, with severities, per-rule thresholds and a transparent score.
- **SLM engine** — local, in-process rewriting of flagged prose windows, behind a runtime-neutral engine boundary; runtime crate and default model chosen by a spike.
- **Engine selection** — both engines by default; `--rules-only` and `--model-only`.
- **Protection layer** — a Markdown parser marks protected spans; engines see prose only; output is verified against the input.
- **Config file** — `sloperator.toml`, discovered upward from the file, tunes rules, severities, thresholds, protected spans, includes/excludes and model settings.
- **Introspection commands** — `rules`, `config`, `model`.
- **Eval harness** — a repo-local fixture set measuring slop score before and after and protected-span integrity; the input to any later fine-tuning decision.
- **Crate shell** — a `Makefile` with build, test, test-integration, check, clean and help targets over cargo and cargo-nextest, JUnit XML from nextest profiles, and a test that keeps the repository self-contained.
- **Language comparison record** — Rust vs Java+GraalVM vs Python, in D-1.

### Out of Scope

- **Fine-tuning or training data in v1** — no source shows it helps and no paired data exists; the eval harness gates the decision (`research/inference-and-models.md` §5).
- **AI-text detection or "undetectable" claims** — humans and detectors are unreliable, and stripping signs "could just make detection harder" (`research/competitive-analysis.md` §4). The tool reports style tells only.
- **Bundled model weights** — files are 1.8-4.3 GB (`research/inference-and-models.md` §2); they stay out of git and the binary.
- **User-authored rule languages or external style packs** — Vale-style packs are a later question; v1 ships a built-in rule set.
- **Non-English prose** — the signal catalogue and candidate models are English-centred; needs Brad's say-so.
- **Formats other than Markdown and plain text** — AsciiDoc, reStructuredText, HTML and source-code comments are not requested.
- **Editor or LSP integration** — not requested.
- **SARIF, JUnit and GitLab output formats** — not requested by name; JSON and GitHub cover tooling and CI; add on demand.
- **Score-threshold failure (`--max-score`)** — Brad asked that findings be scored, not that a score fail the run; exit code follows findings.
- **Verifying facts beyond numbers, URLs and protected terms** — a deterministic fact checker does not exist; the PRD does not claim one (see Risk 1).
- **Dependencies on code outside the repository** — no path dependencies or external build plumbing; the product must stay standalone (D-2).

---

## 3. Resolved Decisions

Non-negotiable for the SPEC. Each has a rationale.

### D-1: Rust, with the language comparison recorded

sloperator is implemented in Rust. Java + GraalVM native and Python are not used because inference does not make Rust hard.

| Axis | Rust (chosen) | Java + GraalVM native | Python |
|------|---------------|-----------------------|--------|
| Inference binding | `llama-cpp-2`, `mistralrs`, `candle` (research §1) | `java-llama.cpp` 4.1.0: prebuilt CPU libraries for Linux, macOS, Windows; GPU needs a self-build; `LlamaModel` is `AutoCloseable` because native memory is not GC'd | `llama-cpp-python`: source build needing a C compiler, or prebuilt CPU/CUDA/Metal wheels for Python 3.10-3.12 |
| Self-contained binary | Yes; model file stays external | JNI under native-image needs extra configuration (not verified); native-image builds are heavy | No: interpreter, or Nuitka bundling that takes "several minutes" |
| Markdown with source ranges | `pulldown-cmark` `OffsetIter` | A JVM library (not researched) | Many libraries (not researched) |
| Build tooling | cargo, cargo-nextest, cargo-deny behind a `Makefile` | Maven plus GraalVM | pip or uv plus a bundler |
| Standalone repository | `Cargo.toml` is already repo-ready | Needs its own JVM build | Needs its own `pyproject` |

**Rationale:** Brad prefers Rust "for speed" and allows a fallback only "if inference makes Rust hard." The research confirms two practical routes for GGUF-quantized 1-4B models: `llama-cpp-2` (MIT/Apache-2.0; needs `clang` for bindgen; "does not follow semver meaningfully") and `mistralrs` (CPU by default, `metal`/`cuda` features, GGUF loading with the tokenizer and chat template read from the file, async API). `candle` is the weakest fit because Phi-4-mini, Gemma 3 and dense Qwen3 are absent from its documented model lists, so the app would own tokenization, sampling and templates (`research/inference-and-models.md` §1, §6). The costs of Rust are toolchain costs (a C++ build, long first compiles), not inference blockers. Neither fallback removes the external model file. This comparison is documentation-level; the spike (D-6) confirms build, speed and quality on real hardware before the runtime is frozen.

### D-2: The crate is a standalone repository

The crate sits at the repository root: `Cargo.toml`, `src/`, `tests/`, `fixtures/`, `eval/`, README, `LICENSE`, `.gitignore` and its own `Makefile`, with its own CI workflow. Nothing outside the repository is needed to build or test it, and `Cargo.toml` has no path dependencies.

**Rationale:** a crate at the root is the Rust-ecosystem default and is what `cargo`, crates.io and CI runners expect. Keeping every build input inside the repository means a fresh clone builds and tests offline once dependencies are fetched, and FR-1.8 plus NFR-2 hold that property under test. See Question 1.

### D-3: Two engines; lint never writes; `--fix` is the only write path

Rules engine and model engine are both enabled by default. `--rules-only` and `--model-only` select one. `check` without `--fix` never writes a file and never loads a model. `check --fix` runs rules first, then the model on the windows that still carry findings.

**Rationale:** Brad's requirement (2) and "lint-only mode flags slop patterns and scores them without rewriting." The model only produces rewrites, so loading it for a read-only lint would cost seconds for no output; lint therefore stays cheap and deterministic, which suits CI. In `--model-only` mode the rule detectors still run, but only to choose which windows go to the model; no rule fix is applied. This matches Ruff's split of `check` and `--fix` (`research/competitive-analysis.md` §1.1, §2.1). Without this reading, "both on by default" would force a model load on every lint.

### D-4: Two fix classes — safe and model

Deterministic substitutions that keep meaning (for example "delve into" → "look at", a lone em dash → a comma) are *safe* fixes and are applied by `--fix`. Every model rewrite is a *model* fix. A model fix is written only if post-rewrite verification passes (FR-8.5 to FR-8.8). An unverified rewrite is never written. Detection-only rules, whose fix needs a rewrite (negative parallelism, tricolons, bullet spam), are fixed by the model engine only.

**Rationale:** Ruff's safe/unsafe model is the closest precedent (`research/competitive-analysis.md` §2.1). A pure opt-in for model rewrites would contradict "both engines on by default"; verify-then-apply honours both Brad's default and his requirement to preserve facts and numbers. `--diff` (FR-2.5) lets a user stage model output without writing. How far verification can be trusted is Question 3.

### D-5: Model weights are external, fetched explicitly, never silently

No weights in the repo or binary. `sloperator model pull` downloads the configured model to a local cache. With the model engine enabled and no model present, `check --fix` fails with exit 2 and tells the user to run `model pull` or pass `--rules-only`.

**Rationale:** weights are 1.8-4.3 GB (`research/inference-and-models.md` §2). The project rule is to fail loudly on local infra and never degrade silently (`research/feasibility-summary.md` RD-3, OQ-3). A silent fall back to rules-only would make "both engines on" a lie. Network use and Hugging Face as a source are Brad's decisions; see Question 2.

### D-6: A spike picks the runtime crate; the model sits behind an engine boundary

Phase 1 is a 2-3 day spike comparing `llama-cpp-2` and `mistralrs`, each with Qwen3-1.7B and Qwen3-4B at Q4_K_M or the nearest published quant, on CPU and on Brad's GPU path, before the SPEC freezes the crate. The runtime is hidden behind one engine boundary so it can be replaced. Pass criteria are in FR-9.9.

**Rationale:** build time, Metal support for `llama-cpp-2`, load time, tokens/s and rewrite quality are all unmeasured (`research/inference-and-models.md` §1, §7). Runtime crates churn (`llama-cpp-2` tracks llama.cpp; `mistralrs` changed options between 0.9.4 and 0.9.5), so versions are pinned and the boundary stays narrow (`research/feasibility-summary.md` Risks). Default model candidates: Qwen3-4B Q4_K_M (quality) and Qwen3-1.7B (speed). Phi-4-mini (MIT) is the licence-safe alternative. Llama 3.2 is excluded by default because its Acceptable Use Policy lists "Representing that the use of Llama 3.2 or outputs are human-generated" (`research/inference-and-models.md` §2); Qwen's licence text was not seen on the pages read. See Question 4.

### D-7: Mask, rewrite, verify

A CommonMark parser with source ranges identifies protected spans. Both engines receive only prose spans. After any edit, the output is checked against the input for the multiset of numbers, URLs and protected tokens and for unchanged block structure. Any mismatch keeps the original text and reports a finding.

**Rationale:** Vale's default of skipping code and inline code is exactly requirement (4) (`research/competitive-analysis.md` §1.1, §2.3); `pulldown-cmark` `OffsetIter` provides source ranges, MIT, with tables, footnotes, task lists and strikethrough as opt-in options (`research/inference-and-models.md` §4). Verification matters because the model is the unreliable component. Facts beyond numbers, URLs and user-declared terms cannot be checked deterministically; the PRD states that limit rather than claim more.

### D-8: Linter conventions adopted from Ruff and Vale

- Exit codes: 0 clean, 1 findings at or above the minimum severity, 2 tool failure. `--exit-zero` forces 0 for findings only.
- Severities: `suggestion`, `warning`, `error`; default minimum for exit 1 is `warning`.
- Config: nearest file wins upward from the file; no implicit parent merge; optional `extend`; `--config` override; CLI flags beat config.
- Rule selection: `select`, `extend_select`, `ignore`; the default enabled list is explicit and versioned, with no `ALL`.
- Suppression: line, block and file level; a block `disable` needs an explicit `enable`.

**Rationale:** these are the conventions both peers share or the documented better practice (`research/competitive-analysis.md` §1.3, §1.4, §2.2, §5, §6). The 0/1/2 split keeps "model missing" distinct from "findings". Vale ties exit 1 to `error` only; Brad asked for non-zero "on findings", so the default threshold is `warning`. Ruff warns that `ALL` adopts new rules silently on upgrade, and the signal catalogue ages (em-dash overuse is already listed among the older, fading indicators), so new rules start opt-in. Whether the threshold default is right is Question 7.

### D-9: Config file `sloperator.toml`

Format TOML; file name `sloperator.toml`; keys are snake_case.

**Rationale:** Ruff uses TOML; Vale uses INI; Brad asked for "linter-style, discoverable in the project" (`research/competitive-analysis.md` §1.4). TOML is the Rust-ecosystem idiom, and so are snake_case keys. See Question 5.

### D-10: Input scope is Markdown and plain text, English only

**Rationale:** Brad's requirements are Markdown-centric; the catalogue and candidate models are English-centred (`research/feasibility-summary.md` A-2, A-4). Other formats and languages would need new parsers, rules and model evidence.

### D-11: Make targets over cargo and nextest, with JUnit XML from nextest profiles

The `Makefile` wraps cargo and cargo-nextest in `build`, `test`, `test-integration`, `check`, `clean` and `help`. Tests run under nextest profiles (`ci` for the unit tier, `ci-perf` for the release-profile benchmark, `it` for the real-model tier) defined in `.config/nextest.toml`, and each profile writes JUnit XML to `target/nextest/<profile>/junit.xml`. `TESTS='<selector> ...'` narrows a run through a nextest filterset. Real-model tests are marked `#[ignore = "integration tier"]` and run only under `make test-integration`.

**Rationale:** CI and humans need per-test results; cargo emits none, while nextest writes JUnit natively, so no copy or conversion step is needed. Make gives one stable entry point locally and in CI (`.github/workflows/ci.yml` runs `make check`, `make build`, `make test`). Model-loading tests stay opt-in so the default run needs no model. The mechanism is ADR-07; FR-1.4 and FR-1.5 fix the outcome.

### D-12: Fine-tuning is deferred behind a measurement

v1 ships prompt design, few-shot examples and constrained output on an untuned model, plus the eval harness. A LoRA is considered only if the best untuned model misses a threshold Brad sets.

**Rationale:** tooling exists (LoRA in `mistralrs` and `llama-cpp-2`), but no source shows fine-tuning improves "sounds human", and no training data exists or may be fabricated (`research/inference-and-models.md` §5). See Question 8.

---

## 4. Functional Requirements

### FR-1: Crate shell and build targets

The system SHALL be a self-contained repository that its own `Makefile` builds, tests and checks.

- FR-1.1: The repository root SHALL hold a `Cargo.toml`, a README, a `LICENSE`, a `.gitignore` listing `target/`, and a `Makefile`.
- FR-1.2: The `Makefile` SHALL define the targets `build`, `test`, `test-integration`, `check`, `clean` and `help`.
- FR-1.3: `make test` SHALL run the unit tests, exit 0 when all pass, and exit non-zero when any fails.
- FR-1.4: After a test run, `target/nextest/ci/junit.xml` SHALL contain JUnit XML with one `testcase` per executed test.
- FR-1.5: `make test TESTS='<name>'` SHALL run only the tests matching each selector, and a selector matching no test SHALL fail the run.
- FR-1.6: `make build` SHALL produce the release binary, and `make check` SHALL fail on a formatting difference or on any compiler or lint warning.
- FR-1.7: `make test-integration` SHALL run the real-model tests; the unit tier SHALL NOT download or load a model.
- FR-1.8: A unit test SHALL fail when any file in the repository names the project the crate was developed in or its build-contract paths, or when `Cargo.toml` has a path dependency.
- FR-1.9: When `cargo` (or `cargo-nextest` / `cargo-deny`) is not on `PATH`, the `Makefile` SHALL print a one-line banner on stderr naming the missing tool and the install step, and SHALL exit non-zero. (Question 9.)

### FR-2: CLI surface

The system SHALL offer a linter-style command line.

- FR-2.1: `sloperator check [PATHS...]` SHALL report findings and scores and SHALL NOT modify any file or load a model.
- FR-2.2: `check` SHALL accept files, directories (walked recursively for `.md`, `.markdown` and `.txt`), glob patterns, and `-` for stdin; with no path it SHALL check the current directory.
- FR-2.3: For stdin, `--stdin-filename <name>` SHALL select the input type and the config section that applies; without it stdin SHALL be treated as Markdown.
- FR-2.4: `check` SHALL skip files matched by `.gitignore` and by config `exclude`, unless config `respect_gitignore` is `false`.
- FR-2.5: `check --diff` SHALL print a unified diff of what `--fix` would write, SHALL write nothing, and SHALL load the model only if the model engine is enabled.
- FR-2.6: `check --fix` SHALL apply fixes in place for file input and SHALL print the fixed text to stdout for stdin input.
- FR-2.7: The process SHALL exit 0 when no finding is at or above the minimum severity, 1 when at least one is, and 2 on tool failure (unreadable file, invalid config, unknown rule, missing model, runtime error).
- FR-2.8: `--exit-zero` SHALL force exit 0 on findings and SHALL NOT mask exit 2.
- FR-2.9: `--min-severity <suggestion|warning|error>` SHALL set which findings are reported and counted toward exit 1.
- FR-2.10: `--rules-only` and `--model-only` SHALL each select one engine; passing both SHALL exit 2 with a usage error; with neither, both engines SHALL be enabled.
- FR-2.11: `--select`, `--extend-select` and `--ignore` SHALL override the matching config keys.
- FR-2.12: `sloperator rules` SHALL list every rule with code, name, default severity, default state and fix class, and `sloperator rules <CODE>` SHALL print that rule's explanation and a before/after example.
- FR-2.13: `sloperator config [PATH]` SHALL print the resolved configuration for that path as JSON.
- FR-2.14: `sloperator model pull`, `model list` and `model path` SHALL download, list and locate models in the cache.
- FR-2.15: An unknown flag, subcommand, rule code or config key SHALL exit 2 with a message naming it.
- FR-2.16: `--help` and `--version` SHALL exit 0.

### FR-3: Output

The system SHALL emit human and machine-readable results.

- FR-3.1: `--output-format` SHALL accept `text` (default), `concise`, `json` and `github`; an unknown value SHALL exit 2.
- FR-3.2: `text` SHALL show for each finding the path, line, column, rule code, severity, message and the source line; `concise` SHALL print one line per finding.
- FR-3.3: `json` SHALL emit one JSON document on stdout with, per finding: path, rule code, rule name, severity, start and end line and column, message, matched text, whether a fix exists and its class (`safe` or `model`); plus a per-file score and an overall summary.
- FR-3.4: `github` SHALL emit GitHub Actions workflow-command annotations, one per finding, with severity mapped to `notice`, `warning` or `error`.
- FR-3.5: Findings SHALL go to stdout and diagnostics and progress to stderr; colour SHALL be off when stdout is not a terminal or `NO_COLOR` is set.
- FR-3.6: Output order SHALL be stable: files in sorted path order, findings in source order.

### FR-4: Rule engine

The system SHALL ship a built-in, versioned rule set.

- FR-4.1: Every rule SHALL have a code `SLP###`, a kebab-case name, a default severity, a default on/off state and a fix class (`safe`, `model` or `none`).
- FR-4.2: The rule set SHALL include at least one rule for each of: AI-vocabulary words (including "delve"), negative parallelism ("it's not X, it's Y" and "not just X, but also Y"), the rule of three, avoided copulatives ("serves as"), significance puffery, hedging disclaimers, markup irregularities (bullet spam and over-bolding), chatbot residue, and em-dash density; and SHALL contain at least 15 rules in total.
- FR-4.3: Em-dash detection SHALL fire on density per 1,000 prose words above a configurable threshold, not on a single occurrence.
- FR-4.4: Rules SHALL inspect only prose spans (FR-8.1); a matching string inside code, a link destination or front matter SHALL NOT produce a finding.
- FR-4.5: Rules SHALL be on only if they are in the explicit default list or selected by config; a rule added in a later release SHALL start off.
- FR-4.6: Each rule SHALL have fixtures with a positive case, a negative case and, where a fix exists, the expected fixed text; `cargo test` SHALL fail if a rule lacks them.
- FR-4.7: Rules with fix class `safe` SHALL be applied by `--fix`; rules with fix class `model` or `none` SHALL NOT be rewritten by the rules engine.

### FR-5: Scoring

The system SHALL report a transparent slop score.

- FR-5.1: The score SHALL be computed deterministically from the weighted findings normalised per 1,000 prose words, and the formula SHALL be documented in `docs/REFERENCE.md`.
- FR-5.2: `check` SHALL report the score per file and an overall score in `text` and `json` output.
- FR-5.3: Identical input and config SHALL yield an identical score.
- FR-5.4: `docs/REFERENCE.md` SHALL state that the score measures style tells, not text quality or provenance, and the tool SHALL NOT claim to make text undetectable.

### FR-6: Inline directives

The system SHALL honour suppression directives written as Markdown comments.

- FR-6.1: `<!-- sloperator-ignore SLP001 -->` SHALL suppress the named rules on the next prose line; with no codes it SHALL suppress all rules on that line.
- FR-6.2: `<!-- sloperator-disable SLP001 -->` ... `<!-- sloperator-enable SLP001 -->` SHALL suppress the named rules for the lines between them.
- FR-6.3: A `disable` without a matching `enable` before end of file SHALL exit 2 with the file and line of the directive.
- FR-6.4: `<!-- sloperator-ignore-file -->` SHALL suppress all findings in the file.
- FR-6.5: A directive naming an unknown rule code SHALL exit 2 with the file, line and code.
- FR-6.6: Suppressed regions SHALL NOT be edited by `--fix`, by either engine.
- FR-6.7: Plain-text input SHALL honour the same directives written as `[sloperator-ignore ...]` on their own line. (Assumption; plain text has no comment syntax.)

### FR-7: Configuration

The system SHALL read a linter-style config file.

- FR-7.1: For each input file the system SHALL use the nearest `sloperator.toml` found by searching upward from the file's directory, and SHALL NOT merge parent files unless `extend` names them.
- FR-7.2: `--config <path>` SHALL override discovery.
- FR-7.3: The config SHALL support keys for: `select`, `extend_select`, `ignore`; per-rule severity and thresholds; `include`, `exclude`, `respect_gitignore`; `min_severity`; per-file overrides by glob; the protected-span settings (FR-7.4); and the model settings (model name, cache path, window size).
- FR-7.4: The config SHALL let a user extend the protected set with regex patterns, literal terms that must survive unchanged, and toggles for front matter, math and HTML blocks.
- FR-7.5: CLI flags SHALL win over config; config SHALL win over built-in defaults.
- FR-7.6: An unknown key, a wrong type or an unknown rule code in config SHALL exit 2 with the file, key and reason.

### FR-8: Protection and verification

The system SHALL preserve code, links, numbers and Markdown structure through both engines.

- FR-8.1: Fenced and indented code blocks, inline code, link destinations and titles, images, autolinks, bare URLs, reference definitions, HTML blocks and front matter SHALL be protected spans; engines SHALL see and change only the remaining prose spans.
- FR-8.2: Heading text, list item text, table cell text and block quote text SHALL be prose spans; heading levels, list nesting and markers, table shape and block order SHALL NOT change.
- FR-8.3: User-configured patterns and terms (FR-7.4) SHALL be protected spans.
- FR-8.4: `--fix` SHALL preserve line endings (LF or CRLF), a trailing newline and the original bytes of all protected spans.
- FR-8.5: After any edit, the system SHALL verify that the multiset of numbers (integers, decimals, percentages, dates written with digits), URLs and configured terms in each edited window equals the multiset before.
- FR-8.6: The system SHALL verify that an edited window keeps the same Markdown block type and introduces no new block markers (heading, list, quote, fence, table).
- FR-8.7: When verification fails, the system SHALL keep the original text for that window, SHALL emit a finding naming the window and the failed check, and SHALL NOT exit 0 silently on that finding.
- FR-8.8: Input that is not valid UTF-8 SHALL exit 2 without modification.
- FR-8.9: Running `--fix` a second time on its own output SHALL change no text from the rules engine.

### FR-9: Model engine

The system SHALL rewrite flagged windows with a local SLM.

- FR-9.1: The model engine SHALL run in-process and SHALL make no network request during `check`; only `model pull` may use the network.
- FR-9.2: The model engine SHALL send to the model only the sentence or paragraph windows (size configurable) that still carry at least one finding at or above the minimum severity after rule fixes (or, in `--model-only`, after detection).
- FR-9.3: The prompt SHALL instruct the model to return only the rewritten window, with no preamble, and the output SHALL be checked against that contract (no thinking block, no wrapper text) before verification.
- FR-9.4: The model SHALL be loaded at most once per invocation, and not at all when no window needs rewriting.
- FR-9.5: A model rewrite SHALL be written only when it passes FR-8.5 and FR-8.6 and the detectors report no more weighted findings for the window than before.
- FR-9.6: With the model engine enabled and no model in the cache, `check --fix` and `check --diff` SHALL exit 2 naming `sloperator model pull` and `--rules-only`.
- FR-9.7: A model or runtime error SHALL exit 2 with the cause; no window SHALL be skipped silently.
- FR-9.8: Sampling SHALL use a fixed default seed, and `--seed <n>` SHALL override it, so a run is reproducible for the same binary, model and hardware.
- FR-9.9: The spike SHALL record, for each crate and model, cold build time, binary size, Metal or GPU availability, load time, tokens/s for a 150-token rewrite on CPU and on GPU, and results on 20 slop paragraphs; the runtime and default model SHALL be chosen only after protected-span integrity on those paragraphs is 100%.
- FR-9.10: Model weights SHALL NOT be committed to git or embedded in the binary.

### FR-10: Eval harness

The system SHALL measure rewrite quality so the fine-tuning decision rests on data.

- FR-10.1: The repo SHALL contain an eval set of slop paragraphs and protected-content fixtures, authored for this project (no fabricated training pairs).
- FR-10.2: An eval command or integration test SHALL report, per model: slop score before and after, protected-span integrity rate, and the count of rewrites rejected by verification.

---

## 5. Non-Functional Requirements

### NFR-1: No Behavioral Regression

A change keeps every existing test green and every documented command working. Verified by: `make check`, `make build` and `make test` pass locally and in CI before a change lands; tests are not weakened to make a change pass.

### NFR-2: Self-Contained Repository

Everything needed to build and test sloperator is inside the repository, with no path dependencies. Verified by: FR-1.8's test; the integration-tier `builds_when_extracted` test copies the crate to a fresh temp directory and runs `cargo test --offline` there, which passes.

### NFR-3: Performance

Rules-only `check` SHALL process a 100 KB Markdown file in under 1 second on the development machine in a release build and SHALL NOT load a model or read the model cache. Model-engine latency budgets are set from the spike results (FR-9.9) and recorded in the SPEC. Verified by: a benchmark test in the unit tier for the first; an integration test counting model loads (FR-9.4) for the second. The 1-second figure is a requirement set here, not a measured fact.

### NFR-4: Fail Loudly

Every failure that could change output or hide a problem exits 2 or emits a finding with the file and reason: invalid config, unknown rule code, unclosed directive, non-UTF-8 input, missing model, runtime error, failed verification. No silent fallback from the model engine to the rules engine. Verified by: one test per case in the list asserting exit code and message.

### NFR-5: Write Safety

`--fix` SHALL write a file only when its content changes, SHALL write atomically (temp file and rename), and SHALL write nothing when any error in that file would exit 2. Verified by: tests that inject a failure mid-run and assert the original file is byte-identical.

### NFR-6: Determinism

`check` (no model) output is a pure function of input, config and rule-set version. Verified by: running the same input twice and comparing JSON bytes; running with directory entries in different orders.

### NFR-7: Dependency and Licence Hygiene

The runtime crate and every other dependency SHALL be pinned to an exact version and have a licence compatible with the sloperator licence; `cargo` lockfile SHALL be committed. Verified by: `Cargo.lock` present; a lint step (`cargo deny` or equivalent, chosen in the SPEC) in `make check`.

### NFR-8: Test Coverage

Unit tests for each rule (FR-4.6), protection and verification (FR-8), directives (FR-6), config validation (FR-7.6), exit codes (FR-2.7, FR-2.8) and every output format (FR-3), in the unit tier with no model. Integration tests with a real model for FR-9 and FR-10. One end-to-end test runs the built binary on a fixture containing code, links, numbers and a table and asserts protected bytes are unchanged. Verified by: the JUnit report lists these tests.

### NFR-9: Clean Breaking Changes

No backward-compatibility shims: when a rule code, config key or flag is renamed during v0, every fixture, test and doc is updated in the same change. Verified by: grep for the old name returns nothing.

---

## 6. Architecture Overview

```
BEFORE:
  make test
     └─ no Makefile, no crate ──► nothing to run
  (no prose-lint tool)

AFTER:
  make test
     └─ Makefile ──► cargo nextest run --profile ci ──► JUnit XML in target/nextest/ci/junit.xml

  sloperator check|fix
  ┌────────┐   ┌───────────────┐   ┌──────────────────┐
  │ paths/ │──►│ config lookup │──►│ Markdown parse   │
  │ stdin  │   │ (nearest      │   │ → prose spans +  │
  └────────┘   │  sloperator.  │   │   protected spans│
               │  toml)        │   └────────┬─────────┘
               └───────────────┘            │ prose spans only
                                            ▼
                             ┌──────────────────────────┐
                             │ directives filter        │
                             └────────────┬─────────────┘
                                          ▼
                             ┌──────────────────────────┐
                             │ rules engine: findings,  │──► score
                             │ safe fixes               │
                             └────────────┬─────────────┘
                                 (--fix / --diff only)
                                          ▼
                             ┌──────────────────────────┐
                             │ model engine: rewrite    │
                             │ windows with findings    │
                             └────────────┬─────────────┘
                                          ▼
                             ┌──────────────────────────┐
                             │ verify: numbers, URLs,   │
                             │ terms, block structure   │
                             │ fail → keep original     │
                             └────────────┬─────────────┘
                                          ▼
                    reporter (text | concise | json | github) → exit 0/1/2
                    writer (--fix: atomic file write, stdin → stdout)
```

```
Engine selection:
  default        rules fixes → model on remaining windows
  --rules-only   rules fixes only
  --model-only   detectors choose windows → model rewrite, no rules fixes
  check (no --fix)  detectors + score only; model never loads
```

### Repository Layout

```
(repository root)
  ├── Cargo.toml, src/, tests/: new — CLI, config, parser masking, rule engine, model engine, verifier, reporters
  ├── Makefile:    new — build, test, test-integration, check, clean, help over cargo and nextest
  ├── .config/nextest.toml: new — ci, ci-perf and it profiles, JUnit XML per profile
  ├── .github/workflows/ci.yml: new — make check, make build, make test
  ├── fixtures/, eval/: new — rule fixtures, protection fixtures, eval set
  └── README, LICENSE, .gitignore: new — standalone packaging

docs
  └── PRD, research: new
```

---

## 7. Implementation Strategy

Tasks are numbered continuously across phases so the SPEC can reference them.

### Phase 1: Inference spike (gates the runtime choice)

1. Build a throwaway harness for `llama-cpp-2` and `mistralrs`, each loading Qwen3-1.7B and Qwen3-4B (Q4_K_M or nearest) on CPU and on the GPU path.
2. Record the FR-9.9 measurements, including a grammar or schema constraint test and 20-paragraph rewrite results.
3. Check Qwen3, Phi-4-mini licence terms against the product's purpose and record the model recommendation (Question 4).
4. Verify that a nextest filterset built from `TESTS` selects exactly the named tests and that a selector matching nothing can be detected.

### Phase 2: Crate shell and build targets

5. Create the crate skeleton at the repository root with `.gitignore`, README, `LICENSE`.
6. Write the `Makefile` with build, test, test-integration, check, clean and help, the nextest profiles, `TESTS` handling, the missing-tool banner and the integration tier.
7. Add the standalone-independence test (FR-1.8) and the extraction test (NFR-2).
8. Show `make test`, `make build` and `make check` pass.

### Phase 3: Protection layer

9. Parse Markdown with source ranges; compute prose and protected spans; honour config protected-set additions.
10. Implement the verifier (numbers, URLs, terms, block structure) and the edit applier with newline and CRLF preservation.
11. Add protection fixtures: code, links, front matter, tables, nested lists, CRLF, math.

### Phase 4: Rule engine and scoring

12. Implement the rule trait, registry, `SLP###` codes, severities, thresholds and the explicit default list.
13. Implement at least 15 rules across the nine families, each with positive, negative and fix fixtures.
14. Implement the safe fixes and the score formula; write the README score section.
15. Implement directives (FR-6) including unclosed-block and unknown-code errors.

### Phase 5: CLI, config and output

16. Implement `check`, input resolution (files, directories, globs, stdin, `.gitignore`), exit codes, `--exit-zero`, `--min-severity`.
17. Implement config discovery, `extend`, per-file overrides, validation; CLI-over-config precedence.
18. Implement `text`, `concise`, `json`, `github` reporters and stable ordering.
19. Implement `rules` and `config` subcommands and `--diff`; atomic writer.

### Phase 6: Model engine

20. Implement the engine boundary around the runtime chosen in Phase 1; pin versions.
21. Implement `model pull|list|path`, cache location, missing-model error.
22. Implement window selection, prompt, output-contract check, seed, single load, accept-only-if-verified logic and `--rules-only` / `--model-only` wiring.
23. Add integration tests with the real model, run by `make test-integration`.

### Phase 7: Eval and hand-off

24. Build the eval set and the eval report (FR-10).
25. Run the end-to-end fixture test, benchmarks (NFR-3) and the full suite via `make test` and `make test-integration`.
26. Update the README's Development section and the docs to match the shipped targets and commands.

---

## 8. Risks

### Risk 1: The SLM alters or drops facts that the verifier cannot see

**Risk:** Verification covers numbers, URLs, configured terms and structure. A model can still change a name, a date written in words, or a claim's polarity, and the check passes.
**Mitigation:** Window-sized rewrites bound the damage; `--diff` lets users review; the README states the limit; the eval harness counts rejected rewrites; a configurable term list protects names. The PRD does not claim fact preservation beyond the verified classes.

### Risk 2: Small models do not rewrite convincingly at 1-4B

**Risk:** Output reads no more human than the input, or the model introduces new tells.
**Mitigation:** FR-9.5 rejects a rewrite that does not lower the weighted findings; the spike tests 4B before 1.7B; rules-only mode stays useful on its own.

### Risk 3: The runtime crate churns or fails to build

**Risk:** `llama-cpp-2` "does not follow semver meaningfully" and needs `clang`; `mistralrs` changed options between 0.9.4 and 0.9.5; Metal for `llama-cpp-2` is unconfirmed.
**Mitigation:** Pin exact versions and commit the lockfile; one narrow engine boundary; the spike records toolchain needs and picks between the two.

### Risk 4: The build targets mis-report Rust tests

**Risk:** Tests pass but the JUnit report holds zero tests, or a `TESTS` selector silently matches nothing and the run reports green.
**Mitigation:** FR-1.4 and FR-1.5 specify the outcome; the `Makefile` fails a run whose reports hold no `testcase`; Phase 1 task 4 verifies the mechanism; the commands in Phase 2 task 8 are run and their reports inspected.

### Risk 5: Rust is absent on a developer machine

**Risk:** `make test`, `make build` or `make check` is run where `cargo`, `cargo-nextest` or `cargo-deny` is missing, and fails with an obscure shell error.
**Mitigation:** FR-1.9's one-line banner names the missing tool and its install step, and the target exits non-zero.

### Risk 6: Model licence terms conflict with the product's purpose

**Risk:** Llama 3.2's Acceptable Use Policy lists "Representing that the use of Llama 3.2 or outputs are human-generated"; Qwen's licence text was not read.
**Mitigation:** Llama 3.2 excluded by default; Qwen and Phi-4-mini terms read in Phase 1 task 3; Question 4 puts the choice to Brad.

### Risk 7: Over-triggering annoys users

**Risk:** A human-written em dash or tricolon is flagged; a fix flattens a deliberate style.
**Mitigation:** Density thresholds per 1,000 words; three severity levels; inline ignore directives; per-rule switches; new rules start off (FR-4.5).

### Risk 8: Rule defaults age as LLM style shifts

**Risk:** Em-dash overuse is already listed among the older, fading indicators (`research/competitive-analysis.md` §4).
**Mitigation:** The default list is explicit and versioned; the rule set version appears in `rules` and JSON output; thresholds are configurable.

### Risk 9: Unmeasured performance misses expectations

**Risk:** The only load-time datum is a 7B model on an RTX 3090 with CUDA (`research/inference-and-models.md` §1.1); a laptop CPU may take many seconds per window.
**Mitigation:** The model loads once and never in lint mode; window size is configurable; the spike measures before the budget is frozen (FR-9.9).

### Risk 10: Docs drift

**Risk:** The README and `docs/` name make targets, flags and test commands that can drift from the `Makefile`.
**Mitigation:** Phase 7 task 26 updates them; NFR-9 requires renames to update every doc in the same change.

---

## 9. Follow-up Questions

Brad is not being asked now. Each question carries the assumption the PRD is written on.

- [ ] **Question 1 — Where does the crate live?** Options: (a) a standalone repository with the crate at its root; (b) a subdirectory of a larger workspace. Recommendation: (a). Context: the request asks for a tool that can stand on its own GitHub repository.
  - Assumption: (a), per D-2.
- [ ] **Question 2 — Missing model, network and download.** Options: fail with exit 2 and require an explicit `model pull`; auto-download on first use; silently fall back to rules-only. Recommendation: explicit `model pull`, fail loudly. Context: "both engines on by default" conflicts with "fail loudly, never silently degrade"; downloading from Hugging Face is a new network dependency.
  - Assumption: explicit `model pull`, fetched from Hugging Face, exit 2 when missing, `--rules-only` as the escape (D-5, FR-9.6).
- [ ] **Question 3 — Are verified model rewrites written in place by default?** Options: write when verification passes; stage as a diff unless `--unsafe-fixes`; never write. Recommendation: write only when verification passes, with `--diff` for review. Context: the biggest tension in the request; verification cannot see names or claim polarity.
  - Assumption: verified model rewrites are written by `--fix`; unverified are never written (D-4, FR-9.5). Lint-only `check` never loads the model.
- [ ] **Question 4 — Default model and who reads the licence.** Options: Qwen3-4B Q4_K_M, Qwen3-1.7B, Phi-4-mini (MIT), Llama 3.2 (AUP concern). Recommendation: pick from Qwen3-4B and Qwen3-1.7B after the spike; Phi-4-mini as the licence-safe fallback. Context: the Qwen licence text was not seen; Gemma and Llama are gated.
  - Assumption: Qwen3-4B is the default if the spike passes FR-9.9 and its licence permits the use; otherwise Phi-4-mini. Llama 3.2 excluded (D-6).
- [ ] **Question 5 — Config naming.** Options: `sloperator.toml` with snake_case keys; camelCase keys; YAML. Recommendation: TOML, snake_case. Context: snake_case is the Rust and TOML idiom.
  - Assumption: TOML, snake_case (D-9).
- [ ] **Question 6 — Which machine formats ship in v1?** Options: JSON only; JSON plus GitHub; plus JUnit and SARIF. Recommendation: JSON plus GitHub. Context: Brad asked for "machine-readable output formats" without naming them.
  - Assumption: `json` and `github`; others out of scope.
- [ ] **Question 7 — Default exit threshold.** Options: exit 1 on `warning` or above; on `error` only (Vale's rule); on any finding. Recommendation: `warning`. Context: Brad said "non-zero exit code on findings"; single em dashes are soft findings.
  - Assumption: `warning` or above, `--exit-zero` and `--min-severity` as overrides (D-8).
- [ ] **Question 8 — Fine-tuning threshold and owner.** Options: defer entirely; set an eval threshold now. Recommendation: defer; build the harness. Context: no evidence yet that tuning helps; no training data exists.
  - Assumption: no fine-tuning in v1; Brad sets the threshold after the first untuned results (D-12).
- [ ] **Question 9 — Machines without the Rust tools.** Options: fail with a one-line banner naming the tool and install step; fail with whatever error the shell gives. Recommendation: the banner, with a non-zero exit. Context: `make` targets call `cargo`, `cargo-nextest` and `cargo-deny`.
  - Assumption: as recommended (FR-1.9).
- [ ] **Question 10 — Licence of the sloperator code.** Options: MIT OR Apache-2.0 (Rust convention); MIT; proprietary until release. Recommendation: MIT OR Apache-2.0. Context: the standalone repository needs a licence file, and dependencies (`llama-cpp-2` is MIT/Apache-2.0, `pulldown-cmark` MIT) constrain the choice.
  - Resolved: MIT, in `LICENSE` and `Cargo.toml`.
- [ ] **Question 11 — In-process model or a client of a local server?** Options: embedded runtime; client of a user-run server (`mistralrs serve`, `ollama`). Recommendation: embedded. Context: a server lowers build weight but "both engines on by default" would then need a running server.
  - Assumption: in-process only (FR-9.1); a server client is out of scope for v1.
