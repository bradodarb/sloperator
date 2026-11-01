# Competitive Analysis: linter CLI conventions and the AI-writing signal catalogue (sloperator)

**Date:** 2026-10-22
**Status:** Complete
**Related:** [Inference and Models](./inference-and-models.md) | [Feasibility Summary](./feasibility-summary.md)

---

## Purpose

Brad asked for a CLI "familiar from other linters". This file compares two mature linters on the axes he named:
modes, input, output formats, exit codes, suppression and config. It also records what the one authoritative public
catalogue of AI-writing signs says, because it seeds the rule set. The question for the PRD: which conventions are
universal (adopt without debate), where do the peers differ (a real choice), and what is new here (a rewrite engine
that is non-deterministic)? **Limits:** Ruff's page for exit codes and its full `--help` were not read; those cells say
"not verified". No other linter was surveyed. Model and runtime comparison lives in [Inference and Models](./inference-and-models.md).

---

## Sources Surveyed

| Peer System | Version Window Surveyed | Primary Doc / Repo |
|-------------|-------------------------|---------------------|
| Ruff (Python linter, written in Rust) | docs as served 2026-10-22 (Ruff version not stated on the pages read) | [Linter](https://docs.astral.sh/ruff/linter/), [Configuration](https://docs.astral.sh/ruff/configuration/), [Settings](https://docs.astral.sh/ruff/settings/) |
| Vale (prose linter, written in Go) | docs as served 2026-10-22 (pages show "last updated" 2 months and 14 days ago) | [Introduction](https://vale.sh/docs), [CLI](https://docs.vale.sh/topics/cli) |
| Wikipedia WikiProject AI Cleanup: "Signs of AI writing" | page states "As of October 2026" | [page](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing) |

Vale is the closest peer by purpose (prose, Markdown-aware). Ruff is the closest by engineering shape (fast Rust CLI,
safe/unsafe fixes, rich config and suppression). Neither rewrites text with a model.

---

## 1. Framework Comparison Matrix

### 1.1 CLI surface

| Feature | Ruff | Vale | Implication for sloperator |
|---------|------|------|----------------------------|
| Lint verb vs fix switch | `ruff check` lints; `--fix` applies fixes; `fix` setting defaults to `false`[^ruff-linter][^ruff-settings] | No fix switch in the CLI page read; fixes appear as "actions" on alerts (the CLI page links to an actions topic)[^vale-cli] | Adopt Ruff's shape: `check` reports, `--fix` rewrites. Matches Brad's "check/lint vs fix modes". |
| Input | files and directories, recursive; no args means current directory[^ruff-linter] | files, directories, stdin, or a literal string; stdin format via `--ext`, its path identity via `--path`[^vale-cli] | Files, globs, directories and stdin (`-`); stdin needs a way to say "this is Markdown" and which config section applies. |
| Glob narrowing | `include`/`exclude`/`extend-exclude` globs in config[^ruff-settings] | `--glob` flag narrows a directory walk[^vale-cli] | Both: config `include`/`exclude` and a CLI override. |
| Respect `.gitignore` | `respect-gitignore` default `true`[^ruff-settings] | not stated on the page read | Default on; the repo is git-based. |
| Markdown handled | `*.md` is in Ruff's default `include`[^ruff-settings] (it lints Python snippets, not prose) | First-class: syntax-aware scopes; code blocks and inline code are ignored by default; `--ignore-syntax` switches parsing off[^vale-intro][^vale-cli] | Vale's default (skip code) is exactly Brad's requirement (4); make it the baseline and make extras configurable. |
| Minimum severity | not verified | `--minAlertLevel` (suggestion, warning, error)[^vale-cli] | Needed: sloperator has soft findings (a single em dash) and hard ones. |
| Rule-selection flags | `--select`, `--extend-select`, `--ignore`; CLI beats file config[^ruff-linter] | `--filter` expression choosing rules[^vale-cli] | Needed for "rules-only / model-only" (`--engine`) and per-rule selection. |
| Introspection | not verified | `ls-config` prints the resolved config as JSON; `vale test` runs rule test cases (with `--coverage`)[^vale-cli] | Cheap to add and very useful: `sloperator config`, `sloperator rules`, and rule tests. |

### 1.2 Output formats

| Feature | Ruff | Vale |
|---------|------|------|
| Human format | `full` (default, shows source), `concise`, `grouped`[^ruff-settings] | `CLI` table (default), `line` (one alert per line)[^vale-cli] |
| Machine formats | `json`, `junit`, `github` (Actions annotations), `gitlab` (code-quality report), `pylint`, `azure`[^ruff-settings] | `JSON` keyed by file; custom Go templates (the docs link example templates for rdjsonl and SARIF, not opened)[^vale-cli] |
| Fix metadata in JSON | each fix has an `applicability` field (safe/unsafe)[^ruff-linter] | alerts carry the action with what it resolves to[^vale-cli] |

**Implication:** every peer ships a human format, a one-line-per-finding format, JSON, and at least one CI
annotation or report format. JSON must carry the rule code, severity, span (line and column), message, the matched text
and, for fixes, whether the fix is safe. Which CI formats ship in v1 is OQ-9.

### 1.3 Exit codes

| Code | Vale | Ruff |
|------|------|------|
| 0 | run completed, no `error`-level alert; warnings and suggestions do not change it[^vale-cli] | not verified |
| 1 | at least one `error`; `--no-exit` forces 0[^vale-cli] | not verified |
| 2 | the tool itself could not run (missing config, rule failed to load, unreadable doc)[^vale-cli] | not verified |

**Implication:** the 0 / 1 / 2 split (clean / findings / tool failure) is the documented convention in the peer read.
A tool failure such as "model missing" must not be confused with "findings" (OQ-3, OQ-8).

### 1.4 Suppression and configuration

| Feature | Ruff | Vale |
|---------|------|------|
| Config files | `pyproject.toml`, `ruff.toml`, `.ruff.toml`; nearest file wins, parents are **not** merged; `extend = "../x"` inherits[^ruff-config] | `.vale.ini` (and `_vale.ini`, `vale.ini`, `.vale`, `_vale`) searched from the working directory upward, then home; also `--config` and `VALE_CONFIG_PATH`; a user-level config underneath, project wins[^vale-cli] |
| Rule selection | `select`, `extend-select`, `ignore`, prefix selectors (`E`, `F401`), `ALL`; docs advise explicit `select` over `ALL` because `ALL` "will implicitly enable new rules whenever you upgrade"[^ruff-linter] | styles and checks (the intro page); alert levels suggestion/warning/error, filterable with `--minAlertLevel`[^vale-cli] |
| Per-file overrides | `lint.per-file-ignores` glob table[^ruff-config] | config sections keyed by file glob (stdin `--path` "puts it under that path's configuration section")[^vale-cli] |
| Inline suppression | line-level `# noqa: CODE`, own-line `# ruff: ignore[CODE]`, block `# ruff: disable[...]` ... `# ruff: enable[...]`, file-level `# ruff: noqa`; explicit ranges recommended over implicit ones[^ruff-linter] | not covered in the pages read |
| Fix safety | safe vs unsafe; `--unsafe-fixes` opt-in; per-rule `extend-safe-fixes` / `extend-unsafe-fixes`; `fixable` / `unfixable`[^ruff-linter] | n/a |
| Rule tests | n/a on pages read | `vale test` and coverage check that each rule fires in some case[^vale-cli] |

---

## 2. Architectural Patterns

### 2.1 Safe vs unsafe fixes (Ruff)

**Where it shows up:** Ruff. Vale has no direct equivalent in the pages read.

**Pattern:** a fix is "safe" if it keeps the meaning of the input; anything that might change meaning is "unsafe" and
needs an explicit flag. The default only applies safe ones, and JSON exposes the label.[^ruff-linter]

**Why it works:** it makes `--fix` trustworthy in CI and pre-commit while still allowing aggressive rewrites on request.

**Applicability here:** a close fit. Deterministic substitutions (for example "delve into" to "look at", a lone
em dash to a comma) are close to "safe". Any SLM rewrite is by nature "unsafe": it can drop or invent a fact.
Brad wants both engines on by default; applying that literally means model rewrites run without an opt-in. Whether
model edits are written in place by default, or staged as a suggestion or diff, is OQ-10. This is the largest
design tension in the request.

### 2.2 Directory-nearest config, no implicit merge (Ruff) vs upward search with user layer (Vale)

**Pattern:** both search upward from the file or working directory. Ruff takes the nearest config file and ignores
parents unless `extend` is used;[^ruff-config] Vale uses the first config found upward then home, with a user-level
file read underneath and the project winning.[^vale-cli]

**Applicability here:** "discoverable in the project" is satisfied by either. Ruff's no-implicit-merge plus `extend`
is the simpler mental model. Recommended: nearest file wins, optional `extend`, plus an explicit `--config`.

### 2.3 Protected syntax by default (Vale)

**Pattern:** the parser assigns scopes; code blocks and inline code are skipped unless the user says otherwise.[^vale-intro]

**Applicability here:** this is Brad's requirement (4) as a design principle: detection and rewriting operate on prose
spans only. The user's config then extends the protected set (for example front matter, math, custom directives).

---

## 3. Where Peers Diverge

- **Question:** are fixes part of the lint command?
  - **Ruff:** yes, a flag on the lint command.[^ruff-linter]
  - **Vale:** fixing is expressed as actions attached to alerts, with the CLI page showing no write-back flag.[^vale-cli]
  - **Implication:** Brad's brief ("check/lint vs fix modes") matches Ruff. Adopt `--fix`.
- **Question:** is the tool a rule framework or an opinionated rule set?
  - **Vale:** framework: "Vale doesn't offer any of its own advice", and rules live in external styles.[^vale-intro]
  - **Ruff:** ships a large built-in rule set organised by Flake8-style code prefixes (the page names several; I did not count the rules).[^ruff-linter]
  - **Implication:** sloperator should ship a built-in, opinionated, documented rule set (it is the product) with
    config to disable or tune them. A user-written rule format is a later question, not v1.
- **Question:** how are rules versioned relative to upgrades?
  - **Ruff:** warns that `ALL` silently adopts new rules on upgrade.[^ruff-linter]
  - **Implication:** the rule set will change as LLM style changes (see section 4: the em-dash entry is now filed
    under past indicators). Default-enabled rules must be an explicit, versioned list; new rules start opt-in or preview.

---

## 4. The signal catalogue (seed for the rule set)

Wikipedia's AI-cleanup project keeps a field guide to AI-writing signs.[^wiki-signs] Its section titles, read via the
page's section index and the introduction, include the following. The researcher mapped each to a candidate rule
family; the mapping is not from the source.

| Page section | Candidate rule family | Deterministic? |
|--------------|-----------------------|----------------|
| "High density of 'AI vocabulary' words" | word and phrase list (Brad's "delve") | Yes (list) |
| "Negative parallelisms" (subsections "Not just X, but also Y", "Not X, but Y", "Y rather than X") | "it's not X, it's Y" patterns | Detect: yes. Fix: needs rewrite |
| "Rule of three" | tricolon detection | Detect: heuristic. Fix: needs rewrite |
| "Avoidance of basic copulatives ('is'/'are' phrases)" | "serves as" to "is" substitutions | Mostly yes |
| "Undue emphasis on significance, legacy, and broader trends" | puffery phrases ("stands as a testament", "pivotal moment") | Yes (list) |
| "Hedging disclaimers about source availability and recommended usage" | hedging family (Brad's "hedging") | Partly |
| "Markup irregularities", "Use of Markdown" | bullet spam, over-bolding, stray markup | Yes |
| "Collaborative communication" | chatbot residue ("I hope this helps") | Yes (list) |
| "Overuse of em dashes (2022-September 2026)" in the section of past indicators | em-dash density | Yes |

Two cautions the source states that change the design:

1. **Signs are not the problem.** The page asks editors not to "merely treat these signs as the problems to be fixed;
   that could just make detection harder", and says the list is "descriptive, not prescriptive".[^wiki-signs] A tool
   that strips the signs does not make text truthful or specific; sloperator's scoring and docs should say it targets
   style tells, not quality or provenance.
2. **The catalogue moves.** The page files em-dash overuse in its section of past indicators (dated 2022 to September 2026);[^wiki-signs-toc]
   I read the section title, not the body, so I cannot say what it concludes. The point stands: rule defaults age,
   and each rule needs an on/off switch and a threshold (for em dashes, density per 1,000 words rather than any
   occurrence).

The source also reports that humans and detectors are poor at telling LLM from human text, and that LLM output is
not converging to human writing.[^wiki-signs] That is why "human-sounding" should be reported as a transparent rule
score, never as a claim of undetectability.

---

## 5. Best-Practice Signals

Universal in the peers read (seen in: Ruff, Vale), so skipping any needs a reason:

- Nearest-config discovery with an explicit `--config` override (Ruff, Vale).[^ruff-config][^vale-cli]
- Rule codes or names plus severity levels that can be filtered (Ruff select/ignore; Vale `--minAlertLevel`).
- A machine-readable format that includes spans and rule ids (Ruff json; Vale JSON).[^ruff-settings][^vale-cli]
- CLI flags beat config files (Ruff states it; Vale's `--minAlertLevel` "overrides the configuration").[^ruff-linter][^vale-cli]
- Suppression at more than one granularity (Ruff: line, block, file; per-file config) and a way to see the resolved config (Vale `ls-config`).
- A tool-failure exit code distinct from the findings code (Vale 0/1/2).[^vale-cli]

---

## 6. Anti-Patterns Observed

- **Implicit blanket enabling.** `ALL` adopts new rules on upgrade (Ruff docs warn against it).[^ruff-linter]
- **Implicit range suppressions.** Ruff warns that a `disable` with no matching `enable` ends at scope change and
  recommends explicit ranges.[^ruff-linter] In Markdown there is no indentation scope, so require explicit `enable`.
- **Treating style tells as the defect** (Wikipedia caution above).[^wiki-signs]

---

## 7. Implications for Our Design

- Adopt Ruff's `check` / `--fix` split and Vale's `0 / 1 / 2` exit code meaning; add `--exit-zero` style override.
- Ship a built-in, versioned rule list with codes (for example `SLP###`), severity levels, per-file overrides, nearest
  config file plus `extend`, and a way to print the resolved config and the rule list.
- Protected spans (code, links, URLs, front matter, numbers, user globs) are a parser-level concept, not a rewrite-time
  check alone. See [Inference and Models](./inference-and-models.md) section 4.
- Treat model edits as the "unsafe" class: they need verification and, per OQ-10, probably an explicit write policy.
- Output formats for v1: human (full and concise) and JSON; CI formats are OQ-9.
- No source read establishes how any peer behaves with a model-based fixer. That part of sloperator has no peer in
  this survey and must be validated by the spike and the eval harness.

---

## 8. References

[^ruff-linter]: The Ruff Linter. https://docs.astral.sh/ruff/linter/. Retrieved 2026-10-22. Establishes `ruff check` / `--fix`, rule selection and CLI-over-config precedence, safe vs unsafe fixes, `--unsafe-fixes`, `fixable`/`unfixable`, JSON `applicability`, and the `noqa` / `ruff: ignore` / `disable`/`enable` / file-level suppression forms, plus the advice on `ALL` and explicit ranges.
[^ruff-config]: Configuring Ruff. https://docs.astral.sh/ruff/configuration/. Retrieved 2026-10-22. Establishes config file names, nearest-file discovery with no implicit parent merging, the `extend` field, `--config`, per-file ignores and the user-level fallback.
[^ruff-settings]: Ruff Settings. https://docs.astral.sh/ruff/settings/. Retrieved 2026-10-22. Establishes `fix` default false, the `output-format` values, default `include` containing `*.md`, `respect-gitignore` default true, and `exclude`/`extend-exclude` glob settings.
[^vale-intro]: Introduction, Vale docs. https://vale.sh/docs. Retrieved 2026-10-22. Establishes that Vale is a prose linter in Go, "not a general-purpose writing aid", offers no advice of its own, and ignores block and inline code by default.
[^vale-cli]: CLI, Vale docs. https://docs.vale.sh/topics/cli. Retrieved 2026-10-22. Establishes input forms (files, directories, stdin with `--ext` and `--path`), config discovery order, `--output` formats (CLI, line, JSON, template), `--minAlertLevel`, `--no-exit`, `ls-config`, `vale test`, and exit codes 0, 1 and 2.
[^wiki-signs]: Wikipedia:Signs of AI writing. https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing. Retrieved 2026-10-22. Establishes the descriptive-not-prescriptive caveat, the warning against treating signs as the problem, the weakness of detectors and human judgement (2025 studies cited by the page), and the "Words to watch" and rule-of-three style families.
[^wiki-signs-toc]: Section index of Wikipedia:Signs of AI writing (MediaWiki parse API). https://en.wikipedia.org/w/api.php?action=parse&page=Wikipedia:Signs_of_AI_writing&prop=sections&format=json. Retrieved 2026-10-22. Establishes the section titles used in section 4, including "Overuse of em dashes (2022-September 2026)" in the section of past indicators. Titles only; bodies of those sections were not read.
