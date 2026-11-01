# Eval corpus

`paragraphs/p01.md` … `p20.md` are twenty short paragraphs of LLM-style prose ("slop") written for this project during
the inference spike (SPEC §1, `docs/research/spike-results.md`). They are the input to the eval harness,
`tests/eval.rs::eval_report_per_model` (SPEC §22, FR-10.2).

## How they were written

- Each paragraph was composed for sloperator as a deliberate imitation of LLM house style (FR-10.1); none is taken
  from a dataset or a third-party text. Each is one paragraph of roughly 35–55 words, the window size the latency
  budget `MODEL_WINDOW_BUDGET_S` was measured on (spike Decision). They entered the repo with the spike record.
- Together they cover every model-class rule family: inflated vocabulary ("delve", "tapestry", "pivotal"),
  significance padding ("a testament to", "a turning point"), stock openers ("In today's fast-paced …"), hedges and
  filler.
- They also carry the content a rewrite must not damage: inline code (`build_index()`), Markdown links and bare URLs,
  numbers and years, and a table row (`p10.md`). The engine masks these, and the eval checks that every protected span
  survives (integrity n/20).
- No paragraph contains `⟦` or `⟧`; the engine reserves them for placeholders (ADR-04). `tests/eval_corpus.rs` checks
  this and that exactly `p01.md` … `p20.md` are present.

## What they are not

They are **not training pairs**. There is no "after" text: the eval scores the model's own rewrite with the rule
engine and the verifier. They must not be used to fine-tune a model (fine-tuning is deferred, D-12; this eval's report
is the input to that decision). Nothing here was fabricated as a human-written reference.

The protected-content fixtures (code, links, front matter, tables, CRLF, math, HTML, reference definitions) are
`fixtures/protect/*`, used by the unit tier (SPEC §23).

## Running the eval

```sh
sloperator model pull     # the default model must be in the cache
cargo test --test eval -- --ignored eval_report_per_model
```

The test runs `pipeline::fix` with both engines on every paragraph, once for every registry model in the cache. It
writes `target/eval/report.json` and prints one table per model: mean score before and after, integrity n/20,
accepted rewrites, rejections per `Check`, SLP902 count and mean seconds per window. It fails when integrity is below
20/20 or the mean seconds per window exceeds `MODEL_WINDOW_BUDGET_S`.

## Latency budget

`MODEL_WINDOW_BUDGET_S = 3.0` seconds per window (the mean). This line is the crate's record of the spike Decision
(`docs/research/spike-results.md`). The constant in `tests/eval.rs` must equal it;
`tests/eval_corpus.rs::budget_matches_eval_readme` checks that in the unit tier. Re-set both if this corpus or
`max_window_chars` changes.
