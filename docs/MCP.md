# MCP server

`sloperator mcp` serves sloperator's linter and fixer to MCP clients (agents, editors) over stdio. It is part of
the normal binary, behind the default cargo feature `mcp` (ADR-10). Every tool runs the same pipeline as
`sloperator check` on the text it is given.

**Text in, text out.** The server never writes files and never reads the file a `path` names. `path` is only a
hint: it picks the format (`.txt` is plain text, anything else Markdown) and which `sloperator.toml` applies. The
agent decides whether to apply a fix.

## Tools

| Tool | Input | Output |
|------|-------|--------|
| `lint` | `text`, `path?` | `findings` (code, name, severity, line, col, endLine, endCol, message, matched) and `score` |
| `fix` | `text`, `path?`, `rules_only?` (default `false`) | `fixed`, `changed`, `diff` (unified), remaining `findings`, `rejections` (SLP901/SLP902), `scoreBefore`, `scoreAfter`, `engines` |
| `rules` | none | `ruleSetVersion` and `rules` (code, name, severity, defaultOn, fix class) |
| `explain_rule` | `code` | the rule's explanation, `params` and a before/after `example` |

Results come back as `structuredContent`, with the same JSON as text in `content`. A bad argument, input over
1 MiB, an unknown rule code, a bad `sloperator.toml` or a model error is a tool error (`isError: true`) with a
message.

`lint` with `{"text": "It is important to note that we delve into the data.\n", "path": "notes.md"}`:

```json
{"findings": [
  {"code": "SLP008", "name": "hedging-preamble", "severity": "warning", "line": 1, "col": 1, "endLine": 1,
   "endCol": 29, "matched": "It is important to note that",
   "message": "`It is important to note that` is a hedging preamble"},
  {"code": "SLP001", "name": "ai-vocabulary-replaceable", "severity": "warning", "line": 1, "col": 33,
   "endLine": 1, "endCol": 43, "matched": "delve into",
   "message": "`delve into` is padded wording; use `look at`"}],
 "score": {"value": 545.45, "proseWords": 11, "weighted": 6}}
```

`fix` with `{"text": "It is important to note that this is great.\n", "path": "notes.md", "rules_only": true}`:

```json
{"changed": true, "fixed": "This is great.\n",
 "diff": "--- a/notes.md\n+++ b/notes.md\n@@ -1 +1 @@\n-It is important to note that this is great.\n+This is great.\n",
 "findings": [], "rejections": [], "engines": {"rules": true, "model": false},
 "scoreBefore": {"value": 333.33, "proseWords": 9, "weighted": 3},
 "scoreAfter": {"value": 0.0, "proseWords": 3, "weighted": 0}}
```

A model rewrite can pass every verification check and still flatten a deliberate voice. The `fix` description
tells the agent to show the user the diff before applying it.

## Flags

| Flag | Default | Effect |
|------|---------|--------|
| `--rules-only` | off | Never load the model; every `fix` uses the rules engine only (`engines.model` is `false`). |
| `--idle-unload <secs>` | `600` | Unload the model after this many seconds without a model-backed fix; `0` = never. |
| `-v` | off | Model load/unload notes and llama.cpp logs on stderr. |

stdout carries only the protocol; every log goes to stderr.

## Model lifecycle

- The model loads lazily, on the first `fix` that needs a model rewrite. `lint`, `rules`, `explain_rule` and
  `fix` with `rules_only` never load it.
- One worker thread owns the model and serves model-backed fixes one at a time, so model use is serialised.
  Rules-only work runs in parallel beside it.
- After `--idle-unload` seconds with no model-backed fix the model is freed; the next one loads it again.
- A missing model is a tool error naming `sloperator model pull`. The server never downloads anything.

## Client configuration

Claude Desktop / Claude Code style (`mcpServers` in the client's JSON config):

```json
{
  "mcpServers": {
    "sloperator": { "command": "sloperator", "args": ["mcp"] }
  }
}
```

Any stdio MCP client: run `sloperator mcp [--rules-only] [--idle-unload SECS]` as the server command, with the
working directory set to the project, so relative `path` hints find its `sloperator.toml`:

```json
{ "command": "/usr/local/bin/sloperator", "args": ["mcp", "--rules-only"], "cwd": "/path/to/project",
  "transport": "stdio" }
```
