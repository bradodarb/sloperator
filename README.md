<img width="1448" height="1086" alt="image" src="https://github.com/user-attachments/assets/ca01b805-0d0f-4a20-89a2-e330ea63d3b6" />


# sloperator

sloperator makes prose shorter and plainer, whether a person or an LLM wrote it.

## Why

LLMs write faster than we can read. A three-line answer comes back as six paragraphs. We skim, then we stop reading.

Padding makes it worse: openers that say nothing, hedges, filler words, em dashes everywhere. Sometimes it's AI slop.
Often it's just too long.

## What it does

- 23 deterministic rules catch padding, AI tics and reading patterns, and fix the safe ones.
  The four reading-metric rules, weak-intensifier rule and redundant-sentence rule are opt-in.
- A small local model tightens what's left.
- Every rewrite is verified before it's kept. If it drops a number, a link or a code block, or doesn't reduce the
  findings, the original stays.
- Nothing leaves your machine.
- It also runs as an MCP server (`sloperator mcp`), so an agent can tighten its own writing before you read it. See
  [MCP server](#mcp-server).

It ships as one crate: a library and a binary, both named `sloperator`.

## Example

This is `sloperator check --fix` with the default model. Model output can vary by seed.

Before:

```text
🚀 Just wanted to circle back on the migration and give everyone a quick update on where things currently stand. ✅ It's important to note that, as of right now, we are basically on track, more or less, with what we had originally planned to do. 🔍 In order to make sure that nothing falls through the cracks, the team has been working really hard to leverage the existing tooling — and honestly it has been going pretty well so far 💪. ⏳ At the end of the day, the main thing that is still outstanding is the data backfill, which we are hoping to have wrapped up by Friday 🤞, assuming that nothing unexpected comes up between now and then. 🙏 Please don't hesitate to reach out if you have any questions or concerns about any of this! 🙌
```

After:

```text
I want to circle back on the migration and give a quick update. We're basically on track with the original plan. The team has been working hard to use existing tools, and it's going well. The main thing still pending is the data backfill, which we hope to finish by Friday. Let me know if you have questions or concerns.
```

134 words down to 60. Same facts, 55% shorter, every emoji gone. Nothing left to flag.

## Install

Homebrew (macOS on Apple silicon, Linux):

```sh
brew tap bradodarb/sloperator https://github.com/bradodarb/sloperator
brew install bradodarb/sloperator/sloperator
```

Tarballs and `.deb` packages are on [GitHub Releases](https://github.com/bradodarb/sloperator/releases); the
[install reference](docs/REFERENCE.md#install) has the download, checksum and `.deb` commands. To build from source,
see [Development](docs/DEVELOPMENT.md). Run `sloperator model pull` once for model-backed fixes (about 2.5 GB).

## Quick start

```sh
sloperator check notes.md               # report findings and a score
sloperator check --diff notes.md        # preview the fixes as a diff
sloperator check --fix notes.md         # apply the verified fixes
sloperator check --fix --rules-only .   # rule fixes only, no model
sloperator check --fix - < notes.md     # fix stdin, print to stdout
sloperator rules                        # list the rules
```

Each rule, with its fix class and an example, is in the [rule reference](docs/REFERENCE.md#rule-core-rules), or run
`sloperator rules SLP001`. Flags, exit codes, directives and `sloperator.toml` are in the
[reference](docs/REFERENCE.md).

## Classical NLP, before the model

The 0.2.0 additions use classical natural language processing: word lists, grammatical patterns and counts.
These techniques run locally without a model download, including with `--rules-only`.

- **Shared sentence and word boundaries.** Rules, reading metrics and model-window planning reuse one cached
  segmentation per document pass. Analysis uses prose spans, keeping protected content such as code out of word
  counts.
- **Reading metrics.** An English syllable heuristic counts vowel groups, adjusts silent endings and checks an
  exception list. Together with paragraph word and sentence counts, it produces a Flesch–Kincaid grade estimate,
  average sentence length, sentence-length variation and type–token ratio (distinct lowercased words divided by
  total words). Reports include these metrics even when readability rules are off. By default, derived values
  require at least 5 sentences and 100 words; configure those floors in `[metrics]`.
- **Part-of-speech tagging.** A small Brill-style tagger starts with a lexicon, guesses unknown words from their
  endings and applies contextual rules. SLP006 uses only certain tags to authorize additional fixes:
  “The library serves as a guide” becomes “The library is a guide.” Ambiguous subjects remain unfixed.
  Tags are computed only when needed and cached for that pass.
- **Phrase and template rewrites.** SLP001 replaces padded wording such as “make a decision” with “decide”.
  SLP003 and SLP004 use guarded templates for short contrast and “not just … but also” constructions.
  A matching phrase gets an automatic fix only when its surrounding grammar meets the rule's checks.
- **Stemming and sentence similarity.** SLP023 removes stop words, reduces inflections with a small suffix
  stripper, then compares content-word counts using weighted Jaccard similarity: shared counts divided by
  combined counts. It flags overlapping sentences within a paragraph, with a default threshold of 0.6.
  This measures word overlap; it cannot establish that two claims mean the same thing, so it never deletes a
  sentence or requests a model rewrite.

SLP018–SLP021 flag grade level, uniform sentence lengths, long average sentences and low lexical diversity.
SLP022 flags weak intensifiers; SLP023 flags possible redundancy. All six are opt-in suggestions with no automatic
fix. To see them alongside the default rules, add these top-level keys to `sloperator.toml`:

```toml
extend_select = ["SLP018", "SLP019", "SLP020", "SLP021", "SLP022", "SLP023"]
min_severity = "suggestion"
```

The verifier also checks negations, modal verbs and capitalised words as a proxy for names, alongside existing
number, link and structure checks. These are heuristics, not a guarantee of preserved meaning. Reading scores
and similarity thresholds are provisional signals for review, not measures of writing quality. See the
[rule reference](docs/REFERENCE.md#rule-core-rules) for parameters and the
[classical NLP design](docs/classical-nlp/ARCHITECTURE-classical-nlp.md) for implementation details.

## MCP server

`sloperator mcp` serves four tools over stdio: `lint`, `fix`, `rules` and `explain_rule`. They run the same
pipeline as `sloperator check`.

```json
{"mcpServers": {"sloperator": {"command": "sloperator", "args": ["mcp"]}}}
```

It never writes files. Text goes in and text comes out. `fix` returns a diff for you to review before the agent
applies anything. The full guide is [docs/MCP.md](docs/MCP.md).

## Docs

- The [reference](docs/REFERENCE.md) documents the CLI, the config file and every rule in full.
- [docs/MCP.md](docs/MCP.md) is the MCP server guide.
- [Development](docs/DEVELOPMENT.md) explains how to build and test.
- The design docs are the [SPEC](docs/SPEC.md), [ARCHITECTURE](docs/ARCHITECTURE.md), [ADRs](docs/adr/) and
  [PRD](docs/PRD.md). Changes are in the [changelog](CHANGELOG.md).

## License

MIT. See [LICENSE](LICENSE).
