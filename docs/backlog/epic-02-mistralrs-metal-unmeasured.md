# sloperator epic-02: mistralrs Metal row unmeasured

**Source:** acceptance audit of epic-02-inference-spike (reviewer).
**Severity:** minor (non-blocking).

PRD FR-9.9 asks for GPU tok/s and load time for each crate and model. In `docs/research/spike-results.md:20,30`, the `mistralrs =0.8.1` Metal build failed because the machine has no Metal Toolchain (`xcodebuild -downloadComponent MetalToolchain`), so that row has no numbers. Per-paragraph tables exist only for the chosen pair (llama-cpp-2 + Qwen3-4B); the other pairs appear as summary rows.

The decision (llama-cpp-2) is not affected: llama-cpp-2 is faster on CPU, smaller and builds Metal without the toolchain. If the runtime choice is ever reopened, install the toolchain and fill the row.
