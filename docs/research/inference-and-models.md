# Deep Dive: Local SLM Inference, Candidate Models and Language Choice (sloperator)

**Date:** 2026-10-22
**Status:** Complete (documentation-level; nothing here was benchmarked)
**Related:** [Competitive Analysis](./competitive-analysis.md) | [Feasibility Summary](./feasibility-summary.md)

---

## Purpose

Answers Brad's requirements (3) and (5): which local small models are candidates, how to run them locally, whether
Rust inference is practical (so the Java/Python fallback is not needed), and whether fine-tuning is worth it later.
Every external claim cites a page opened in this session. **Limit of this file:** it is built from documentation and
model cards. No model was run and no throughput, latency or rewrite-quality number was measured. Where the sources
are silent the text says "not verified". The PRD must treat those items as spike work, not facts.

---

## 1. Is Rust SLM inference practical? Yes, three routes exist

| Route | What it is | Evidence read | Caveats found |
|-------|-----------|---------------|---------------|
| `llama-cpp-2` crate | Safe-ish Rust wrappers over llama.cpp, with a `-sys` crate for raw bindings. Dual MIT/Apache-2.0.[^llamacpp-rs] | Modules for `model`, `context`, `sampling`, `gguf`, `llama_batch`, `vocab`; chat-template application; GBNF grammar and `json_schema_to_grammar`; LoRA adapter load/set/remove error types.[^llamacpp2-docs] The repo's `simple` example loads a GGUF from Hugging Face and generates.[^llamacpp-rs] | README: needs `clang` for bindgen; "this crate is *not safe*"; "does not follow semver meaningfully" (it tracks llama.cpp).[^llamacpp2-readme][^llamacpp-rs] The docs.rs feature list names only `cuda` and `sampler`; **Metal support was not confirmed** from the page read.[^llamacpp2-docs] |
| `mistralrs` crate | In-process Rust SDK of the mistral.rs engine: `cargo add mistralrs`, `ModelBuilder` and `GgufModelBuilder`.[^mistralrs][^mistralrs-rust] | Default features build for CPU; `metal`, `cuda`, `mkl` features exist; 4-bit in-situ quantization (`with_auto_isq`) or GGUF load by exact file; GGUF reads tokenizer and chat template from the file; dynamic LoRA on GGUF; offline mode via `HF_HUB_OFFLINE=1`.[^mistralrs-rust][^mistralrs-gguf] | API is async (`tokio`, `build().await`); `build()` "is expensive. Call it once".[^mistralrs-rust] IQ-type GGUF files are not supported yet.[^mistralrs-gguf] Published benchmarks are CUDA server GPUs (GB10, B200, H100) on Gemma 4 models, not CPU or laptop numbers.[^mistralrs] The project moves fast (0.9.x; an upgrade guide for changed memory options is linked).[^mistralrs-rust] |
| `candle` | Hugging Face's minimalist Rust ML framework; CPU backend (optional MKL on x86, Accelerate on Mac), CUDA, WASM; loads safetensors, npz, ggml, PyTorch files; quantization with llama.cpp quant types.[^candle] | Model list in the README names Phi 1/1.5/2/3, Gemma v1 and v2, LLaMA v1-v3, Qwen1.5, Qwen1.5 MoE and Qwen3 MoE, and "Quantized LLMs" for Llama, Mistral, Mixtral and Qwen3 MoE.[^candle] | It is a framework, so the app owns tokenization, sampling and chat templates. **None of Phi-4-mini, Gemma 3 or dense Qwen3 appears in the README lists read**, so each would be new port work or an external crate (unverified).[^candle] |

**Conclusion:** Rust inference is practical for GGUF-quantized 1-4B models through `llama-cpp-2` or `mistralrs`.
`candle` is the weakest fit for this product, because the candidate models are not in its documented list. The
Java/Python fallback is **not triggered**. The spike (section 7) must still confirm build, size, speed and quality on
real hardware before the PRD commits to one crate.

### 1.1 What the product needs from the runtime

Design implications for the engine choice. These are the researcher's reading of the requirements, not facts from
a source:

- **Constrained output.** Rewriting must not drop code, links or numbers. A runtime with grammar or JSON-schema
  constraints makes "return only the rewritten sentences" cheaper to enforce. `llama-cpp-2` exposes a grammar sampler
  and `json_schema_to_grammar`;[^llamacpp2-docs] mistral.rs documents grammar enforcement for tool calling and
  linked examples for grammar, JSON schema and llguidance.[^mistralrs]
- **Chat template from the GGUF.** Both routes read it from the model file (mistral.rs states this
  explicitly;[^mistralrs-gguf] llama-cpp-2 has `ApplyChatTemplateError` and `ChatTemplateError`).[^llamacpp2-docs]
- **One model load per invocation.** A CLI pays load time on every run. The one data point read is from the
  llama-cpp-rs README: a 7B Q4_K_M model loaded in about 0.73 s and decoded about 106 tokens/s **on an RTX 3090 with
  CUDA**.[^llamacpp-rs] It says nothing about a laptop CPU. Treat it as "works", not as a budget.
- **No JVM dependency.** None of the three routes depends on the JVM, so the tool stays a single native binary
  with no runtime to install.

---

## 2. Candidate models

All figures come from the model pages or cards named. "Not verified" means the page read did not say.

| Model | Params | Context | Quantized artifact seen | License as read | Notes |
|-------|--------|---------|-------------------------|-----------------|-------|
| Qwen3-1.7B | 1.7B (1.4B non-embedding) | 32,768 | GGUF `Q8_0`, 1.83 GB (the only quant listed)[^qwen3-1.7b] | not shown on the page read | Switchable thinking and non-thinking mode (`/no_think`); card recommends non-thinking sampling `temp 0.7, top-p 0.8, top-k 20, presence-penalty 1.5`, and "DO NOT use greedy decoding" in thinking mode.[^qwen3-1.7b] |
| Qwen3-4B | 4.0B (3.6B non-embedding) | 32,768 native, 131,072 with YaRN | GGUF `Q4_K_M` 2.5 GB, `Q5_K_M` 2.89 GB, `Q6_K` 3.31 GB, `Q8_0` 4.28 GB[^qwen3-4b] | not shown on the page read | Same modes and sampling advice; the card recommends presence penalty 1.5 "for quantized models to suppress repetitive outputs".[^qwen3-4b] Card says it excels at "creative writing ... natural, engaging" dialogue (vendor claim).[^qwen3-4b] |
| Gemma 3 1B IT | 1.0B | 32K in, 8,192 out | GGUF not verified (the repo page shows BF16 safetensors) | Gated: must accept Google's terms; terms page not read[^gemma3] | Card cautions models "might struggle to grasp subtle nuances, sarcasm, or figurative language".[^gemma3] |
| Phi-4-mini-instruct | 3.8B | 128K | GGUF not verified (ONNX variants named on the card) | MIT[^phi4] | Trained mainly on English; card says it lacks capacity "to store too much factual knowledge" and may be factually wrong; ships a TRL fine-tuning example.[^phi4] |
| Llama 3.2 3B Instruct (and 1B) | 3.21B (1.23B) | 128k | GGUF not verified; Meta's own quantized variants target ExecuTorch/Arm[^llama32] | Custom community license: "Built with Llama" attribution, a 700M-MAU clause, and an Acceptable Use Policy; gated[^llama32] | **Policy risk for this product:** the AUP lists under "Intentionally deceive or mislead others" the item "Representing that the use of Llama 3.2 or outputs are human-generated".[^llama32] A tool whose purpose is to make text read as human sits close to that line. Whether it applies is a legal reading for Brad, not a research finding. |

**Reading across the table:**

- Size band: a 4-bit 4B model is about 2.5 GB on disk (Qwen3-4B Q4_K_M);[^qwen3-4b] a 1-2B model is roughly 1-2 GB.
  Runtime memory also includes KV cache, which was not measured.
- Qwen3 is the only family here with confirmed official GGUF files across sizes on pages read, and the only one
  whose card documents how to turn reasoning off, which matters because a rewrite step should not emit a
  thinking block. Phi-4-mini is the cleanest license (MIT) but has no official GGUF confirmed.
- Rewrite quality of any of these on "de-slop while preserving facts" is **unknown**. Cards report general
  benchmarks (for example Phi-4-mini's MMLU 67.3, Llama-3.2-3B's 61.8 in Microsoft's own table),[^phi4] none of which
  measures rewrite fidelity or style.
- Small models hallucinate facts. Two cards say so explicitly.[^phi4][^gemma3] This is the central risk for the
  "preserve facts and numbers" requirement and is why post-rewrite verification belongs in the design (section 4).

---

## 3. Running them locally

| Option | How | Evidence |
|--------|-----|----------|
| In-process, GGUF file via `llama-cpp-2` | Load a local `.gguf`; or download from Hugging Face in the example | [^llamacpp-rs] |
| In-process via `mistralrs` | `GgufModelBuilder::new(repo, [file])` or `ModelBuilder` with ISQ; offline with `HF_HUB_OFFLINE=1` | [^mistralrs-rust][^mistralrs-gguf] |
| Out-of-process server | `mistralrs serve -m <model> --quant 4` exposes OpenAI- and Anthropic-compatible endpoints; Qwen's card also documents `llama-cli` and `ollama run hf.co/...` | [^mistralrs][^qwen3-4b] |

The out-of-process option is a different product shape: sloperator would be an HTTP client of a model server the
user runs. It lowers build weight, but "both engines on by default" would then need a running server, so first
run needs setup. It goes to the PRD as OQ-11.

Weights are never bundled in the repo or binary (RD-3). Model location, download and offline behaviour are
product decisions (OQ-3).

---

## 4. Safety design directions for rewriting (not sourced, for the PRD)

The researcher's directions, labelled as design, because small models can alter or drop facts:

1. **Mask, rewrite, verify.** Parse Markdown with a CommonMark parser that yields source ranges
   (`pulldown-cmark`'s `OffsetIter`, "Markdown event and source range iterator"; MIT; CommonMark by default,
   tables, footnotes, task lists and strikethrough as opt-in `Options`)[^pulldown] and mark code blocks, inline code,
   link destinations, front matter and user-configured spans as protected. Send the model only the prose spans.
2. **Verify after.** For each rewritten span, compare the multiset of numbers, URLs and protected tokens in and out.
   On mismatch, keep the original span and report a finding. Never accept silently (project rule: fail loudly).
3. **Sentence- or paragraph-sized windows** keep prompts short for 1-4B models and bound damage.
4. **Rules fix first, then the model** only where rules cannot: "It's not X, it's Y", tricolons and bullet spam can
   be detected deterministically but a safe fix is a rewrite.

---

## 5. Fine-tuning: not worth it for v1; decide later behind a measurement

What the sources establish:

- Tooling exists: mistral.rs supports LoRA and, for GGUF, dynamic LoRA adapters;[^mistralrs][^mistralrs-gguf]
  `llama-cpp-2` has LoRA adapter load/set/remove APIs;[^llamacpp2-docs] Phi-4-mini's card ships a TRL SFT sample
  script;[^phi4] candle's README lists an external `candle-lora` crate.[^candle]
- There is no source read showing that fine-tuning improves "sounds human" for a rewrite model.
- Measuring "human" is itself weak: Wikipedia's AI-cleanup guide reports a 2025 study in which human ability to tell
  LLM text from human text was no better than chance, that detectors "have non-trivial error rates", and that LLM
  output is not converging to human writing.[^wiki-signs]

So the fine-tuning decision needs an evaluation harness first. Proposed gate (for the PRD):

1. Build the eval set: slop-containing paragraphs plus protected-content fixtures; metrics are (a) rule-engine
   slop score before and after, (b) protected-span integrity rate, (c) a small blind human preference sample.
2. Try prompt design, few-shot examples and grammar-constrained output on the untuned model first.
3. Only if the best untuned model fails a threshold Brad sets (OQ-7) is a LoRA worth the data-collection cost.
   Training data (paired slop and clean text) does not exist in the repo and must not be fabricated.

---

## 6. Language comparison (the PRD records this)

Brad: Rust preferred; fall back to Java+GraalVM native or Python "only if inference makes Rust hard".

| Axis | Rust | Java + GraalVM native | Python |
|------|------|-----------------------|--------|
| Inference binding | `llama-cpp-2`, `mistralrs`, `candle` (section 1) | `java-llama.cpp` 4.1.0 on Maven: prebuilt CPU libraries for Linux x86-64/aarch64, macOS x86-64/aarch64, Windows; GPU needs a self-build with CMake; loads a native `jllama` library; `LlamaModel` is `AutoCloseable` because native memory is not GC'd[^jllama] | `llama-cpp-python`: `pip install` builds llama.cpp from source and needs a C compiler, or install a prebuilt CPU/CUDA/Metal wheel (Python 3.10-3.12 for the wheels read)[^llamacpp-python] |
| Single self-contained binary | Yes for the Rust side; model file stays external | A JNI library under native-image needs extra configuration (not verified); native-image builds of large JVM applications are slow and produce large binaries | No: needs an interpreter, or bundling (for example Nuitka), which takes minutes per build |
| Startup cost per CLI call | Native process + model load | Native image is fast; model load same | Interpreter and import time plus model load |
| Markdown parsing | `pulldown-cmark` (source ranges)[^pulldown] | Would use a JVM Markdown library (not researched) | Many libraries (not researched) |
| Build tooling | `cargo` (plus `cargo-nextest` for tests) driven by a small `Makefile` | Maven plus GraalVM | `pyproject` plus a bundler |
| Standalone repository | `Cargo.toml` is self-contained | Needs its own JVM build | Needs its own `pyproject` |
| Verdict | **Recommended; inference does not make it hard** | Not needed | Not needed |

Rust's costs are real but not inference-related: a C++ toolchain (`clang`, cmake-style build) if `llama-cpp-2` is
chosen,[^llamacpp2-readme] and long first compiles. Neither fallback
removes the model-file requirement, so neither simplifies the product.

---

## 7. Spike (to run before the SPEC freezes the crate)

Run on Brad's actual machine class (macOS; Apple Silicon is **not** established). For `llama-cpp-2` and `mistralrs`, each with Qwen3-1.7B and
Qwen3-4B at Q4_K_M or the nearest published quant:

1. Cold `cargo build --release` time, binary size, and whether a C/C++ toolchain is needed.
2. Whether Metal (or the user's GPU path) works for each crate. `llama-cpp-2` is unconfirmed here.
3. Model load time and tokens/s for a 150-token paragraph rewrite on CPU and on GPU.
4. Whether a grammar or schema constraint keeps output well-formed and inside the prose-span contract.
5. Rewrite quality on 20 slop paragraphs: protected-span integrity (target 100%) and rule-engine slop score delta.

Pass criteria are a PRD decision. The spike is about 2-3 days and is part of Pillar 5 in the [feasibility summary](./feasibility-summary.md).

---

## References

[^llamacpp-rs]: llama-cpp-rs (utilityai), repository README. https://github.com/utilityai/llama-cpp-rs. Retrieved 2026-10-22. Establishes the workspace layout (`llama-cpp-2`, `llama-cpp-sys-2`), dual MIT/Apache-2.0 licensing, "does not follow semver meaningfully", the `simple` example loading a Hugging Face GGUF, and the one CUDA data point (7B Q4_K_M, ~106 tokens/s, ~0.73 s load on an RTX 3090).
[^llamacpp2-readme]: llama-cpp-2 crate README. https://raw.githubusercontent.com/utilityai/llama-cpp-rs/main/llama-cpp-2/README.md. Retrieved 2026-10-22. Establishes the bindgen/clang dependency and the "not safe" disclaimer.
[^llamacpp2-docs]: llama_cpp_2 (docs.rs, version 0.1.160 link on the page). https://docs.rs/llama-cpp-2/latest/llama_cpp_2/. Retrieved 2026-10-22. Establishes the module list (model, context, sampling, gguf, vocab, batch), grammar/JSON-schema helpers, chat-template and LoRA error types, and feature flags `cuda` and `sampler`.
[^mistralrs]: mistral.rs repository README. https://github.com/EricLBuehler/mistral.rs. Retrieved 2026-10-22. Establishes the Rust SDK (`cargo add mistralrs`), GGUF 2-8 bit and ISQ support, LoRA, grammar-enforced tool calling, CUDA/Metal support, and that the published benchmarks are CUDA server GPUs.
[^mistralrs-rust]: Rust SDK getting started, mistral.rs docs. https://docs.mistralrs.dev/guides/rust/getting-started/. Retrieved 2026-10-22. Establishes `mistralrs = "0.9.5"`, default CPU build with `cuda`/`metal`/`mkl` features, the async `ModelBuilder` flow, and that `build()` is expensive and should be called once.
[^mistralrs-gguf]: Run GGUF models, mistral.rs docs. https://docs.mistralrs.dev/guides/models/run-gguf/. Retrieved 2026-10-22. Establishes local/Hub GGUF loading in Rust (`GgufModelBuilder`), tokenizer and chat template read from the GGUF, dynamic LoRA on GGUF, offline mode, and no IQ-GGUF support.
[^candle]: candle repository README. https://github.com/huggingface/candle. Retrieved 2026-10-22. Establishes the backends (CPU/MKL/Accelerate, CUDA, WASM), the model lists named in the text, llama.cpp quant-type support, and the external `candle-lora`.
[^qwen3-1.7b]: Qwen/Qwen3-1.7B-GGUF model card. https://huggingface.co/Qwen/Qwen3-1.7B-GGUF. Retrieved 2026-10-22. Establishes 1.7B params, 32,768 context, `Q8_0` 1.83 GB, thinking/non-thinking switch and sampling advice.
[^qwen3-4b]: Qwen/Qwen3-4B-GGUF model card. https://huggingface.co/Qwen/Qwen3-4B-GGUF. Retrieved 2026-10-22. Establishes 4.0B params, 32,768/131,072 context, the Q4_K_M-Q8_0 file sizes, presence-penalty advice for quantized models, and the `llama-cli`/`ollama` run commands.
[^gemma3]: google/gemma-3-1b-it model card. https://huggingface.co/google/gemma-3-1b-it. Retrieved 2026-10-22. Establishes 1.0B params, 32K input / 8,192 output context, the gated-license requirement, and the stated limitations.
[^phi4]: microsoft/Phi-4-mini-instruct model card. https://huggingface.co/microsoft/Phi-4-mini-instruct. Retrieved 2026-10-22. Establishes 3.8B params, 128K context, MIT license, English-primary training, the stated factual-knowledge limit, benchmark table, and the TRL fine-tuning sample.
[^llama32]: meta-llama/Llama-3.2-3B-Instruct model card and license text. https://huggingface.co/meta-llama/Llama-3.2-3B-Instruct. Retrieved 2026-10-22. Establishes 3.21B/1.23B params, 128k context, the Community License terms (attribution, 700M-MAU clause) and the Acceptable Use Policy item on representing outputs as human-generated.
[^jllama]: java-llama.cpp repository README. https://github.com/kherud/java-llama.cpp. Retrieved 2026-10-22. Establishes the Maven artifact 4.1.0, prebuilt CPU platforms, GPU self-build, the native `jllama` library, and `AutoCloseable` memory handling.
[^llamacpp-python]: llama-cpp-python repository README. https://github.com/abetlen/llama-cpp-python. Retrieved 2026-10-22. Establishes the source build needing a C compiler, the prebuilt CPU/CUDA/Metal wheels and their Python-version limits.
[^pulldown]: pulldown_cmark crate docs. https://docs.rs/pulldown-cmark/latest/pulldown_cmark/. Retrieved 2026-10-22. Establishes the CommonMark pull parser, `OffsetIter` (events with source ranges) and opt-in extensions; version 0.13.6 and MIT license appear in the page's link metadata.
[^wiki-signs]: Wikipedia:Signs of AI writing. https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing. Retrieved 2026-10-22. Establishes the caveats on detector error rates and on human detection ability (2025 studies cited by the page) and that LLM output is not becoming more human-like.
