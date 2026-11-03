# sloperator reference

The full reference for the `sloperator` CLI and library: install, commands, flags, output, exit codes, directives,
configuration, rules, score, limits and model setup. The [README](../README.md) has the short version. The MCP
server has its own guide, [MCP.md](MCP.md). Building and testing are in [DEVELOPMENT.md](DEVELOPMENT.md).

## Install

Prebuilt binaries cover macOS on Apple silicon and Linux on x86_64 and aarch64. Each release is on the
[releases page](https://github.com/bradodarb/sloperator/releases).

Homebrew (macOS on Apple silicon, Linux):

```sh
brew tap bradodarb/sloperator https://github.com/bradodarb/sloperator
brew install bradodarb/sloperator/sloperator
```

Direct download. Set `TARGET` to `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu` or `aarch64-unknown-linux-gnu`:

```sh
VERSION=0.1.0
TARGET=aarch64-apple-darwin
BASE=https://github.com/bradodarb/sloperator/releases/download/v$VERSION
curl -fLO "$BASE/sloperator-$VERSION-$TARGET.tar.gz"
curl -fLO "$BASE/sloperator-$VERSION-$TARGET.tar.gz.sha256"
shasum -a 256 -c "sloperator-$VERSION-$TARGET.tar.gz.sha256"
tar -xzf "sloperator-$VERSION-$TARGET.tar.gz"
sudo install "sloperator-$VERSION-$TARGET/sloperator" /usr/local/bin/
```

Debian and Ubuntu. Use `amd64` or `arm64`:

```sh
curl -fLO "https://github.com/bradodarb/sloperator/releases/download/v0.1.0/sloperator_0.1.0_amd64.deb"
sudo apt install ./sloperator_0.1.0_amd64.deb
```

The Linux binaries are built on Ubuntu 24.04 (glibc 2.39). An older distribution may need a source build. There is
no Intel macOS binary, so build from source there as well. See [DEVELOPMENT.md](DEVELOPMENT.md).

Fixes that use the model need it downloaded once (about 2.5 GB):

```sh
sloperator model pull
```

## Usage

```text
sloperator [-v|-q] check [PATHS...] [--fix | --diff] [--rules-only | --model-only] [--output-format FMT]
                                    [--min-severity SEV] [--exit-zero] [--select CODES] [--extend-select CODES]
                                    [--ignore CODES] [--config FILE] [--stdin-filename PATH] [--seed N]
sloperator rules [CODE]
sloperator config [PATH] [--config FILE]
sloperator model pull [NAME] | list | path [NAME]
sloperator [-v] mcp [--rules-only] [--idle-unload SECS]
```

**Commands.**

- `check` lints files, directories (walked; `include`, `exclude` and `.gitignore` apply), globs, or `-` for stdin.
  No paths means `.`. It reads nothing but the inputs and their config and writes nothing (FR-2.1).
  - `--fix` applies the verified fixes in place, atomically and keeping permissions. If any path fails, nothing is
    written (DQ-10). With stdin, the fixed text goes to stdout.
  - `--diff` prints the same changes as a unified diff on stdout and writes nothing (FR-2.5). `--fix` together with
    `--diff` is a usage error.
  - `--rules-only` / `--model-only` pick one engine for `--fix`/`--diff`; with neither, both run. Passing both is a
    usage error (FR-2.10). Lint never loads the model. When the model engine is on, the model is resolved and
    checked before any file is read, so a missing model (or a build without the `model` feature) exits 2 naming
    `sloperator model pull` and `--rules-only` (FR-9.6).
  - Files are analysed in parallel and fixed one at a time in path order (ADR-09).
- `rules` lists every rule (code, name, default severity, on/off by default, fix class) under the rule-set
  version. `rules SLP001` prints that rule's explanation, parameters and before/after example. An unknown code
  exits 2 (FR-2.12).
- `config [PATH]` prints the settings that apply to PATH (a file, or a directory's files; default `.`) as pretty
  JSON with the config file's snake_case keys (FR-2.13, DQ-9). `--config FILE` replaces discovery, as for `check`.

**Flags.** `--select`, `--extend-select` and `--ignore` take comma-separated codes and override the config keys of
the same name; an unknown code exits 2 naming it. `--min-severity suggestion|warning|error` (default `warning`, or
the config's `min_severity`) decides which findings are reported and counted toward exit 1. `--seed` sets the
model's sampling seed. `--stdin-filename PATH` names stdin: it picks the format (`.txt` is plain text, anything else
Markdown) and the config lookup. `-v` adds stage timings on stderr; `-q` prints errors only (no report, no
skip notes).

**Output formats** (`--output-format`): `text` (default: each finding with its source line and a caret, a score
per file, then a summary), `concise` (`path:line:col: SLP003 [warning] message`, one line per finding), `json`
(schema v1, camelCase keys) and `github` (workflow annotations). See Report below. Colour is used only when stdout
is a terminal and `NO_COLOR` is unset (FR-3.5). When stdout carries content (a diff, or stdin `--fix`), the report
goes to stderr instead, uncoloured (DQ-11).

**Exit codes.** 2 on any error: an unreadable or missing path, non-UTF-8 input, a bad config file or key, an
unknown rule code in a flag or directive, a directive error, a usage error (unknown flag or value, conflicting
flags), a missing model, a failed write. Otherwise 0 with `--exit-zero`. Otherwise 1 when any finding was reported,
and 0 when none was (FR-2.7, FR-2.8, D-8). An error on one path does not stop the others from being reported.
`--help` and `--version` exit 0.

**Directives** in the text suppress findings: `<!-- sloperator-ignore SLP001 -->` (the next prose line),
`sloperator-disable` / `sloperator-enable` (a region), `sloperator-ignore-file`; in plain text, `[sloperator-ignore
SLP001]`. No codes means every rule. See Rule core below.

**Config reference:** see Configuration below (`sloperator.toml`, discovered upward from each file).

## Document model (`doc`, `types`)

`doc::parse(src, format, &ProtectSettings)` builds a `Document`: a span map that covers every source byte exactly
once, plus the blocks (paragraphs, headings, list items, table cells) that rules and rewrites work on. A span is
either prose or protected, so an edit that stays inside prose cannot change code, link destinations, images,
reference definitions, front matter, HTML, math, or block markup such as `#`, list markers, `|` and `>`. Link text,
heading text, list-item text, table-cell text and quote text are prose.

- Bare URLs, configured `patterns` (regex) and configured `terms` (literals) are carved out of prose as protected
  runs. `parse` compiles `patterns` itself; build settings with `ProtectSettings::new` to have an invalid regex
  reported.
- `ProtectSettings::default()` has no patterns or terms, and `front_matter`, `math` and `html` on. HTML is always
  protected: `html = false` only changes what rules may read, never what an edit may touch.
- Plain text (`Format::PlainText`): each run of non-blank lines is one paragraph. A whole line of the form
  `[sloperator-<kind> ...]` is a protected directive line and ends the paragraph before it.
- `Document::newline` is `Crlf` when the first line ending is `\r\n`.
- `Document::protected_strings()` is a sorted multiset. A block's continuation prefixes (`> `, list indentation)
  count once per block and distinct text, so re-wrapping a block does not change it.
- `check` picks the format from the file extension (`.md` and `.markdown` are Markdown, anything else is plain
  text) and parses each file with the default settings.

`src/types.rs` holds the leaf value types `RuleCode` (`SLP###`; 900 and above are system codes), `Proposal` and
`ProposalOrigin`. Neither `doc` nor `types` uses any other crate module, apart from `error` for `doc`.

## Edit gate (`verify`)

Every edit goes through `verify::edit(doc, proposal, frozen, protect, resolves)`, which returns `Verified<Proposal>` or a
`Rejection { range, failed: Check, detail }`. Only `verify` can build a `Verified` (private constructor, ADR-05).
A rejection is reported, never written: the pipeline turns it into SLP901 (see Pipeline below).

- Numbers (`42`, `1,000.5`, `42%`), digit dates (`2024-01-05`), URLs and configured `terms` are compared as
  multisets between the replaced bytes and the replacement. A loss, addition or change is rejected
  (`Numbers`, `Urls`, `Terms`). Reordering is fine.
- Meaning checks run after configured terms and before reparsing. `Negation` rejects a reduction beyond the
  allowance: one for rule-origin SLP003/SLP004, or one per SLP003/SLP004 finding in a model window's `resolves`
  list (duplicates count). Other origins have no allowance. `ProperNouns` preserves the multiset of capitalised,
  non-sentence-initial words; only rule-origin edits skip closed-list common words. `Modals` preserves the modal
  multiset, normalising contractions such as “won't” to “will”. Safe fixes pass an empty `resolves` list.
- An edit overlapping a protected span or a frozen (suppressed) range, or outside the document, is `Protected`.
- The document is re-parsed with the edit applied. A different block count is `NewBlockMarker` (a new heading or
  list), a different block kind or level is `BlockType`, and different protected bytes (a new fence, say) is
  `Protected`. A document whose directives `directives::extract` re-tagged is compared through a bare parse of its
  own text, so the re-tag's split spans never read as changed bytes.
- `verify::improvement(before, after, range)` is `Ok` only when `after < before` weighted findings; an equal or
  higher count is `NoImprovement` (FR-9.5, DQ-3).
- `verify::document(before, after)` checks the whole file: equal block skeleton and protected strings, else
  `Document`.

## Plan and apply (`edit`)

`edit` re-exports `Proposal` and `ProposalOrigin` from `types`. It does not use `rules`; the pipeline is its caller.

- `edit::plan(proposals, frozen)` sorts by start, then end, then origin (rules by code, before the model), then
  replacement text. It drops every proposal that touches a frozen range (FR-6.6) and every proposal that overlaps an
  earlier kept one (§7). The same input gives the same `Plan { kept, dropped }` in any order. Dropped proposals are
  only skipped, and their findings stay in the report.
- `edit::apply(src, &[Verified<Proposal>], newline)` takes only verified edits, copies every byte outside them, and
  writes replacement newlines in the document's style, so CRLF files stay CRLF. The final newline of a file is in a
  protected run, so no edit covers it and it is kept. A no-op plan returns `src` byte for byte (ADR-03).
- `edit::unified_diff(path, before, after)` returns a unified diff with `--- a/<path>` / `+++ b/<path>` headers and
  `@@` hunks with 3 lines of context (FR-2.5).

## Rule core (`rules`)

`rules` holds the `Rule` trait (`meta()` and `check(&RuleCtx, &mut Vec<Finding>)`), the metadata types (`Severity`,
`FixClass`, `Family`, `ParamSpec`, `RuleMeta`), `Finding`, `Enabled` (a rule with its resolved severity and params)
and the `Registry`. Each `ParamSpec` declares inclusive `min` and `max` bounds; `f64::MAX` denotes an unbounded
upper limit. Config rejects NaN and values outside these bounds, naming the rule and parameter. Existing count
and rate parameters accept `0..=f64::MAX`. `rules CODE` prints the bounds after each parameter's help;
MCP `explain_rule` includes `min` and `max`, with `null` for an unbounded maximum.
`rules::RuleCode` and `rules::Proposal` are re-exports from `types`. `rules` never uses
`edit`; a fix is a `types::Proposal`. The built-in list is `rules::builtin::all()`; each rule story adds one file
`src/rules/builtin/slpNNN_<name>.rs` and one line there.

- `rules::detect(doc, &Prose, &Suppressions, &[Enabled])` runs every enabled rule, drops each finding whose range overlaps a
  protected span (FR-4.4) or whose start line is suppressed, and sorts by start byte then code. A system code
  (SLP900 and above) is never suppressed.
- `Registry::builtin()` is built once. `get(code)` and `default_set()` query it. System codes SLP901 and SLP902 are
  not rules and are never in it.
- The catalogue has 22 rules across eleven populated families. The original SLP001–SLP017 remain on by default
  with `since = "0.1.0"`; SLP018–SLP021 are opt-in Readability rules with `since = "0.2.0"`. `tests/rules_registry.rs` pins this: `default_set_is_explicit` compares `default_set()` with a
  literal list, so changing the defaults shows up as a test diff, and `new_rules_start_off` fails when a rule whose
  `since` is newer than 0.1.0 is `default_on` (FR-4.5).

SLP001 `ai-vocabulary-replaceable` remains a default-on warning in AiVocabulary with safe fixes, and now also
covers generic wordiness and nominalisations. Inflected rows include “make a decision” → “decide”,
“reach a conclusion” → “conclude”, “take into consideration/account” and “give consideration to” → “consider”,
“conduct/perform an analysis of” → “analyze”, “conduct an investigation into” → “investigate”,
“provide assistance to” → “help”, “make an attempt to” → “try to”, “have a tendency to” → “tend to”,
and “be in agreement with” → “agree with”. Padded phrases include “due to/owing to/in light of the fact that”
→ “because”, “despite/in spite of the fact that” → “although”, “at this point in time/at the present time”
→ “now”, “in the near future” → “soon”, “on a daily/regular basis” → “daily/regularly”,
“until such time as” → “until”, “prior to” → “before”, “for the purpose of” → “for”,
“with the exception of” → “except”, and “in close proximity to” → “near”.

Noun-final rows fire only in closed, proven contexts: at sentence end, with one allowed complement word plus
one terminal word, or for fronted adverbial rows followed by a comma and subject pronoun. Relative clauses,
coordinations and noun modifiers such as “a decision tree” are left alone. “The fact that” rows reject a preceding
form of “be”. SLP001's new default-on rows can change default findings, scores and safe fixes.

| Code | Name | Family | Severity | Fix | Default | Detects |
|------|------|--------|----------|-----|---------|---------|
| SLP022 | `weak-intensifier` | Wordiness | suggestion | none | off | absolutely, actually, basically, extremely, highly, incredibly, literally, quite, really, totally, truly, very |

SLP022 has no params and is available since 0.2.0. Enable it with `extend_select = ["SLP022"]` and
`min_severity = "suggestion"`. It reports emphasis without deleting it: removal changes strength, which the
verifier does not check. No automatic fix or model window is selected.

The four Readability rules are suggestions with fix class `None`, default-off and detection-only: they never
select a model window. Each reports once on the first paragraph's prose and adds suggestion weight 1 when
reported. Enable them with `extend_select = ["SLP018", "SLP019", "SLP020", "SLP021"]`; use
`min_severity = "suggestion"` to include them in the report and score. Thresholds remain provisional.

| Code | Name | Fires when | Threshold (default; inclusive bounds) |
|------|------|------------|---------------------------------------|
| SLP018 | `grade-level` | Flesch–Kincaid grade exceeds maximum | `max_grade = 12`; 0–30 |
| SLP019 | `sentence-length-variation` | population CV falls below minimum | `min_cv = 0.35`; 0–5 |
| SLP020 | `long-sentences` | mean words per sentence exceeds maximum | `max_avg_words = 25`; 1–1000 |
| SLP021 | `lexical-diversity` | distinct lowercase words / words falls below minimum | `min_ttr = 0.4`; 0–1 |

All four declare `min_sentences` (default 5) and `min_words` (default 100), static minimum 1 and no finite upper
bound. Unset minimums inherit the file's `[metrics]` report floors after configuration layers merge.
An explicit rule minimum may raise a floor but cannot be below it; violations name the rule parameter and floor.
SLP019 measures sentence-length variation, not perplexity-based burstiness. See [Text metrics](#text-metrics)
for formulas, counting scope and limits. Every numeric `ParamSpec` has inclusive `min`/`max` bounds; invalid
config values fail with the dotted key. CLI and MCP rule explanations expose these bounds.

- `tests/rules_fixtures.rs` checks every rule's fixtures (SPEC §10, FR-4.6): `every_rule_has_fixtures` (directory,
  `positive.md`, `negative.md`, and `fixed.md` exactly when the rule is safe), `only_safe_rules_attach_fixes`
  (FR-4.7, over every fixture), and `fix_is_idempotent_over_all_fixtures` (FR-8.9: each safe rule's fixes, applied
  through `edit::plan` → `verify::edit` → `edit::apply`, change nothing on a second pass).
- `rules::helpers::WordList` is a case-insensitive, whole-word, longest-match phrase table of
  `(phrase, Option<replacement>)`. `find` returns source ranges, with a case-preserving replacement when the entry has
  one and `None` when it does not (the phrase is flagged and left to the model). `PatternRule` holds a `RegexSet`
  and its regexes, and `find` returns `(pattern index, source range)`. Neither reports a match across a non-prose run.
- SLP006 `copula-avoidance` (`builtin/slp006_copula_avoidance.rs`) reports every inflection of "serve/stand/function
  as". Its fix is `Safe` but attached only when the guards hold: "serves/stands/functions as" → "is" after
  `SINGULAR_SUBJECT` (`it`, `this`, `that`, `which`, `who`, `he`, `she`, same sentence), a base form after a modal or
  "to" (one adverb allowed between) → "be", and in both cases only when the next word after "as" is in
  `AS_COMPLEMENT` (`a`, `an`, `the`, `its`, `his`, `her`, `their`, `our`, `your`, `my`, `one`, `another`). With no next
  word in the block, or a protected run after "as", there is no fix. A further "is" branch accepts
  sentence-initial determiner/noun subjects, optionally with one adjective, only when all subject tags are
  certain, the determiner is singular-compatible, and whitespace/protection guards hold. It excludes
  "as a matter/result/rule/whole" and any following verb or modal in the clause. Thus "The library serves as
  a guide" becomes "The library is a guide", while ambiguous "The tool serves as a bridge" has no fix.
  Every other form fires with no fix. Tags are computed lazily once per document pass.
- Pattern and residue rules (`builtin/slp003_*` … `slp015_*`), all on by default in v0.1.0. Each is a `PatternRule`
  over the prose of a block, so a match never crosses code or a link.
  - SLP003 `not-x-but-y` and SLP004 `not-just-but-also` (negative parallelism), SLP007 `significance-puffery` and
    SLP015 `ai-disclaimer` (chatbot residue) are fix class `Model`: they report and attach no fix.
  - SLP005 `rule-of-three` is `Model`. It counts triads "A, B(,) and C" of items of 1-3 words and reports every one
    only when the count per 1,000 prose words is above its param `max_per_1000_words` (default 4).
  - SLP012 `chatbot-closer` (severity `error`, `Safe`) matches a whole sentence ("I hope this helps!", "Let me know if
    you have any (other) questions.", "Feel free to …") that starts a sentence. The fix deletes the sentence and one
    adjoining space (the one after it, or the one before it at the end of a block). A sentence alone in its block gets
    no fix.
  - SLP013 `chatbot-opener` (severity `error`, `Safe`) matches "Certainly!", "Great question!", "Absolutely!" or
    "Sure!" at the start of a block, after only block markup. The fix is `delete_lead_in`: delete it and capitalise
    the next letter.
  - The fixture tests resolve each rule's `ParamSpec` defaults, as config does, so a param rule fires there.
- Markup and density rules (`builtin/slp010_*`, `slp011_*`, `slp014_*`), on by default in v0.1.0.
  `rules::helpers::markdown_events(doc)` re-parses a Markdown document for the structure the span map does not keep
  (lists, strong spans; plain text yields none), and `first_prose(doc, range)` gives the trimmed first prose run in a
  range, where a structure rule reports so `detect` does not drop the finding as markup.
  - SLP010 `bullet-spam` (`None`, D-4) reports a list (once, at its first prose) of at least `min_items` (5) items in
    which at least 80% open with a bold lead-in or have no more than `max_words` (6) words.
  - SLP011 `over-bolding` (`None`, D-4) reports every outermost strong span of a block with more than
    `max_per_100_words` (2) strong spans per 100 of the block's words.
  - SLP014 `em-dash-density` (severity `warning`, `Safe`) counts em dashes (U+2014, or the `--` of ` -- `) in prose
    and reports each one only when the count per 1,000 prose words is above `max_per_1000_words` (3). The fix turns a
    dash between two words, with one space on each side when present, into ", "; a dash next to punctuation, a
    non-prose run or a block edge gets no fix.
  - SLP017 `decorative-emoji` (`builtin/slp017_*`, severity `warning`, `Safe`, family Markup) reports each pictographic
    emoji in prose, one finding per grapheme cluster, so a ZWJ sequence, skin tone, flag or keycap counts once. A
    supplementary-plane `Extended_Pictographic` code point or a BMP one with default emoji presentation (✅, ⏳) is an
    emoji; any other BMP pictograph (©, ®, ™, ↔) only when U+FE0F follows it, and a digit, `#` or `*` only as a
    keycap. The fix deletes a cluster of emoji (separated only by spaces) with one adjacent space: the space after it,
    or the space before it when the line ends or closing punctuation follows (`so far 💪.` → `so far.`). An emoji
    that is the only prose on its line (a bare list item or heading) or sits between letters or after opening markup
    is flagged with no fix.
- `directives::Suppressions` holds the suppressed lines. `Default` suppresses nothing; `is_suppressed(line, code)` and
  `frozen_ranges()` are the queries. `with_lines` and `with_file_ignored` (`#[doc(hidden)]`) build a value by hand
  for tests.
- `directives::extract(&mut doc, path, registry)` reads the directive lines: `<!-- sloperator-<kind> [SLP### ...] -->`
  in Markdown, `[sloperator-<kind> [SLP### ...]]` in plain text, each alone on its line (`MD_DIRECTIVE`,
  `TXT_DIRECTIVE`). Kinds: `ignore` (the next line with prose; blank lines and other directives are skipped),
  `disable`/`enable` (a region; an `enable` closes the `disable` with the same code set), `ignore-file` (everything
  is suppressed and frozen). No codes means every rule. A Markdown line counts only when one HTML span covers all of
  it, so a directive-shaped line in a code fence, an indented block, inline code or front matter does nothing.
  Directive lines are re-tagged `Protected(Directive)`. A word that is not a registered rule code (system codes
  included) is `Error::UnknownRule` naming `path:line`; an unclosed `disable`, or an `enable` with no `disable`, is
  `Error::Directive` with the line.
- `Registry::from_rules` (`#[doc(hidden)]`) builds a registry from given rules, for tests.

## Configuration (`config`)

`config::Resolver` turns `sloperator.toml` files into an `Arc<Settings>` per input file; `check` builds it from the
flags and `sloperator config` prints its result. The keys are snake_case TOML (D-9):

```toml
extend = "../sloperator.toml"          # relative to this file; may chain; a cycle is an error
select = ["SLP001", "SLP003"]          # replaces the default set
extend_select = ["SLP005"]
ignore = ["SLP002"]
min_severity = "warning"               # suggestion | warning | error
include = ["**/*.md", "**/*.txt"]      # default: **/*.md, **/*.markdown, **/*.txt
exclude = ["CHANGELOG.md", "vendor/**"]
respect_gitignore = true

[rules.SLP014]
severity = "error"
max_per_1000_words = 5.0               # parameter names and defaults come from RuleMeta::params

[[per_file]]                           # glob is relative to this file's directory
glob = "docs/legal/**"
ignore = ["SLP008"]
min_severity = "error"

[protect]
patterns = ['\bv\d+\.\d+\.\d+\b']
terms = ["Acme Cloud", "Dr. Okafor"]
front_matter = true
math = true
html = true

[metrics]
min_sentences = 5                       # positive integer; default 5
min_words = 100                         # positive integer; default 100

[model]
name = "qwen3-4b"
path = "/models/my.gguf"               # relative paths resolve against this file; requires `profile`
profile = "qwen3"
cache_dir = "~/.cache/sloperator/models"
window = "paragraph"                   # paragraph | sentence
max_window_chars = 4000
max_tokens = 0
seed = 42
```

- **Discovery (FR-7.1).** The nearest `sloperator.toml` above the input file's directory is used, searching up to
  the filesystem root. Parent files are never merged; only `extend` brings another file in. `Resolver::new(cli,
  Some(path))` (`--config`) skips discovery for every file (FR-7.2). A missing `--config` file is `Error::Io`.
- **Resolution (FR-7.5).** Lowest to highest: built-in defaults, the `extend` chain root first, the file itself,
  every matching `[[per_file]]` entry in file order (entries from every file of the chain), then `CliOverrides`.
  At each layer the rule set is `select` (when present, it replaces the set), plus `extend_select`, minus `ignore`.
  The defaults are the registry's default set at each rule's own severity and param defaults, `min_severity`
  `warning`, model `qwen3-4b`, window `paragraph`, `max_window_chars` 4000, `max_tokens` 0 and seed 42.
- **Metric floors.** `[metrics]` is top-level only and inherits each omitted value through `extend`.
  Both values must be integers at least 1; unknown keys and invalid values name `metrics.<key>` in the error.
  `Settings.metrics` holds the resolved floors. `Resolver::run_floors(cwd)` uses the explicit `--config` chain,
  else the nearest config above `cwd`, else 5 sentences and 100 words.
- **Errors (FR-7.6).** Each of these is `Error::Config { file, key, reason }` with a dotted key such as
  `rules.SLP014.max_per_1000_words`, `per_file[0].glob` or `protect.patterns[1]`: a TOML syntax error (key
  `(syntax)`), an unknown key, a wrong type, an unknown or system rule code, an unknown or non-numeric rule
  parameter, an invalid regex or glob, `model.path` without `model.profile` anywhere in the chain, and an `extend`
  cycle (key `extend`, naming every file in the loop). Codes passed on the command line get the same checks, with
  `file` = `<command line>`. A `model.name` missing from the model registry is reported when the model is resolved
  (see Model setup), not while loading config.
- `Settings::enabled(registry)` returns the enabled rules in registry order, each with its resolved severity and
  params. `RuleCode` serializes as `SLP014`, so `Settings` serializes with the same keys the file uses.

## Input (`input`)

`input::resolve(args, stdin_filename, cwd, &mut resolver, notes)` turns the command-line paths into a sorted,
de-duplicated `Vec<SourceRef>`, each with its `Format` and `Arc<Settings>`. `input::read` loads one; `check` uses
both.

- **Forms (FR-2.2).** A file, a directory (walked; `include` picks the files, default `*.md`, `*.markdown`, `*.txt`),
  a glob (an argument with `*`, `?` or `[`; the walk starts at its literal prefix, `*` stays inside one directory,
  `**` crosses them, and `include` does not apply), and `-` for stdin. No arguments means `cwd`. File paths are
  joined onto `cwd`.
- **Stdin (FR-2.3).** `--stdin-filename` sets the format and the config lookup path. Without it, stdin is `-`,
  Markdown, and resolved for `cwd`.
- **Format.** `format_for`: `.txt` is plain text; everything else (`.md`, `.markdown`, `-`) is Markdown.
- **Skipping (FR-2.4).** `.gitignore` files (in walked and parent directories) and `.git/info/exclude` apply unless
  `respect_gitignore = false`. `exclude` always applies. Both are matched relative to the config file's directory,
  or `cwd` when there is none. An explicit file that either one matches is skipped with
  `sloperator: skipped (excluded): PATH` on stderr.
- **Pinned walker defaults (Gap 5).** Hidden directories such as `.github/` are walked; `.git/` is not. `.ignore`
  files and the user's global gitignore are never read. `.gitignore` applies outside a git repo too.
- **Errors.** A missing path, or a glob that matches nothing, is `Error::Io` (exit 2). Invalid UTF-8 is
  `Error::NotUtf8` with the byte offset of the first bad byte; the file is never written.
- **Order (FR-3.6, NFR-6).** The result is sorted by `Origin` (files by path, then stdin), whatever the argument or
  walk order.

## Pipeline (`pipeline`)

`pipeline::analyse(source)` parses the file, extracts its directives and runs the enabled rules. It is pure and safe to
run in parallel. Then one of the following runs:

- `pipeline::lint(analysis)` reports the findings. It takes no rewriter and never calls `verify`, so lint never
  touches the model (FR-2.1, D-3). `score_before` is `None` and `fixed` is `None`.
- `pipeline::fix(analysis, engines, rw, seed)` serves Fix and Diff and runs serially:
  1. **Safe fixes** (`engines.rules`): the findings' fixes are planned against the frozen ranges, and each one is
     checked by `verify::edit`. Verified fixes are applied, and the result is parsed and checked again. A rejected
     fix becomes SLP901 `rewrite-rejected` at warning with the message `rewrite rejected (<check>): <detail>`, and
     its bytes are kept (FR-8.7).
  2. **Model windows** (`engines.model` with `rw` set; FR-9.2). Windows are cut from the step-1 text, or from the
     original with `--model-only`. A `Skip` is SLP902, and so is a window that overlaps one already sent (a sentence
     window across two sentences). Each `Rewrite` goes to `rw.get()?.rewrite`, then through `mask::propose` and
     `verify::edit`. The one-edit candidate is detected again, and `verify::improvement` accepts it only with strictly
     fewer weighted findings in the window (FR-9.5, DQ-3). Any rejection is SLP901 and the window's bytes are kept.
     The model opens on the first window and never when there is none (FR-9.4). Every request carries the same seed
     (FR-9.8). A runtime error is `Err`, so the run exits 2 and writes nothing (FR-9.7). With `--rules-only` the step
     does nothing and the model never loads (FR-2.10).
  3. **Whole-document check**: `verify::document(original, final)`. If it fails, every edit is discarded and one
     SLP901 (`document`) is reported.
  4. **Report**: the findings left in the final text that pass `--min-severity`, plus every SLP9xx, in source order.
     `score` is the score of the final text and `score_before` the score of the original (FR-5.2). `fixed` is set
     only when the text changed.

Running `fix` again on its own output changes nothing (FR-8.9). Each finding is reported as a `Located` with 1-based
line and character column, the matched text and its source line. `pipeline` never uses `cli`, `report`, `write` or
`engine::runtime`.

## Model windows (`engine`)

The model fix path's pure parts (SPEC §16, ADR-04 Option 1). None of them loads or calls a model; `pipeline::fix` step 2
drives them.

- **Windows (`engine::window::windows`, FR-9.2).** One window per prose block (per sentence with
  `model.window = "sentence"`) that overlaps a finding at or above `min_severity` whose rule is `Model` class, or
  `Safe` class with no fix attached. A `None`-class finding never selects a window. A window is a `Rewrite` or a
  `Skip`, and `WindowPlan::skipped_finding` turns a `Skip` into SLP902 `window-skipped` (warning). A window is
  skipped, never silently, when its source already holds `⟦` or `⟧`, its masked text is longer than
  `model.max_window_chars` (default 4000), its continuation lines carry different container prefixes (a lazy
  continuation, mixed indentation), it holds a hard line break, or it overlaps a suppressed region (FR-9.7, DQ-5).
- **Masking (`engine::mask::mask`, DQ-7).** Each run of protected bytes (inline code, link brackets and destinations,
  HTML, math, URLs, user patterns and terms, emphasis markup) becomes `⟦n⟧`, numbered from 1 in source order. A soft
  break becomes one space, together with the next line's container prefix (`> `, list indentation) and any
  trailing whitespace before it; those bytes are never sent to the model.
- **Output contract and unmasking (`engine::mask::propose`, FR-9.3).** An empty `<think></think>` is stripped when
  the profile says so. The output is rejected (`outputContract`) when it is empty, still holds a think block, opens
  with a preamble (`Here is …:`), is wrapped in quotes the window was not wrapped in or in a code fence, holds a
  blank line, or is more than twice the masked length. It is rejected (`placeholders`) unless every `⟦n⟧` appears
  exactly once and no other appears; the model may move them. The placeholders are then restored from the source.
  A multi-line window is re-wrapped greedily to its widest source line, and every line after the first starts with
  the window's prefix (FR-8.2, Gap 7); a word that would open a line as a block marker stays on the line before.
- **Prompt (`engine::prompt`, D-12).** `SYSTEM`, two `FEW_SHOT` pairs, and `max_tokens`: `model.max_tokens`, or
  3/2 × (masked chars / 4) + 16 when that is 0.

## Report (`report`)

`report::render(format, results, &summary, color, out)` writes the file results in one `OutputFormat` (`text`,
`concise`, `json`, `github`; a `clap::ValueEnum`, `text` by default). `Summary::of(results, errors, &floors)` totals a run:
files, findings, `bySeverity` (suggestion, warning, error), files fixed, SLP901 `rejected`, SLP902 `skipped`, the
overall word-weighted score, metrics derived from merged file counts with the run floors, and failed paths. `check --output-format` picks the format.

- Order is stable (FR-3.6): `render` sorts files by `Origin` (files by path, then stdin) and findings by start
  position, so the caller's collection order never shows.
- **text:** `path:line:col SLP001 [warning] message`, the source line indented four spaces, and a caret line under
  the matched characters (to the line's end for a multi-line finding). Each file then gets
  `path: score 12.40 (37 weighted / 2,984 words)`, followed by `path: metrics …`.
  Fix/Diff adds `path: metrics before …` between score and final metrics. After the blank line and unchanged
  summary line, `overall metrics …` shows values derived from merged final counts.
- **concise:** exactly one line per finding, `path:line:col: SLP001 [warning] message`. Line breaks in a message
  become spaces.
- **json:** one document and a newline, camelCase keys: `schemaVersion` (1), `ruleSetVersion`, `files` (each with
  `path`, `score`, `scoreBefore` (`null` for lint), `metrics`, `metricsBefore` (`null` for lint) and `findings`),
  and `summary` (including `metrics`). The additive metrics keys keep `schemaVersion` at 1 and preserve the score shape. Every finding carries its own
  `path`, plus `code`, `name`, `severity`, `start`/`end` (`line`, `column`), `message`, `matched` and
  `fix: {available, class}`. `available` is true for a safe-class finding with a fix attached and for every
  model-class finding. The system codes are named `rewrite-rejected` and `window-skipped`, class `none`. A code that
  is neither a built-in rule nor a system code fails the render with an error.
- **github:** `::warning file=…,line=…,col=…,endLine=…,endColumn=…,title=SLP001 <name>::message`. Suggestion maps to
  `notice`, warning to `warning`, error to `error`. `gh_escape(s, property)` escapes `%`, `\r` and `\n` in the
  message, and also `:` and `,` in the `file` and `title` properties.
- Colour (FR-3.5, DQ-16): `color` is a plain `bool` the caller decides. Only `text` and `concise` colour (the
  severity tag, plus the location and caret in `text`), and only when it is `true`. `json` and `github` never colour.

## Write (`write`)

`write::commit(results, fs, stdout)` commits `--fix` results in order and returns the number of files written. It is
called only when the run produced no exit-2 error (DQ-10).

- **Files.** A file is written only when `fixed` is set, which means its text changed. An unchanged file never
  reaches `Fs`.
- **Atomic (NFR-5).** `RealFs::write_atomic` writes a `NamedTempFile` in the same directory, syncs it, copies the
  original's permissions (the temp file is created 0600) and renames it over the original. A failed write is
  `Error::Io` naming that path (exit 2). Files before it stay fully replaced, and it and later files stay
  byte-identical.
- **Stdin (FR-2.6).** The fixed text, or the original when nothing changed, goes to `stdout`. Stdin is never written
  to disk.
- **Seam.** `Fs` is the injectable file system. Tests use `FailingFs` (`tests/common`), which fails its Nth write.

## Score formula

`score::score(findings, prose_words)` returns a `Score { value, prose_words, weighted }` (JSON keys `value`,
`proseWords`, `weighted`):

```
value = round2(1000 × Σ weight(severity) / max(prose_words, 1))
```

Weights: suggestion 1, warning 3, error 9. `round2` rounds to two decimals, so the JSON is byte-stable.

- The sum runs over the reported findings, so the number and the listing agree.
- System findings (SLP901 and SLP902) are excluded from the score.
- `score::overall` is word-weighted across files: `Σ weighted / Σ prose_words` over all files, not a mean of the
  per-file values.
- The same input and config give the same score. `score` is pure and reads only findings and word counts.

## Text metrics

Metrics are style indicators, not a measure of quality or provenance.

Metrics count paragraph blocks only: headings, list items, table cells and protected spans are excluded.
Words follow Unicode word segmentation; sentences without a prose word are skipped. Syllables are approximate:
vowel groups, silent-e / -ed / -es rules and a small exception table estimate English pronunciation.

With W words, S sentences, Y syllables and T distinct lowercased word types:

- Average sentence length = W / S.
- Grade level (Flesch–Kincaid) = 0.39 × (W / S) + 11.8 × (Y / W) − 15.59.
- Sentence-length variation is the population standard deviation, SD = sqrt((S × Σ sentence_length² − W²) / S²).
  Its coefficient of variation, CV, is SD / (W / S). Sometimes called “burstiness”, this is sentence-length
  variation, not the perplexity measure used by detectors.
- Type/token ratio = T / W. It tends to fall as text grows; avoid comparing documents of very different lengths.

Values are derived from integer counts and rounded once to two decimals. Overall metrics merge the counts and
union the word types across files before deriving values; they never average per-file metrics.
Below either sample floor, every derived value is unavailable (JSON `null`). Defaults are 5 sentences and
100 words; top-level `[metrics] min_sentences` and `min_words` accept integers at least 1.
Per-file reports use that file's resolved floors. Overall metrics use the explicit `--config` chain, else the
nearest config above the working directory, else defaults. Unavailable text names those floors.

JSON keys are `files[].metrics`, `files[].metricsBefore` and `summary.metrics`. A metrics object has `words`,
`sentences`, `avgSentenceLength`, `gradeLevel`, `sentenceLengthSd`, `sentenceLengthCv` and `typeTokenRatio`.
`metricsBefore` is null for lint and holds the original counts' values for Fix/Diff; `metrics` describes the
surviving final text. These additive fields keep `schemaVersion` at 1. Concise and GitHub output omit metrics.

## Limits

The score measures style tells, not text quality or provenance. sloperator does not make text undetectable and
makes no such claim.

Verification covers numbers, URLs, configured terms, placeholders, block structure, negations, modal verbs and
capitalised words as a proxy for names. It does not detect every change of meaning: a reworded claim with the same
negations, modals and capitalised words passes.

## Model setup

`sloperator model pull [NAME]` downloads a model into the cache; `model list` and `model path [NAME]` report what is
there. NAME defaults to the configured `model.name`. The registry:

| Name | Repository | File | Size | Licence | Profile |
|------|------------|------|------|---------|---------|
| `qwen3-4b` (default) | `Qwen/Qwen3-4B-GGUF` | `Qwen3-4B-Q4_K_M.gguf` | 2,497,280,256 B | apache-2.0 | `qwen3` |
| `qwen3-1.7b` | `Qwen/Qwen3-1.7B-GGUF` | `Qwen3-1.7B-Q8_0.gguf` | 1,834,426,016 B | apache-2.0 | `qwen3` |

- **Registry (`model::registry`, ADR-08).** Each `ModelSpec` carries the file's sha256 (from the Hugging Face API),
  its size, its licence and its `PromptProfile`. `qwen3` uses the Qwen3 card's non-thinking sampling settings
  (temp 0.7, top_p 0.8, top_k 20, presence penalty 1.5, ` /no_think` suffix, empty `<think>` block stripped).
- **Cache.** `model.cache_dir`, else the platform cache directory + `sloperator/models` (e.g.
  `~/.cache/sloperator/models`). A model lives at `CACHE/<repo with / as __>/<file>`.
- **Resolve (FR-9.6).** `model.path` with `model.profile` wins. Otherwise `model.name` is looked up in the registry
  (an unknown name is `Error::Usage`) and its cached file is used. A file that is not there is
  `Error::ModelMissing`, whose message names `sloperator model pull <name>` and `--rules-only`. Resolving never
  touches the network (FR-9.1).
- **Pull (`model::pull`, ADR-08, FR-2.14).** The only network code (`tests/layering.rs::network_only_in_pull`).
  `pull(spec, dir, base, progress)` GETs `{base}/{repo}/resolve/main/{file}` and streams it into `FILE.part`, hashing
  as it goes; progress lines go to stderr (`-q` silences them). Only a matching sha256 and size renames `.part` to
  `FILE` and writes the `FILE.sha256` sidecar. An HTTP error, a short or oversized body or a sha256 mismatch removes
  `.part` and is `Error::Download` naming the URL (exit 2). The CLI always passes `HF_BASE`
  (`https://huggingface.co`); tests pass a local server. When the file and a matching sidecar are already there,
  `model pull` prints "up to date" and exits 0.
- **`model list`** prints one tab-separated line per registry entry: name, file, size, licence, `present`/`absent`
  and the cache path. **`model path`** prints the resolved path (honouring `model.path` when no NAME is given) and
  exits 0, or exits 2 with `Error::ModelMissing` when the file is absent.
- **Port (`engine`, ADR-02).** `LazyRewriter::new` resolves the model and runs the factory's `preflight`, so a
  missing or unsupported model fails before any file is processed. The model is opened on the first `get` and
  then reused (FR-9.4). `DisabledFactory` fails preflight with `Error::ModelUnsupported`; `main` uses it only in a
  build without the `model` feature.
- **Runtime (`engine::runtime`, feature `model`, SPEC §21).** `RuntimeFactory::new(verbose)` is the real factory and
  the only code that names `llama_cpp_2` (`tests/layering.rs::runtime_crate_only_in_adapter`). `open` initialises
  the backend (once per process), offloads every layer when the build has a GPU backend (`metal`, `cuda`; Metal is
  also on in a default macOS build) and none otherwise, and loads the model; nothing touches the network (FR-9.1).
  llama.cpp's and ggml's logs are dropped unless `-v`. Each window gets a fresh 4096-token context, a chat-template
  prompt (system, few-shot pairs, window + the profile's suffix) and a sampler chain from the `PromptProfile`
  (presence penalty, top_k, top_p, temp) ending in `dist(seed)`, so the same seed gives the same output on the same
  binary, model and hardware (FR-9.8). A prompt plus `max_tokens` over the context is `Error::Runtime` ("context
  overflow", exit 2); `windows` should have skipped it first. Hitting `max_tokens` is not an error. The model is
  owned and dropped before exit, not leaked: a model alive at `process::exit` trips ggml's Metal teardown assert.
- **Integration tests (`tests/model_real.rs`).** Need the default model in the cache (`sloperator model pull`) and
  fail naming that command when it is absent. They cover one load for many windows, same seed → same output, every
  real rewrite either written after verify or reported as SLP901, and `check --fix` with the proxies pointed at a
  closed port.

Weights are never committed or embedded: `tests/model_cache.rs::no_weights_in_crate` fails on any `*.gguf` file or any
file over 10 MB in the repository (FR-9.10).
