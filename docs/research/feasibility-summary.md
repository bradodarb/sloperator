# Initial Feasibility Summary: sloperator

**Date:** 2026-10-22
**Status:** Feasible with caveats. Rust SLM inference is practical on paper, but nothing was run.
**Scope:** new standalone Rust crate with its own `Makefile` (build, test, check over cargo and cargo-nextest)
**Next Step:** PRD (decide OQ-1 to OQ-13), then a 2-3 day inference spike before the SPEC freezes the runtime crate
**Related:** [Competitive Analysis](./competitive-analysis.md) | [Inference and Models](./inference-and-models.md)

---

## Vision

After this ships, a writer or CI job runs `sloperator check README.md` and gets a findings list and a slop score for
LLM-isms (em-dash overuse, "delve", "it's not X, it's Y", tricolons, hedging, bullet spam). `sloperator check --fix`
cleans what rules can fix safely and asks a local small model to rewrite the rest, while code blocks, links, numbers
and Markdown structure come out untouched. It reads a project config file, honours inline ignore directives, exits
non-zero on findings and emits JSON for tooling. It is a standalone repository with no dependency on any other project.
Flavor: greenfield build.

---

## Why This Matters

1. **No prose-lint capability exists to build on.** No prose rules and no AI-writing detection; everything is new.
2. **The "standalone, with `make test`" requirement** means the crate owns its build: a root `Makefile` drives
   cargo and cargo-nextest and writes JUnit reports, with no external build system.
3. **The model rewrite is the risky half.** Small models "may generate incorrect or outdated factual statements"
   (Gemma 3 card) and lack capacity "to store too much factual knowledge" (Phi-4-mini card)
   (`inference-and-models.md` section 2). Brad requires facts and numbers preserved, so verification, not the model, is
   what makes `--fix` trustworthy.

---

## What Already Works

| Component | Status | Notes |
|-----------|--------|-------|
| Test runner with JUnit output (external) | Exists | `cargo-nextest` profiles write `junit.xml` per profile |
| Integration tier as a model-download opt-in (pattern) | Exists | ignored tests run only by an explicit target with the model feature |
| Rust inference stacks (external) | Exist, unbenchmarked here | `llama-cpp-2`, `mistralrs`, `candle` (`inference-and-models.md` section 1) |
| CommonMark parser with source ranges (external) | Exists | `pulldown-cmark` `OffsetIter` (`inference-and-models.md` section 4) |
| Peer CLI conventions (external) | Documented | Ruff and Vale (`competitive-analysis.md`) |

---

## What Needs Work

### Pillar 1: Crate shell and build (S, blocks everything)

**Problem:** the crate needs a build, test and check entry point that developers and CI share, with test selection
and machine-readable test reports.

**Solution:** a crate with its own `.gitignore` (`target/`), a `Makefile` with `build`, `test`, `test-integration`,
`check`, `clean` and `help`, nextest profiles that write JUnit XML under `target/nextest/<profile>/junit.xml`,
`TESTS` selector handling in the recipe, a standalone test, and a loud failure when a required tool is missing
(OQ-13).

**Effort:** ~2-3 days.

### Pillar 2: CLI, config and output (M)

**Problem:** the peer-familiar surface (`check`, `--fix`, files/globs/stdin, 0/1/2 exit codes, JSON, inline
directives, nearest-file config with `extend`) must be specified precisely.

**Solution:** adopt the conventions in `competitive-analysis.md` sections 1 and 7: Ruff's `check`/`--fix` split and
rule-selection precedence, Vale's exit codes and `--minAlertLevel`, an `ls-config`-style introspection command.

**Effort:** ~4-6 days.

### Pillar 3: Deterministic rule engine (M)

**Problem:** Brad's rule list (em dash, "delve", "it's not X, it's Y", tricolons, hedging, bullet spam, "etc.")
needs detectors, severities, scores, thresholds and safe fixes.

**Solution:** a versioned built-in rule list seeded from the Wikipedia signal catalogue (`competitive-analysis.md`
section 4), operating only on prose spans produced by the Markdown parser. Detection for negative parallelism,
tricolons and bullet spam is rule-based; their fixes are rewrite-class (model) fixes, and the PRD must say so.

**Effort:** ~6-8 days for ~15 rules with fixtures and tests.

### Pillar 4: Protection layer (M)

**Problem:** preserve code, links, URLs, front matter, numbers and user-configured spans through both engines.

**Solution:** parse with `pulldown-cmark`, mark protected ranges, hand only prose spans to either engine, and
verify afterwards that numbers, URLs and protected tokens are unchanged, keeping the original on mismatch and
reporting loudly (`inference-and-models.md` section 4; project rule "fail loud").

**Effort:** ~4-5 days.

### Pillar 5: SLM engine, spike first (L, highest uncertainty)

**Problem:** model choice, runtime crate, load time and rewrite quality are unmeasured. Whether Metal works through
`llama-cpp-2` is unconfirmed (`inference-and-models.md` section 1).

**Solution:** run the spike in `inference-and-models.md` section 7, then implement a pluggable engine boundary,
prompt and windowing design, model location and download, and an eval harness (needed anyway for the fine-tuning
decision).

**Effort:** spike ~2-3 days; engine ~6-10 days; eval harness ~3-4 days.

### Pillar 6: Packaging and extraction readiness (S)

**Problem:** the crate is published as its own GitHub repository.

**Solution:** self-contained `Cargo.toml`, its own README and licence file, no path references outside the
repository, a test that fails if any file names the development host.

**Effort:** ~1-2 days.

---

## What We're NOT Building

- **Fine-tuning or training data in v1**: no evidence it helps and no training data exists; gate is in
  `inference-and-models.md` section 5.
- **An AI-text detector or "undetectable" guarantee**: the source says detectors and humans are unreliable and that
  stripping signs can make detection harder (`competitive-analysis.md` section 4). The tool reports style tells only.
- **Bundled model weights**: files of 1.8-4.3 GB (`inference-and-models.md` section 2) do not belong in git or the binary (RD-3).
- **User-authored rule languages**: Vale-style external styles are a later question; v1 ships a built-in rule set.
- **Non-English prose**: the catalogue and candidate models are English-centred (Phi-4-mini card says so); out of scope until Brad says otherwise (A-4).
- **Editor/LSP integration**: not requested.
- **Any dependency on code outside the crate's own repository** (other than published crates): the product must stay standalone.

---

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| SLM rewrites alter or drop facts and numbers | High | High | Mask, rewrite prose spans only, verify protected tokens, keep original on mismatch, treat model fixes as the unsafe class (OQ-10) |
| Small models cannot rewrite convincingly at 1-4B | Med | High | Spike and eval harness before committing; rules-only mode remains useful; test the 4B before the 1.7B |
| CI cannot read Rust test results | Med | High | nextest JUnit profiles are RD-4; add an acceptance command to the PRD |
| Developers without Rust or cargo-nextest get a confusing failure | Med | Med | OQ-13: each target checks its tools and fails with a one-line banner and install hint |
| Runtime crate churn (`llama-cpp-2` "does not follow semver meaningfully"; `mistralrs` changed options between 0.9.4 and 0.9.5) | High | Med | Pin exact versions; hide the runtime behind one engine trait; keep the spike result in the SPEC |
| Native build toolchain friction (`llama-cpp-2` needs clang for bindgen) | Med | Med | Record in the spike; `mistralrs` is the alternative |
| Model licence terms conflict with the product's purpose | Med | High | Llama 3.2's Acceptable Use Policy lists "representing that ... outputs are human-generated" (`inference-and-models.md` section 2); Qwen licence text was not seen on the pages read; Brad/legal reads them (OQ-4) |
| Rule defaults age as LLM style shifts | High | Low | Versioned default list; thresholds and per-rule switches; new rules start opt-in |
| Over-triggering annoys users (a human-written em dash flagged) | Med | Med | Density thresholds per 1,000 words; severity levels; inline ignore directives |
| Docs drift | Med | Low | README and `make help` are updated with every target change |

---

## Effort Estimate

| Pillar | Scope | Effort |
|--------|-------|--------|
| 1. Crate shell and build | crate, `Makefile`, nextest JUnit profiles, guards | ~2-3 days |
| 2. CLI, config, output | commands, config discovery, formats, directives | ~4-6 days |
| 3. Rule engine | ~15 rules, scoring, fixtures | ~6-8 days |
| 4. Protection layer | parser masking, verification | ~4-5 days |
| 5. SLM engine | spike 2-3, engine 6-10, eval 3-4 | ~11-17 days |
| 6. Packaging and extraction | standalone checks, README | ~1-2 days |
| **Total** | | **~28-41 days** (single engineer; estimates are the researcher's, not measured) |

---

## Assumptions

- **A-1:** The acceptance command (`make test`) is the binding requirement; the crate sits at the repository root
  (OQ-1).
- **A-2:** "Markdown structure" means CommonMark plus the common extensions; plain text is also accepted; other
  formats (AsciiDoc, reStructuredText, HTML) are out of v1.
- **A-3:** The developer machine is macOS; Apple Silicon and GPU
  availability are **not established** and the spike must test them.
- **A-4:** English only.
- **A-5:** "Both engines on by default" means a plain `check` runs rules and model; the model step may fall back
  loudly (never silently) if no model is installed (OQ-3).
- **A-6:** The rule set is built in and versioned; users tune it by config, not by writing new rules.
- **A-7:** Exit codes follow the Vale convention (0 clean, 1 findings at or above the configured severity, 2 tool
  failure), with an override flag for report-only CI (`competitive-analysis.md` section 1.3).
- **A-8:** Quality is judged by a repo-local eval set; no external benchmark measures rewrite fidelity.

---

## Resolved Decisions

| # | Decision | Rationale |
|---|----------|-----------|
| RD-1 | Implement in Rust. | Brad's preference; inference is practical through `llama-cpp-2` or `mistralrs`, so the Java/Python fallback is not triggered (`inference-and-models.md` sections 1 and 6). The comparison goes in the PRD. |
| RD-2 | Two engines, both on by default; flags select rules-only or model-only. | Brad's requirement (2). |
| RD-3 | Model weights are not stored in the repo or the binary. | Sizes of 1.8-4.3 GB per file (`inference-and-models.md` section 2); how they are fetched is OQ-3. |
| RD-4 | Test runs emit JUnit XML under `target/nextest/<profile>/junit.xml`. | CI and tooling read one report per nextest profile (ci, ci-perf, it). |
| RD-5 | Standalone, enforced by a test. | Brad's requirement; `tests/standalone.rs` scans the repository so no file names the development host. |
| RD-6 | Lint mode never writes; `--fix` is the only write path. | Brad's "lint-only mode flags and scores without rewriting"; matches Ruff (`competitive-analysis.md` section 1.1). |

---

## Open Questions

| # | Question | Context |
|---|----------|---------|
| OQ-1 | **Where does the crate live?** *Resolved: the crate is a standalone repository; `Cargo.toml`, `src/`, `tests/`, `fixtures/` and `eval/` sit at the repository root next to its `Makefile`.* | PRD D-2. |
| OQ-2 | Which runtime crate and which hardware must the spike cover? *Recommendation: spike `llama-cpp-2` and `mistralrs` with Qwen3-1.7B and Qwen3-4B on Brad's Mac and CPU-only; decide after.* | Metal for `llama-cpp-2` unconfirmed; no local benchmark exists. |
| OQ-3 | What happens when no model is installed or the model fails: skip the model engine loudly and exit 2, fail the run, or auto-download? Where do models live and how are they fetched? *Recommendation: explicit `sloperator model pull`; a missing model is a loud error (exit 2) under the default settings, with `--rules-only` as the escape.* | "Both engines on by default" vs "fail loud, never silently degrade" (project rule). |
| OQ-4 | Which default model, and who reads the licence? *Recommendation: Qwen3-4B Q4_K_M as the quality candidate and Qwen3-1.7B as the fast candidate, pending the spike; Phi-4-mini (MIT) as the licence-safe alternative; avoid Llama 3.2 for this product unless counsel clears its AUP.* | Qwen licence text was not seen on the pages read; Gemma and Llama are gated; Llama's AUP wording (`inference-and-models.md` section 2). |
| OQ-5 | Config format and file name (`sloperator.toml`, `.sloperator.toml`, YAML, or a table in `pyproject`-like file)? *Recommendation: `sloperator.toml`, nearest file wins, optional `extend`.* | Peers use TOML (Ruff) and INI (Vale); Brad said "linter-style, discoverable in the project". |
| OQ-6 | Which key style does the config file use? *Recommendation: the Rust/TOML idiom (snake_case or kebab-case).* | Matches peer TOML-configured tools. |
| OQ-7 | What eval threshold would justify fine-tuning, and who owns it? *Recommendation: defer; build the eval harness in v1 and revisit once untuned results are known.* | `inference-and-models.md` section 5. |
| OQ-8 | Which severities exist and which default level triggers exit 1? Does score above a threshold fail the run? *Recommendation: three levels (suggestion, warning, error); exit 1 on warning or above by default; `--exit-zero` for report-only.* | Vale ties the code to `error` only (`competitive-analysis.md` section 1.3); Brad said "non-zero on findings". |
| OQ-9 | Which machine formats ship in v1: JSON only, or also JUnit, GitHub annotations, SARIF? *Recommendation: JSON plus GitHub annotations; JUnit and SARIF later.* | Ruff ships many; Vale links templates for rdjsonl and SARIF (not opened). |
| OQ-10 | Are model edits written in place by `--fix`, or staged (diff/suggestion) unless `--unsafe-fixes` (or similar) is given? *Recommendation: rules-engine safe fixes apply by default; model rewrites apply only with an explicit flag or after protected-token verification passes; unverified rewrites are never written.* | The largest tension in the request: "both engines on by default" vs a rewrite that can corrupt facts (`competitive-analysis.md` section 2.1). Brad alone decides. |
| OQ-11 | In-process model (embedded runtime) or a client of a user-run server (mistral.rs `serve`, `ollama`)? *Recommendation: in-process by default; optional OpenAI-compatible endpoint later.* | Embedded: self-contained binary, heavier build. Server: lighter build, but "both engines on by default" needs a running server. |
| OQ-12 | Which input types in v1: Markdown and plain text only? *Recommendation: yes (A-2).* | Brad's list is Markdown-centric. |
| OQ-13 | What happens when Rust or a required cargo tool is absent? *Resolved: every `Makefile` target checks its tools (`cargo`, `cargo-nextest`, `cargo-deny`) and fails loudly with a one-line banner and install hint, exit 1; nothing is skipped silently.* | Fail loud, never silently degrade. |

---

## References

- [Competitive Analysis](./competitive-analysis.md): source for the Ruff, Vale and Wikipedia claims
- [Inference and Models](./inference-and-models.md): source for runtime, model, licence and language-comparison claims

No new external claims are made in this summary beyond those cited in the two sibling files.
