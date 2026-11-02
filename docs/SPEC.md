# Technical Specification: sloperator — a standalone CLI that makes LLM-generated text read as human

**Version:** 0.4 (revision 3: adversarial review round 3, final pass)
**Date:** 2026-10-30
**Status:** Draft
**Module(s):** `sloperator` (the crate at the repository root, PRD D-2)
**Companion:** [PRD](./PRD.md) · [Architecture](./ARCHITECTURE.md) · ADRs [01](./adr/ADR-01-single-crate-layered-modules.md) [02](./adr/ADR-02-rewriter-port-feature-gated-runtime.md) [03](./adr/ADR-03-source-range-edits.md) [04](./adr/ADR-04-placeholder-masking.md) [05](./adr/ADR-05-verify-every-edit.md) [06](./adr/ADR-06-rules-as-code.md) [07](./adr/ADR-07-nextest-junit-bridge.md) [08](./adr/ADR-08-model-registry-and-pull.md) [09](./adr/ADR-09-parallel-analysis-serial-model.md) · UX: none (the CLI surface is PRD FR-2/FR-3; the architecture records "no UX scope")
**Depends On:** none

> **How the code blocks were checked.** No Rust toolchain is installed on this machine (`command -v cargo rustc`
> printed nothing this session, architecture §2 and Risk R1). So no Rust block could be compiled. Every
> third-party call in this SPEC was checked against the docs.rs page of the pinned version, fetched this session:
> `llama-cpp-2` 0.1.160, `pulldown-cmark` 0.13.6, `ignore` 0.4.34, `globset` 0.4.21, `aho-corasick` 1.1.5,
> `similar` 3.2.1, `tempfile` 3.27.0, `ureq` 3.5.0, `dirs` 7.0.0, `unicode-segmentation` 1.13.3 and `clap` 4.6.7.
> The make code in §3 *was* executed: its guard and selector logic ran under GNU make with the `Makefile`'s own
> shell settings (`bash`, `.ONESHELL`, `-eu -o pipefail`). The Phase 2 gate (§3) is the first real compile.
> Compile-checking the crate is Phase 0's prerequisite, not something this SPEC can claim.

---

## Table of Contents

0. [Phase 0 — toolchain gate](#0-phase-0--toolchain-gate)
1. [Inference spike and results record](#1-inference-spike-and-results-record)
2. [Crate packaging](#2-crate-packaging)
3. [`Makefile` — build, test and check targets](#3-makefile--build-test-and-check-targets)
4. [Crate root and the error type](#4-crate-root-and-the-error-type)
5. [Layering and standalone tests](#5-layering-and-standalone-tests)
6. [`doc` — parse and span map](#6-doc--parse-and-span-map)
7. [`edit` — proposals, plan, apply, diff](#7-edit--proposals-plan-apply-diff)
8. [`verify` — the edit gate](#8-verify--the-edit-gate)
9. [`rules` — core types and registry](#9-rules--core-types-and-registry)
10. [Built-in rule catalogue](#10-built-in-rule-catalogue)
11. [`score`](#11-score)
12. [`directives`](#12-directives)
13. [`config`](#13-config)
14. [`input`](#14-input)
15. [`model` — registry, cache, pull](#15-model--registry-cache-pull)
16. [`engine` — port, windows, masking, prompt](#16-engine--port-windows-masking-prompt)
17. [`pipeline`](#17-pipeline)
18. [`report`](#18-report)
19. [`write`](#19-write)
20. [`cli` and `main`](#20-cli-and-main)
21. [`engine::runtime` — the llama-cpp-2 adapter](#21-engineruntime--the-llama-cpp-2-adapter)
22. [Eval harness](#22-eval-harness)
23. [Testing Strategy](#23-testing-strategy)
24. [Resolved Design Questions](#24-resolved-design-questions)
- [Open Decisions](#open-decisions) · [PRD Deviation Notes](#prd-deviation-notes) · [Architecture Deviation Notes](#architecture-deviation-notes) · [Self-Audit](#self-audit) · [Revision Log](#revision-log)

Sections follow the delivery phases in architecture §12. One change of phase is recorded in the Architecture
Deviation Notes: the spike-independent parts of the model path (§15, §16) move from Phase 6 to Phase 5.

---

## 0. Phase 0 — toolchain gate

**File:** none (machine setup; recorded in `README.md` §"Building", §2)
**Component:** §12 Phase 0, Risk R1
**Implements:** prerequisite for FR-1.3, FR-1.6, FR-9.9

There is no code here, only a precondition. Phases 1-7 need `rustup` with the pinned toolchain (§2), plus
`cargo-nextest` 0.9.150 and `cargo-deny` 0.20.2. The `llama-cpp-2` build also needs `clang` and `cmake`.
All four versions are the latest on crates.io and the Rust release channel, checked this session.

**Gate:** `cargo --version`, `cargo nextest --version`, `cargo deny --version`, `clang --version` and
`cmake --version` all exit 0. The `rustup` toolchain resolves to `1.99.1` from the repository root.

---

## 1. Inference spike and results record

**File:** `docs/research/spike-results.md` (new). `eval/paragraphs/*.md` (new; the 20
spike paragraphs, reused by §22). The harness itself lives outside the repo in `$TMPDIR/sloperator-spike/` and is not
committed (architecture §10).
**Component:** §12 Phase 1; §4.10 engine (ADR-02, ADR-04), §4.14 model (ADR-08)
**Implements:** FR-9.9, D-6, PRD Phase 1 tasks 1-4

The spike decides three things: the runtime crate behind §21, the default registry entry in §15, and whether
placeholder masking (ADR-04 Option 1) holds up or the degraded mode (ADR-04 Option 2) is needed. The results file is
a fixed form, so reviewers can check every FR-9.9 field:

```markdown
# Spike results — sloperator inference

Machine: <model, CPU, RAM, GPU, OS>       Toolchain: rustc 1.99.1, clang <v>, cmake <v>
| Crate @version | Model file | Device | Cold build (s) | Binary (MB) | GPU used | Load (s) | tok/s (150-token rewrite) |
| llama-cpp-2 =0.1.160 | Qwen3-4B-Q4_K_M.gguf | CPU | … |
| llama-cpp-2 =0.1.160 (feature metal) | Qwen3-4B-Q4_K_M.gguf | Metal | … |
| mistralrs =0.8.1 | … | CPU / Metal | … |
| … Qwen3-1.7B-Q8_0.gguf rows … |
## 20-paragraph results (per crate × model)
| Paragraph | Findings before | After | Placeholders kept | Numbers/URLs kept | Output contract | Accepted |
Protected-span integrity after verify: N/20 (must be 20/20 for the chosen pair)
Placeholder survival before verify: N/M placeholders (decides ADR-04 Option 1 vs 2)
Grammar constraint (LlamaSampler::grammar) tried: yes/no — effect on contract violations
## Licence reading (PRD Question 4)
## Test selectors through the Makefile (PRD Phase 1 task 4)
## Decision: crate, default model, masking mode, model latency budget (NFR-3 second half)
```

**Design notes:**

- **`llama-cpp-2` is the primary candidate in this SPEC; the spike can still pick `mistralrs`.** The research
  could not confirm Metal for `llama-cpp-2`. This session's crates.io query shows `metal`, `cuda` and `vulkan`
  features on 0.1.160 (licence `MIT OR Apache-2.0`), so that gap is closed on paper. `mistralrs` is at **0.8.1** on
  crates.io, not the "0.9.x" in the research and the architecture; the architecture's §9 row is corrected this pass.
  If the spike picks `mistralrs`, only §21 and the `model` feature line in §2 change. The `Rewriter` port does not
  (ADR-02).
- **Licences already seen.** The Hugging Face API (`/api/models/Qwen/Qwen3-4B-GGUF`) reports `license: apache-2.0`
  for both Qwen3 GGUF repos. That is card metadata, not the licence text, so the spike still records the text, as
  PRD Phase 1 task 3 requires.
- **Pass/fail is the architecture's §12 Phase 1 gate, unchanged.** If neither crate passes, the spike stops and goes
  to Brad (D-1's fallback question reopens). Phases 2-5 do not wait for it.
- **The model latency budget** that NFR-3 leaves "set from the spike" goes in the Decision section. §22's eval test
  asserts it.

---

## 2. Crate packaging

**File:** repository root (new): `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `clippy.toml`, `deny.toml`,
`.config/nextest.toml`, `.gitignore`, `README.md`, `LICENSE`
**Component:** §4.15 module shell (ADR-01, ADR-02, ADR-07)
**Implements:** FR-1.1, FR-5.1, FR-5.4, FR-9.10, NFR-2, NFR-7, D-1, D-2, D-12, PRD Question 10 (MIT)

One crate: library `sloperator` (`src/lib.rs`) and binary `sloperator` (`src/main.rs`). Every version is an exact
pin taken from crates.io this session (NFR-7). `Cargo.lock` is committed, and every cargo call uses `--locked`.

```toml
# Cargo.toml
[package]
name = "sloperator"
version = "0.1.0"
edition = "2024"
rust-version = "1.88"
license = "MIT"
description = "Lint and rewrite LLM-generated prose so it reads as human-written"
readme = "README.md"
publish = false

[features]
default = ["model"]
model = ["dep:llama-cpp-2"]                  # ADR-02: the only feature that pulls the runtime
metal = ["model", "llama-cpp-2/metal"]       # passthrough (ADR-02 follow-up)
cuda = ["model", "llama-cpp-2/cuda"]

[dependencies]
aho-corasick = "=1.1.5"
clap = { version = "=4.6.7", features = ["derive"] }
dirs = "=7.0.0"
globset = "=0.4.21"
ignore = "=0.4.34"
llama-cpp-2 = { version = "=0.1.160", optional = true }
pulldown-cmark = { version = "=0.13.6", default-features = false }
rayon = "=1.12.0"
regex = "=1.13.1"
serde = { version = "=1.0.229", features = ["derive"] }
serde_json = "=1.0.154"
sha2 = "=0.11.1"
similar = "=3.2.1"
tempfile = "=3.27.0"
thiserror = "=2.0.21"
toml = "=1.1.8"
unicode-segmentation = "=1.13.3"
ureq = "=3.5.0"

[dev-dependencies]
assert_cmd = "=2.2.2"
predicates = "=3.1.4"

[lints.clippy]
unwrap_used = "deny"     # architecture §8: no unwrap/expect in src/
expect_used = "deny"
```

```toml
# rust-toolchain.toml
[toolchain]
channel = "1.99.1"
components = ["rustfmt", "clippy"]
```

```toml
# clippy.toml — tests may unwrap (architecture §8)
allow-unwrap-in-tests = true
allow-expect-in-tests = true
```

```toml
# .config/nextest.toml — test profiles and JUnit reports under target/nextest/<profile>/junit.xml (ADR-07)
[profile.ci]
fail-fast = false
[profile.ci.junit]
path = "junit.xml"
report-name = "sloperator"

[profile.ci-perf]                  # NFR-3 benchmark, run with --cargo-profile release (§3)
fail-fast = false
test-threads = 1
[profile.ci-perf.junit]
path = "junit.xml"
report-name = "sloperator-perf"

[profile.it]                       # real model: one test at a time (memory)
fail-fast = false
test-threads = 1
slow-timeout = { period = "120s", terminate-after = 10 }
[profile.it.junit]
path = "junit.xml"
report-name = "sloperator-integration"
```

```toml
# deny.toml (NFR-7)
[licenses]
allow = ["MIT", "Apache-2.0", "Apache-2.0 WITH LLVM-exception", "BSD-2-Clause", "BSD-3-Clause", "ISC",
         "Unicode-3.0", "Zlib", "CDLA-Permissive-2.0"]
confidence-threshold = 0.9
[bans]
multiple-versions = "warn"
wildcards = "deny"
[advisories]
yanked = "deny"
[sources]
unknown-registry = "deny"
unknown-git = "deny"
```

`.gitignore` contains `/target`, which also covers the nextest JUnit reports under `target/nextest/` and the perf
build under `target/perf/`.

**README.md** sections (each one a deliverable the tests or reviewers check):
- Usage: `check`, `--fix`, `--diff`, input forms, exit codes, output formats, directives, config reference.
- Score formula (FR-5.1): the formula from §11, with the weights.
- Limits, stating FR-5.4 word for word: "The score measures style tells, not text quality or provenance.
  sloperator does not make text undetectable and makes no such claim." Also: verification covers numbers, URLs,
  configured terms, placeholders and block structure, not names or claim polarity (PRD Risk 1).
- Model setup: `sloperator model pull`, the cache location and the registry table. Weights are never committed or
  embedded (FR-9.10).
- Building: Phase 0 prerequisites, `cargo build --release --locked`, and `--no-default-features` for a rules-only
  binary.
- Licence: MIT.

**Design notes:**

- **No `path =` dependency, no workspace.** A copy of the repository builds on its own (NFR-2). §5's tests enforce it.
- **`pulldown-cmark` without default features** drops the `html` renderer and `getopts`. ADR-03 never re-renders
  Markdown.
- **`ureq` 3 defaults to rustls**, so HTTPS comes without OpenSSL. `CDLA-Permissive-2.0` is on the allow-list for the
  `webpki-roots` data that rustls pulls in. Phase 2 runs `cargo deny check licenses`. Any licence it reports that
  is not on the list is added only if it is permissive; otherwise the dependency is replaced. The list never grows
  to make the build pass.
- **If the spike picks `mistralrs`:** `model = ["dep:mistralrs", "dep:tokio"]` with `mistralrs = "=0.8.1"` and
  `tokio = { version = "…", features = ["rt"] }` pinned at that point; `metal`/`cuda` map to `mistralrs/metal`
  and `mistralrs/cuda` (features confirmed on crates.io for 0.8.1).

---

## 3. `Makefile` — build, test and check targets

**File:** `Makefile`, `.github/workflows/ci.yml`
**Component:** §4.15 module shell (ADR-07)
**Implements:** FR-1.2, FR-1.3, FR-1.4, FR-1.5, FR-1.6, FR-1.7, FR-1.9, D-11, NFR-1, NFR-3 (benchmark wiring)

The crate's own `Makefile` is the entry point for building, testing and checking (D-11). It defines the targets
`build`, `test`, `test-integration`, `check`, `clean` and `help` (FR-1.2); `help` is the default goal. Every
recipe runs from the repository root under `bash`, `.ONESHELL` and `-eu -o pipefail`. The test targets run
cargo-nextest with the profiles in `.config/nextest.toml` (§2), and each profile writes JUnit XML to
`target/nextest/<profile>/junit.xml` (ADR-07). CI (`.github/workflows/ci.yml`, macOS) runs `make check`,
`make build` and `make test`.

```make
################################################################################
### sloperator: build, test and check with cargo.
###
### make build | test | test-integration | check | clean | help
### TESTS='<selector> ...' narrows test and test-integration (see `make help`).
################################################################################

SHELL       := bash
.SHELLFLAGS := -eu -o pipefail -c
.ONESHELL:
.DEFAULT_GOAL := help
MAKEFLAGS   += --no-builtin-rules

# Cargo features for the integration tier, e.g. `make test-integration FEATURES=metal`.
FEATURES ?= model
# The NFR-3 perf pass builds in the release profile; its own target dir keeps it from replacing the model-enabled
# target/release/sloperator that `make build` produces with a --no-default-features binary.
PERF_TARGET_DIR := target/perf

RUSTUP_HINT  := install Rust: https://rustup.rs (toolchain pinned in rust-toolchain.toml)
NEXTEST_HINT := cargo install --locked cargo-nextest@0.9.150
DENY_HINT    := cargo install --locked cargo-deny@0.20.2

# TESTS -> one nextest filterset. `binary_id#test` selects one test exactly; a bare word is a substring of the binary
# id or test name. nextest rejects a `binary_id(~word)` that matches no binary id and fails the whole filterset, so
# the binary_id clause is emitted only for a word inside one of the binary ids (the lib, the bin and one
# `sloperator::<name>` per tests/*.rs).
EMPTY :=
SPACE := $(EMPTY) $(EMPTY)
HASH  := \#
SEL_TESTS := $(if $(filter undefined,$(origin TESTS)),,$(TESTS))
BINIDS    := sloperator sloperator::bin/sloperator $(patsubst tests/%.rs,sloperator::%,$(wildcard tests/*.rs))
sel_bare   = $(if $(findstring $(1),$(BINIDS)),binary_id(~$(1))|)test(~$(1))
sel_one    = $(if $(findstring $(HASH),$(1)),(binary_id(=$(firstword $(subst $(HASH), ,$(1))))&test(=$(lastword $(subst $(HASH), ,$(1))))),($(call sel_bare,$(1))))
SEL       := $(subst $(SPACE), or ,$(strip $(foreach t,$(SEL_TESTS),$(call sel_one,$(t)))))
nt_filter  = -E '$(1)$(if $(SEL), and ($(SEL)))'

# The release-profile perf pass runs only when no selector is given or a selector names `perf`: nextest builds every
# test binary before it filters, so a targeted run would otherwise pay for a full release build.
RUN_PERF := $(if $(SEL_TESTS),$(findstring perf,$(SEL_TESTS)),all)

# need <tool>,<install hint>: fail with a one-line banner when a tool is missing.
define need
command -v $(1) >/dev/null 2>&1 || { echo "sloperator: '$(1)' not found on PATH — $(2)" >&2; exit 1; }
endef

# count <junit.xml...>: fail when the reports hold no testcase, so a selector that matched nothing is not green.
define count
n=$$(cat $(1) 2>/dev/null | grep -c '<testcase' || true)
if [ "$$n" -eq 0 ]; then echo "sloperator: no test ran (TESTS='$(SEL_TESTS)')" >&2; exit 1; fi
endef

.PHONY: help build test test-integration check clean

help: ## list the targets
	@grep -E '^[a-z-]+:.*## ' $(firstword $(MAKEFILE_LIST)) | awk -F':.*## ' '{printf "  %-17s %s\n", $$1, $$2}'
	echo "  TESTS='<sel> ...'  narrow test / test-integration: 'binary_id#test' exact, a bare word is a substring"
	echo "  FEATURES=<list>    cargo features for test-integration (default: model)"

build: ## release build with default features (model)
	@$(call need,cargo,$(RUSTUP_HINT))
	cargo build --release --locked

test: ## unit tier (no model) plus the release-profile perf test
	@$(call need,cargo,$(RUSTUP_HINT))
	$(call need,cargo-nextest,$(NEXTEST_HINT))
	rm -f target/nextest/ci/junit.xml target/nextest/ci-perf/junit.xml
	rc=0
	cargo nextest run --locked --no-default-features --profile ci --no-tests=pass $(call nt_filter,not binary_id(=sloperator::perf)) || rc=$$?
	if [ -n '$(RUN_PERF)' ]; then
	  cargo nextest run --locked --no-default-features --cargo-profile release --target-dir $(PERF_TARGET_DIR) --profile ci-perf --no-tests=pass $(call nt_filter,binary_id(=sloperator::perf)) || rc=$$?
	else
	  echo "sloperator: perf pass skipped (TESTS names no perf test)" >&2
	fi
	$(call count,target/nextest/ci/junit.xml target/nextest/ci-perf/junit.xml)
	exit $$rc

test-integration: ## integration tier (real model; tests marked #[ignore = "integration tier"])
	@$(call need,cargo,$(RUSTUP_HINT))
	$(call need,cargo-nextest,$(NEXTEST_HINT))
	rm -f target/nextest/it/junit.xml
	rc=0
	cargo nextest run --locked --features $(FEATURES) --profile it --run-ignored only --no-tests=pass $(call nt_filter,all()) || rc=$$?
	$(call count,target/nextest/it/junit.xml)
	exit $$rc

check: ## fmt --check, clippy (no-default and default features, -D warnings), cargo deny check
	@$(call need,cargo,$(RUSTUP_HINT))
	$(call need,cargo-deny,$(DENY_HINT))
	cargo fmt --check
	cargo clippy --locked --all-targets --no-default-features -- -D warnings
	cargo clippy --locked --all-targets -- -D warnings
	cargo deny check

clean: ## remove target/
	rm -rf target
```

**How a test run works:** `make test TESTS='verify'`:
1. The guard checks that `cargo` and `cargo-nextest` are on `PATH`; a missing tool prints the one-line banner and
   the recipe exits 1, so `make` exits non-zero.
2. The previous `target/nextest/ci/junit.xml` and `target/nextest/ci-perf/junit.xml` are deleted, so a stale report
   cannot count.
3. The unit-tier run uses profile `ci` with `--no-default-features` and the filterset built from `TESTS`.
4. The perf pass (profile `ci-perf`, `--cargo-profile release`, `--target-dir target/perf`) runs only when `TESTS`
   is unset or names `perf`; here it is skipped with a banner on stderr.
5. `count` fails the target if the JUnit files hold no `<testcase`; otherwise the target exits with the first
   failing run's status.

`make test-integration` is the same shape with profile `it`, `--features $(FEATURES)` (default `model`) and
`--run-ignored only`, writing `target/nextest/it/junit.xml`.

**Design notes:**

- **Verified.** The selector, guard and perf-skip logic was run under GNU make with the Makefile's shell flags:
  - `TESTS='sloperator::cli_exit#exit_two_on_bad_config verify::tests'` produced
    `-E '(binary_id(=sloperator::cli_exit)&test(=exit_two_on_bad_config)) or (test(~verify::tests))'`
    (the bare word occurs in no binary id, so it gets only the `test(~…)` term).
  - With no `TESTS` the selector is empty and `$(origin TESTS)` is `undefined`, so no filter clause is added.
  - `RUN_PERF` was empty for `TESTS='verify'`, and non-empty for `TESTS='sloperator::perf#x'` and for no `TESTS`.
  - With `cargo` missing from `PATH`, `make test` printed the banner, its recipe exited 1 and make exited
    non-zero.
- **Selector form (architecture Risk R7).** nextest writes one `<testsuite>` per test binary and one `<testcase>`
  per test, with `classname` = the binary id (`sloperator` for the library's in-crate tests, `sloperator::cli_exit`
  for `tests/cli_exit.rs`) and `name` = the test path (nexte.st JUnit docs). A `binary_id#test` selector maps that
  id back to an exact filter, so failing ids round-trip into `TESTS`. A bare word is a substring of the test name,
  and also of the binary id when the word occurs in one of the binary ids (`BINIDS`): nextest rejects a
  `binary_id(~word)` that matches no binary and would fail the whole filterset. Story Gates name tests as
  `<binary_id>#<test name>` or as a bare substring.
- **`--no-tests=pass` plus `count`.** The unit tier is two nextest runs (main and perf), and a selector may match
  only one of them, so neither run may fail for having no tests. `count` then fails the target if *no* testcase
  was written at all. A mistyped selector exits non-zero instead of passing with zero tests (FR-1.5, fail loudly).
- **Targeted selectors skip the perf run (Gap 9).** nextest builds every test binary before it filters, so a
  release-profile run for a selector that cannot match `perf` would pay for a full release build for nothing. The
  skip banner goes to stderr, so a skipped pass is visible, not silent.
- **The perf pass has its own target dir.** `--target-dir target/perf` keeps the `--no-default-features` release
  build from replacing the model-enabled `target/release/sloperator` that `make build` produces.
- **JUnit is written even when tests fail.** `rc` keeps the failing status, so the report holds the failing
  testcases and the target still exits non-zero (FR-1.3).
- **Missing tools fail loudly (FR-1.9).** `need` prints `sloperator: '<tool>' not found on PATH — <install hint>`
  on stderr and the recipe exits 1, so `make` fails. No JUnit is written. The hints name rustup (toolchain pinned in `rust-toolchain.toml`),
  `cargo install --locked cargo-nextest@0.9.150` and `cargo install --locked cargo-deny@0.20.2`.
- **`check` runs clippy twice** (rules-only and default features) to keep both build configurations green
  (ADR-02 Consequences), then `cargo deny check` (NFR-7). The second clippy run needs `clang`/`cmake`
  (architecture R4).
- **`clean` needs no cargo.** It deletes `target/`, which holds the build output and every JUnit report.

---

## 4. Crate root and the error type

**File:** `src/lib.rs`, `src/error.rs` (new)
**Component:** §8 cross-cutting (errors); §4.1 cli (exit mapping)
**Implements:** NFR-4, FR-2.7, FR-2.15, FR-6.3, FR-6.5, FR-7.6, FR-8.8, FR-9.6, FR-9.7

```rust
// src/lib.rs
pub mod cli;
pub mod config;
pub mod directives;
pub mod doc;
pub mod edit;
pub mod engine;
pub mod error;
pub mod input;
pub mod model;
pub mod pipeline;
pub mod report;
pub mod rules;
pub mod score;
pub mod types;
pub mod verify;
pub mod write;

pub use error::Error;
```

```rust
// src/error.rs
use std::path::PathBuf;

/// Every failure that exits 2 (NFR-4). Rejected edits are findings (SLP901/SLP902), never errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{path}: {source}")]
    Io { path: PathBuf, #[source] source: std::io::Error },
    #[error("{path}: not valid UTF-8 (byte {offset}); file left unchanged")]
    NotUtf8 { path: PathBuf, offset: usize },
    #[error("{file}: config key `{key}`: {reason}")]
    Config { file: PathBuf, key: String, reason: String },
    #[error("unknown rule code `{code}`{}", where_.as_ref().map(|w| format!(" in {w}")).unwrap_or_default())]
    UnknownRule { code: String, where_: Option<String> },
    #[error("{file}:{line}: {reason}")]
    Directive { file: PathBuf, line: u32, reason: String },
    #[error("usage: {0}")]
    Usage(String),
    #[error("model `{name}` is not in the cache ({path}); run `sloperator model pull {name}` or pass --rules-only")]
    ModelMissing { name: String, path: PathBuf },
    #[error("this binary was built without model support; rebuild with the `model` feature or pass --rules-only")]
    ModelUnsupported,
    #[error("loading model {path}: {reason}")]
    ModelLoad { path: PathBuf, reason: String },
    #[error("model runtime: {0}")]
    Runtime(String),
    #[error("download {url}: {reason}")]
    Download { url: String, reason: String },
}
```

**Design notes:**

- The 11 variants are exactly the ones architecture §8 lists. The fields are the "file, line or key" that NFR-4
  requires in each message.
- There is no `From<io::Error>` impl. Every I/O error must name its path, so call sites build `Error::Io { path, .. }`
  themselves. That keeps `?` from producing a pathless error.

---

## 5. Layering and standalone tests

**File:** `tests/layering.rs`, `tests/standalone.rs` (new)
**Component:** §5 module rules; §4.15 module shell (ADR-01)
**Implements:** FR-1.8, NFR-2, FR-9.1 (network only in `model::pull`)

These are text scans over the crate, using `std::fs` (tests may do I/O). Each test holds its table as a `const`
slice, so adding a module means adding a row.

```rust
// tests/layering.rs — rows derived from architecture §5
const FORBIDDEN_USES: &[(&str, &[&str])] = &[
    ("src/doc/",        &["crate::"]),
    ("src/types.rs",    &["crate::"]),   // leaf: RuleCode, Proposal, ProposalOrigin (Gap 3)
    ("src/rules/",      &["crate::engine", "crate::verify", "crate::edit", "crate::write", "crate::report"]),
    ("src/score/",      &["crate::engine", "crate::write", "crate::report", "crate::input"]),
    ("src/edit/",       &["crate::engine", "crate::rules", "crate::input", "crate::write"]),
    ("src/verify/",     &["crate::engine", "crate::write", "crate::report"]),
    ("src/config/",     &["crate::pipeline", "crate::engine", "crate::report", "crate::write"]),
    ("src/input/",      &["crate::rules", "crate::engine", "crate::write"]),
    ("src/directives/", &["crate::engine", "crate::edit"]),
    ("src/model/",      &["crate::pipeline", "crate::rules", "crate::engine"]),
    ("src/engine/",     &["crate::write", "crate::report", "crate::cli"]),
    ("src/pipeline/",   &["crate::cli", "crate::report", "crate::write", "crate::engine::runtime"]),
    ("src/report/",     &["crate::engine", "crate::write", "crate::config::Resolver"]),
    ("src/write/",      &["crate::engine", "crate::rules"]),
    ("src/cli/",        &["crate::engine::runtime", "llama_cpp_2", "mistralrs"]),
];
const PURE_DIRS: &[&str] = &["src/doc/", "src/types.rs", "src/rules/", "src/score/", "src/edit/", "src/verify/"];
const IO_PATHS: &[&str] = &["std::fs", "std::net", "std::process", "std::env"];
```

| Test | Asserts |
|------|---------|
| `forbidden_uses` | No file under a row's directory contains a forbidden path. (`doc` may use only `crate::error`; that one exception is coded in.) A row may name one file (`src/types.rs`). The shared value types live there, so `rules` and `edit` never import each other (Gap 3). |
| `pure_modules_have_no_io` | No `IO_PATHS` entry appears under `PURE_DIRS`. |
| `runtime_crate_only_in_adapter` | `llama_cpp_2` and `mistralrs` appear only in `src/engine/runtime.rs`. |
| `network_only_in_pull` | `ureq` appears only in `src/model/pull.rs`. |
| `single_exit_point` | `std::process::exit` appears only in `src/main.rs`. |

```rust
// tests/standalone.rs
/// The scan itself, so the planted-reference test exercises the same code as `no_host_references`.
/// Returns (path relative to `root`, first needle found), matching case-insensitively; non-UTF-8 files are skipped.
fn find_markers(root: &std::path::Path) -> Vec<(std::path::PathBuf, &'static str)>;
```

| Test | Asserts |
|------|---------|
| `no_host_references` | `find_markers` over the whole repository except `.git/` and `target/` (and *including* `tests/standalone.rs`) returns nothing. The needles are the name of the project the crate was developed in and that project's build-contract path patterns. They are assembled from fragments at run time, so the test file never spells them and does not match itself. Matching is case-insensitive, so no identifier, comment or string in the file contains the name in any case: the constant is `HOST_MARKERS` and the test `no_host_references`. FR-1.8. |
| `planted_reference_is_caught` | Copies `src/lib.rs` and `tests/standalone.rs` into a `tempfile::TempDir`, appends a comment naming the host project (built at run time from two halves) to the copied `lib.rs`, and asserts `find_markers` reports exactly that file. The unmodified `standalone.rs` copy reports nothing, so the fragment form cannot hide a real reference (Gap 4). |
| `no_external_path_deps` | Parse `Cargo.toml` with `toml`: no dependency table (including `target.<cfg>` tables) has a `path` key. FR-1.8. |
| `builds_when_extracted` | `#[ignore = "integration tier"]`. Copies the crate without `.git/` and `target/` to a `tempfile::TempDir`, then runs `cargo test --no-default-features --locked --offline` there and asserts exit 0. NFR-2. |

**Design notes:**

- **The host project's name is banned outright.** One case-insensitive rule catches crate names, paths and package
  names. The docs are scanned too, so design records name the crate's own files only.
- **`builds_when_extracted` runs in the integration tier.** It is a cold build, so it is too slow for the unit tier
  (architecture §5). `--offline` works because the registry cache is already warm from the unit tier.

---

## 6. `doc` — parse and span map

**File:** `src/doc/mod.rs`, `src/doc/markdown.rs`, `src/doc/plain.rs`, `src/doc/protect.rs` (new)
**Component:** §4.4 doc (ADR-03)
**Implements:** FR-8.1, FR-8.2, FR-8.3, FR-4.4, FR-6.7 (directive lines protected), D-7, D-10

`doc` is a pure function of text and settings. Apart from `crate::error` it uses no other crate module (§5). So the
input types it needs, `Format` and `ProtectSettings`, are defined here; `input` and `config` re-export them.
That is an Architecture Deviation Note, because the architecture put `Format` in `input`.

```rust
// src/doc/mod.rs
use std::ops::Range;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Format { Markdown, PlainText }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Newline { Lf, Crlf }

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockKind {
    Paragraph,
    Heading(u8),                          // 1..=6
    ListItem { depth: u8, ordered: bool },
    TableCell { row: u32, col: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block { pub id: BlockId, pub kind: BlockKind, pub quote_depth: u8, pub range: Range<usize> }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtectReason { CodeBlock, InlineCode, LinkDest, LinkTitle, Image, Autolink, BareUrl, RefDef,
    HtmlBlock, Html, FrontMatter, Math, Markup, UserPattern, UserTerm, Directive }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpanKind { Prose, Protected(ProtectReason) }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span { pub range: Range<usize>, pub kind: SpanKind, pub block: Option<BlockId> }

/// [protect] in sloperator.toml (FR-7.4). Defaults: patterns/terms empty; front_matter, math, html all true.
#[derive(Clone, Debug, serde::Serialize)]
pub struct ProtectSettings {
    pub patterns: Vec<String>,   // regex sources; compiled by config into `compiled` (invalid regex -> Error::Config)
    pub terms: Vec<String>,      // literal terms that must survive unchanged
    pub front_matter: bool,
    pub math: bool,
    pub html: bool,
    #[serde(skip)]
    pub compiled: Option<Arc<regex::RegexSet>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockSig { pub kind: BlockKind, pub quote_depth: u8 }

#[derive(Clone, Debug)]
pub struct Document {
    pub src: Arc<str>,
    pub format: Format,
    pub spans: Vec<Span>,        // ordered, non-overlapping, covering 0..src.len()
    pub blocks: Vec<Block>,      // ordered by range.start
    pub newline: Newline,
    line_starts: Vec<usize>,
}

impl Document {
    pub fn block_text(&self, id: BlockId) -> BlockText;            // prose of one block, see below
    pub fn prose_words(&self) -> usize;                            // unicode_words() count over prose spans
    pub fn line_col(&self, offset: usize) -> (u32, u32);           // 1-based line; 1-based column in chars
    pub fn line_range(&self, line: u32) -> Range<usize>;
    pub fn is_protected(&self, range: &Range<usize>) -> bool;      // overlaps any Protected span
    pub fn protected_strings(&self) -> Vec<&str>;                  // sorted multiset (§16); a block's continuation-prefix
                                                                   // spans count once per block, not per line (Gap 7)
}

/// The prose of one block. Each non-prose run inside it becomes one U+FFFC, so patterns never match across
/// code or links. `src_range` maps a text range back to source bytes; None if the range covers an OBJ.
pub struct BlockText { pub block: BlockId, pub text: String, map: Vec<(usize, usize, usize)> /* (text_off, src_off, len) */ }
impl BlockText {
    pub fn to_src(&self, text_off: usize) -> Option<usize>;
    pub fn src_range(&self, r: Range<usize>) -> Option<Range<usize>>;
}

pub const OBJ: char = '\u{FFFC}';

pub fn parse(src: Arc<str>, format: Format, protect: &ProtectSettings) -> Document {
    match format { Format::Markdown => markdown::parse(src, protect), Format::PlainText => plain::parse(src, protect) }
}
pub fn skeleton(doc: &Document) -> Vec<BlockSig> {
    doc.blocks.iter().map(|b| BlockSig { kind: b.kind, quote_depth: b.quote_depth }).collect()
}
```

```rust
// src/doc/markdown.rs — pulldown-cmark 0.13.6 (Parser::new_ext, into_offset_iter, Options::* checked on docs.rs)
use pulldown_cmark::{Options, Parser};

pub(super) fn options(protect: &ProtectSettings) -> Options {
    let mut o = Options::ENABLE_TABLES | Options::ENABLE_FOOTNOTES | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS;
    if protect.front_matter {
        o |= Options::ENABLE_YAML_STYLE_METADATA_BLOCKS | Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS;
    }
    if protect.math { o |= Options::ENABLE_MATH; }
    o
}
```

The event walk over `Parser::new_ext(&src, options(protect)).into_offset_iter()` gives `(Event, Range<usize>)`
pairs. It classifies each byte as follows:

| Event / region | Span |
|----------------|------|
| `Start(Paragraph / Heading / Item / TableCell)` … `End` | opens and closes a `Block`. List depth and `ordered` come from the `List` stack; `quote_depth` from the `BlockQuote` stack |
| `Text` inside an open block | `Prose` |
| `SoftBreak` | `Prose` (whitespace; §16 keeps line breaks) |
| `Code` | `Protected(InlineCode)` |
| `Start(CodeBlock)` … `End` | `Protected(CodeBlock)`, the whole range |
| `Start(Link)`: bytes after the link text (`](dest "title")`, or the reference label) | `Protected(LinkDest)`. The link text stays prose |
| `Start(Link)` with `LinkType::Autolink` / `Email` | `Protected(Autolink)`, the whole range |
| `Start(Image)` | `Protected(Image)`, the whole range, alt text included |
| `Html`, `InlineHtml` | `Protected(HtmlBlock / Html)`. Always protected, even with `html = false`: the toggle only lets *rules* see HTML block text, and edits there stay forbidden (DQ-8) |
| `Start(MetadataBlock)` | `Protected(FrontMatter)` |
| `InlineMath`, `DisplayMath` | `Protected(Math)` |
| other bytes inside a block (emphasis markers, list markers, `#`, `\|`, `>`) | `Protected(Markup)` |
| bytes outside every block (blank lines, fences, rules, reference definitions) | `Protected(Markup)` / `Protected(RefDef)` |

Then `protect::split` carves `BareUrl`, `UserPattern` and `UserTerm` out of prose spans.

```rust
// src/doc/protect.rs
/// Bare URLs; trailing `.,;:!?)` trimmed after matching.
pub const BARE_URL: &str = r"(?:https?://|www\.)[^\s<>\x{FFFC}]+";
pub(super) fn split(spans: Vec<Span>, src: &str, protect: &ProtectSettings) -> Vec<Span>;
```

**Plain text** (`plain.rs`): each run of non-blank lines is one `Paragraph` block. Every byte is prose except
bare URLs, user patterns, user terms and lines matching `directives::TXT_DIRECTIVE` (FR-6.7), which become
`Protected(Directive)`.

**Design notes:**

- **The span map covers every byte.** A byte is either prose or protected. So "engines see only prose" (FR-8.1)
  comes down to "proposals stay inside prose", which §8's `Protected` check enforces. Markup bytes are protected,
  so no edit can change a heading level, list marker or table pipe (FR-8.2).
- **Heading, list-item, table-cell and block-quote text are prose.** They arrive as `Text` events inside those
  blocks (FR-8.2).
- **Markdown directive comments** are already `Html` and therefore protected. §12 re-tags them as `Directive`.
- **`newline`** is `Crlf` when the first line ending is `\r\n`.
- The event walk is described as a table rather than written as code. That choice is deliberate: the
  `doc_spans.rs` tests (§23) pin its behaviour byte for byte against `fixtures/protect/*`.

---

## 7. `edit` — proposals, plan, apply, diff

**File:** `src/edit/mod.rs` (new)
**Component:** §4.8 edit (ADR-03, ADR-05)
**Implements:** FR-8.4, FR-8.9, FR-6.6, NFR-5 (content side)

```rust
use std::path::Path;
use crate::doc::Newline;
use crate::verify::Verified;
pub use crate::types::{Proposal, ProposalOrigin};   // defined in the leaf `types` module (§5, Gap 3)

/// Conflict-free and sorted by start. Overlaps are dropped deterministically: earlier start wins, then lower origin.
pub struct Plan { pub kept: Vec<Proposal>, pub dropped: Vec<Proposal> }

pub fn plan(mut props: Vec<Proposal>, frozen: &[Range<usize>]) -> Plan {
    props.sort_by(|a, b| (a.range.start, a.range.end, a.origin).cmp(&(b.range.start, b.range.end, b.origin)));
    let mut kept: Vec<Proposal> = Vec::new();
    let mut dropped = Vec::new();
    for p in props {
        let hits_frozen = frozen.iter().any(|f| f.start < p.range.end && p.range.start < f.end);
        let overlaps = kept.last().is_some_and(|k| p.range.start < k.range.end);
        if hits_frozen || overlaps { dropped.push(p) } else { kept.push(p) }
    }
    Plan { kept, dropped }
}

/// Splices verified edits (ascending, non-overlapping) into the original bytes. Every other byte is copied (ADR-03).
/// Replacement newlines are normalised to the document's style, so CRLF files stay CRLF (FR-8.4).
pub fn apply(src: &str, edits: &[Verified<Proposal>], nl: Newline) -> String {
    let mut out = String::with_capacity(src.len());
    let mut at = 0;
    for e in edits {
        let p = e.get();
        debug_assert!(p.range.start >= at);
        out.push_str(&src[at..p.range.start]);
        match nl {
            Newline::Lf => out.push_str(&p.replacement),
            Newline::Crlf => out.push_str(&p.replacement.replace("\r\n", "\n").replace('\n', "\r\n")),
        }
        at = p.range.end;
    }
    out.push_str(&src[at..]);
    out
}

/// `--diff` output (similar 3.2.1: TextDiff::from_lines(..).unified_diff(), checked on docs.rs).
pub fn unified_diff(path: &Path, before: &str, after: &str) -> String {
    let name = path.display().to_string();
    similar::TextDiff::from_lines(before, after).unified_diff().context_radius(3)
        .header(&format!("a/{name}"), &format!("b/{name}")).to_string()
}
```

**Design notes:**

- **Trailing newline.** The final newline byte is in a protected run (§6), so no edit covers it. FR-8.4 holds by
  construction.
- **Dropped proposals are not lost.** Their findings remain in the report. Only the fix is skipped, and the next
  `--fix` run picks it up.
- **FR-8.9** is a property of the safe fixes: no fixed text matches its own rule. §23
  `fix_is_idempotent_over_all_fixtures` asserts it.

---

## 8. `verify` — the edit gate

**File:** `src/verify/mod.rs`, `src/verify/tokens.rs` (new)
**Component:** §4.9 verify (ADR-05)
**Implements:** FR-8.5, FR-8.6, FR-8.7, FR-9.5, D-4, D-7

```rust
use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::Arc;
use crate::doc::{self, Document, ProtectSettings};
use crate::types::Proposal;

/// Only this module can construct one. The tuple field and `new` are private (ADR-05).
#[derive(Debug)]
pub struct Verified<T>(T);
impl<T> Verified<T> {
    fn new(t: T) -> Self { Verified(t) }
    pub fn get(&self) -> &T { &self.0 }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Tokens { pub numbers: BTreeMap<String, u32>, pub urls: BTreeMap<String, u32>, pub terms: BTreeMap<String, u32> }

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Check { Numbers, Urls, Terms, Placeholders, Protected, BlockType, NewBlockMarker, OutputContract,
    NoImprovement, Document }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rejection { pub range: Range<usize>, pub failed: Check, pub detail: String }

/// numbers: `\d+(?:[.,]\d+)*%?` and digit dates `\d{1,4}[-/.]\d{1,2}[-/.]\d{1,4}` (dates matched first);
/// urls: doc::protect::BARE_URL; terms: each configured term, non-overlapping occurrences.
pub fn tokens(text: &str, protect: &ProtectSettings) -> Tokens;

/// Per-edit verification (FR-8.5, FR-8.6). `frozen` = directives' suppressed ranges (FR-6.6).
pub fn edit(doc: &Document, p: Proposal, frozen: &[Range<usize>], protect: &ProtectSettings)
    -> Result<Verified<Proposal>, Rejection> {
    let r = p.range.clone();
    let reject = |failed, detail: String| Rejection { range: r.clone(), failed, detail };
    if frozen.iter().any(|f| f.start < r.end && r.start < f.end) || doc.is_protected(&r) {
        return Err(reject(Check::Protected, "edit touches a protected or suppressed region".into()));
    }
    let before = tokens(&doc.src[r.clone()], protect);
    let after = tokens(&p.replacement, protect);
    if before.numbers != after.numbers { return Err(reject(Check::Numbers, diff(&before.numbers, &after.numbers))); }
    if before.urls != after.urls { return Err(reject(Check::Urls, diff(&before.urls, &after.urls))); }
    if before.terms != after.terms { return Err(reject(Check::Terms, diff(&before.terms, &after.terms))); }
    let candidate: Arc<str> = format!("{}{}{}", &doc.src[..r.start], p.replacement, &doc.src[r.end..]).into();
    let next = doc::parse(candidate, doc.format, protect);
    let (a, b) = (doc::skeleton(doc), doc::skeleton(&next));
    if a.len() != b.len() {
        return Err(reject(Check::NewBlockMarker, format!("{} blocks became {}", a.len(), b.len())));
    }
    if a != b { return Err(reject(Check::BlockType, "a block changed type or level".into())); }
    if doc.protected_strings() != next.protected_strings() {
        return Err(reject(Check::Protected, "protected bytes changed".into()));
    }
    Ok(Verified::new(p))
}

/// FR-9.5 as resolved in DQ-3: strictly fewer weighted findings in the window after the rewrite.
pub fn improvement(before: u32, after: u32, range: Range<usize>) -> Result<(), Rejection> {
    if after < before { Ok(()) } else {
        Err(Rejection { range, failed: Check::NoImprovement, detail: format!("weighted findings {before} -> {after}") })
    }
}

/// Whole-file defence (architecture §7.2): protected strings and skeleton equal before and after.
pub fn document(before: &Document, after: &Document) -> Result<(), Rejection>;

fn diff(a: &BTreeMap<String, u32>, b: &BTreeMap<String, u32>) -> String; // e.g. "lost 42 ×1; added 43 ×1"
```

**Design notes:**

- **`Verified` lives in `verify`, not `edit`.** The architecture puts it in `edit` with a constructor visible only
  `pub(in crate::verify)`. Rust rejects that: a `pub(in path)` path must be an ancestor of the item's own module
  (E0742), and `crate::verify` is not an ancestor of `crate::edit`. With the type in `verify` and the constructor
  private, the guarantee is the same. See Architecture Deviation Notes.
- **Two `Check` variants are added to the architecture's enum.** `Protected` covers edits into protected or frozen
  bytes, which §6 extends to markup bytes. `Document` names the whole-file check.
- **A rejection is a finding, not an error.** `pipeline` turns each `Rejection` into SLP901 with message
  `rewrite rejected (<check>): <detail>`; DQ-4 fixes its severity and exit behaviour. The original text stays.
- **Cost.** Each proposal costs one `pulldown-cmark` re-parse, and only in `--fix`/`--diff`. Lint never calls
  `verify` (NFR-3).

---

## 9. `rules` — core types and registry

**File:** `src/rules/mod.rs`, `src/rules/registry.rs`, `src/rules/helpers.rs` (new)
**Component:** §4.6 rules (ADR-06)
**Implements:** FR-4.1, FR-4.4, FR-4.5, FR-4.7, FR-2.12 (metadata), D-4, D-8

```rust
// src/types.rs — the leaf module for value types that `rules`, `edit`, `verify` and `directives` all share (Gap 3).
// It uses no `crate::` module, like `doc`, so every row of the §5 layering table may depend on it.
use std::fmt;
use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuleCode(pub u16);
impl RuleCode {
    pub const REWRITE_REJECTED: RuleCode = RuleCode(901);
    pub const WINDOW_SKIPPED: RuleCode = RuleCode(902);
    pub fn is_system(self) -> bool { self.0 >= 900 }
}
impl fmt::Display for RuleCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "SLP{:03}", self.0) }
}
impl std::str::FromStr for RuleCode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        s.strip_prefix("SLP").filter(|d| d.len() == 3 && d.bytes().all(|b| b.is_ascii_digit()))
            .and_then(|d| d.parse().ok()).map(RuleCode).ok_or_else(|| format!("`{s}` is not a rule code (SLP###)"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProposalOrigin { Rule(RuleCode), Model }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proposal { pub range: Range<usize>, pub replacement: String, pub origin: ProposalOrigin }
```

```rust
// src/rules/mod.rs
use std::collections::BTreeMap;
use std::ops::Range;
use crate::doc::{BlockText, Document};
pub use crate::types::{Proposal, RuleCode};   // re-exported so callers keep writing `rules::RuleCode` (Gap 3)

pub mod builtin;
pub mod helpers;
pub mod registry;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Severity { Suggestion, Warning, Error }

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FixClass { Safe, Model, None }

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Family { AiVocabulary, NegativeParallelism, RuleOfThree, CopulaAvoidance, SignificancePuffery, Hedging,
    Markup, ChatbotResidue, EmDash }

/// One tunable threshold (e.g. SLP014 `max_per_1000_words`); config validates names and types against these (FR-7.6).
#[derive(Clone, Copy, Debug)]
pub struct ParamSpec { pub name: &'static str, pub default: f64, pub help: &'static str }

pub struct RuleMeta {
    pub code: RuleCode, pub name: &'static str, pub family: Family, pub severity: Severity, pub default_on: bool,
    pub fix: FixClass, pub explain: &'static str, pub example: (&'static str, &'static str),
    pub params: &'static [ParamSpec], pub since: &'static str,
}

/// What a rule sees: prose only (FR-4.4), its resolved params, the prose word count (FR-4.3).
pub struct RuleCtx<'a> { pub doc: &'a Document, pub params: &'a BTreeMap<&'static str, f64>, pub severity: Severity }
impl RuleCtx<'_> {
    pub fn blocks(&self) -> impl Iterator<Item = BlockText> + '_ { self.doc.blocks.iter().map(|b| self.doc.block_text(b.id)) }
    pub fn prose_words(&self) -> usize { self.doc.prose_words() }
    pub fn param(&self, name: &str) -> f64 { self.params.get(name).copied().unwrap_or(f64::NAN) }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Finding { pub code: RuleCode, pub range: Range<usize>, pub message: String, pub severity: Severity,
    pub fix: Option<Proposal> }

pub trait Rule: Send + Sync {
    fn meta(&self) -> &RuleMeta;
    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>);
}

/// The rule-set version (architecture §6): the crate version at which the default list last changed.
pub const RULE_SET_VERSION: &str = "0.1.0";

/// A rule with its resolved severity and params for one file (built by config::Settings::enabled, §13).
pub struct Enabled<'a> { pub rule: &'a dyn Rule, pub severity: Severity, pub params: BTreeMap<&'static str, f64> }

/// Run every enabled rule, drop suppressed findings, sort by (start, code): source order (FR-3.6).
pub fn detect(doc: &Document, sup: &crate::directives::Suppressions, rules: &[Enabled<'_>]) -> Vec<Finding>;
```

```rust
// src/rules/registry.rs
use std::collections::BTreeSet;
use std::sync::OnceLock;
use super::{Rule, RuleCode, RULE_SET_VERSION};

pub struct Registry { rules: Vec<Box<dyn Rule>>, pub version: &'static str }
impl Registry {
    pub fn builtin() -> &'static Registry {
        static R: OnceLock<Registry> = OnceLock::new();
        R.get_or_init(|| Registry { rules: super::builtin::all(), version: RULE_SET_VERSION })
    }
    pub fn all(&self) -> impl Iterator<Item = &dyn Rule> { self.rules.iter().map(|r| r.as_ref()) }
    pub fn get(&self, code: RuleCode) -> Option<&dyn Rule> { self.all().find(|r| r.meta().code == code) }
    pub fn default_set(&self) -> BTreeSet<RuleCode> {
        self.all().filter(|r| r.meta().default_on).map(|r| r.meta().code).collect()
    }
}
```

```rust
// src/rules/helpers.rs — shared detectors (ADR-06). One aho-corasick automaton per word-list rule (NFR-3).
pub struct WordList { ac: aho_corasick::AhoCorasick, entries: &'static [(&'static str, Option<&'static str>)] }
impl WordList {
    /// entries: (phrase, safe replacement). Built with AhoCorasickBuilder::ascii_case_insensitive(true) and
    /// match_kind(MatchKind::LeftmostLongest) (aho-corasick 1.1.5, checked on docs.rs).
    pub fn new(entries: &'static [(&'static str, Option<&'static str>)]) -> WordList;
    /// Whole-word matches in one block as source ranges, with a case-preserving replacement when Some.
    pub fn find(&self, bt: &crate::doc::BlockText) -> Vec<(std::ops::Range<usize>, Option<String>)>;
}
pub struct PatternRule { pub set: regex::RegexSet, pub regexes: Vec<regex::Regex> }
```

**Design notes:**

- **System codes SLP9xx are not rules.** They are not in the registry. `select`, `ignore` and directives reject
  them as unknown codes (exit 2). They never count toward FR-4.2's 15 rules.
- **FR-4.5 is enforced by tests.** `rules_registry::default_set_is_explicit` compares `default_set()` with a
  literal list, so a change to the defaults shows up as a diff. `new_rules_start_off` asserts that every rule
  whose `since` is newer than `"0.1.0"` has `default_on == false`.
- **FR-4.7.** Only a rule whose class is `Safe` attaches `Finding::fix`. Model and none rules never do, and
  `rules_fixtures` asserts this per rule.

---

## 10. Built-in rule catalogue

**File:** `src/rules/builtin/mod.rs` (`pub fn all() -> Vec<Box<dyn Rule>>`), one file per rule
`src/rules/builtin/slpNNN_<name>.rs`, fixtures `fixtures/rules/SLPNNN/{positive.md,negative.md[,fixed.md]}` (new)
**Component:** §4.6 rules (ADR-06); architecture §6 fixtures
**Implements:** FR-4.2, FR-4.3, FR-4.6, FR-4.7, D-4

The catalogue has 17 rules across all nine families (FR-4.2). Every rule is in the default set for v0.1.0, and
`since = "0.1.0"`.

| Code | Name | Family | Severity | Fix | Detects | Safe fix |
|------|------|--------|----------|-----|---------|----------|
| SLP001 | `ai-vocabulary-replaceable` | AiVocabulary | warning | safe | "delve(s/d) into" / "delving into", "utilize(s/d)" / "utilizing", "in order to", "facilitate(s/d)" / "facilitating"; "leverages", "leveraged", "leveraging" (verb-only inflections). Bare "leverage" is reported with **no fix** (noun/verb ambiguous, Gap 2) | inflection-matched: "look(s/ed) at" / "looking at", "use(s/d)" / "using", "to", "help(s/ed)" / "helping", "uses" / "used" / "using" (case kept) |
| SLP002 | `ai-vocabulary` | AiVocabulary | suggestion | model | "tapestry", "testament", "multifaceted", "pivotal", "realm", "seamless", "robust" | — |
| SLP003 | `not-x-but-y` | NegativeParallelism | warning | model | "it's not X, it's Y"; "this isn't X — it's Y", in one sentence or two adjacent sentences | — |
| SLP004 | `not-just-but-also` | NegativeParallelism | warning | model | "not just X, but (also) Y"; "not only X but also Y" | — |
| SLP005 | `rule-of-three` | RuleOfThree | suggestion | model | triads "A, B(,) and C" of items of 1-3 words, above `max_per_1000_words` (default 4) | — |
| SLP006 | `copula-avoidance` | CopulaAvoidance | warning | safe | "serve/stand/function as" in every inflection | Only where the part of speech and the agreement are both certain from a fixed guard on the preceding token: "serves/stands/functions as" → "is" **only** when the token immediately before it, in the same sentence and matched case-insensitively, is one of `it`, `this`, `that`, `which`, `who`, `he`, `she`, **and** the token immediately after "as" is in `AS_COMPLEMENT` (Gap 16, design note). Each of the three words is also a plural noun ("export these functions as a module", "the market stands as …"), so after any other token the rule fires with **no fix** (Gap 14). A base form after a modal or "to" (`can`, `could`, `may`, `might`, `must`, `shall`, `should`, `will`, `would`, `to`, optionally with one adverb between) → "be", again **only** when the token after "as" is in `AS_COMPLEMENT`. "as" also opens fixed phrases ("as follows", "as well as", "as long as", "as such") where the verb is no copula stand-in, so any other following token fires with **no fix** (Gap 16). Every other form ("served as", "serving as", a base form after a plural subject) is reported with **no fix**, so the model engine takes it (Gap 2) |
| SLP007 | `significance-puffery` | SignificancePuffery | warning | model | "plays a crucial/pivotal/vital role", "underscores the importance", "a testament to", "marks a significant milestone" | — |
| SLP008 | `hedging-preamble` | Hedging | warning | safe | sentence-initial "It's important to note that", "It is worth noting that", "It should be noted that" | delete it and capitalise the next letter |
| SLP009 | `stacked-hedge` | Hedging | suggestion | safe | "may potentially", "could possibly", "might perhaps" | keep the first word |
| SLP010 | `bullet-spam` | Markup | suggestion | none | a list of at least `min_items` (5) items in which at least 80% open with a bold lead-in or have no more than `max_words` (6) words | — |
| SLP011 | `over-bolding` | Markup | suggestion | none | more than `max_per_100_words` (2) strong spans per 100 prose words in one block | — |
| SLP012 | `chatbot-closer` | ChatbotResidue | error | safe | the whole sentence "I hope this helps!", "Let me know if you have any (other) questions.", or "Feel free to …" | delete the sentence and one adjoining space |
| SLP013 | `chatbot-opener` | ChatbotResidue | error | safe | block-initial "Certainly!", "Great question!", "Absolutely!", or "Sure!" | delete it and the next space, then capitalise |
| SLP014 | `em-dash-density` | EmDash | warning | safe | em dashes (U+2014, or ` -- ` in prose) above `max_per_1000_words` (3) per 1,000 prose words in the file; once over the threshold, one finding per dash | a dash between words → ", " |
| SLP015 | `ai-disclaimer` | ChatbotResidue | error | model | "As of my last (knowledge) update", "As an AI language model" | — |
| SLP016 | `summary-signpost` | SignificancePuffery | suggestion | safe | "In conclusion,", "In summary,", "Overall,", or "Ultimately," at the start of a block's last sentence | delete it and capitalise |
| SLP017 | `decorative-emoji` | Markup | warning | safe | pictographic emoji in prose, one finding per grapheme cluster (ZWJ sequence, skin tone, flag, keycap): supplementary-plane `Extended_Pictographic`, BMP `Emoji_Presentation` (✅, ⏳), other BMP pictographs (©, ®, ™) only with U+FE0F, digits/`#`/`*` only as keycaps | delete the emoji (a space-separated cluster at once) and one adjacent space: the one after, or the one before at a line end or before closing punctuation; no fix when it is the only prose on its line (bare list item, heading) or sits between letters or after opening markup |

**Fixture contract (FR-4.6).** `tests/rules_fixtures.rs` uses a macro, `rule_fixtures!(1, 2, …, 17)`. For each
code it generates `slpNNN_positive_fires`, `slpNNN_negative_silent` and, for safe rules only,
`slpNNN_fix_matches`. A further test, `every_rule_has_fixtures`, walks `Registry::builtin()` and fails if any of
these holds:

- a code has no fixture directory;
- `positive.md` or `negative.md` is missing;
- a safe rule has no `fixed.md`;
- a non-safe rule has a `fixed.md`.

So `cargo test` fails for any rule added without fixtures. Each `negative.md` also contains the rule's trigger
text inside inline code, a fenced block and a link destination, which proves FR-4.4.

**Design notes:**

- **SLP010 and SLP011 are detection-only. The model does not fix them either.** Fixing bullet spam means
  turning a list into prose or merging items. Either way the list structure changes, which FR-8.2 and FR-8.6
  forbid, so verify would reject every such rewrite. The honest class is `none` (PRD Deviation Notes: D-4).
- **SLP014 is a density rule (FR-4.3).** Below the threshold it reports nothing. Above it, the safe fixes bring the
  file back under. Configure it with `[rules.SLP014] max_per_1000_words`.
- **SLP017 never leaves empty markup.** Deleting an emoji that is a list item's or heading's whole body would leave
  an empty bullet or heading, so such an emoji, and one alone on its line, is flagged with no fix. ©, ® and ™ are
  `Extended_Pictographic` but read as plain symbols in prose, so they count only when U+FE0F asks for emoji
  presentation.
- **The safe-fix rule (Gap 2).** A finding carries a fix only if the replacement is grammatical and keeps meaning
  in *every* context the matcher can see. So a safe fix may never depend on part of speech or subject agreement that
  the matcher cannot observe. Concretely: (1) verb entries are listed per inflection, each with its own replacement,
  never as a lemma; (2) a form that is also a noun ("leverage", "use") or whose agreement depends on the subject
  ("served as" → was/were, bare "serve as" → are/am) gets `None` from `WordList` and is left to the model; (3) a
  context guard is a fixed word list checked on the preceding token, or on the following token, in the same
  `BlockText`, never a heuristic. Where a guard protects against an open set of fixed phrases, it is an **allow**
  list (the safe contexts), never a deny list of the unsafe ones, because a deny list can miss a phrase.
  `WordList` entries are `(phrase, Option<&str>)`, so (2) is expressed in the table itself, not in code. Each rule's
  `negative.md` (no fix) and `fixed.md` hold the failing contexts:
  - SLP001 `positive.md`: "we have leverage over suppliers" fires with no fix and is unchanged in `fixed.md`;
    "she leveraged the API" → "she used the API".
  - SLP006 `fixed.md`: "These files can serve as the templates" → "These files can be the templates"; "designed to
    serve as a guide" → "designed to be a guide"; "It serves as a bridge" → "It is a bridge"; "a cache, which
    functions as a buffer" → "a cache, which is a buffer". These fire with no fix and stay unchanged: "They served
    as mentors", "Both serve as anchors", "The tool serves as a bridge" (a noun before "serves" is not in the guard
    list), "export these functions as a module" and "we expose helper functions as a convenience" (plural noun,
    Gap 14), "It functions as follows", "It serves as well as the old one" and "This can serve as well as any"
    (the token after "as" is not in `AS_COMPLEMENT`, Gap 16), and "These files can serve as templates" (a bare
    plural after "as" is not in the list either, so the model takes it).
  - The guard for "serves/stands/functions as" is `SINGULAR_SUBJECT: &[&str] = &["it", "this", "that", "which",
    "who", "he", "she"]`, checked with `BlockText`'s previous word token. Both fixes ("→ is" and the modal/"to" "→ be") also require the
    next word token after "as" to be in `AS_COMPLEMENT: &[&str] = &["a", "an", "the", "its", "his", "her",
    "their", "our", "your", "my", "one", "another"]`, matched case-insensitively with `BlockText`'s next word token
    (Gap 16). Each entry opens a noun phrase, so "is"/"be" + that phrase is grammatical. "follows", "well", "long",
    "soon", "much", "far", "such", "if" and "though" are not in the list, so no fixed "as …" phrase is ever fixed.
    If no next word token exists in the same `BlockText` (end of block, or a protected run follows), the fix is
    dropped. A noun subject ("The tool serves as") gets no
    fix, because the matcher cannot tell the noun "functions" from the verb after a noun. The model engine takes it.
  - `rules_fixtures::safe_fix_contexts_grammatical` asserts these pairs exactly. FixClass stays `Safe` for both rules
    (the class says a fix *may* exist); `FixClass::Safe` findings with `fix: None` select a model window like a
    `Model` finding (§16 `windows`).
- **Safe fixes never cross a protected run.** `BlockText::src_range` returns `None` across `OBJ`. The fixes contain
  no digits or URLs. No fix output matches its own pattern (FR-8.9).

---

## 11. `score`

**File:** `src/score/mod.rs` (new)
**Component:** §4.7 score
**Implements:** FR-5.1, FR-5.2, FR-5.3

```rust
use crate::rules::{Finding, Severity};

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Score { pub value: f64, pub prose_words: usize, pub weighted: u32 }

pub const fn weight(s: Severity) -> u32 { match s { Severity::Suggestion => 1, Severity::Warning => 3, Severity::Error => 9 } }

/// value = round2(1000 × Σ weight(severity) / max(prose_words, 1)). System findings (SLP9xx) are excluded.
pub fn score(findings: &[Finding], prose_words: usize) -> Score {
    let weighted: u32 = findings.iter().filter(|f| !f.code.is_system()).map(|f| weight(f.severity)).sum();
    Score { value: round2(1000.0 * f64::from(weighted) / prose_words.max(1) as f64), prose_words, weighted }
}
/// Word-weighted: Σ weighted / Σ words over all files, not a mean of means.
pub fn overall(files: &[Score]) -> Score {
    let words: usize = files.iter().map(|s| s.prose_words).sum();
    let weighted: u32 = files.iter().map(|s| s.weighted).sum();
    Score { value: round2(1000.0 * f64::from(weighted) / words.max(1) as f64), prose_words: words, weighted }
}
fn round2(v: f64) -> f64 { (v * 100.0).round() / 100.0 }
```

**Design notes:**

- The README prints this formula and these weights (FR-5.1).
- Rounding to two decimals keeps the JSON byte-stable (FR-5.3, NFR-6).
- The score counts the *reported* findings, after `--min-severity`, so the number and the listing agree.

---

## 12. `directives`

**File:** `src/directives/mod.rs` (new)
**Component:** §4.5 directives
**Implements:** FR-6.1, FR-6.2, FR-6.3, FR-6.4, FR-6.5, FR-6.6, FR-6.7

```rust
use std::collections::BTreeSet;
use std::ops::Range;
use std::path::Path;
use crate::doc::Document;
use crate::rules::{registry::Registry, RuleCode};
use crate::Error;

/// Markdown: a whole line `<!-- sloperator-<kind> [SLP### ...] -->`. Plain text: `[sloperator-<kind> [SLP### ...]]`.
pub const MD_DIRECTIVE: &str = r"^\s*<!--\s*sloperator-(ignore-file|ignore|disable|enable)((?:\s+[A-Za-z0-9]+)*)\s*-->\s*$";
pub const TXT_DIRECTIVE: &str = r"^\s*\[sloperator-(ignore-file|ignore|disable|enable)((?:\s+[A-Za-z0-9]+)*)\s*\]\s*$";

#[derive(Debug, Default)]
pub struct Suppressions {
    file: bool,                                            // ignore-file
    lines: Vec<(Range<u32>, Option<BTreeSet<RuleCode>>)>,  // None = all rules
    frozen: Vec<Range<usize>>,                             // byte ranges of suppressed lines (FR-6.6)
}

/// Candidate lines (Gap 6): in Markdown, a line counts only when one `Protected(Html)` or `Protected(HtmlBlock)` span
/// covers all of its non-whitespace bytes; the regex is then applied to that line. A directive-shaped line inside a
/// code fence (`Protected(CodeBlock)`), inline code, front matter or prose is never a directive and has no effect.
/// In plain text, a line counts when `plain::parse` tagged it `Protected(Directive)` (§6).
/// Also re-tags each directive line's span as ProtectReason::Directive, so `doc` is updated in place.
pub fn extract(doc: &mut Document, path: &Path, registry: &Registry) -> Result<Suppressions, Error>;

impl Suppressions {
    pub fn is_suppressed(&self, line: u32, code: RuleCode) -> bool {
        !code.is_system() && (self.file || self.lines.iter().any(|(r, codes)| r.contains(&line)
            && codes.as_ref().is_none_or(|c| c.contains(&code))))
    }
    pub fn frozen_ranges(&self) -> &[Range<usize>] { &self.frozen }
}
```

**Semantics:**

- `ignore` applies to the next line that holds a prose byte. Blank lines and other directives in between are
  skipped.
- `disable CODES` opens a region; with no codes it covers all rules. It is closed by an `enable` with the same code
  set. An unclosed region at end of file is `Error::Directive { file, line, reason: "sloperator-disable without
  matching sloperator-enable" }` (FR-6.3).
- Any word that is not a registry code is `Error::UnknownRule { code, where_: Some("path:line") }` (FR-6.5).
  System codes count as unknown here.
- `ignore-file` suppresses everything and freezes the whole document (FR-6.4, FR-6.6).
- Plain text uses the same logic with `TXT_DIRECTIVE` (FR-6.7).

**Design notes:**

- Suppression is per line. A finding that spans lines is suppressed when its start line is.
- System findings are never suppressed.
- Frozen regions cannot be edited in the first place, so no rejection is ever needed for them.

---

## 13. `config`

**File:** `src/config/mod.rs`, `src/config/file.rs` (new)
**Component:** §4.2 config
**Implements:** FR-7.1, FR-7.2, FR-7.3, FR-7.4, FR-7.5, FR-7.6, FR-2.11, FR-2.13, D-8, D-9 (TOML with snake_case keys, per Brad's PRD-gate answer Q5 (a))

```toml
# sloperator.toml: every key (snake_case, D-9)
extend = "../sloperator.toml"          # optional; relative to this file; may chain; a cycle is an error
select = ["SLP001", "SLP003"]          # replaces the default set
extend_select = ["SLP005"]
ignore = ["SLP002"]
min_severity = "warning"               # suggestion | warning | error
include = ["**/*.md", "**/*.txt"]      # walker filter (default: *.md, *.markdown, *.txt)
exclude = ["CHANGELOG.md", "vendor/**"]
respect_gitignore = true

[rules.SLP014]
severity = "error"
max_per_1000_words = 5.0               # parameter names and defaults come from RuleMeta::params

[[per_file]]
glob = "docs/legal/**"
ignore = ["SLP008"]
min_severity = "error"

[protect]
patterns = ['\bv\d+\.\d+\.\d+\b']      # regexes that extend the protected set (FR-7.4)
terms = ["Acme Cloud", "Dr. Okafor"]   # literals that must survive unchanged; verify counts them (FR-8.5)
front_matter = true
math = true
html = true

[model]
name = "qwen3-4b"                      # registry name (§15)
path = "/models/my.gguf"               # optional local GGUF instead of the cache; requires `profile`
profile = "qwen3"
cache_dir = "~/.cache/sloperator/models"
window = "paragraph"                   # paragraph | sentence
max_window_chars = 4000                # longer windows are skipped with SLP902 (DQ-5)
max_tokens = 0                         # 0 = 1.5 × the window's token estimate (architecture §8)
seed = 42
```

```rust
// src/config/file.rs: the shape on disk. Unknown keys fail through deny_unknown_fields (FR-7.6).
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFile {
    pub extend: Option<PathBuf>,
    pub select: Option<Vec<String>>,
    pub extend_select: Option<Vec<String>>,
    pub ignore: Option<Vec<String>>,
    pub min_severity: Option<crate::rules::Severity>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub respect_gitignore: Option<bool>,
    pub rules: Option<BTreeMap<String, toml::Table>>,   // validated against the registry and RuleMeta::params
    pub per_file: Option<Vec<PerFile>>,
    pub protect: Option<ProtectFile>,
    pub model: Option<ModelFile>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerFile { pub glob: String, pub select: Option<Vec<String>>, pub extend_select: Option<Vec<String>>,
    pub ignore: Option<Vec<String>>, pub min_severity: Option<crate::rules::Severity>,
    pub rules: Option<BTreeMap<String, toml::Table>> }

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtectFile { pub patterns: Option<Vec<String>>, pub terms: Option<Vec<String>>,
    pub front_matter: Option<bool>, pub math: Option<bool>, pub html: Option<bool> }

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelFile { pub name: Option<String>, pub path: Option<PathBuf>, pub profile: Option<String>,
    pub cache_dir: Option<PathBuf>, pub window: Option<WindowMode>, pub max_window_chars: Option<usize>,
    pub max_tokens: Option<u32>, pub seed: Option<u32> }

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowMode { Paragraph, Sentence }
```

```rust
// src/config/mod.rs: the resolved, immutable view (architecture §6)
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
pub use crate::doc::ProtectSettings;
pub use file::WindowMode;
use crate::rules::{registry::Registry, Enabled, RuleCode, Severity};
use crate::Error;

#[derive(Clone, Debug, serde::Serialize)]
pub struct RuleConf { pub severity: Severity, pub params: BTreeMap<&'static str, f64> }

#[derive(Clone, Debug, serde::Serialize)]
pub struct ModelSettings { pub name: String, pub path: Option<PathBuf>, pub profile: Option<String>,
    pub cache_dir: Option<PathBuf>, pub window: WindowMode, pub max_window_chars: usize, pub max_tokens: u32,
    pub seed: u32 }

#[derive(Clone, Debug, serde::Serialize)]
pub struct Settings {
    pub source: Option<PathBuf>,                 // the sloperator.toml in effect; None = built-in defaults
    pub rules: BTreeMap<RuleCode, RuleConf>,     // enabled rules only
    pub min_severity: Severity,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub respect_gitignore: bool,
    pub protect: ProtectSettings,
    pub model: ModelSettings,
}
impl Settings { pub fn enabled(&self, reg: &'static Registry) -> Vec<Enabled<'static>>; }   // in registry order

#[derive(Clone, Debug, Default)]
pub struct CliOverrides { pub select: Vec<String>, pub extend_select: Vec<String>, pub ignore: Vec<String>,
    pub min_severity: Option<Severity>, pub seed: Option<u32> }

pub struct Resolver { /* CLI overrides, the explicit --config chain, and a cache dir -> parsed chain */ }
impl Resolver {
    pub fn new(cli: CliOverrides, explicit: Option<&Path>) -> Result<Resolver, Error>;   // --config (FR-7.2)
    /// The nearest sloperator.toml found upward from the file's directory (FR-7.1), then per_file, then CLI (FR-7.5).
    pub fn for_file(&mut self, path: &Path) -> Result<Arc<Settings>, Error>;
}
```

**Resolution order (FR-7.5).** Lowest to highest:

1. Built-in defaults.
2. The `extend` chain, root first.
3. The discovered file, or the `--config` file.
4. Every matching `[[per_file]]` entry, in file order.
5. CLI flags.

At each layer the rule set is computed as `select` (which replaces the set when present), plus `extend_select`,
minus `ignore`.

**Validation (FR-7.6).** Each failure below becomes `Error::Config { file, key, reason }` with a dotted key
(e.g. `rules.SLP014.max_per_1000_words`):

- a TOML parse error;
- an unknown key;
- a wrong type;
- an unknown rule code or a system rule code;
- an unknown or non-numeric rule parameter;
- an invalid regex in `protect.patterns`;
- an invalid glob;
- `model.path` without `model.profile`;
- a `model.name` that is not in the registry;
- an `extend` cycle.

Codes given on the CLI go through the same checks, with `file` = `<command line>` (FR-2.11, FR-2.15).

**Design notes:**

- The parent file is never merged implicitly (FR-7.1, D-8). Only `extend` pulls another file in. The upward search
  stops at the filesystem root.
- The `Resolver` is filled on the main thread while inputs are resolved. Parallel analysis only reads the
  `Arc<Settings>` (ADR-09).
- `sloperator config [PATH]` prints `Settings` as pretty JSON with snake_case keys, the same as the file (FR-2.13,
  DQ-9).
- `per_file` globs match the input path relative to the directory of the config file that declares them.

---

## 14. `input`

**File:** `src/input/mod.rs` (new)
**Component:** §4.3 input
**Implements:** FR-2.2, FR-2.3, FR-2.4, FR-8.8, FR-3.6

```rust
use std::path::{Path, PathBuf};
use std::sync::Arc;
pub use crate::doc::Format;
use crate::config::{Resolver, Settings};
use crate::Error;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Origin { File(PathBuf), Stdin { as_path: PathBuf } }   // as_path = --stdin-filename, else "-"

#[derive(Clone, Debug)]
pub struct SourceRef { pub origin: Origin, pub format: Format, pub settings: Arc<Settings> }

#[derive(Clone, Debug)]
pub struct Source { pub origin: Origin, pub format: Format, pub text: Arc<str>, pub settings: Arc<Settings> }

/// Files, directories (walked), globs and "-" become sorted, de-duplicated refs. No args means the current directory.
pub fn resolve(args: &[String], stdin_filename: Option<&Path>, cwd: &Path, resolver: &mut Resolver,
    notes: &mut dyn std::io::Write) -> Result<Vec<SourceRef>, Error>;

/// Reads a file, or takes the stdin text cli already read. Non-UTF-8 input returns Error::NotUtf8, whose offset is
/// Utf8Error::valid_up_to() (FR-8.8).
pub fn read(r: &SourceRef, stdin_text: Option<&str>) -> Result<Source, Error>;

pub fn format_for(path: &Path) -> Format;   // .txt → PlainText; everything else (.md, .markdown, "-") → Markdown
```

**Walking.** A directory is walked with `ignore::WalkBuilder::new(dir)` and these settings:

- every filter set explicitly, never left at the crate default (Gap 5):

  ```rust
  let g = s.respect_gitignore;
  ignore::WalkBuilder::new(dir)
      .standard_filters(false)   // start from nothing; each filter below is a stated choice
      .hidden(false)             // `.github/*.md` and other dot-dirs are checked; `.git/` is skipped by filter_entry
      .ignore(false)             // `.ignore` files are not read: FR-2.4 names only .gitignore and `exclude`
      .git_ignore(g)             // .gitignore, in the walked dirs ...
      .parents(g)                // ... and in their parents
      .git_exclude(g)            // .git/info/exclude is part of git's ignore set
      .git_global(false)         // a user's global gitignore never silently hides a project file
      .require_git(false)        // .gitignore applies outside a git repo too
      .follow_links(false)
      .filter_entry(|e| e.file_name() != ".git")
      .sort_by_file_path(|a, b| a.cmp(b));
  ```

  With `respect_gitignore = false`, only `exclude` skips files. All methods are on `WalkBuilder` in ignore 0.4.x,
  each taking `bool` and returning `&mut WalkBuilder` (docs.rs, fetched this revision);
- `include` and `exclude` compiled as `globset::GlobSet`.

All of these are ignore 0.4.34 / globset 0.4.21 signatures, checked on docs.rs. An argument containing `*`, `?` or
`[` is treated as a glob: the walk starts at its literal prefix and keeps the paths that match. The final list is
sorted by `Origin` and de-duplicated (FR-3.6, NFR-6).

**Design notes:**

- An explicit file that matches `exclude` or `.gitignore` is skipped, as FR-2.4 says. A note goes to stderr:
  `sloperator: skipped (excluded): PATH`.
- A path that does not exist, or a glob that matches nothing, returns `Error::Io` (exit 2). A typo never passes as
  "clean".
- For stdin (FR-2.3), `--stdin-filename` sets the format and the config lookup path. Without it, stdin is Markdown,
  and the config is resolved for `cwd`.

---

## 15. `model` — registry, cache, pull

**File:** `src/model/mod.rs`, `src/model/registry.rs`, `src/model/pull.rs` (new)
**Component:** §4.14 model (ADR-08)
**Implements:** FR-2.14, FR-9.1, FR-9.6, FR-9.10, D-5 / Brad Q2 (a) (explicit `model pull`, exit 2 when missing), D-6 / Brad Q4 (a) (Qwen3-4B Q4_K_M default)

```rust
// src/model/registry.rs
#[derive(Debug)]
pub struct PromptProfile { pub name: &'static str, pub temp: f32, pub top_p: f32, pub top_k: i32,
    pub presence_penalty: f32, pub user_suffix: &'static str, pub strip_empty_think: bool }

/// The Qwen3 card's non-thinking sampling settings (research inference-and-models.md §2).
pub const QWEN3: PromptProfile = PromptProfile { name: "qwen3", temp: 0.7, top_p: 0.8, top_k: 20,
    presence_penalty: 1.5, user_suffix: " /no_think", strip_empty_think: true };
pub const PROFILES: &[&PromptProfile] = &[&QWEN3];

#[derive(Debug)]
pub struct ModelSpec { pub name: &'static str, pub repo: &'static str, pub file: &'static str,
    pub sha256: &'static str, pub bytes: u64, pub licence: &'static str, pub profile: &'static PromptProfile }

/// sha256 and sizes read from the Hugging Face API (`/api/models/<repo>?blobs=true`) on 2026-10-30.
pub const REGISTRY: &[ModelSpec] = &[
    ModelSpec { name: "qwen3-4b", repo: "Qwen/Qwen3-4B-GGUF", file: "Qwen3-4B-Q4_K_M.gguf",
        sha256: "7485fe6f11af29433bc51cab58009521f205840f5b4ae3a32fa7f92e8534fdf5", bytes: 2_497_280_256,
        licence: "apache-2.0", profile: &QWEN3 },
    ModelSpec { name: "qwen3-1.7b", repo: "Qwen/Qwen3-1.7B-GGUF", file: "Qwen3-1.7B-Q8_0.gguf",
        sha256: "061b54daade076b5d3362dac252678d17da8c68f07560be70818cace6590cb1a", bytes: 1_834_426_016,
        licence: "apache-2.0", profile: &QWEN3 },
];
pub const DEFAULT_MODEL: &str = "qwen3-4b";
```

```rust
// src/model/mod.rs
use std::path::{Path, PathBuf};
use crate::config::ModelSettings;
use crate::Error;
pub mod pull;
pub mod registry;
pub use registry::{ModelSpec, PromptProfile, DEFAULT_MODEL, REGISTRY};

#[derive(Debug)]
pub struct ResolvedModel { pub name: String, pub path: PathBuf, pub profile: &'static PromptProfile }

/// model.cache_dir, else dirs::cache_dir()/sloperator/models (dirs 7.0.0: `cache_dir() -> Option<PathBuf>`).
pub fn cache_dir(s: &ModelSettings) -> Result<PathBuf, Error>;
/// CACHE/<repo with "/" replaced by "__">/<file>
pub fn cached_path(dir: &Path, spec: &ModelSpec) -> PathBuf;
/// model.path (with its profile), or the registry entry's cached file. Absent → Error::ModelMissing (FR-9.6).
pub fn resolve(s: &ModelSettings) -> Result<ResolvedModel, Error>;
pub fn spec(name: &str) -> Option<&'static ModelSpec>;
```

```rust
// src/model/pull.rs: the only network code (tests/layering.rs::network_only_in_pull)
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};
use super::ModelSpec;
use crate::Error;

pub const HF_BASE: &str = "https://huggingface.co";

/// GET {base}/{repo}/resolve/main/{file}, streamed and hashed into FILE.part; then rename to FILE and write FILE.sha256.
/// A sha256 mismatch or an HTTP error deletes .part and returns Error::Download.
pub fn pull(spec: &ModelSpec, dir: &Path, base: &str, progress: &mut dyn Write) -> Result<PathBuf, Error> {
    let url = format!("{base}/{}/resolve/main/{}", spec.repo, spec.file);
    let fail = |reason: String| Error::Download { url: url.clone(), reason };
    let mut resp = ureq::get(&url).call().map_err(|e| fail(e.to_string()))?;      // ureq 3.5.0
    let reader = resp.body_mut().with_config().limit(spec.bytes + 1).reader();
    // Loop: read 1 MiB chunks → write FILE.part → Sha256::update → a progress line on each 1%.
    // Compare the hex digest with spec.sha256; on a mismatch remove .part and return Err(fail("sha256 mismatch ...")).
    todo!("streaming loop as specified")
}
```

**CLI (FR-2.14):**

- `model pull [NAME]` downloads `NAME`, or the configured `model.name`, which defaults to `qwen3-4b`. Progress goes to
  stderr. If the file and a matching `.sha256` sidecar are already present, it prints "up to date" and exits 0.
- `model list` prints, for each registry entry: name, file, size, licence, and present/absent with the path.
- `model path [NAME]` prints the resolved path and exits 0. If the model is absent it exits 2 with `ModelMissing`.

**Design notes:**

- The spike may change `DEFAULT_MODEL` or add entries, but not the shape. Each new entry gets its real sha256 from
  the same API.
- `base` is a function parameter, not a config key. Tests point it at a local `TcpListener` (§23.11), and the CLI
  always passes `HF_BASE`.
- The runtime only ever receives a file path (ADR-08), so `check` makes no network request (FR-9.1).

---

## 16. `engine` — port, windows, masking, prompt

**File:** `src/engine/mod.rs`, `src/engine/window.rs`, `src/engine/mask.rs`, `src/engine/prompt.rs` (new)
**Component:** §4.10 engine (ADR-02, ADR-04, ADR-09)
**Implements:** FR-9.2, FR-9.3, FR-9.4, FR-9.6, FR-9.7, FR-9.8, D-3, D-12 (prompt and few-shot examples only; no tuning)

```rust
// src/engine/mod.rs
use crate::config::ModelSettings;
use crate::model::{PromptProfile, ResolvedModel};
use crate::Error;
pub mod mask;
pub mod prompt;
pub mod window;
#[cfg(feature = "model")]
pub mod runtime;

pub const DEFAULT_SEED: u32 = 42;

pub struct RewriteRequest<'a> { pub system: &'static str, pub few_shot: &'static [(&'static str, &'static str)],
    pub window: &'a str, pub profile: &'static PromptProfile, pub max_tokens: u32, pub seed: u32 }

/// The port (ADR-02). Synchronous, one per process, and it owns the loaded model.
pub trait Rewriter { fn rewrite(&mut self, req: &RewriteRequest<'_>) -> Result<String, Error>; }

pub trait RewriterFactory {
    /// A cheap check that runs before any file is processed (Fix/Diff with the model engine on). It loads nothing.
    fn preflight(&self, m: &ResolvedModel) -> Result<(), Error> { let _ = m; Ok(()) }
    fn open(&self, m: &ResolvedModel) -> Result<Box<dyn Rewriter>, Error>;
}

/// The factory in a build without feature `model`. Preflight fails, so the run exits 2 before doing any work.
pub struct DisabledFactory;
impl RewriterFactory for DisabledFactory {
    fn preflight(&self, _: &ResolvedModel) -> Result<(), Error> { Err(Error::ModelUnsupported) }
    fn open(&self, _: &ResolvedModel) -> Result<Box<dyn Rewriter>, Error> { Err(Error::ModelUnsupported) }
}

/// Resolves the model and runs preflight eagerly (FR-9.6). The model itself is opened on the first `get` (FR-9.4).
pub struct LazyRewriter<'f> { factory: &'f dyn RewriterFactory, model: ResolvedModel,
    open: Option<Box<dyn Rewriter>>, pub loads: u32 }
impl<'f> LazyRewriter<'f> {
    pub fn new(factory: &'f dyn RewriterFactory, s: &ModelSettings) -> Result<Self, Error> {
        let model = crate::model::resolve(s)?;      // ModelMissing → exit 2, naming `model pull` and --rules-only
        factory.preflight(&model)?;
        Ok(LazyRewriter { factory, model, open: None, loads: 0 })
    }
    pub fn profile(&self) -> &'static PromptProfile { self.model.profile }
    pub fn get(&mut self) -> Result<&mut dyn Rewriter, Error> {
        if self.open.is_none() { self.open = Some(self.factory.open(&self.model)?); self.loads += 1; }
        match self.open.as_deref_mut() { Some(r) => Ok(r), None => Err(Error::Runtime("model not open".into())) }
    }
}
```

```rust
// src/engine/window.rs
use std::ops::Range;
pub struct Window {
    pub block: crate::doc::BlockId,
    pub range: Range<usize>,            // the source bytes the rewrite replaces
    pub masked: String,                 // prose with ⟦n⟧ placeholders; soft breaks folded to single spaces
    pub slots: Vec<Range<usize>>,       // slots[n-1] = source range of placeholder ⟦n⟧
    pub wrap: Option<Wrap>,             // Some when the window spans more than one source line (Gap 7)
    pub weight_before: u32,             // weighted findings overlapping `range` (FR-9.5)
}
/// Line structure of a multi-line window (Gap 7). `prefix` is the container continuation text that every line after
/// the first begins with: the bytes from the line start to the first prose byte, e.g. `"> "` or `"   "` (list-item
/// indentation). All continuation lines must carry identical prefix bytes. If they differ (lazy continuation, mixed
/// indentation) the window is `Skip` with reason "inconsistent container prefix" (SLP902), never guessed.
pub struct Wrap { pub width: usize, pub prefix: String, pub newline: crate::doc::Newline }
pub enum WindowPlan { Rewrite(Window), Skip { range: Range<usize>, reason: String } }   // Skip → finding SLP902

/// One window per block (or per sentence when `model.window = "sentence"`) that overlaps a finding with
/// severity ≥ min_severity whose rule's fix class is Safe or Model. A FixClass::None finding never selects a window.
pub fn windows(doc: &crate::doc::Document, findings: &[crate::rules::Finding], frozen: &[Range<usize>],
    s: &crate::config::Settings) -> Vec<WindowPlan>;
```

```rust
// src/engine/mask.rs
use std::ops::Range;
pub const OPEN: char = '\u{27E6}';    // ⟦
pub const CLOSE: char = '\u{27E7}';   // ⟧

/// Each protected run inside the window becomes ⟦n⟧, numbered from 1 in source order. Protected runs are:
/// inline code, link destination, inline HTML, math, bare URL, user pattern/term, emphasis/hard-break markup.
pub fn mask(doc: &crate::doc::Document, block: crate::doc::BlockId, range: Range<usize>) -> (String, Vec<Range<usize>>);

/// The output contract (FR-9.3), then unmasking (ADR-04). Applied in order:
///  1. If profile.strip_empty_think: remove one leading `<think>` + whitespace + `</think>`.
///  2. Trim. Reject with Check::OutputContract if the result is empty; still contains `<think>`; opens with a
///     preamble matching `(?i)^(here(?:'s| is)|sure|certainly|rewritten|revised)\b[^\n]*:\s`; is wrapped in
///     quotes or a ``` fence; contains a blank line; or is longer than 2 × the masked window.
///  3. Each ⟦n⟧ for n in 1..=slots.len() must appear exactly once, and no other ⟦…⟧ may appear.
///     Otherwise reject with Check::Placeholders.
///  4. Replace each ⟦n⟧ with doc.src[slots[n-1]]. If `wrap` is Some, re-wrap at that width with its break text.
pub fn propose(doc: &crate::doc::Document, w: &crate::engine::window::Window, raw: &str,
    profile: &crate::model::PromptProfile) -> Result<crate::edit::Proposal, crate::verify::Rejection>;
```

```rust
// src/engine/prompt.rs: written for this project (D-12); not training data
pub const SYSTEM: &str = "You edit one passage of prose so it reads as written by a careful human. \
Keep every fact, name, number and claim. Keep every placeholder like ⟦1⟧ exactly once, unchanged. \
Remove filler, hype, hedging preambles, 'not X but Y' framing and lists of three. Use plain words. \
Return only the rewritten passage: no preamble, no quotes, no notes.";
pub const FEW_SHOT: &[(&str, &str)] = &[
    ("It's important to note that ⟦1⟧ plays a crucial role in the build — it's not just a tool, it's a foundation.",
     "⟦1⟧ drives the build."),
    ("Our team is committed to delivering a seamless, robust, and scalable experience for 30% more users.",
     "We want the service to keep working as it grows to 30% more users."),
];
/// model.max_tokens, or 3/2 × (masked chars / 4) + 16 when that setting is 0 (architecture §8).
pub fn max_tokens(masked: &str, s: &crate::config::ModelSettings) -> u32;
```

**Design notes:**

- A window is skipped with finding SLP902 `window-skipped` (warning) in two cases: its source already contains `⟦`
  or `⟧` (ADR-04 follow-up), or its masked text is longer than `model.max_window_chars`. It is never skipped
  silently (FR-9.7).
- **Container prefixes in wrapped windows (Gap 7).** In a block quote or list item, each continuation line starts
  with `Protected(Markup)` bytes (`> `, indentation) that lie *inside* the window range. They are never masked and
  never sent to the model:
  1. `windows` strips them. Each `SoftBreak` plus the next line's prefix becomes one space in `masked`, and
     `Wrap.prefix` records the prefix bytes (identical on every continuation line, else `Skip`).
  2. `propose` unmasks, then greedily re-wraps the prose to `Wrap.width` (the widest source line, measured without
     its prefix), joining lines with `Wrap.newline` + `Wrap.prefix`. Every output line therefore carries the original
     prefix; the first line's marker lies outside the window range and is untouched.
  3. Re-wrapping may change the line count, and with it the number of prefix spans. So `protected_strings` (§6)
     counts a block's continuation-prefix spans once per block. `verify::edit` then compares the same markup, and the
     skeleton check still catches any structural change.
  A single-line window has `wrap: None` and needs none of this.
- The model may move a placeholder within the window (ADR-04 Option 1). So `Document::protected_strings` returns a
  **sorted multiset**, not a sequence, and §8 compares it that way.
- Determinism (FR-9.8): the seed is `--seed`, else `model.seed`, else `DEFAULT_SEED`. Each window gets the same
  seed and a cleared KV cache, so results do not depend on window order (architecture §6).
- ADR-04 Option 2 (degraded mode) is not built unless the spike's placeholder-survival figure demands it. If it
  does, that becomes a SPEC revision, never a runtime fallback.

---

## 17. `pipeline`

**File:** `src/pipeline/mod.rs` (new)
**Component:** §4.11 pipeline (ADR-03, ADR-05, ADR-09)
**Implements:** D-3, D-4, FR-2.1, FR-2.5, FR-2.6, FR-2.10, FR-4.7, FR-8.7, FR-8.9, FR-9.2, FR-9.5, FR-5.2

```rust
use std::sync::Arc;
use crate::{directives, doc, engine, rules, score};
use crate::input::{Origin, Source};
use crate::rules::{registry::Registry, Finding};
use crate::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode { Lint, Fix, Diff }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Engines { pub rules: bool, pub model: bool }   // both by default; --rules-only / --model-only (FR-2.10)

pub struct Analysis { pub source: Source, pub doc: doc::Document, pub sup: directives::Suppressions,
    pub findings: Vec<Finding> }

#[derive(Clone, Debug)]
pub struct Located { pub finding: Finding, pub start: (u32, u32), pub end: (u32, u32), pub matched: String,
    pub line_text: String }

pub struct FileResult {
    pub origin: Origin,
    pub original: Arc<str>,
    pub findings: Vec<Located>,              // findings with severity ≥ min_severity, plus every SLP9xx (DQ-4)
    pub score: score::Score,                 // score of the final text
    pub score_before: Option<score::Score>,  // Fix/Diff only
    pub fixed: Option<String>,               // Fix/Diff only, and only when the text changed
}

/// Pure and safe to run in parallel (ADR-09).
pub fn analyse(source: Source) -> Result<Analysis, Error> {
    let s = Arc::clone(&source.settings);
    let mut d = doc::parse(Arc::clone(&source.text), source.format, &s.protect);
    let path = match &source.origin { Origin::File(p) => p.clone(), Origin::Stdin { as_path } => as_path.clone() };
    let sup = directives::extract(&mut d, &path, Registry::builtin())?;
    let findings = rules::detect(&d, &sup, &s.enabled(Registry::builtin()));
    Ok(Analysis { source, doc: d, sup, findings })
}

/// Lint never touches a Rewriter (FR-2.1).
pub fn lint(a: Analysis) -> FileResult;

/// Fix/Diff, run serially (ADR-09).
pub fn fix(a: Analysis, engines: Engines, rw: Option<&mut engine::LazyRewriter<'_>>, seed: u32)
    -> Result<FileResult, Error>;
```

**`fix` in order** (architecture §7.2):

1. **Safe fixes** (`engines.rules`; D-4, FR-4.7).
   - Collect each finding's `fix`.
   - `edit::plan` the fixes against the frozen ranges.
   - Run `verify::edit` on each fix. Each `Err` becomes SLP901.
   - `edit::apply` produces `text₁`.
   - Re-parse, re-extract directives and re-detect to get `doc₁` and `findings₁`.
2. **Model windows** (`engines.model`, with `rw` set to `Some`).
   - Call `engine::windows`. A `Skip` becomes SLP902.
   - For each `Rewrite`:
     - Call `rw.get()?.rewrite(&req)?`. A runtime error returns `Err` and the run exits 2 (FR-9.7).
     - Run `mask::propose`, then `verify::edit`.
     - Detect again on the one-edit candidate and run `verify::improvement(w.weight_before, after, ..)`.
     - Any rejection becomes SLP901.
   - Windows never overlap. `edit::apply` produces `text₂`.
   - With `--model-only`, step 1 is skipped and windows come from the original findings.
3. **Whole-file check.** Run `verify::document(doc₀, doc₂)`. If it fails, every edit is discarded (`fixed = None`)
   and one SLP901 with `Check::Document` is reported.
4. **Report.** Report the findings that remain in the final text and pass the min-severity filter, plus every
   SLP901/902. `score` is computed on the final text and `score_before` on the original.

**Design notes:**

- `edit::apply` accepts only `Verified<Proposal>`, so an unverified byte is never written (ADR-05).
- If there are no windows, `get()` is never called and the model never loads (FR-9.4). `LazyRewriter::new` has
  already run in `cli`, so a missing model fails every `--fix`/`--diff` run that has the model engine on (FR-9.6).

---

## 18. `report`

**File:** `src/report/mod.rs`, `src/report/{text,concise,json,github}.rs` (new)
**Component:** §4.12 report
**Implements:** FR-3.1, FR-3.2, FR-3.3, FR-3.4, FR-3.5, FR-3.6, FR-5.2, NFR-9 (`schemaVersion`)

```rust
use std::io::{self, Write};
use crate::pipeline::FileResult;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum OutputFormat { #[default] Text, Concise, Json, Github }

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary { pub files: usize, pub findings: usize, pub by_severity: [usize; 3], pub fixed_files: usize,
    pub rejected: usize, pub skipped: usize, pub score: crate::score::Score, pub errors: usize }

pub fn render(fmt: OutputFormat, results: &[FileResult], summary: &Summary, color: bool, out: &mut dyn Write)
    -> io::Result<()>;
/// GitHub workflow-command escaping. Data: `%`→`%25`, `\r`→`%0D`, `\n`→`%0A`. Properties also escape `:`→`%3A`, `,`→`%2C`.
pub fn gh_escape(s: &str, property: bool) -> String;
```

- **text:** one finding per entry: `path:line:col SLP001 [warning] message`, then the source line, then a caret
  line. Each file then gets `score 12.40 (37 weighted / 2,984 words)`, and the run ends with a summary.
- **concise:** one line per finding, `path:line:col: SLP001 [warning] message`.
- **json** (camelCase field names, architecture §6):

```json
{"schemaVersion":1,"ruleSetVersion":"0.1.0",
 "files":[{"path":"docs/a.md","score":{"value":12.4,"proseWords":2984,"weighted":37},"scoreBefore":null,
   "findings":[{"code":"SLP001","name":"ai-vocabulary-replaceable","severity":"warning",
     "path":"docs/a.md","start":{"line":3,"column":5},"end":{"line":3,"column":15},"message":"…","matched":"delve into",
     "fix":{"available":true,"class":"safe"}}]}],
 "summary":{"files":1,"findings":1,"bySeverity":[0,1,0],"fixedFiles":0,"rejected":0,"skipped":0,
   "score":{"value":12.4,"proseWords":2984,"weighted":37},"errors":0}}
```

- **github:** `::warning file=docs/a.md,line=3,col=5,endLine=3,endColumn=15,title=SLP001 ai-vocabulary-replaceable::message`.
  Severity maps suggestion→`notice`, warning→`warning`, error→`error` (FR-3.4).

**Design notes:**

- **Each finding carries its own `path`** (FR-3.3, Gap 8), equal to its file object's `path`, so a consumer that
  flattens findings needs no join. The file object keeps `path` for its score.
- Columns are 1-based character counts.
- The system findings are named `rewrite-rejected` and `window-skipped`, with fix class `none`.
- Output order is the order of `results`, which `cli` sorts by `Origin`. Findings within a file are in source
  order (FR-3.6).

---

## 19. `write`

**File:** `src/write/mod.rs` (new)
**Component:** §4.13 write
**Implements:** FR-2.6, NFR-5

```rust
use std::io::{self, Write};
use std::path::Path;
use crate::pipeline::FileResult;
use crate::Error;

pub trait Fs { fn write_atomic(&self, path: &Path, bytes: &[u8]) -> io::Result<()>; }

/// tempfile 3.27.0: NamedTempFile::new_in(dir) → write_all → sync → copy the original's permissions → persist(path).
pub struct RealFs;
impl Fs for RealFs {
    fn write_atomic(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        let dir = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
        let perms = std::fs::metadata(path)?.permissions();
        let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
        tmp.write_all(bytes)?;
        tmp.as_file().sync_all()?;
        std::fs::set_permissions(tmp.path(), perms)?;
        tmp.persist(path).map_err(|e| e.error)?;
        Ok(())
    }
}

/// Called only when the run produced no exit-2 error (DQ-10). A file is written when `fixed` differs.
/// With stdin, the fixed text (or the original, if unchanged) is printed to `stdout` (FR-2.6).
pub fn commit(results: &[FileResult], fs: &dyn Fs, stdout: &mut dyn Write) -> Result<usize, Error>;
```

**Design notes:**

- Permissions are copied because `NamedTempFile` creates files with mode 0600. Without the copy, `--fix` would
  silently tighten the permissions of every file it touches.
- A failed write returns `Error::Io { path }` (exit 2). Every write is atomic, so a crash before the rename leaves
  the original byte-identical (NFR-5).

---

## 20. `cli` and `main`

**File:** `src/cli/mod.rs`, `src/cli/args.rs`, `src/main.rs` (new)
**Component:** §4.1 cli (ADR-09 ordering)
**Implements:** FR-2.1–FR-2.16, FR-3.5, D-3, D-8; Brad Q6 (CLI surface as written), Q3 (a) (verified model rewrites written by `--fix`)

```rust
// src/cli/args.rs (clap 4.6.7 derive)
use std::path::PathBuf;
use crate::report::OutputFormat;
use crate::rules::Severity;

#[derive(Debug, clap::Parser)]
#[command(name = "sloperator", version, about = "Lint and rewrite LLM-generated prose")]
pub struct Cli {
    #[arg(short, long, global = true, action = clap::ArgAction::Count)] pub verbose: u8,
    #[arg(short, long, global = true)] pub quiet: bool,
    #[command(subcommand)] pub cmd: Command,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Report findings and scores; --fix writes, --diff previews
    Check(CheckArgs),
    /// List rules, or explain one
    Rules { code: Option<String> },
    /// Print the resolved configuration for a path as JSON
    Config { path: Option<PathBuf>, #[arg(long)] config: Option<PathBuf> },
    /// Manage local models
    #[command(subcommand)] Model(ModelCmd),
}

#[derive(Debug, clap::Subcommand)]
pub enum ModelCmd { Pull { name: Option<String> }, List, Path { name: Option<String> } }

#[derive(Debug, clap::Args)]
pub struct CheckArgs {
    pub paths: Vec<String>,
    #[arg(long)] pub fix: bool,
    #[arg(long, conflicts_with = "fix")] pub diff: bool,
    #[arg(long, conflicts_with = "model_only")] pub rules_only: bool,
    #[arg(long)] pub model_only: bool,
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)] pub output_format: OutputFormat,
    #[arg(long, value_enum)] pub min_severity: Option<Severity>,
    #[arg(long)] pub exit_zero: bool,
    #[arg(long, value_delimiter = ',')] pub select: Vec<String>,
    #[arg(long, value_delimiter = ',')] pub extend_select: Vec<String>,
    #[arg(long, value_delimiter = ',')] pub ignore: Vec<String>,
    #[arg(long)] pub config: Option<PathBuf>,
    #[arg(long)] pub stdin_filename: Option<PathBuf>,
    #[arg(long)] pub seed: Option<u32>,
}
```

```rust
// src/cli/mod.rs
use std::io::{Read, Write};
use std::path::PathBuf;
mod args;
pub use args::{CheckArgs, Cli, Command, ModelCmd};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitCode { Clean = 0, Findings = 1, Failure = 2 }

/// The injectable seams (ADR-02, architecture §11).
pub struct Deps { pub rewriter_factory: Box<dyn crate::engine::RewriterFactory>, pub fs: Box<dyn crate::write::Fs>,
    pub cwd: PathBuf, pub color: bool }

/// The only place where the exit code is decided.
pub fn run(cli: Cli, deps: &mut Deps, stdin: &mut dyn Read, out: &mut dyn Write, err: &mut dyn Write) -> ExitCode;
```

```rust
// src/main.rs: the single std::process::exit call (tests/layering.rs::single_exit_point)
use std::io::IsTerminal;
use clap::Parser;
use sloperator::cli::{self, Cli, Deps};

fn main() {
    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            let code = match e.kind() {
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => 0,   // FR-2.16
                _ => 2,                                                                               // FR-2.15
            };
            let _ = e.print();
            std::process::exit(code);
        }
    };
    #[cfg(feature = "model")]
    let factory: Box<dyn sloperator::engine::RewriterFactory> =
        Box::new(sloperator::engine::runtime::RuntimeFactory::new(cli.verbose > 0));
    #[cfg(not(feature = "model"))]
    let factory: Box<dyn sloperator::engine::RewriterFactory> = Box::new(sloperator::engine::DisabledFactory);
    let cwd = match std::env::current_dir() { Ok(d) => d, Err(e) => { eprintln!("sloperator: {e}"); std::process::exit(2) } };
    let color = std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();       // FR-3.5
    let mut deps = Deps { rewriter_factory: factory, fs: Box::new(sloperator::write::RealFs), cwd, color };
    let code = cli::run(cli, &mut deps, &mut std::io::stdin().lock(), &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock());
    std::process::exit(code as i32);
}
```

**`run(Check)`, step by step:**

1. Build `CliOverrides` and call `Resolver::new` with `--config`.
2. Call `input::resolve`. If `-` is among the paths, read stdin into a `String`; invalid UTF-8 returns `NotUtf8`.
3. Choose the engines from the flags and the `Mode`.
4. If the mode is not Lint and `engines.model` is set, call `LazyRewriter::new` before touching any file (FR-9.6, D-5).
5. Read each source with `input::read` and run `pipeline::analyse` on it. This uses rayon `par_iter`; results are
   collected into a `Vec` sorted by `Origin` (ADR-09).
6. In Lint mode, call `pipeline::lint`. Otherwise call `pipeline::fix` serially, in sorted order.
7. Print each per-file `Error` to `err` with its file, line or key, and set `failed`.
8. In Diff mode, print `edit::unified_diff` to `out` for each changed file. In Fix mode, call `write::commit` only
   if `!failed`. With stdin, the fixed text goes to `out`.
9. Call `report::render` on `out`. When `out` carries content (a diff, or stdin `--fix`), render to `err` instead
   (DQ-11).
10. Pick the exit code: `Failure` on any error; else `Clean` if `--exit-zero`; else `Findings` if anything was
    reported; else `Clean`.

**Other subcommands:**

- `rules` prints code, name, severity, on/off, fix class and the rule-set version.
- `rules CODE` prints the rule's `explain` text, its parameters and its before/after example. An unknown code
  returns `UnknownRule` (FR-2.12, FR-2.15).
- `config` is specified in §13 and `model` in §15.

**Design notes:**

- `-v` adds stage timings, model load time and tokens/s per window, and keeps llama.cpp's logs (§21). `-q` prints
  errors only (architecture §8).
- Passing both `--fix` and `--diff` is a usage error (exit 2).

---

## 21. `engine::runtime` — the llama-cpp-2 adapter

**File:** `src/engine/runtime.rs` (new; compiled only with feature `model`)
**Component:** §4.10 engine::runtime (ADR-02)
**Implements:** FR-9.1, FR-9.3, FR-9.4, FR-9.7, FR-9.8, NFR-7

Written against `llama-cpp-2 =0.1.160`, the primary candidate (§1). Every call matches its docs.rs page:

- `send_logs_to_tracing(LogOptions::default().with_logs_enabled(false))`
- `LlamaBackend::init` / `supports_gpu_offload`
- `LlamaModelParams::with_n_gpu_layers`
- `LlamaModel::load_from_file` / `new_context` / `chat_template` / `apply_chat_template` / `vocab` / `n_vocab`
- `LlamaChatMessage::new(String, String)`
- `LlamaVocab::tokenize(&[u8], bool, bool)` / `is_eog` / `token_to_piece_into`
- `LlamaContextParams::with_n_ctx(Option<NonZeroU32>)` / `with_n_batch`
- `LlamaContext::decode`
- `LlamaBatch::new(usize, i32)` / `add` / `clear` / `n_tokens`
- `LlamaSampler::chain_simple` / `penalties` / `top_k` / `top_p` / `temp` / `dist(u32)` / `sample` / `accept`

One backend and one model per process: `LlamaBackend::init` fails on a second call, so a factory opens once
(`LazyRewriter` guarantees it, FR-9.4). `LlamaRewriter` **owns** the model; it is not leaked. Its field order is its
drop order, so the model is freed before the backend. A model still alive at `process::exit` aborts ggml's Metal
teardown (`GGML_ASSERT([rsets->data count] == 0)`, exit 134). Each window gets a fresh `LlamaContext` that borrows the
owned model for the length of one `rewrite` call, so nothing is self-referential and windows stay independent
(FR-9.8). Unless `-v` is given, llama.cpp's and ggml's logs are silenced with `send_logs_to_tracing` **before**
`LlamaBackend::init`, because the Metal device logs during backend and model setup.

If the spike picks `mistralrs`, only this file changes. It would then use `GgufModelBuilder` behind a private
current-thread tokio runtime and `block_on` (architecture §8).

```rust
use std::num::NonZeroU32;

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{LlamaChatMessage, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_2::{LogOptions, send_logs_to_tracing};

use crate::Error;
use crate::engine::{RewriteRequest, Rewriter, RewriterFactory};
use crate::model::ResolvedModel;

pub const N_CTX: u32 = 4096;
const ALL_LAYERS: u32 = 999;     // offloaded when the build has a GPU backend (metal/cuda)
const PENALTY_LAST_N: i32 = 64;  // tokens the presence penalty looks at

fn rt(e: impl std::fmt::Display) -> Error { Error::Runtime(e.to_string()) }

pub struct RuntimeFactory { verbose: bool }
impl RuntimeFactory { pub fn new(verbose: bool) -> Self { RuntimeFactory { verbose } } }

impl RewriterFactory for RuntimeFactory {
    fn open(&self, m: &ResolvedModel) -> Result<Box<dyn Rewriter>, Error> {
        if !self.verbose {
            // Before init: the Metal device logs during backend and model setup.
            send_logs_to_tracing(LogOptions::default().with_logs_enabled(false));
        }
        let backend = LlamaBackend::init().map_err(rt)?;
        let gpu_layers = if backend.supports_gpu_offload() { ALL_LAYERS } else { 0 };
        let params = LlamaModelParams::default().with_n_gpu_layers(gpu_layers);
        let model = LlamaModel::load_from_file(&backend, &m.path, &params)
            .map_err(|e| Error::ModelLoad { path: m.path.clone(), reason: e.to_string() })?;
        Ok(Box::new(LlamaRewriter { model, backend }))
    }
}

/// Field order is drop order: the model is freed before the backend.
struct LlamaRewriter { model: LlamaModel, backend: LlamaBackend }

impl Rewriter for LlamaRewriter {
    fn rewrite(&mut self, req: &RewriteRequest<'_>) -> Result<String, Error> {
        let msg = |role: &str, text: String| LlamaChatMessage::new(role.to_string(), text).map_err(rt);
        let mut chat = vec![msg("system", req.system.to_string())?];
        for (u, a) in req.few_shot {
            chat.push(msg("user", (*u).to_string())?);
            chat.push(msg("assistant", (*a).to_string())?);
        }
        chat.push(msg("user", format!("{}{}", req.window, req.profile.user_suffix))?);
        let tmpl = self.model.chat_template(None).map_err(rt)?;
        let prompt = self.model.apply_chat_template(&tmpl, &chat, true).map_err(rt)?;

        let vocab = self.model.vocab();
        let tokens = vocab.tokenize(prompt.as_bytes(), false, true);
        if tokens.len() + req.max_tokens as usize > N_CTX as usize {
            return Err(Error::Runtime(format!(
                "context overflow: a prompt of {} tokens plus {} output tokens exceeds the {N_CTX}-token context",
                tokens.len(), req.max_tokens)));
        }
        if tokens.is_empty() { return Err(Error::Runtime("the prompt tokenized to nothing".into())); }
        // A fresh context per window: windows are independent (FR-9.8).
        let cparams = LlamaContextParams::default().with_n_ctx(NonZeroU32::new(N_CTX)).with_n_batch(N_CTX);
        let mut ctx = self.model.new_context(&self.backend, cparams).map_err(rt)?;
        let mut batch = LlamaBatch::new(N_CTX as usize, 1);
        let last = tokens.len() - 1;
        for (i, t) in tokens.iter().enumerate() { batch.add(*t, pos(i)?, &[0], i == last).map_err(rt)?; }
        ctx.decode(&mut batch).map_err(rt)?;

        let p = req.profile;
        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::penalties(self.model.n_vocab(), PENALTY_LAST_N, 1.0, 0.0, p.presence_penalty),
            LlamaSampler::top_k(p.top_k),
            LlamaSampler::top_p(p.top_p, 1),
            LlamaSampler::temp(p.temp),
            LlamaSampler::dist(req.seed),
        ]);
        let mut out = Vec::new();
        let first = pos(tokens.len())?;
        for next in (first..).take(req.max_tokens as usize) {
            let tok = sampler.sample(&ctx, batch.n_tokens() - 1);
            sampler.accept(tok);
            if vocab.is_eog(tok) { break; }
            vocab.token_to_piece_into(tok, &mut out, false, None);
            batch.clear();
            batch.add(tok, next, &[0], true).map_err(rt)?;
            ctx.decode(&mut batch).map_err(rt)?;
        }
        String::from_utf8(out).map_err(|e| Error::Runtime(format!("model output is not UTF-8: {e}")))
    }
}

/// A token position; the context bound keeps it far below `i32::MAX`.
fn pos(i: usize) -> Result<i32, Error> { i32::try_from(i).map_err(rt) }
```

**Design notes:**

- Every runtime failure maps to `Runtime` or `ModelLoad`, and both exit 2 (FR-9.7). Hitting `max_tokens` is not an
  error. The truncated text goes through the output contract and verify, which normally reject it.
- The context-overflow error is a backstop. `windows` already skips over-long windows with SLP902, so reaching
  this error means a bug, and the run fails loudly.
- The model is owned and dropped before the backend, never leaked (open question 4, §Self-Audit): a model alive at
  `process::exit` trips ggml's Metal teardown assert (exit 134).
- A fresh context per window costs one context allocation per window; it replaces a single reused context, which
  would have needed a `'static` model borrow.
- `supports_gpu_offload()` is true only in a `metal`/`cuda` build (§2 passthrough). A CPU build offloads 0 layers.
- `dist(seed)` makes output reproducible for the same binary, model and hardware (FR-9.8).

---

## 22. Eval harness

**File:** `eval/README.md`, `eval/paragraphs/p01.md` … `p20.md` (new; the spike's 20 paragraphs, §1), `tests/eval.rs` (new)
**Component:** architecture §6 (eval), §11 integration tier, §12 Phase 7
**Implements:** FR-10.1, FR-10.2, D-12, NFR-3 (second half: the model latency budget)

- **`eval/paragraphs/`** holds twenty slop paragraphs written for this project. Together they cover every
  model-class family, with code spans, links, numbers and a table row among them. `eval/README.md` says how they
  were written and that they are not training pairs (FR-10.1). The protected-content fixtures are
  `fixtures/protect/*` (§23).
- **`tests/eval.rs::eval_report_per_model`** is marked `#[ignore = "integration tier"]` and the file is gated by
  `#![cfg(feature = "model")]`. It works as follows:
  - It runs `pipeline::fix` with both engines, once for every registry model in the cache.
  - It fails when the default model is absent, with "run `sloperator model pull`" (architecture §11).
  - It writes `target/eval/report.json` and prints one table per model. The table shows the mean score before and
    after, protected-span integrity (n/20), rewrites accepted, rewrites rejected per `Check`, SLP902 count, and mean
    seconds per window (FR-10.2).
  - It asserts that integrity is 20/20.
  - It asserts that mean seconds per window is at most `MODEL_WINDOW_BUDGET_S`. That constant comes from the spike's
    Decision section (§1). Until then it is `0.0`, so the test cannot pass by accident.

**Design notes:** Fine-tuning stays deferred (D-12; Brad accepted it at the PRD gate). This report is the input to
that later decision.

---

## 23. Testing Strategy

**Implements:** NFR-8 and the "Verified by" lines of NFR-1–NFR-9

Every file below is a cargo integration-test binary under `tests/`. A Gate selector names a test
as `sloperator::<file>#<test>`, or uses a bare substring (§3). The unit tier runs every test without `#[ignore]`,
built with `--no-default-features`. The integration tier runs the `#[ignore]` tests with `--features model`.

**Test infrastructure:** `tests/common/mod.rs` provides:
- `fixture(path)` and `settings_default()`;
- `run_cli(args, stdin) -> (ExitCode, String, String)`, which calls `cli::run` in-process with `Vec<u8>` writers;
- `FakeFactory`, whose scripted `FakeRewriter` outputs drive the model paths, and which counts `open`/`preflight` calls;
- `FailingFs`, which fails on its Nth write;
- `TempModelCache`, which writes a dummy `.gguf` and `.sha256` into a `TempDir`.

The fixtures in `fixtures/protect/` are `code.md`, `links.md`, `front_matter.md`, `table.md`, `nested_lists.md`,
`crlf.md`, `math.md`, `html.md`, `refdefs.md` and `mixed.md`. `mixed.md` combines code, links, numbers and a table
for the e2e test.

### 23.1 `tests/layering.rs`, `tests/standalone.rs`

The tests listed in §5. They cover FR-1.8, NFR-2 and FR-9.1.

### 23.2 Make target checks (Phase 2 gate commands; run and recorded as story evidence, not test methods)

| Check | Verifies |
|-------|----------|
| `make help` lists `build`, `test`, `test-integration`, `check`, `clean` and `help` | FR-1.2 |
| `make test`; `target/nextest/ci/junit.xml` holds one `testcase` per executed test | FR-1.3, FR-1.4 |
| `make test TESTS='sloperator::cli_exit#exit_two_on_bad_config'`: exactly one testcase | FR-1.5 |
| `make test TESTS='no_such_test'` prints "no test ran" and exits non-zero | FR-1.5, fail loudly |
| With `cargo` removed from `PATH`: `make test`, `make build` and `make check` each print the one-line banner and exit non-zero; likewise `make test` without `cargo-nextest` and `make check` without `cargo-deny` | FR-1.9 (Gaps 1, 15) |
| `make test TESTS='verify'` starts no `--cargo-profile release` build: the log holds the "perf pass skipped" banner and no `ci-perf` run, and `target/nextest/ci-perf/junit.xml` does not exist. `TESTS='perf'` does run the pass | FR-1.5 (Gaps 9, 15) |
| `make build`; `make check` | FR-1.6, NFR-7 |
| CI (`.github/workflows/ci.yml`, macOS) runs `make check`, `make build` and `make test` and passes | NFR-1 |

### 23.3 `tests/doc_spans.rs`

| Test | Verifies |
|------|----------|
| `code_blocks_and_inline_code_protected` | FR-8.1 |
| `link_destination_protected_link_text_prose` | FR-8.1, ADR-04 |
| `autolink_bare_url_image_refdef_protected` | FR-8.1 |
| `front_matter_html_math_protected` | FR-8.1, FR-7.4 |
| `heading_list_cell_quote_text_is_prose_markup_protected` | FR-8.2 |
| `user_patterns_and_terms_split_prose` | FR-8.3, FR-7.4 |
| `spans_cover_every_byte_without_overlap` | §6 invariant |
| `plain_text_paragraphs_and_directive_lines` | D-10, FR-6.7 |
| `crlf_detected` | FR-8.4 |

### 23.4 `tests/edit_apply.rs`

| Test | Verifies |
|------|----------|
| `noop_plan_round_trips_every_protect_fixture` | FR-8.4, ADR-03 |
| `crlf_preserved_in_replacements` / `trailing_newline_preserved` | FR-8.4 |
| `plan_drops_overlaps_deterministically` | §7 |
| `plan_drops_frozen` | FR-6.6 |
| `unified_diff_has_headers_and_hunks` | FR-2.5 |

### 23.5 `tests/verify_edits.rs`

| Test | Verifies |
|------|----------|
| `rejects_lost_number` / `rejects_added_number` / `rejects_changed_percentage` / `rejects_changed_digit_date` | FR-8.5 |
| `rejects_changed_url` / `rejects_lost_term` | FR-8.5 |
| `rejects_new_heading_marker` / `rejects_list_marker` / `rejects_fence` | FR-8.6 |
| `rejects_edit_into_protected_or_frozen` | FR-8.1, FR-6.6 |
| `accepts_reordered_placeholders_restored` | ADR-04, §16 |
| `improvement_requires_strictly_fewer` | FR-9.5, DQ-3 |
| `document_check_fires_on_forged_edit_pair` | architecture §7.2 |

### 23.6 `tests/rules_registry.rs`, `tests/rules_fixtures.rs`

| Test | Verifies |
|------|----------|
| `at_least_15_rules_covering_nine_families` | FR-4.2 |
| `every_rule_has_code_name_severity_state_fix` | FR-4.1 |
| `default_set_is_explicit` / `new_rules_start_off` | FR-4.5 |
| `system_codes_not_in_registry` | §9 |
| `every_rule_has_fixtures` | FR-4.6 |
| `slpNNN_positive_fires` / `slpNNN_negative_silent` / `slpNNN_fix_matches` (×16, macro) | FR-4.2, FR-4.4, FR-4.6 |
| `only_safe_rules_attach_fixes` | FR-4.7 |
| `em_dash_fires_on_density_not_single` | FR-4.3 |
| `fix_is_idempotent_over_all_fixtures` | FR-8.9 |

### 23.7 `tests/score.rs`

| Test | Verifies |
|------|----------|
| `formula_matches_readme` | FR-5.1 |
| `overall_is_word_weighted` | FR-5.1 |
| `identical_input_identical_score` | FR-5.3 |
| `readme_states_limits` | FR-5.4 |

### 23.8 `tests/directives.rs`

| Test | Verifies |
|------|----------|
| `ignore_next_line_named_and_all` | FR-6.1 |
| `disable_enable_region` | FR-6.2 |
| `unclosed_disable_exits_two_with_line` | FR-6.3, NFR-4 |
| `ignore_file` | FR-6.4 |
| `unknown_code_exits_two_with_line` / `system_code_rejected` | FR-6.5 |
| `suppressed_region_not_edited_by_either_engine` | FR-6.6 |
| `plain_text_bracket_directives` | FR-6.7 |
| `directive_in_code_fence_ignored` (a fenced and an indented code block, each holding `<!-- sloperator-disable SLP001 -->` with no `enable`: exit by findings, never 2; findings outside are not suppressed) | FR-6.3, FR-8.1 (Gap 6) |
| `directive_in_inline_code_ignored` | FR-6.1, FR-8.1 (Gap 6) |

### 23.9 `tests/config.rs`

| Test | Verifies |
|------|----------|
| `nearest_file_wins_no_parent_merge` / `extend_chain_and_cycle_error` | FR-7.1 |
| `explicit_config_overrides_discovery` | FR-7.2 |
| `all_documented_keys_parse` (the §13 example) | FR-7.3, FR-7.4 |
| `cli_beats_file_beats_default` | FR-7.5, FR-2.11 |
| `per_file_glob_overrides` | FR-7.3 |
| `unknown_key_wrong_type_unknown_code_unknown_param_bad_regex_exit_two` | FR-7.6, NFR-4 |
| `config_command_prints_resolved_json` | FR-2.13 |

### 23.10 `tests/input.rs`

| Test | Verifies |
|------|----------|
| `files_dirs_globs_and_stdin` / `no_args_checks_cwd` | FR-2.2 |
| `stdin_filename_selects_format_and_config` / `stdin_defaults_markdown` | FR-2.3 |
| `gitignore_and_exclude_skipped_respect_gitignore_false` | FR-2.4 |
| `hidden_dir_walked_dot_ignore_and_global_ignore_not_read` (a `.github/x.md` is checked; a `.ignore` and a global `core.excludesFile`, set via `HOME` in a `TempDir`, skip nothing) | FR-2.2, FR-2.4 (Gap 5) |
| `gitignore_applies_outside_git_repo` (a `TempDir` with `.gitignore` and no `.git`) | FR-2.4 (Gap 5) |
| `missing_path_exits_two` | NFR-4 |
| `non_utf8_exits_two_unmodified` | FR-8.8 |
| `order_independent_of_walk_order` | FR-3.6, NFR-6 |

### 23.11 `tests/model_cache.rs`

| Test | Verifies |
|------|----------|
| `registry_entries_have_sha256_and_profile` | ADR-08 |
| `missing_model_error_names_pull_and_rules_only` | FR-9.6 |
| `pull_from_local_server_verifies_sha256` / `pull_rejects_corrupt_file_and_removes_part` | FR-2.14, ADR-08 |
| `list_and_path_report_presence` | FR-2.14 |
| `no_weights_in_crate` (no `*.gguf`; no file over 10 MB under the module) | FR-9.10 |

### 23.12 `tests/engine.rs` (FakeRewriter)

| Test | Verifies |
|------|----------|
| `windows_only_for_findings_at_or_above_min_severity` / `none_class_selects_no_window` | FR-9.2 |
| `mask_replaces_protected_runs_with_numbered_placeholders` | ADR-04 |
| `contract_rejects_think_block_preamble_quotes_blank_line` / `strips_empty_think` | FR-9.3 |
| `placeholder_dropped_or_duplicated_rejected` | ADR-04 |
| `no_window_means_no_load` / `many_windows_load_once` | FR-9.4 |
| `good_rewrite_written` / `number_drop_rejected_original_kept` | FR-9.5, D-4 |
| `missing_model_exits_two_before_any_file` / `disabled_factory_exits_two` | FR-9.6, ADR-02 |
| `runtime_error_exits_two_nothing_written` | FR-9.7, NFR-5 |
| `seed_passed_through_same_for_every_window` | FR-9.8 |
| `input_with_placeholder_glyph_skipped_slp902` / `over_long_window_skipped_slp902` | FR-9.7, DQ-5 |
| `hard_wrapped_window_rewrapped` | §16 |
| `wrapped_blockquote_and_list_item_rewrapped_with_prefixes` (a 3-line `> ` quote and a 3-line list item with 3-space continuation: a FakeRewriter rewrite is accepted, every output line keeps its prefix, the skeleton is unchanged) | FR-8.2, FR-9.5 (Gap 7) |
| `inconsistent_prefix_window_skipped_slp902` | FR-9.7 (Gap 7) |

### 23.13 `tests/pipeline.rs`

| Test | Verifies |
|------|----------|
| `lint_never_constructs_rewriter` | FR-2.1, D-3 |
| `default_runs_rules_then_model_on_remaining` | D-3, FR-9.2 |
| `rules_only_never_calls_model` / `model_only_skips_safe_fixes` | FR-2.10 |
| `rejection_is_slp901_warning_counts_for_exit` | FR-8.7, DQ-4 |
| `second_fix_changes_nothing_rules_engine` | FR-8.9 |

### 23.14 `tests/report.rs`

| Test | Verifies |
|------|----------|
| `text_shows_path_line_col_code_severity_message_source_line` / `concise_one_line_per_finding` | FR-3.2 |
| `unknown_output_format_exits_two` | FR-3.1 |
| `json_shape_matches_schema` | FR-3.3, FR-5.2, NFR-9 |
| `github_annotations_and_escaping` | FR-3.4 |
| `no_color_when_not_terminal` | FR-3.5 |
| `stable_order` | FR-3.6 |

### 23.15 `tests/write.rs`

| Test | Verifies |
|------|----------|
| `unchanged_file_not_written` / `atomic_write_keeps_permissions` | NFR-5 |
| `failure_on_second_file_leaves_files_byte_identical` | NFR-5 |
| `stdin_fix_prints_fixed_text` | FR-2.6 |

### 23.16 `tests/cli_exit.rs`

| Test | Verifies |
|------|----------|
| `exit_zero_clean` / `exit_one_on_warning` / `exit_zero_on_suggestion_only` | FR-2.7, D-8 |
| `exit_two_on_bad_config` / `exit_two_unknown_rule_flag` / `exit_two_unknown_flag_names_it` | FR-2.7, FR-2.15 |
| `exit_zero_flag_masks_findings_not_errors` | FR-2.8 |
| `min_severity_filters_report_and_exit` | FR-2.9 |
| `rules_only_and_model_only_together_exit_two` | FR-2.10 |
| `rules_list_and_explain` | FR-2.12 |
| `help_and_version_exit_zero` | FR-2.16 |
| `fix_and_diff_together_exit_two` | DQ-11 |

### 23.17 `tests/e2e.rs` (the built binary, via `assert_cmd`)

| Test | Verifies |
|------|----------|
| `mixed_fixture_protected_bytes_unchanged_after_rules_fix` | NFR-8, FR-8.1, FR-8.4 |
| `diff_writes_nothing` | FR-2.5 |
| `check_modifies_nothing` | FR-2.1 |
| `json_identical_across_runs_and_shuffled_dirs` | NFR-6 |
| `fix_without_model_exits_two` | FR-9.6, NFR-4 |

### 23.18 `tests/perf.rs` (release profile, a separate run, §3)

| Test | Verifies |
|------|----------|
| `rules_only_100kb_under_1s` (also asserts that the model cache dir was never read) | NFR-3 |

### 23.19 Integration tier (`#[ignore = "integration tier"]`)

| File / test | Verifies |
|-------------|----------|
| `tests/model_real.rs::loads_once_for_many_windows` | FR-9.4 |
| `tests/model_real.rs::same_seed_same_output` | FR-9.8 |
| `tests/model_real.rs::real_rewrites_accepted_or_rejected_never_unverified` | FR-9.5, D-4 |
| `tests/model_real.rs::no_network_during_check` (`HTTPS_PROXY` points to a closed port) | FR-9.1 |
| `tests/eval.rs::eval_report_per_model` | FR-10.2, NFR-3 |
| `tests/standalone.rs::builds_when_extracted` | NFR-2 |

`model_real.rs` is gated by `#![cfg(feature = "model")]`. When the model is absent it fails with "run
`sloperator model pull`" (architecture §11).

---

## 24. Resolved Design Questions

| # | Question | Resolution |
|---|----------|------------|
| DQ-0 | Brad's PRD-gate answers | Built exactly as given: Q1 (a) one self-contained crate (now the repository root, D-2); Q2 (a) explicit `model pull`, exit 2 when the model is missing; Q3 (a) verified model rewrites written by `--fix`, with `--diff` available; Q4 (a) Qwen3-4B Q4_K_M as default after the spike; Q5 (a) `sloperator.toml` with snake_case keys; Q6 the CLI surface as written; Q7 a missing tool prints a one-line banner and the target fails (§3, FR-1.9); Q8 MIT; fine-tuning deferred; in-process runtime only. His follow-up on distribution binaries is OD-1. |
| DQ-1 | Integration-tier JUnit location (gate DEFER 1) | `target/nextest/it/junit.xml`, written by the `it` nextest profile (§2) that `make test-integration` runs (§3). The unit tier writes `target/nextest/ci/junit.xml` and the perf pass `target/nextest/ci-perf/junit.xml`. |
| DQ-2 | How do test selectors reach nextest? (gate DEFER 2) | `make test TESTS='<selector> ...'`: the `Makefile` reads `TESTS` through `$(origin TESTS)` (so an unset `TESTS` adds no filter) and maps the words to one nextest filterset (§3). Probed under GNU make (§3). |
| DQ-3 | FR-9.5 "no more" vs PRD Risk 2 "does not lower" (gate DEFER 3) | **Strictly fewer** weighted findings in the window. A rewrite that does not lower the count adds unverifiable risk (names, polarity) for no gain. |
| DQ-4 | Severity and exit behaviour of a verification failure (gate DEFER 4) | SLP901 `rewrite-rejected` is always `warning`. It cannot be configured, suppressed or filtered by `--min-severity`, and it always counts toward exit 1. `--exit-zero` forces exit 0, but the finding is still printed, so nothing is silent (FR-8.7). SLP902 `window-skipped` behaves the same way. Both are excluded from the score. |
| DQ-5 | Windows the model cannot take | Skipped with SLP902 when the source contains `⟦`/`⟧`, or when the masked window is over `model.max_window_chars` (default 4000, about 1,000 tokens; context 4,096). Never silent (FR-9.7). |
| DQ-6 | A local GGUF outside the registry (ADR-08 follow-up) | `model.path` together with a required `model.profile`. There is no sha256 check, because the file is the user's own. |
| DQ-7 | Placeholder glyphs (ADR-04 follow-up) | `⟦n⟧` (U+27E6 / U+27E7), numbered from 1 in source order. Input that already contains these glyphs is not sent (DQ-5). |
| DQ-8 | Meaning of `protect.html = false` | Rules see the text inside HTML, so they can report findings there. Edits inside HTML stay forbidden. |
| DQ-9 | Key case in `sloperator config` output | snake_case, matching the file. Report JSON stays camelCase (architecture §6). |
| DQ-10 | Writes when another file errors | Nothing is written if any exit-2 error occurred in the run. This is stricter than NFR-5 and matches architecture §7.3. |
| DQ-11 | Stdin with `--fix`/`--diff`; `--fix` together with `--diff` | Content goes to stdout and the report to stderr, as Ruff does. `--fix` with `--diff` is a usage error (exit 2). |
| DQ-12 | Exit code in fix/diff mode | Follows the findings that *remain* after fixing, plus any SLP9xx (Ruff behaviour). The JSON carries `scoreBefore` and `score`. |
| DQ-13 | An explicit file that matches `exclude`/`.gitignore` | Skipped (FR-2.4) with a note on stderr. A missing path or a glob that matches nothing exits 2. |
| DQ-14 | Default seed and sampling | Seed 42. Sampling follows the Qwen3 card's non-thinking settings via the prompt profile, and `/no_think` is appended. An empty `<think></think>` block is stripped. |
| DQ-15 | Supply-chain lint (NFR-7 leaves the choice to the SPEC) | `cargo-deny` 0.20.2 in `make check`, configured by the `deny.toml` in §2. |
| DQ-16 | Colour detection | `std::io::IsTerminal` plus a `NO_COLOR` check in `main`. `report` takes `color: bool`. `anstream` is not used. |

---

## Open Decisions

| # | Question | Options (consequence each) | Recommendation |
|---|----------|----------------------------|----------------|
| OD-1 | **Brad asked: "Q7 is really about building the project, right? We should be able to build binaries for distribution on any machine, correct?"** **Part one: yes.** Q7/FR-1.9 only governs what the `Makefile` does on a machine that has no Rust: it prints a one-line banner naming the missing tool and fails. It places no limit on distribution. A built `sloperator` is a self-contained native executable: users need no Rust, and only the model file stays external (D-5). llama.cpp links statically by default; `llama-cpp-2`'s `dynamic-link` feature is opt-in. **Part two is still open.** A binary is built per OS and CPU architecture, and building one always needs Rust plus clang/cmake (§0). The PRD does not say which targets ship or how. | (a) **One native build per target in CI.** A GitHub Actions matrix covers macOS arm64 (`metal`), Linux x86_64/arm64 and Windows x86_64 (CPU), and produces release archives. Inside this repo, `make build` produces only the host binary. *Consequence:* reliable, and Metal works. It costs CI minutes and extends `.github/workflows/ci.yml`, outside these stories. (b) **Cross-compile every target from one host** (`cargo-zigbuild`/`cross`). *Consequence:* one build machine, but llama.cpp's cmake cross-builds are fragile, and Metal cannot be built off macOS. (c) **Source only** (`cargo install --path`). *Consequence:* no release engineering, but every user needs Rust, clang and cmake. | **(a).** It is the only option that ships a GPU-capable macOS binary plus CPU binaries everywhere without a fragile cross toolchain. v1 in this repo needs nothing beyond the host build already specified. If Brad wants archives sooner, a `make package` target is a one-story addition. |

---

## PRD Deviation Notes

- **FR-1.4:** The PRD names only `target/nextest/ci/junit.xml`. Unit-tier results go there. The perf pass writes
  `target/nextest/ci-perf/junit.xml` and the integration tier `target/nextest/it/junit.xml` (DQ-1), one file per
  nextest profile.
- **FR-9.5:** The PRD says "no more" weighted findings. The SPEC requires *strictly fewer* (DQ-3), which follows
  PRD Risk 2. The acceptance is stricter, never looser.
- **D-4 / FR-9.2 (bullet spam, over-bolding):** D-4 says these are "fixed by the model engine only". The SPEC
  classes SLP010/SLP011 as `none`, and they select no model window (§10, §16). Any fix would change list structure
  or block markup. FR-8.2/8.6 forbid that, so verify would reject every attempt. Negative parallelism and tricolons
  stay model-fixed.
- **NFR-5:** The per-file rule still holds, and the SPEC is stricter: nothing is written when any file in the run hit
  an exit-2 error (DQ-10).
- **FR-3.5 with stdin `--fix`/`--diff`:** The report goes to stderr because stdout carries the content (DQ-11).

---

## Architecture Deviation Notes

- **§4.8 edit / ADR-05:** The architecture places `Verified<T>` in `edit`, with a constructor visible
  `pub(in crate::verify)`. Rust rejects this: a `pub(in path)` must name an ancestor module (E0742), and
  `crate::verify` is not an ancestor of `crate::edit`. The SPEC defines `Verified` in `verify` with a private
  constructor (§8), and `edit::apply` takes `&[crate::verify::Verified<Proposal>]`. The guarantee is unchanged.
  *Fixed in the architecture this pass* (§4.8, §4.9, §5 row).
- **§4.6 rules / §4.8 edit (Gap 3):** The architecture put `RuleCode` in `rules` and `Proposal` in `edit`, and it
  forbade each module from using the other. But `Finding.fix` needs `Proposal`, and `ProposalOrigin::Rule` needs
  `RuleCode`. Both now live in a leaf module, `src/types.rs`, which uses no `crate::` module; `rules` and `edit`
  re-export them (§9, §7). *Fixed in the architecture this pass* (§4.6, §4.8, §5).
- **§4.15 / ADR-07 guard (Gap 1):** One missing-tool guard covers every target that needs a tool (`build`, `test`,
  `test-integration`, `check`): banner on stderr, exit non-zero (§3). There is no separate dependency-fetch target;
  cargo fetches as part of each `--locked` build. *Recorded in ADR-07 and §4.15.*
- **§4.3 input / §4.4 doc:** To keep "doc depends on nothing internal", `Format` and `ProtectSettings` are defined
  in `doc` (§6). `input` and `config` re-export them.
- **§4.9 verify / §4.6 rules:** `Check` gains `Protected` and `Document` (§8). A second system code is added:
  SLP902 `window-skipped` (DQ-5).
- **§4.10 engine:** `RewriterFactory` gains `preflight`. This keeps FR-9.6 (a missing model or a model-less build
  exits 2 even when no window needs the model) and FR-9.4 (lazy load). `RewriteRequest` carries the
  `PromptProfile`. `seed` is a `u32`, the type `LlamaSampler::dist` takes.
- **§4.1 cli:** `run` takes `stdin: &mut dyn Read`. `Deps` holds `cwd` and `color` instead of `now_seed`, because
  a clock-derived seed would break FR-9.8.
- **§9 tech stack:** `anstream` is dropped (DQ-16). `mistralrs` is at 0.8.1 on crates.io, not "0.9.x". *Corrected in
  the architecture this pass.*
- **§12 phases:** The spike-independent model code (registry, `pull`, port, windows, masking, FakeRewriter tests)
  may be built in Phase 5, so the unit-tier `fix_without_model_exits_two` test has its code. Phase 6 keeps
  `engine::runtime` and the integration tier, and remains gated on Phase 1.
- **§2 row "Make targets":** The targets are `build`, `test`, `test-integration`, `check`, `clean` and `help`, all
  written in the crate's own `Makefile` (§3). No other make target exists.
---

## Self-Audit

- **FR coverage:**
  - FR-1: 1.1→§2; 1.2→§3; 1.3→§3; 1.4→§3; 1.5→§3; 1.6→§3; 1.7→§3, §23.19; 1.8→§5; 1.9→§3.
  - FR-2: 2.1→§17, §20; 2.2→§14; 2.3→§14; 2.4→§14; 2.5→§7, §20; 2.6→§19, §20; 2.7→§20; 2.8→§20; 2.9→§20;
    2.10→§20; 2.11→§13; 2.12→§9, §20; 2.13→§13; 2.14→§15; 2.15→§4, §13, §20; 2.16→§20.
  - FR-3: 3.1→§18; 3.2→§18; 3.3→§18; 3.4→§18; 3.5→§18, §20; 3.6→§14, §18.
  - FR-4: 4.1→§9; 4.2→§10; 4.3→§10; 4.4→§6, §9; 4.5→§9; 4.6→§10; 4.7→§9, §17.
  - FR-5: 5.1→§11, §2; 5.2→§11, §18; 5.3→§11; 5.4→§2.
  - FR-6: 6.1–6.5→§12; 6.6→§7, §8, §12; 6.7→§6, §12.
  - FR-7: 7.1–7.3→§13; 7.4→§6, §13; 7.5–7.6→§13.
  - FR-8: 8.1–8.3→§6; 8.4→§7; 8.5–8.6→§8; 8.7→§8, §17; 8.8→§14; 8.9→§7, §10.
  - FR-9: 9.1→§15, §21; 9.2→§16; 9.3→§16; 9.4→§16, §17; 9.5→§8, §17; 9.6→§15, §16; 9.7→§16, §21; 9.8→§16, §21;
    9.9→§1; 9.10→§2, §15.
  - FR-10: 10.1→§22; 10.2→§22.
- **NFR coverage:** NFR-1→§3, §23.2; NFR-2→§2, §5; NFR-3→§3, §8, §22, §23.18; NFR-4→§4, §13, §14, §16;
  NFR-5→§19, §20; NFR-6→§11, §14, §23.17; NFR-7→§2, §3; NFR-8→§23; NFR-9→§18 (`schemaVersion`) and the
  no-shim rule.
- **Component coverage:**
  - §4.1→§20; §4.2→§13; §4.3→§14; §4.4→§6; §4.5→§12; §4.6→§9, §10; §4.7→§11; §4.8→§7; §4.9→§8; §4.10→§16, §21;
    §4.11→§17; §4.12→§18; §4.13→§19; §4.14→§15; §4.15→§2, §3, §5.
  - Phase 0→§0; Phase 1→§1; Phase 7→§22.
  - ADR-01→§2, §5; ADR-02→§2, §16, §21; ADR-03→§6, §7; ADR-04→§16; ADR-05→§8; ADR-06→§9, §10; ADR-07→§3;
    ADR-08→§15; ADR-09→§17, §20.
- **Deviations:** 5 PRD notes and 8 architecture notes. The gate's four DEFERs are resolved as DQ-1, DQ-2 (probed), DQ-3 and DQ-4.
- **Self-review (passes 0–5):**
  - **Grounding.** Every build and test claim describes the crate's own files (`Makefile`, `.config/nextest.toml`,
    `.github/workflows/ci.yml`, `tests/standalone.rs`). The make logic was executed. The Rust code was checked against docs.rs signatures but **not compiled**,
    because no toolchain is installed. That is the main residual risk; the Phase 2 gate is the first compile.
  - **Consistency.** These were checked across sections:
    - where `Verified` lives (§7, §8, notes);
    - `seed: u32` (§13, §16, §20, §21);
    - `preflight` and `LazyRewriter::new` (§16, §17, §20);
    - SLP901/902 (§8, §9, §12, §16, §17, §18, DQ-4/5);
    - `protected_strings` as a sorted multiset (§6, §8, §16);
    - the 16 rules and their fix classes (§10, §16, PRD note);
    - the per-profile JUnit paths (§2, §3, DQ-1);
    - `anstream` absent from §2.
  - **Defects found and fixed in this pass:**
    - E0742 on `Verified`;
    - tempfile's 0600 mode;
    - FR-9.6 vs lazy load (fixed by `preflight`);
    - placeholder reordering vs sequence comparison;
    - `deps`/`deploy` in the architecture.
  - **Bodies left unwritten.** Two bodies are specified in prose and pinned by tests rather than written: the `doc`
    event walk (§6) and the `pull` streaming loop (§15).
- **For the adversarial review:**
  1. DQ-3: is strictly-fewer the right acceptance rule?
  2. DQ-4: should SLP901 bypass `--min-severity`?
  3. Is bullet spam correctly classed `none`?
  4. Should the model be leaked for a `'static` context borrow (§21)? **Answered: no.** `LlamaRewriter` owns the
     model and drops it before the backend, and each window builds a fresh context that borrows it. A model still
     alive at `process::exit` trips ggml's Metal teardown assert (`GGML_ASSERT([rsets->data count] == 0)`, exit 134).
  5. Is it right that the `count` check fails a selector that matches nothing?
  6. Is `max_window_chars = 4000` a sound value before spike data exists?

---

## Revision Log

### Revision 1: adversarial review round 1 (`docs/gap-analysis.md`)

| Gap | Severity / disposition | Status | Where |
|-----|------------------------|--------|-------|
| 1 | HIGH FIX-SPEC | **Fixed.** A missing tool must never be silent, and a dependency fetch must not be told apart from a direct build by guesswork. The `Makefile` uses one guard, `need`, for every target: it prints `sloperator: '<tool>' not found on PATH — <hint>` and the recipe exits 1. There is no separate fetch target. §23.2 gains the check "no cargo: `make test`/`build`/`check` print the banner and exit non-zero". ADR-07 and architecture §4.15 are updated. | §3 (`need`, design notes), §23.2, Architecture Deviation Notes |
| 2 | HIGH FIX-SPEC | **Fixed.** §10 now states the safe-fix rule: a fix must never depend on part of speech or subject agreement that the matcher cannot see. SLP001 fixes only the verb inflections `leverages`/`leveraged`/`leveraging`. Bare "leverage" fires with no fix, which leaves it to the model. SLP006 fixes only two forms: `serves/stands/functions as` → "is", and a base form after a modal or "to" → "be". Every other form gets no fix. The failing contexts are added to the fixtures and to `safe_fix_contexts_grammatical`. | §10 table + design note |
| 3 | MEDIUM FIX-SPEC | **Fixed.** A new leaf module, `src/types.rs`, holds `RuleCode`, `Proposal` and `ProposalOrigin`. `rules`, `edit` and `verify` import from it, so neither forbidden edge exists. Added a layering row and a `PURE_DIRS` entry. | §4 (`pub mod types`), §5, §7, §8, §9, Architecture Deviation Notes |
| 4 | MEDIUM FIX-SPEC | **Fixed.** The host-name needles are assembled from fragments, so no needle appears literally. The scan still covers `standalone.rs`, with no exemption. The new `planted_reference_is_caught` proves a real reference is still found. | §5 |
| 5 | MEDIUM FIX-SPEC | **Fixed.** The walker starts from `standard_filters(false)` and sets every flag explicitly: `hidden(false)` (`.git` is skipped by `filter_entry`), `ignore(false)`, `git_ignore`/`parents`/`git_exclude` = `respect_gitignore`, `git_global(false)`, `require_git(false)`. Signatures checked on docs.rs this pass. Two new tests. | §14, §23.10 |
| 6 | MEDIUM FIX-SPEC | **Fixed.** A Markdown directive counts only when one `Protected(Html)`/`HtmlBlock` span covers the whole line. A plain-text directive counts only on a line that `plain::parse` tagged as `Directive`. Lines in code fences and inline code never count. Added `directive_in_code_fence_ignored` and `directive_in_inline_code_ignored`. | §12, §23.8 |
| 7 | MEDIUM FIX-SPEC | **Fixed.** A new `Wrap { width, prefix, newline }` carries the line layout. Continuation prefixes (`> `, indentation) are stripped before masking and re-inserted on every re-wrapped line. Inconsistent prefixes → SLP902. `protected_strings` counts a block's prefix spans once per block. Two new tests. | §16, §6, §23.12 |
| 8 | LOW FIX-SPEC | **Fixed.** Each JSON finding carries `path`. | §18 example + design note |
| 9 | LOW FIX-SPEC | **Fixed.** `RUN_PERF` skips the release perf run when `TESTS` is set and no word in it contains `perf`. Probed: empty for `verify`; set for `…perf#x` and when `TESTS` is unset. Added a §23.2 check. | §3, §23.2 |
| 10 | LOW FIX-ARCHITECTURE | **Fixed in the architecture.** §4.6 and §4.8: the types home and dependency lines. §4.9: the `edit` signature takes `frozen` + `ProtectSettings`; `Tokens` has no `placeholders`; `Check` adds `Protected`/`Document`; `rules` is gone from its dependencies; `improvement` is added. §4.10: `Wrap`, `slots`, `preflight`, `profile`, `seed: u32`, and the `propose` signature. §5 graph: `verify` no longer depends on `rules` or `edit`, and `types` is added. §4.15 and ADR-07: the `binary_id#test` selector and the missing-tool guard. | `ARCHITECTURE-sloperator.md`, `adr/ADR-07-nextest-junit-bridge.md` |
| 11 | DECIDE | Left for Brad; OD-1 unchanged. | Open Decisions |
| 12 | LOW DEFER | Not addressed here. Deferred to `deferred.md` item 5: the story that carries §23.18 fixes the mechanism. | — |
| 13 | LOW ACCEPT | No change. | — |

Every section touched in this pass was re-checked against its neighbours:
- the `types` imports (§4, §5, §7, §8, §9);
- the SLP001/SLP006 classes against §16 window selection: a Safe finding with no fix still selects a window;
- `Wrap` (§16) against `protected_strings` (§6);
- §3 against §23.2.

The Rust is still uncompiled (no toolchain; see the header note). The make changes were executed.

### Revision 2: adversarial review round 2 (`docs/gap-analysis.md`, Round 2)

| Gap | Severity / disposition | Status | Where |
|-----|------------------------|--------|-------|
| 14 | HIGH FIX-SPEC | **Fixed.** "serves/stands/functions as" → "is" now fires its fix only when the preceding token in the same sentence is in the fixed guard list `SINGULAR_SUBJECT` (`it`, `this`, `that`, `which`, `who`, `he`, `she`). That is safe-fix rule item 3. After any other token, including a noun subject, the rule fires with no fix and leaves the window to the model, because each of the three words is also a plural noun. The `fixed.md` pairs are now "It serves as a bridge" and "which functions as a buffer". "export these functions as a module", "we expose helper functions as a convenience" and "The tool serves as a bridge" are listed as fires-with-no-fix and asserted in `safe_fix_contexts_grammatical`. | §10 SLP006 row + design note |
| 4 | MEDIUM FIX-SPEC (still open) | **Fixed.** The identifiers are renamed to `HOST_MARKERS` and `no_host_references`. The file comment now states that no text in `standalone.rs` may contain the banned word in any case, because the scan is case-insensitive. The scan still covers the file with no exemption, so `planted_reference_is_caught`'s claim that the clean copy reports nothing now holds. The architecture §5 rule table is renamed to match. | §5; `ARCHITECTURE-sloperator.md` module-rules table |
| 9 | LOW FIX-SPEC (still open) | **Fixed.** The perf nextest run in `make test` is wrapped in `if [ -n '$(RUN_PERF)' ]`. The `else` branch prints a stderr skip banner. A probe under the `Makefile`'s shell flags ran the branch for no `TESTS` and for `…perf#x`, and skipped it for `verify`. All three exited 0. | §3 recipe + design notes |
| 15 | MEDIUM FIX-SPEC | **Fixed.** §23.2 now has both rows: no cargo → `make test`/`build`/`check` print the banner and exit non-zero; and `TESTS='verify'` → no `ci-perf` run and no perf XML. | §23.2 |
| 11 | DECIDE | Left for Brad; OD-1 unchanged. | Open Decisions |

Re-checked after these edits:
- §10 SLP006 against §9 `WordList`. The guard is a preceding-token check, as rule item 3 allows. `WordList` entries for the three forms carry `Some("is")`, and the rule drops the fix when the guard fails.
- §10 against §16. A Safe finding with no fix still selects a model window.
- The §3 recipe against `count`. A skipped perf pass writes no XML, and the unit suite's XML still counts.
- §5 against the architecture rule table.

No identifier, comment or string in `tests/standalone.rs` contains the host project's name in any case.

### Revision 3: adversarial review round 3, final pass (`docs/gap-analysis.md`, Round 3)

| Gap | Severity / disposition | Status | Where |
|-----|------------------------|--------|-------|
| 16 | MEDIUM FIX-SPEC | **Fixed.** Both SLP006 fixes ("serves/stands/functions as" → "is" after `SINGULAR_SUBJECT`, and the modal/"to" base form → "be") now also require the next word token after "as" to be in the allow list `AS_COMPLEMENT` (`a`, `an`, `the`, `its`, `his`, `her`, `their`, `our`, `your`, `my`, `one`, `another`). Each entry opens a noun phrase, so the result is grammatical. "follows", "well", "long", "soon", "much", "far", "such", "if" and "though" are not in the list, so "as follows", "as well as", "as long as" and similar phrases fire with no fix. When no next token exists in the same `BlockText`, the fix is dropped. Safe-fix rule item 3 now allows a following-token guard and requires an allow list where the unsafe set is open. I chose the allow list over the deny list the gap also offered, because a deny list misses any phrase nobody thought of. "It functions as follows", "It serves as well as the old one" and "This can serve as well as any" are added as fires-with-no-fix fixtures and asserted in `safe_fix_contexts_grammatical`. The modal `fixed.md` pair becomes "can serve as the templates" → "can be the templates". The bare-plural form "can serve as templates" now fires with no fix, which costs a little coverage and leaves that case to the model. | §10 SLP006 row; §10 design note (safe-fix rule item 3, SLP006 fixtures, `AS_COMPLEMENT`) |
| 11 | DECIDE | Left for Brad; OD-1 unchanged. | Open Decisions |

No FIX-ARCHITECTURE item was open, so the architecture is unchanged. The architecture does not name SLP006 or its guards.

Re-checked after these edits:
- The §10 SLP006 row against the design note. Both fix forms name `AS_COMPLEMENT`, and every `fixed.md` pair has an allowed word after "as": "the", "a", "a", "a".
- The fires-with-no-fix list against the guards. Each entry fails `SINGULAR_SUBJECT` or `AS_COMPLEMENT`, or has no fix form ("served as").
- §10 against §16. A Safe finding with `fix: None` still selects a model window, so the newly unfixed contexts go to the model engine.
- §10 against §9 `WordList`. The new guard is a token-list check on `BlockText`, the same kind of check as the preceding-token guard. It adds no new `WordList` shape.
