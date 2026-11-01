# sloperator epic-07: SPEC §21 still describes Box::leak of the model

**Source:** acceptance audit of epic-07-llama-runtime (reviewer)
**Where:** `docs/SPEC.md` §21 (around line 2080), story-01 Notes "Accepted Gap 13"

The shipped `src/engine/runtime.rs` owns the model in `LlamaRewriter` and drops it before the backend. Leaking it would trip ggml's Metal teardown assert at exit (exit 134); see the comment at `runtime.rs:5-7`. It also uses `send_logs_to_tracing(...with_logs_enabled(false))` instead of `void_logs`. The SPEC text and Accepted Gap 13 should be updated to match.
