# Spike results — sloperator inference

Machine: Apple M3 Max (arm64), 48 GB RAM, integrated GPU (Metal), macOS 26.5.2      Toolchain: rustc 1.99.1, Apple clang 21.0.0, cmake 4.4.5

Harnesses: throwaway crates in `$TMPDIR/sloperator-spike/{llama,mist}/`, not committed (architecture §10). Each one masks protected runs (inline code, link destinations, bare URLs) to `⟦n⟧`, builds the SPEC §16 prompt (system text plus the two few-shot pairs, Qwen ChatML, thinking switched off), generates greedily, applies the output contract and the placeholder check, then unmasks. Models were downloaded from Hugging Face; both file sizes and sha256 values equal the Hugging Face listing.

| File | Bytes | sha256 |
|------|-------|--------|
| Qwen3-4B-Q4_K_M.gguf | 2,497,280,256 | `7485fe6f11af29433bc51cab58009521f205840f5b4ae3a32fa7f92e8534fdf5` |
| Qwen3-1.7B-Q8_0.gguf | 1,834,426,016 | `061b54daade076b5d3362dac252678d17da8c68f07560be70818cace6590cb1a` |

## Crate × model

| Crate @version | Model file | Device | Cold build (s) | Binary (MB) | GPU used | Load (s) | tok/s (150-token rewrite) |
|---|---|---|---|---|---|---|---|
| llama-cpp-2 =0.1.160 | Qwen3-4B-Q4_K_M.gguf | CPU | 32.1 | 12.17 | no (`n_gpu_layers=0`) | 3.22 | 36.2 |
| llama-cpp-2 =0.1.160 (feature `metal`) | Qwen3-4B-Q4_K_M.gguf | Metal | 27.7 | 12.17 | yes (`n_gpu_layers=99`) | 0.21 | 80.5 |
| llama-cpp-2 =0.1.160 | Qwen3-1.7B-Q8_0.gguf | CPU | 32.1 | 12.17 | no | 2.33 | 56.2 |
| llama-cpp-2 =0.1.160 (feature `metal`) | Qwen3-1.7B-Q8_0.gguf | Metal | 27.7 | 12.17 | yes | 0.17 | 129.2 |
| mistralrs =0.8.1 | Qwen3-4B-Q4_K_M.gguf | CPU (`with_force_cpu`) | 99.1 | 64.81 | no | 2.34 | 9.5 |
| mistralrs =0.8.1 (feature `metal`) | either | Metal | build FAILED after 45 s | — | — | — | not measured |
| mistralrs =0.8.1 | Qwen3-1.7B-Q8_0.gguf | CPU (`with_force_cpu`) | 99.1 | 64.81 | no | 2.07 | 13.7–16.0 |

How to read the numbers (every one comes from a run in this session):

- **Cold build** is `cargo build --release` into an empty `target/` with the registry already fetched; the Metal build used a second empty target directory. The llama-cpp-2 CPU build is 32.1 s and the Metal build 27.7 s. The mistralrs figure is 96.1 s plus 3.1 s: my first attempt compiled all dependencies and then failed on two typos in my own harness, so the second invocation only relinked. The sum is the cold cost.
- **Binary** is the size of the spike binary, not of sloperator: llama-cpp-2 12,171,568 bytes (identical with and without `metal`), mistralrs 64,810,768 bytes. The table column is MB (1 MB = 1,000,000 bytes).
- **tok/s** is the mean of the timed runs after one warm-up (llama-cpp-2) or the per-run `avg_compl_tok_per_sec` (mistralrs). The rewrite of p01 stopped at the end-of-generation token after 74–119 tokens, so no run was a full 150 tokens; decode speed is steady-state, so the rate is comparable.
- **GPU used** is what I set (`n_gpu_layers`, `with_force_cpu`) plus the observed speed-up (Metal is 2.2× CPU for the 4B model). I did not read GPU counters.
- **mistralrs Metal failed to build**: `mistralrs-paged-attn-0.8.1/build.rs:210` panicked with `cannot execute tool 'metal' due to missing Metal Toolchain; use: xcodebuild -downloadComponent MetalToolchain`. That is a gap in this machine's Xcode install, not proof that the crate cannot use Metal, but it leaves the row unmeasured. I did not install the toolchain. `llama-cpp-2`'s `metal` feature built on the same machine without it.
- If `mistralrs` had been picked it needs `tokio` `^1.49.0` with features `full` and `rt-multi-thread` (`mistralrs-0.8.1/Cargo.toml:337-343`); the spike lock resolved 1.54.1. The pick below is `llama-cpp-2`, so epic 07 does not change the `model` feature line in `Cargo.toml`.

## 20-paragraph results (per crate × model)

Paragraphs are `eval/paragraphs/p01.md … p20.md`. "Findings" is a crude marker-phrase count in the harness, because the rule engine (epic 04) does not exist yet. It is a relative signal, not a SLP finding count. "Numbers/URLs kept" means every number outside the placeholders appears in the unmasked output, and link destinations and bare URLs, which are masked, survive because a rewrite that keeps its placeholders unmasks them verbatim (checked in the integrity row). "Output contract" is the SPEC §16 check: empty, `<think>`, preamble, quote or fence, blank line, longer than 2× the masked window. "Accepted" means contract ok, placeholders kept exactly once each, numbers kept.

Chosen pair, llama-cpp-2 =0.1.160 + Qwen3-4B-Q4_K_M, Metal, plain sampler:

| Paragraph | Findings before | After | Placeholders kept | Numbers/URLs kept | Output contract | Accepted |
|---|---|---|---|---|---|---|
| p01 | 11 | 11 | 1/1 | yes | ok | yes |
| p02 | 2 | 2 | 1/1 | yes | ok | yes |
| p03 | 0 | 0 | 1/1 | yes | ok | yes |
| p04 | 5 | 0 | 1/1 | yes | ok | yes |
| p05 | 5 | 1 | n/a (no protected run) | yes | ok | yes |
| p06 | 6 | 0 | 1/1 | yes | ok | yes |
| p07 | 0 | 0 | 2/2 | yes | ok | yes |
| p08 | 6 | 1 | 1/1 | yes | ok | yes |
| p09 | 8 | 7 | 1/1 | yes | ok | yes |
| p10 | 6 | 6 | 1/1 | yes | ok | yes |
| p11 | 9 | 0 | 1/1 | yes | ok | yes |
| p12 | 2 | 0 | 1/1 | yes | ok | yes |
| p13 | 2 | 0 | 1/1 | yes | ok | yes |
| p14 | 2 | 1 | 1/1 | yes | ok | yes |
| p15 | 5 | 0 | 1/1 | yes | ok | yes |
| p16 | 4 | 2 | 2/2 | yes | ok | yes |
| p17 | 0 | 0 | 2/2 | yes | ok | yes |
| p18 | 3 | 2 | 1/1 | yes | ok | yes |
| p19 | 10 | 1 | 2/2 | yes | ok | yes |
| p20 | 7 | 1 | 1/1 | yes | ok | yes |

The "before" column is the sum 93 over all 20 paragraphs; "after" is 35 for this run. p01, p09 and p10 are barely changed: the model left them nearly as written. That is a quality observation for epic 06 and 08 (acceptance is about safety, not about whether the rewrite helps).

Summary for every pair that ran the paragraphs (each a full 20-paragraph run):

| Crate | Model | Device / sampler | Accepted | Protected-span integrity after verify | Placeholders kept (slots) | Output-contract violations | Mean s per window (max) |
|---|---|---|---|---|---|---|---|
| llama-cpp-2 | 4B Q4_K_M | Metal, plain | 20/20 | 20/20 | 23/23 | 0 | 0.66 (0.79) |
| llama-cpp-2 | 4B Q4_K_M | Metal, grammar | 20/20 | 20/20 | 23/23 | 0 | 1.26 (1.58) |
| llama-cpp-2 | 4B Q4_K_M | CPU, plain | 20/20 | 20/20 | 23/23 | 0 | 1.30 |
| llama-cpp-2 | 1.7B Q8_0 | Metal, plain | 19/20 | 20/20 | 22/23 | 0 | 0.42 (0.50) |
| llama-cpp-2 | 1.7B Q8_0 | Metal, grammar | 19/20 | 20/20 | 22/23 | 0 | 1.06 |
| llama-cpp-2 | 1.7B Q8_0 | CPU, plain | 19/20 | 20/20 | 22/23 | 0 | 0.88 |
| mistralrs | 4B Q4_K_M | CPU | 20/20 | 20/20 | 23/23 | 0 | 8.29 (13.83) |
| mistralrs | 1.7B Q8_0 | CPU | 18/20 | 20/20 | 21/23 | 0 | 5.59 (9.23) |

Rejected paragraphs (all for a dropped placeholder; verify then keeps the original text, so integrity stays 20/20): the 1.7B model with llama-cpp-2 loses p12's `⟦1⟧` (the link destination, so the rewrite reads "See the importer notes" with the link gone); with mistralrs it loses the same one and, in p07, one of the two. The 4B model lost none with either crate.

Protected-span integrity after verify: 20/20 for the chosen pair (and for every pair listed, because a rejected rewrite leaves the source untouched)
Placeholder survival before verify: 23/23 placeholders (4B, every device and both samplers); 22/23 (1.7B llama-cpp-2); 21/23 (1.7B mistralrs). Option 1 (ADR-04) holds for the 4B model.
Grammar constraint (LlamaSampler::grammar) tried: yes — `root ::= [^\n<]+ "\n"?`, which encodes "one block, no blank line, no `<think>`". Effect on contract violations: none (0 before, 0 after on both models), and no change in acceptance. It roughly doubled the time per window (4B Metal 0.66 s to 1.26 s; 1.7B Metal 0.42 s to 1.06 s), so it is not worth enabling.

Limits of this evidence: one greedy run per paragraph (the sampler is deterministic, so a rerun on the same device gives the same text; Metal and CPU can differ in the last bits and did differ in wording); 20 paragraphs written for this project; only the three protected-run kinds the harness masks (inline code, link destination, bare URL) were exercised, and the real masker (epic 06) also covers inline HTML, math and emphasis markup.

## Licence reading (PRD Question 4)

Qwen3-4B-GGUF (`https://huggingface.co/Qwen/Qwen3-4B-GGUF`): the repository `LICENSE` file, fetched this session from `…/raw/main/LICENSE` (201 lines), is the Apache License, Version 2.0, and the model-card metadata reads `license: apache-2.0`. The file opens:

```
                                 Apache License
                           Version 2.0, January 2004
                        http://www.apache.org/licenses/

   TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION
```

and its appendix notice, which names the licensor, reads:

```
   Copyright 2025 Alibaba Cloud

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
```

Apache-2.0 permits use, redistribution and use in a tool distributed under MIT. sloperator does not bundle the model: `sloperator model pull` downloads it on the user's machine (ADR-08), so the licence terms (including carrying the licence and notices if the file is redistributed) fall on whoever redistributes the GGUF. `llama-cpp-2` is `MIT OR Apache-2.0` (SPEC §1). Llama 3.2 stays excluded and Phi-4-mini (MIT) remains the unmeasured fallback; nothing here rules the chosen model out.

## TESTS propagation through the Makefile (PRD Phase 1 task 4)

Run from the root of the checkout, with one selector naming a bare binary id:

```
$ make test TESTS='sloperator::eval_corpus'
    Starting 1 test across 6 binaries (13 tests and 1 binary skipped)
        PASS [   0.012s] (1/1) sloperator::eval_corpus twenty_paragraphs_present_without_placeholder_glyphs
     Summary [   0.013s] 1 test run: 1 passed, 13 skipped
sloperator: perf pass skipped (TESTS names no perf test)
exit code 0
```

The report `target/nextest/ci/junit.xml` holds exactly one `<testcase name="twenty_paragraphs_present_without_placeholder_glyphs" classname="sloperator::eval_corpus" …/>` (`grep -c '<testcase'` prints 1). So the Makefile turns the selector into one nextest filterset as ADR-07 says, the other 13 tests are skipped, and the testcase count check sees one testcase.

## Decision

- **Verdict: PASS.** Protected-span integrity is 20/20 after verify for the chosen pair, and placeholder survival before verify is 23/23.
- **Crate: `llama-cpp-2` =0.1.160** (unchanged from `Cargo.toml`). It is 3.5–4.1× faster than `mistralrs` on CPU for the same GGUF (4B: 36.2 vs 9.5 tok/s; 1.7B: 56.2 vs 13.7–16.0), builds cold in about 30 s against about 99 s, produces a 12 MB binary against 65 MB, and its `metal` feature builds on a machine where `mistralrs`' Metal build does not. No change to `Cargo.toml` or to epic 07's feature line, and no tokio pin is needed.
- **`DEFAULT_MODEL`: `"qwen3-4b"`** (the registry entry for Qwen3-4B-Q4_K_M.gguf; SPEC §15 already names it). Brad's Q4 default stands: the licence is Apache-2.0 and the 4B model accepted 20/20 where the 1.7B model accepted 19/20 and lost a placeholder in p12. The 1.7B Q8_0 file is faster (0.42 s per window on Metal) and is a reasonable second registry entry, but not the default. The registry sha256 for Qwen3-4B-Q4_K_M.gguf is the value in the table above; for Qwen3-1.7B-Q8_0.gguf likewise.
- **Masking mode: ADR-04 Option 1** (numbered `⟦n⟧` placeholders, unmasked after the check). The degraded Option 2 is not needed. The risk is smaller models: the 1.7B model dropped a link placeholder on its own, which the placeholder check rejects and the pipeline reports.
- **Grammar constraint:** do not use `LlamaSampler::grammar` (no benefit, 1.9× cost).
- **`MODEL_WINDOW_BUDGET_S = 3.0`** seconds per window (the mean, as SPEC §22 asserts it). Measured mean for the default model: 0.66 s on Metal, 1.30 s on CPU, max single window 0.79 s on Metal. 3.0 is about 2.3× the CPU mean, so it holds on a slower CPU-only machine and still fails on a regression such as a model that does not offload or a runaway generation. Windows here are single paragraphs of roughly 35–50 words; epic 08 should keep the eval on windows of similar size. Re-set the constant if the eval corpus or `max_window_chars` changes. The crate records this value in `eval/README.md` (Latency budget), and `tests/eval_corpus.rs` checks the constant against that copy, so update it there too.

Follow-ups for the next epics, not done here: epic 07 should set `n_gpu_layers` high when the `metal` feature is built and fall back to CPU when it is not; epic 06 must test the real masker on inline HTML, math and emphasis, which this spike did not exercise; mistralrs Metal remains unmeasured (needs `xcodebuild -downloadComponent MetalToolchain`) and is not needed for this decision.
