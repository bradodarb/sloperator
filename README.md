<img width="1448" height="1086" alt="image" src="https://github.com/user-attachments/assets/ca01b805-0d0f-4a20-89a2-e330ea63d3b6" />


# sloperator

sloperator makes prose shorter and plainer, whether a person or an LLM wrote it.

## Why

LLMs write faster than we can read. A three-line answer comes back as six paragraphs. We skim, then we stop reading.

Padding makes it worse: openers that say nothing, hedges, filler words, em dashes everywhere. Sometimes it's AI slop.
Often it's just too long.

## What it does

- 21 deterministic rules catch padding, AI tics and reading patterns, and fix the safe ones.
  The four reading-metric rules are opt-in.
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
