# Technical Specification: Classical (Non-Neural) NLP for Sloperator

**Version:** 0.1
**Date:** 2026-11-02
**Status:** Draft
**Module(s):** `sloperator` (one crate): new `src/text/`; changed `src/doc/`, `src/rules/`, `src/engine/window.rs`, `src/engine/mask.rs`, `src/verify/`, `src/pipeline/`, `src/report/`, `src/mcp/`, `src/config/`, `src/cli/`, `tests/`, `fixtures/rules/`, `eval/`, `docs/`
**Companion:** [PRD](./PRD-classical-nlp.md) · [Architecture](./ARCHITECTURE-classical-nlp.md) · ADRs [01](./adr/ADR-01-text-analysis-layer.md)–[09](./adr/ADR-09-param-bounds.md) · no UX spec (the architecture records "no UX scope": CLI, JSON and MCP output only)
**Depends On:** none. It extends the product SPEC ([`docs/SPEC.md`](../SPEC.md)). §10 there (the rule catalogue) and the verification-limits sentence (`docs/SPEC.md:238`) are edited by §18 and §25 here.

This SPEC details the approved architecture. Where the code forced a departure, see [Architecture Deviation Notes](#architecture-deviation-notes). D-1 to D-11 of the PRD and ADR-01 to ADR-07 and ADR-09 are taken as decided. ADR-08 is still **Proposed** and is routed to Brad as OD-1. Phase 2 (§20, §21) is written for its recommended option, and OD-1 states what changes under the other option.

Code blocks were compile-checked where they are new and non-trivial. `text::segment`, `text::metrics`, `text::syllable`, `text::lexicon`, the tagger engine, `Prose`, `verify::meaning` and the two template regexes were built in a scratch crate outside the worktree, against this crate as a path dependency (`default-features = false`), with `cargo test` and `cargo clippy --all-targets` clean. That run also checked the segmenter: its word count equals `Document::prose_words` on all 77 Markdown files under `fixtures/` and `eval/paragraphs/`, and it holds on 500 random 200-character inputs. Code that edits existing files (pipeline, report, config, rules) is written against the signatures read at the cited lines. The story that applies each one compile-checks it through its gate.

Test commands use the repo's own runner (`docs/DEVELOPMENT.md:20-28`): `make test TESTS='<selector> ...'`. A selector is a substring of a test binary or test name, or `binary_id#test`.

---

## Table of Contents

Phase 1a — baseline
1. [Standalone-scan hygiene of the initiative docs](#1-standalone-scan-hygiene-of-the-initiative-docs)
2. [Model-reliance harness](#2-model-reliance-harness)

Phase 1b — one sentence definition
3. [`doc`: `continuation_prefix_len` and `BlockText::to_text`](#3-doc-continuation_prefix_len-and-blocktextto_text)
4. [`text::segment`](#4-textsegment)
5. [Window sentence ranges use the segmenter; `starts_sentence` moves](#5-window-sentence-ranges-use-the-segmenter-starts_sentence-moves)
6. [`text` module root and layering rows](#6-text-module-root-and-layering-rows)

Phase 1c — metrics shipped
7. [`text::syllable`](#7-textsyllable)
8. [`text::metrics`](#8-textmetrics)
9. [`Prose` (segments and counts)](#9-prose-segments-and-counts)
10. [`RuleCtx.prose` and `rules::detect`](#10-rulectxprose-and-rulesdetect)
11. [Pipeline: `Analysis.prose`, `FileResult.metrics`](#11-pipeline-analysisprose-fileresultmetrics)
12. [Text and JSON reports](#12-text-and-json-reports)
13. [MCP results](#13-mcp-results)
14. [REFERENCE: metrics](#14-reference-metrics)

Phase 1d — metric rules
15. [`ParamSpec` bounds, config check, `explain`](#15-paramspec-bounds-config-check-explain)
16. [`Family` additions and metric-rule helpers](#16-family-additions-and-metric-rule-helpers)
17. [SLP018–SLP021 metric rules](#17-slp018slp021-metric-rules)
18. [Rule-set version, CHANGELOG, catalogue](#18-rule-set-version-changelog-catalogue)

Phase 1e — calibration
19. [Calibration corpus](#19-calibration-corpus)

Phase 2 — wordiness
20. [SLP001 wordiness and nominalisation rows](#20-slp001-wordiness-and-nominalisation-rows)
21. [SLP022 `weak-intensifier`](#21-slp022-weak-intensifier)

Phase 3 — verifier invariants
22. [`text::lexicon`](#22-textlexicon)
23. [`verify::meaning`, new `Check` variants, `verify::edit(resolves)`](#23-verifymeaning-new-check-variants-verifyeditresolves)
24. [`Window.codes`, pipeline call sites, check names](#24-windowcodes-pipeline-call-sites-check-names)
25. [Verification-limits sentence in three places](#25-verification-limits-sentence-in-three-places)

Phase 4 — tagger
26. [`text::tagger`](#26-texttagger)
27. [`Prose::tags` (lazy, once per pass)](#27-prosetags-lazy-once-per-pass)
28. [SLP006 tag-gated subjects](#28-slp006-tag-gated-subjects)

Phase 5 — templates
29. [SLP004 template `X and Y`](#29-slp004-template-x-and-y)
30. [SLP003 template `it's Y`](#30-slp003-template-its-y)
31. [Class change fallout (fixtures macro, JSON `available`, docs)](#31-class-change-fallout)

Phase 6 — redundancy
32. [`text::stem`](#32-textstem)
33. [`text::similarity`](#33-textsimilarity)
34. [SLP023 `redundant-sentence`](#34-slp023-redundant-sentence)
35. [Performance test extension](#35-performance-test-extension)

36. [Testing Strategy](#36-testing-strategy)
37. [Resolved Design Questions](#37-resolved-design-questions)

[Open Decisions](#open-decisions) · [PRD Deviation Notes](#prd-deviation-notes) · [Architecture Deviation Notes](#architecture-deviation-notes) · [Self-Audit](#self-audit) · [Revision Log](#revision-log)

---

## 1. Standalone-scan hygiene of the initiative docs

**File:** `docs/classical-nlp/PRD-classical-nlp.md` (changed at line 298), `docs/classical-nlp/ARCHITECTURE-classical-nlp.md` (changed at line 563)
**Component:** §12 Delivery Architecture, phase 1a gate (all existing tests pass); no ADR
**Implements:** NFR-1, NFR-10

`tests/standalone.rs::no_host_references` scans every file in the repository except `.git` and `target`
(`tests/standalone.rs:37-59`). It fails on any line that names the development host's build contract
(`tests/standalone.rs:16-35`). Two lines of this initiative's own documents do so today, so that test fails on this
branch before any code changes:

- `PRD-classical-nlp.md:298` (NFR-10, "Verified by:"): it names the host build contract's test verb.
- `ARCHITECTURE-classical-nlp.md:563` (§11 Test Architecture, row "Unit"): the same, in its "Runs as" column.

The first story rewrites both cells in the repo's own form, `make test TESTS='text::'` and "the listed tests exist
and pass under `make test`" (`docs/DEVELOPMENT.md:20-28`), and changes nothing else in those documents. This SPEC
and every later artifact in `docs/` use only that form.

**Design notes:**

- This is a fix to the gate, not new scope. Without it the phase-1a gate ("test passes") cannot pass.

---

## 2. Model-reliance harness

**File:** `tests/model_reliance.rs` (new), `eval/model-reliance.md` (new)
**Component:** §4.14 Model-reliance harness
**Implements:** FR-10.1, FR-10.2, FR-10.3

A rules-only fix runs over the 20 eval paragraphs. The test counts the findings left that a model window would take:
a `Model`-class finding, or a `Safe`-class finding with no fix. The predicate is the one in
`src/engine/window.rs:110-119`, rebuilt here from `Registry` metadata, so no new public API is needed. The test then
compares the count per 1,000 prose words with the baseline row of `eval/model-reliance.md`.

```rust
//! FR-10 (classical-nlp SPEC §2): what a rules-only fix leaves for the model, per 1,000 prose words, over
//! `eval/paragraphs`, compared with the baseline recorded in `eval/model-reliance.md`. No model, no network.
#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may unwrap (architecture §8)

use std::path::PathBuf;
use std::sync::Arc;

use sloperator::config::{CliOverrides, Resolver};
use sloperator::engine::DEFAULT_SEED;
use sloperator::input::{Format, Origin, Source};
use sloperator::pipeline::{self, Engines};
use sloperator::rules::FixClass;
use sloperator::rules::registry::Registry;

const RULES_ONLY: Engines = Engines { rules: true, model: false };

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Residual model-eligible findings per 1,000 prose words, rounded to two decimals. Each paragraph is analysed as
/// a file in a temp dir, so no `sloperator.toml` of the checkout applies (as `tests/pipeline.rs:25-37` does).
fn residual_per_1000() -> f64 {
    let tmp = tempfile::tempdir().unwrap();
    let (mut residual, mut words) = (0u64, 0u64);
    for n in 1..=20 {
        let name = format!("p{n:02}.md");
        let text = std::fs::read_to_string(manifest().join("eval/paragraphs").join(&name)).unwrap();
        let path = tmp.path().join(&name);
        let settings = Resolver::new(CliOverrides::default(), None).unwrap().for_file(&path).unwrap();
        let source = Source { origin: Origin::File(path), format: Format::Markdown, text: Arc::from(text), settings };
        let r = pipeline::fix(pipeline::analyse(source).unwrap(), RULES_ONLY, None, DEFAULT_SEED).unwrap();
        words += r.score.prose_words as u64;
        residual += r
            .findings
            .iter()
            .filter(|l| !l.finding.code.is_system())
            .filter(|l| match Registry::builtin().get(l.finding.code).map(|x| x.meta().fix) {
                Some(FixClass::Model) => true,
                Some(FixClass::Safe) => l.finding.fix.is_none(),
                Some(FixClass::None) | None => false,
            })
            .count() as u64;
    }
    (100_000.0 * residual as f64 / words.max(1) as f64).round() / 100.0
}

/// The value of the `| baseline |` row of `eval/model-reliance.md`.
fn baseline() -> f64 {
    let text = std::fs::read_to_string(manifest().join("eval/model-reliance.md")).unwrap();
    let row = text.lines().find(|l| l.starts_with("| baseline |")).expect("no `| baseline |` row");
    row.split('|').nth(2).unwrap().trim().parse().expect("baseline value is a number")
}

/// FR-10.3: never above the baseline. The current value is printed so each phase records it (FR-10.2).
#[test]
fn residual_does_not_exceed_baseline() {
    let (now, base) = (residual_per_1000(), baseline());
    println!("model reliance: {now:.2} per 1,000 prose words (baseline {base:.2})");
    assert!(now <= base, "residual {now:.2} per 1,000 words is above the baseline {base:.2}");
}
```

`eval/model-reliance.md` holds a short header that says what is measured and how to run it
(`make test TESTS='model_reliance'`, with the printed line read from the output), then this table. The phase-1a
story fills in the baseline row from the test's printed value, before any rule or table change (FR-10.2):

```markdown
| phase | residual per 1,000 prose words | date |
|-------|-------------------------------|------|
| baseline | 0.00 | 2026-11-DD |
```

Each later phase that touches rules (2, 4, 5, 6) appends a row named for the phase (`phase 2`, …) with its printed
value and date. The test reads only the baseline row.

**Design notes:**

- The paragraphs are read from the checkout and analysed as temp-dir paths. Config discovery walks up from a
  source's directory (`src/config/mod.rs:1-6`), so a `sloperator.toml` above `eval/` would change the result. Temp
  paths avoid that, as `tests/pipeline.rs` does.
- The new metric, intensifier and redundancy rules are default-off, and the harness uses default settings, so they
  never enter the count. Under default settings they are `FixClass::None` anyway.

---

## 3. `doc`: `continuation_prefix_len` and `BlockText::to_text`

**File:** `src/doc/mod.rs` (changed: new method after `is_protected` at line 274; new method on `BlockText` after line 329), `src/engine/mask.rs` (changed: `prefix_len` at lines 105-117 deleted, call site at line 80), `src/engine/window.rs` (call sites at lines 162, 220, 230)
**Component:** §4.1 Segmenter (the ADR-03 move of `prefix_len` to `doc`); §4.10 Verifier (sentence-initial test)
**Implements:** FR-1.4, FR-6.2, NFR-7, NFR-9

`text` may not use `engine` (ADR-01), so `engine::mask::prefix_len` moves to `Document`, unchanged. `doc` still
uses no crate module (`tests/layering.rs:10`). `BlockText::to_text` is the inverse of `to_src`
(`src/doc/mod.rs:324-329`). The verifier needs it to ask `starts_sentence` about a source offset (§23).

```rust
impl Document {
    /// The container prefix at line start `ls`: the leading spaces, tabs and `>` that are block markup (not prose),
    /// up to `end`. (Moved from `engine::mask::prefix_len`, ADR-03.)
    pub fn continuation_prefix_len(&self, ls: usize, end: usize) -> usize {
        let bytes = self.src.as_bytes();
        let mut i = ls;
        while i < end.min(bytes.len())
            && matches!(bytes[i], b' ' | b'\t' | b'>')
            && self.is_protected(&(i..i + 1))
        {
            i += 1;
        }
        i - ls
    }
}

impl BlockText {
    /// The text byte of a source byte inside a prose segment, or at the end of one; `None` otherwise (an offset in
    /// a protected run or outside the block).
    pub fn to_text(&self, src_off: usize) -> Option<usize> {
        self.map
            .iter()
            .find(|&&(_, s, len)| s <= src_off && src_off <= s + len)
            .map(|&(t, s, _)| t + (src_off - s))
    }
}
```

Every call `mask::prefix_len(doc, a, b)` becomes `doc.continuation_prefix_len(a, b)`, at `src/engine/mask.rs:80` and
`src/engine/window.rs:162, 220, 230`, and `src/engine/window.rs:250` moves into §4. The old function is deleted;
nothing re-exports it (NFR-9).

---

## 4. `text::segment`

**File:** `src/text/segment.rs` (new)
**Component:** §4.1 Segmenter (ADR-03)
**Implements:** FR-1.1, FR-1.2, FR-1.3, FR-1.5, FR-2.10, NFR-7

This is the crate's one definition of sentences and words. The boundary scan is the body of
`engine::window::sentence_range` (`src/engine/window.rs:241-263`), moved here without changing behaviour. Words are
read the way `prose_words` reads them (`src/doc/mod.rs:234-241`). Each word is then assigned to the sentence whose
raw range holds its start, so `words.len() == doc.prose_words()` holds by construction (FR-1.3).

```rust
//! Sentences and words over prose (classical-nlp SPEC §4, ADR-03). Pure: a function of the `Document`, no I/O.

use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

use crate::doc::{BlockId, Document, OBJ, SpanKind};

/// One sentence of one block. `range` is its source bytes with surrounding whitespace trimmed; `words` indexes
/// [`Segments::words`]. A sentence with no prose word (only inline code, say) has an empty `words` range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sentence {
    pub block: BlockId,
    pub range: Range<usize>,
    pub words: Range<u32>,
}

/// One `unicode_words()` word of a prose span. `sentence_initial` is true for the first word of a sentence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Word {
    pub range: Range<usize>,
    pub sentence_initial: bool,
}

/// Owned (no borrow of the Document), so it sits beside the Document in pipeline values (ADR-02). Both vectors are
/// in source order; a word in no block (bytes between blocks) is in `words` but in no sentence.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Segments {
    pub sentences: Vec<Sentence>,
    pub words: Vec<Word>,
}

impl Segments {
    /// The index of the last word that ends at or before source byte `src`.
    pub fn word_before(&self, src: usize) -> Option<usize> {
        self.words.partition_point(|w| w.range.end <= src).checked_sub(1)
    }

    /// The index of the word that starts exactly at source byte `src`.
    pub fn word_at(&self, src: usize) -> Option<usize> {
        let i = self.words.partition_point(|w| w.range.start < src);
        (self.words.get(i)?.range.start == src).then_some(i)
    }

    /// The index of the sentence that holds word `word`; `None` for a word in no sentence.
    pub fn sentence_of(&self, word: usize) -> Option<usize> {
        let w = u32::try_from(word).ok()?;
        let i = self.sentences.partition_point(|s| s.words.end <= w);
        self.sentences.get(i).filter(|s| s.words.contains(&w)).map(|_| i)
    }
}

/// The block's bytes with line breaks and continuation prefixes blanked to spaces (UAX #29 breaks after every line
/// feed otherwise). `None` for an unknown block. Only ASCII bytes are replaced with ASCII bytes, so the copy is UTF-8.
pub(crate) fn blanked(doc: &Document, block: BlockId) -> Option<String> {
    let b = doc.blocks.get(block.0 as usize)?.range.clone();
    let mut scan = doc.src.as_bytes().get(b.clone())?.to_vec();
    for byte in scan.iter_mut().filter(|b| **b == b'\n' || **b == b'\r') {
        *byte = b' ';
    }
    for (i, _) in doc.src[b.clone()].match_indices('\n') {
        let ls = b.start + i + 1;
        if ls < b.end {
            let p = doc.continuation_prefix_len(ls, b.end);
            scan[ls - b.start..ls - b.start + p].fill(b' ');
        }
    }
    String::from_utf8(scan).ok()
}

/// UAX #29 sentence starts in a blanked block copy, relative to `block_start`; a bound strictly inside a protected
/// run is not a bound.
pub(crate) fn scan_starts(doc: &Document, block_start: usize, scan: &str) -> Vec<usize> {
    scan.split_sentence_bound_indices()
        .map(|(i, _)| i)
        .filter(|&i| i == 0 || !doc.is_protected(&(block_start + i..block_start + i)))
        .collect()
}

/// Sentence start offsets within `block`'s source range, relative to its start (FR-1.2, FR-1.4).
pub fn sentence_starts(doc: &Document, block: BlockId) -> Vec<usize> {
    match (blanked(doc, block), doc.blocks.get(block.0 as usize)) {
        (Some(scan), Some(b)) => scan_starts(doc, b.range.start, &scan),
        _ => vec![0],
    }
}

/// Whether the text byte `at` of a `BlockText` starts a sentence: only whitespace before it, or `.`, `!` or `?` is
/// the nearest character before it. A non-prose run at the start of a line is block markup and is skipped.
/// (Moved unchanged from `rules::helpers`, `src/rules/helpers.rs:138-157`.)
pub fn starts_sentence(text: &str, at: usize) -> bool {
    let mut before = text.get(..at).unwrap_or_default();
    loop {
        before = before.trim_end();
        if before.is_empty() {
            return true;
        }
        if !before.ends_with(OBJ) {
            return before.ends_with(['.', '!', '?']);
        }
        let line = before.trim_end_matches(OBJ).trim_end_matches([' ', '\t']);
        if !(line.is_empty() || line.ends_with('\n')) {
            return false;
        }
        before = line;
    }
}

/// Every block's sentences and every prose word (FR-1.1). Total: malformed input gives fewer sentences, never a
/// panic.
pub fn segment(doc: &Document) -> Segments {
    let mut sentences: Vec<Sentence> = Vec::new();
    let mut raw: Vec<Range<usize>> = Vec::new(); // untrimmed sentence ranges, parallel to `sentences`
    for block in &doc.blocks {
        let Some(scan) = blanked(doc, block.id) else {
            continue;
        };
        let b0 = block.range.start;
        let mut starts = scan_starts(doc, b0, &scan);
        starts.push(scan.len());
        for pair in starts.windows(2) {
            let (s, e) = (pair[0], pair[1]);
            let piece = &scan[s..e];
            let lead = piece.len() - piece.trim_start().len();
            let trail = piece.len() - piece.trim_end().len();
            if lead + trail >= piece.len() {
                continue;
            }
            sentences.push(Sentence { block: block.id, range: b0 + s + lead..b0 + e - trail, words: 0..0 });
            raw.push(b0 + s..b0 + e);
        }
    }
    let mut words: Vec<Word> = Vec::new();
    let mut k = 0;
    for span in doc.spans.iter().filter(|s| s.kind == SpanKind::Prose) {
        for (off, w) in doc.src[span.range.clone()].unicode_word_indices() {
            let at = span.range.start + off;
            while k < raw.len() && raw[k].end <= at {
                k += 1;
            }
            let i = u32::try_from(words.len()).unwrap_or(u32::MAX);
            let mut initial = false;
            if let Some(s) = sentences.get_mut(k)
                && raw[k].start <= at
                && Some(s.block) == span.block
            {
                if s.words.is_empty() {
                    s.words = i..i + 1;
                    initial = true;
                } else {
                    s.words.end = i + 1;
                }
            }
            words.push(Word { range: at..at + w.len(), sentence_initial: initial });
        }
    }
    // A sentence with no word gets an empty range at the running end, so `words.end` never decreases and
    // `sentence_of` can binary-search.
    let mut end = 0;
    for s in &mut sentences {
        if s.words.is_empty() {
            s.words = end..end;
        }
        end = s.words.end;
    }
    Segments { sentences, words }
}
```

**Design notes:**

- `blanked` and `scan_starts` are `pub(crate)` so the window can keep its trimming on the blanked copy, byte for
  byte as today (§5). `sentence_starts` is the public form the architecture names.
- Words come only from prose spans, so protected runs and `OBJ` placeholders are never words (FR-1.2). A sentence
  bound strictly inside a protected run is dropped by `scan_starts` (FR-1.2).
- Architecture Risk 1 (FR-1.3 vs block-less prose) holds by construction: a word with no sentence is still pushed.
  On today's fixtures no word lacks a sentence (scratch run).

---

## 5. Window sentence ranges use the segmenter; `starts_sentence` moves

**File:** `src/engine/window.rs` (changed at lines 7-15 imports, 237-278 `sentence_range`), `src/rules/helpers.rs` (changed at lines 138-157)
**Component:** §4.11 Pipeline and windows; §4.1 Segmenter (ADR-03)
**Implements:** FR-1.4, NFR-9

`sentence_range` keeps only its range arithmetic and trimming. The boundary scan comes from `text::segment`, so the
crate has one definition of a sentence.

```rust
use crate::text::segment;

/// The sentences of `block` that `r` touches, as one source range with surrounding whitespace trimmed. Sentence
/// bounds are `text::segment`'s (FR-1.4); trimming uses the same blanked copy, so continuation prefixes trim away.
fn sentence_range(doc: &Document, block: BlockId, r: &Range<usize>) -> Range<usize> {
    let b = doc.blocks[block.0 as usize].range.clone();
    let Some(scan) = segment::blanked(doc, block) else {
        return b;
    };
    let mut starts = segment::scan_starts(doc, b.start, &scan);
    starts.push(scan.len());
    let (lo, hi) = (
        r.start.saturating_sub(b.start),
        r.end.max(r.start + 1).saturating_sub(b.start),
    );
    let first = starts.partition_point(|&s| s <= lo).saturating_sub(1);
    let last = starts.partition_point(|&s| s < hi).max(first + 1);
    let (s, e) = (starts[first], starts[last.min(starts.len() - 1)]);
    let piece = &scan[s..e];
    let lead = piece.len() - piece.trim_start().len();
    let trail = piece.len() - piece.trim_end().len();
    if lead + trail >= piece.len() {
        return b;
    }
    b.start + s + lead..b.start + e - trail
}
```

The `use unicode_segmentation::UnicodeSegmentation;` import at `src/engine/window.rs:9` is removed, because nothing
else in the file uses it.

In `src/rules/helpers.rs` the body of `starts_sentence` (lines 138-157) is deleted and replaced by
`pub use crate::text::segment::starts_sentence;`. The call sites in SLP008, SLP012 and SLP016
(`slp008…rs:4`, `slp012…rs:7`, `slp016…rs:4`) still compile. This is the one re-export ADR-03 calls for. There is a
single definition, not an alias kept for compatibility.

---

## 6. `text` module root and layering rows

**File:** `src/lib.rs` (changed: `pub mod text;` between `score` and `types`, line 19), `src/text/mod.rs` (new), `tests/layering.rs` (changed at lines 9-95, 96-103, plus a new helper and tests after line 227)
**Component:** §4.8 Per-document context (root), §5 Module rules (ADR-01)
**Implements:** NFR-5, NFR-7, NFR-8

`src/text/mod.rs` declares the submodules as each phase adds them (`segment` in 1b; `syllable`, `metrics` in 1c;
`lexicon` in 3; `tagger` in 4; `stem`, `similarity` in 6) and holds `Prose` (§9, §27). The layering table gains:

```rust
// tests/layering.rs, FORBIDDEN_USES: a new row (ADR-01) …
(
    "src/text/",
    &[
        "crate::rules", "crate::verify", "crate::engine", "crate::pipeline", "crate::report", "crate::config",
        "crate::input", "crate::write", "crate::edit", "crate::score", "crate::directives", "crate::model",
        "crate::mcp", "crate::cli",
    ],
),
// … and the existing `src/verify/` row (line 40-43) gains "crate::rules" (architecture §5).
("src/verify/", &["crate::engine", "crate::write", "crate::report", "crate::rules"]),

// PURE_DIRS gains "src/text/".
```

Two structural tests guard the "once per document pass" property of NFR-2 (§27). They need a variant of `only_in`
(`tests/layering.rs:189-209`) that takes several allowed files, so `only_in` is reimplemented through it:

```rust
/// Asserts that every needle appears in no `src/` file outside `allowed`.
fn only_in_files(needles: &[&str], allowed: &[&str]) {
    let violations: Vec<String> = src_files()
        .into_iter()
        .filter(|(path, _)| !allowed.contains(&path.as_str()))
        .flat_map(|(path, text)| {
            needles.iter().filter(|n| text.contains(**n)).map(|n| format!("{path} mentions {n}")).collect::<Vec<_>>()
        })
        .collect();
    assert!(violations.is_empty(), "only {allowed:?} may mention these:\n{}", violations.join("\n"));
}

fn only_in(needles: &[&str], allowed: &str) {
    only_in_files(needles, &[allowed]);
}

/// NFR-2 / FR-7.3: tags are computed only by `Prose::tags` (Phase 4 adds the call).
#[test]
fn tagger_runs_only_inside_prose() {
    only_in(&["tagger::tag("], "src/text/mod.rs");
}

/// NFR-2: one `Prose` per document pass. It is built where a pass parses (`detect_in`) and in in-crate unit tests.
#[test]
fn prose_built_only_per_pass() {
    only_in_files(&["Prose::new("], &["src/pipeline/mod.rs", "src/text/mod.rs", "src/rules/mod.rs"]);
}
```

`tagger_runs_only_inside_prose` lands with Phase 4. Until then the needle matches nothing, and the test still
passes.

**Design notes:**

- `src/text/` uses `crate::doc` and, from Phase 3, no `crate::types` at all. The architecture allows `types`, and
  nothing in this SPEC needs it. The `NEGATION_EXEMPT` codes live in `verify` (§23), not in `text`.
- NFR-5's "existing no-network test pattern": no generic one exists. Two existing scans cover `src/text/`:
  `pure_modules_have_no_io` (`std::net`, `std::fs`, `std::process`, `std::env` banned under `PURE_DIRS`) and
  `network_only_in_pull` (`tests/layering.rs:214-217`). See PRD Deviation Notes.

---

## 7. `text::syllable`

**File:** `src/text/syllable.rs` (new)
**Component:** §4.3 Syllable counter (ADR-07)
**Implements:** FR-2.3, FR-2.6, NFR-4, NFR-10

```rust
//! Approximate English syllable count (FR-2.6): vowel groups, silent-e / -ed / -es rules, a sorted exception table.
//! Deterministic and offline; REFERENCE documents it as approximate.

/// Words the heuristic miscounts, sorted by word (binary search). A unit test checks order and uniqueness.
static EXCEPTIONS: &[(&str, u32)] = &[
    ("being", 2), ("business", 2), ("create", 2), ("created", 3), ("creates", 2), ("every", 2), ("idea", 3),
    ("ideas", 3), ("poem", 2), ("quiet", 2), ("science", 2), ("society", 4), ("variety", 4),
];

/// The syllables of one word: 0 for a word with no letter (`42`), otherwise at least 1. Non-ASCII letters count
/// as vowels (`café` = 2).
pub fn syllables(word: &str) -> u32 {
    let w: Vec<char> = word.chars().filter(|c| c.is_alphabetic()).flat_map(char::to_lowercase).collect();
    if w.is_empty() {
        return 0;
    }
    let lower: String = w.iter().collect();
    if let Ok(i) = EXCEPTIONS.binary_search_by(|(e, _)| (*e).cmp(lower.as_str())) {
        return EXCEPTIONS[i].1;
    }
    let vowel = |i: usize| match w[i] {
        'a' | 'e' | 'i' | 'o' | 'u' => true,
        'y' => i > 0,
        c => !c.is_ascii(),
    };
    let n = w.len();
    let mut groups: u32 = 0;
    let mut prev = false;
    for i in 0..n {
        let v = vowel(i);
        if v && !prev {
            groups += 1;
        }
        prev = v;
    }
    let consonant = |i: usize| !vowel(i);
    let silent = if n > 2 && w[n - 1] == 'e' && consonant(n - 2) {
        !(w[n - 2] == 'l' && n > 3 && consonant(n - 3)) // "-le" after a consonant: "table" 2
    } else if n > 3 && w[n - 2] == 'e' && w[n - 1] == 'd' {
        consonant(n - 3) && !matches!(w[n - 3], 't' | 'd') // "jumped" 1, "wanted" 2
    } else if n > 3 && w[n - 2] == 'e' && w[n - 1] == 's' {
        consonant(n - 3)
            && !matches!(w[n - 3], 's' | 'x' | 'z' | 'c' | 'g')
            && !(w[n - 3] == 'h' && n > 4 && matches!(w[n - 4], 'c' | 's')) // "makes" 1, "boxes" 2
    } else {
        false
    };
    if silent {
        groups = groups.saturating_sub(1);
    }
    groups.max(1)
}
```

---

## 8. `text::metrics`

**File:** `src/text/metrics.rs` (new)
**Component:** §4.4 Metrics (ADR-04)
**Implements:** FR-2.1, FR-2.2, FR-2.3, FR-2.4, FR-2.5, FR-2.7, FR-2.8, FR-2.9, FR-2.10, NFR-3

```rust
//! Reading metrics from integer counts over paragraph blocks (classical-nlp SPEC §8, ADR-04, D-5, D-6).

use std::collections::BTreeSet;

use crate::doc::{BlockKind, Document};

use super::segment::Segments;
use super::syllable::syllables;

/// Below either floor every derived value is unavailable (D-5, provisional). Rule params may raise them, never
/// lower them (ADR-09).
pub const MIN_SENTENCES: u64 = 5;
pub const MIN_WORDS: u64 = 100;

/// Sufficient statistics (ADR-04). `merge` is integer addition plus set union, so overall = merge of files, never
/// a mean of per-file values (FR-2.8). `types` holds case-folded words and is not serialised.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub words: u64,
    pub sentences: u64,
    pub syllables: u64,
    /// Σ (words in sentence)², which gives the population SD exactly with `words` and `sentences`.
    pub sentence_len_sq: u64,
    pub types: BTreeSet<Box<str>>,
}

/// The reported values, rounded to two decimals; `None` (JSON `null`) when the counts are below the floors.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metrics {
    pub words: u64,
    pub sentences: u64,
    pub avg_sentence_length: Option<f64>,
    pub grade_level: Option<f64>,
    pub sentence_length_sd: Option<f64>,
    pub sentence_length_cv: Option<f64>,
    pub type_token_ratio: Option<f64>,
}

impl Counts {
    pub fn merge(&mut self, other: &Counts) {
        self.words += other.words;
        self.sentences += other.sentences;
        self.syllables += other.syllables;
        self.sentence_len_sq += other.sentence_len_sq;
        self.types.extend(other.types.iter().cloned());
    }

    /// FR-2.2-2.5 and FR-2.7: every value from the integers, rounded once at the end (FR-2.9).
    pub fn derive(&self) -> Metrics {
        let mut m = Metrics {
            words: self.words,
            sentences: self.sentences,
            avg_sentence_length: None,
            grade_level: None,
            sentence_length_sd: None,
            sentence_length_cv: None,
            type_token_ratio: None,
        };
        if self.sentences < MIN_SENTENCES || self.words < MIN_WORDS {
            return m;
        }
        let (w, s) = (self.words as f64, self.sentences as f64);
        let mean = w / s;
        // Population variance as an exact integer numerator: (n·Σx² − (Σx)²) / n².
        let n = u128::from(self.sentences);
        let spread = (n * u128::from(self.sentence_len_sq))
            .saturating_sub(u128::from(self.words) * u128::from(self.words));
        let sd = (spread as f64 / (n * n) as f64).sqrt();
        m.avg_sentence_length = Some(round2(mean));
        m.grade_level = Some(round2(0.39 * mean + 11.8 * (self.syllables as f64 / w) - 15.59));
        m.sentence_length_sd = Some(round2(sd));
        m.sentence_length_cv = Some(round2(sd / mean));
        m.type_token_ratio = Some(round2(self.types.len() as f64 / w));
        m
    }
}

/// The score's rounding rule (`src/score/mod.rs:54-56`), kept private here because `text` may not use `score`.
fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// The counts of paragraph sentences only (D-5, FR-2.10). A sentence with no prose word is skipped.
pub fn counts(doc: &Document, seg: &Segments) -> Counts {
    let mut c = Counts::default();
    for s in &seg.sentences {
        let paragraph = doc.blocks.get(s.block.0 as usize).is_some_and(|b| b.kind == BlockKind::Paragraph);
        if !paragraph || s.words.is_empty() {
            continue;
        }
        let n = u64::from(s.words.end - s.words.start);
        c.sentences += 1;
        c.words += n;
        c.sentence_len_sq += n * n;
        for w in &seg.words[s.words.start as usize..s.words.end as usize] {
            let text = &doc.src[w.range.clone()];
            c.syllables += u64::from(syllables(text));
            let lower = text.to_lowercase();
            if !c.types.contains(lower.as_str()) {
                c.types.insert(lower.into_boxed_str());
            }
        }
    }
    c
}
```

**Design notes:**

- The SD is computed from an exact integer numerator, so the order of summation cannot change its bits (NFR-3).
  Floats appear only inside `derive`.
- Type/token ratio is exactly FR-2.5: distinct `to_lowercase()` words over words. Its length dependence (PRD Risk 3)
  is documented in REFERENCE (§14), and the calibration (§19) records it at several lengths.

---

## 9. `Prose` (segments and counts)

**File:** `src/text/mod.rs` (new)
**Component:** §4.8 Per-document context (ADR-02)
**Implements:** FR-1.5, FR-3.4, NFR-2, NFR-8

```rust
//! Classical text analysis (classical-nlp SPEC, ADR-01): pure functions over a parsed `Document`, plus `Prose`,
//! the per-document-pass context that holds the segmentation and memoises the optional parts.

use std::sync::OnceLock;

use crate::doc::Document;

pub mod metrics;
pub mod segment;
pub mod syllable;

pub use metrics::{Counts, Metrics};
pub use segment::{Segments, Sentence, Word};

/// One document pass's analysis (ADR-02). Owned, so it travels beside its `Document`; always paired with the
/// `Document` it was built from.
pub struct Prose {
    segments: Segments,
    counts: OnceLock<Counts>,
    src_len: usize,
}

impl Prose {
    /// Segments eagerly, because metrics are always reported.
    pub fn new(doc: &Document) -> Prose {
        Prose { segments: segment::segment(doc), counts: OnceLock::new(), src_len: doc.src.len() }
    }

    pub fn segments(&self) -> &Segments {
        &self.segments
    }

    /// The paragraph metric counts, computed on first use.
    pub fn counts(&self, doc: &Document) -> &Counts {
        debug_assert_eq!(doc.src.len(), self.src_len, "a Prose is used with another Document");
        self.counts.get_or_init(|| metrics::counts(doc, &self.segments))
    }
}

/// NFR-8: `Prose` crosses rayon workers inside `Analysis` (`src/cli/mod.rs:171`).
const _: fn() = || {
    fn ok<T: Send + Sync>() {}
    ok::<Prose>();
};
```

Phase 4 adds the `tags` field, its accessor and the test counter (§27).

---

## 10. `RuleCtx.prose` and `rules::detect`

**File:** `src/rules/mod.rs` (changed at lines 9, 83-98, 155-177), `tests/rules_core.rs` (call sites at lines 80, 98, 103, 119), `tests/rules_fixtures.rs` (line 53)
**Component:** §4.9 Rule integration (ADR-02)
**Implements:** FR-7.3, NFR-2

```rust
use crate::text::Prose;

/// What a rule sees: prose only (FR-4.4), the shared per-pass analysis, its resolved params.
pub struct RuleCtx<'a> {
    pub doc: &'a Document,
    /// Built once per document pass by the caller and shared by every rule (FR-7.3, ADR-02).
    pub prose: &'a Prose,
    pub params: &'a BTreeMap<&'static str, f64>,
    pub severity: Severity,
}

pub fn detect(
    doc: &Document,
    prose: &Prose,
    sup: &crate::directives::Suppressions,
    rules: &[Enabled<'_>],
) -> Vec<Finding> {
    let mut out = Vec::new();
    for enabled in rules {
        let ctx = RuleCtx { doc, prose, params: &enabled.params, severity: enabled.severity };
        enabled.rule.check(&ctx, &mut out);
    }
    out.retain(|f| !doc.is_protected(&f.range) && !sup.is_suppressed(doc.line_col(f.range.start).0, f.code));
    out.sort_by_key(|f| (f.range.start, f.code));
    out
}
```

Test call sites build `let prose = Prose::new(&d);` after parsing and pass `&prose`
(`tests/rules_core.rs:80, 98, 103, 119`; `tests/rules_fixtures.rs:53`, where `findings` gains the line).

---

## 11. Pipeline: `Analysis.prose`, `FileResult.metrics`

**File:** `src/pipeline/mod.rs` (changed at lines 17, 33-39, 52-63, 69-77, 80-92, 99-196, 264-292, 294-309), `tests/write.rs` (line 25), `tests/report.rs` (lines 63, 242)
**Component:** §4.11 Pipeline and windows (ADR-02, ADR-05)
**Implements:** FR-3.4, NFR-2

```rust
use crate::text::{self, Counts};

/// One parsed file with its directives, its per-pass analysis and every finding the enabled rules report.
pub struct Analysis {
    pub source: Source,
    pub doc: Document,
    /// Built from `doc` in `detect_in` (ADR-02).
    pub prose: text::Prose,
    pub sup: directives::Suppressions,
    pub findings: Vec<Finding>,
}

pub struct FileResult {
    // … origin, original, findings, score, score_before, fixed unchanged …
    /// Metric counts of the final text (FR-3.4); `Counts::derive` gives the reported values.
    pub metrics: Counts,
    /// Metric counts of the original text; Fix/Diff only.
    pub metrics_before: Option<Counts>,
}

/// What one detection pass returns. An alias keeps the signatures under clippy's `type_complexity` (`-D warnings`).
type Detected = (Document, text::Prose, directives::Suppressions, Vec<Finding>);

fn detect_in(text: Arc<str>, source: &Source) -> Result<Detected, Error> {
    let s = &source.settings;
    let mut d = doc::parse(text, source.format, &s.protect);
    let path = match &source.origin {
        Origin::File(p) => p.clone(),
        Origin::Stdin { as_path } => as_path.clone(),
    };
    let reg = Registry::builtin();
    let sup = directives::extract(&mut d, &path, reg)?;
    // After `extract`, so directive comments are protected and never words.
    let prose = text::Prose::new(&d);
    let findings = rules::detect(&d, &prose, &sup, &s.enabled(reg));
    Ok((d, prose, sup, findings))
}
```

`analyse` destructures four values into `Analysis`. `lint` adds `metrics: a.prose.counts(&a.doc).clone()` and
`metrics_before: None`. Its body still names nothing from the model port, so `lint_never_constructs_rewriter` holds
(`tests/pipeline.rs:44-55`). `fix` changes as follows (line numbers of today's file):

```rust
    let doc0 = a.doc;
    let prose0 = a.prose;
    let findings0 = a.findings;
    let metrics_before = prose0.counts(&doc0).clone();
    // …
    // :146-148 — the step-1 tuple has four values:
            let (base, base_sup, base_findings) = match &step1 {
                Some((d, _, s, f)) => (d, s, f.as_slice()),
                None => (&doc0, &a.sup, findings0.as_slice()),
            };
    // :162-170 — step 2 keeps each pass's Prose with its Document:
            if m.applied.is_empty() {
                step1.map(|(d, p, _, f)| (d, p, f))
            } else {
                let text2: Arc<str> = edit::apply(&base.src, &m.applied, base.newline).into();
                let (d, p, _, f) = detect_in(text2, &a.source)?;
                Some((d, p, f))
            }
        }
        _ => step1.map(|(d, p, _, f)| (d, p, f)),
    };
    // :174-180 — step 3 carries the Prose of whichever text survives:
    let (final_doc, final_prose, final_findings, system) = match step2 {
        None => (doc0, prose0, findings0, system),
        Some((d, p, f)) => match verify::document(&doc0, &d) {
            Ok(()) => (d, p, f, system),
            Err(r) => (doc0, prose0, findings0, vec![rejection(r)]),
        },
    };
    let metrics = final_prose.counts(&final_doc).clone();
    // … FileResult { …, metrics, metrics_before: Some(metrics_before) }
```

`accept` destructures `let (_, _, _, after) = detect_in(candidate, source)?;` (line 283). Every candidate pass
builds one `Prose`, never one per rule (NFR-2). The test helpers that build a `FileResult` literal add
`metrics: Counts::default(), metrics_before: None` (`tests/write.rs:25`, `tests/report.rs:63, 242`).

**Design notes:**

- When the whole-document check discards every edit, `final_prose` is `prose0`, so `metrics == metrics_before`
  (architecture §7.2).
- `prose0.counts` is cloned before `prose0` can move, so there is no borrow across the match.

---

## 12. Text and JSON reports

**File:** `src/report/mod.rs` (changed at lines 14, 33-44, 46-67), `src/report/text.rs` (changed at lines 6-8, 32-35, new `metrics_text`), `src/report/json.rs` (changed at lines 11, 25-32, 100-105), `tests/report.rs` (lines 83-105, 166-168)
**Component:** §4.12 Report and MCP surfaces (ADR-05)
**Implements:** FR-3.1, FR-3.2, FR-3.4, FR-3.5

```rust
// report/mod.rs
use crate::text::{Counts, Metrics};

pub struct Summary {
    // … files, findings, by_severity, fixed_files, rejected, skipped, score, errors unchanged …
    /// Overall metrics: the merge of every file's counts, derived once (FR-2.8).
    pub metrics: Metrics,
}

impl Summary {
    pub fn of(results: &[FileResult], errors: usize) -> Summary {
        // … existing totals …
        let mut counts = Counts::default();
        for r in results {
            counts.merge(&r.metrics);
        }
        Summary { /* … */ metrics: counts.derive(), errors }
    }
}
```

The field is declared after `score` and before `errors`, so the summary keys serialise in that order.

```rust
// report/text.rs — after the score line (today line 32):
        writeln!(out, "{}: score {}", f.path, score_text(&f.result.score))?;
        if let Some(before) = &f.result.metrics_before {
            writeln!(out, "{}: metrics before {}", f.path, metrics_text(&before.derive()))?;
        }
        writeln!(out, "{}: metrics {}", f.path, metrics_text(&f.result.metrics.derive()))?;
    }
    writeln!(out)?;
    writeln!(out, "{}", summary_line(summary))?;
    writeln!(out, "overall metrics {}", metrics_text(&summary.metrics))
}

/// `avg sentence 18.25 words, grade 11.40, sentence-length variation 6.10 (cv 0.33), type/token 0.52`, or
/// `unavailable (3 sentences, 64 words; needs 5 sentences and 100 words)`.
fn metrics_text(m: &Metrics) -> String {
    match (m.avg_sentence_length, m.grade_level, m.sentence_length_sd, m.sentence_length_cv, m.type_token_ratio) {
        (Some(avg), Some(grade), Some(sd), Some(cv), Some(ttr)) => format!(
            "avg sentence {avg:.2} words, grade {grade:.2}, sentence-length variation {sd:.2} (cv {cv:.2}), \
             type/token {ttr:.2}"
        ),
        _ => format!(
            "unavailable ({}, {}; needs {MIN_SENTENCES} sentences and {MIN_WORDS} words)",
            plural(usize::try_from(m.sentences).unwrap_or(usize::MAX), "sentence"),
            plural(usize::try_from(m.words).unwrap_or(usize::MAX), "word"),
        ),
    }
}
```

(`use crate::text::Metrics; use crate::text::metrics::{MIN_SENTENCES, MIN_WORDS};` at the top of `text.rs`.)

```rust
// report/json.rs
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct File<'a> {
    path: &'a str,
    score: Score,
    score_before: Option<Score>,
    metrics: Metrics,
    metrics_before: Option<Metrics>,
    findings: Vec<Finding<'a>>,
}
// in render: metrics: f.result.metrics.derive(), metrics_before: f.result.metrics_before.as_ref().map(Counts::derive),
```

`SCHEMA_VERSION` stays 1 (D-11, ADR-05). `concise` and `github` are not touched (FR-3.5). They print no score today
(`src/report/concise.rs`, `src/report/github.rs`).

Existing tests that pin the shape are updated only for the added output (NFR-1):

- `tests/report.rs` `text_shows_path_line_col_code_severity_message_source_line`: a `docs/a.md: metrics
  unavailable (…)` line after the score line and an `overall metrics unavailable (…)` line after the summary line;
  the count becomes 8.
- The summary JSON assertion (`tests/report.rs:166-168`): a `metrics` object with every value `null` (the helper's
  `Counts::default()`).

**Design notes:**

- The overall metrics are on their own line, after the summary line. This keeps `summary_line` byte-identical for
  every reader that parses it. See the Architecture Deviation Notes.
- Determinism: `Summary::of` merges counts in caller order. Integer addition and set union commute, so the order
  cannot change the result (NFR-3).

---

## 13. MCP results

**File:** `src/mcp/mod.rs` (changed at lines 109-116, 207-220), `docs/MCP.md` (lines 16, 40-50), `tests/mcp.rs`
**Component:** §4.12 Report and MCP surfaces (ADR-05)
**Implements:** FR-3.3

```rust
    async fn lint(&self, args: &JsonObject) -> Result<Value, String> {
        let source = self.source(args)?;
        let r = blocking(move || pipeline::analyse(source).map(pipeline::lint)).await?;
        Ok(json!({
            "score": r.score,
            "metrics": r.metrics.derive(),
            "findings": r.findings.iter().map(finding_json).collect::<Vec<_>>(),
        }))
    }
// fix_json gains, after "scoreAfter":
//      "metricsBefore": r.metrics_before.as_ref().map(crate::text::Counts::derive),
//      "metricsAfter": r.metrics.derive(),
```

`docs/MCP.md` lists `metrics` in the lint row and `metricsBefore`, `metricsAfter` in the fix row and in its example.
`tests/mcp.rs` gains the assertions in §36.

---

## 14. REFERENCE: metrics

**File:** `docs/REFERENCE.md` (new subsection "Text metrics" after "Score formula", line 421; "Report" section line 380)
**Component:** §4.12 Report and MCP surfaces; §4.4 Metrics
**Implements:** FR-3.6, FR-2.6

REFERENCE gains:

- The formulas of FR-2.2 to FR-2.5, with population SD and CV defined, and "sentence-length variation" as the name,
  with "burstiness" as a colloquial alias that is not the perplexity measure detectors use (D-3).
- What is counted: paragraph blocks only. Headings, list items, table cells and protected spans are excluded. Words
  are `unicode_words()` as in the score's word count. A sentence with no prose word is not counted.
- The floors (5 sentences, 100 words) and that below them every value is `null` / "unavailable".
- That syllables are approximate (vowel groups, silent-e, -ed and -es rules, a small exception table), and that
  type/token ratio falls as a text grows, so documents of very different lengths do not compare.
- The sentence: "Metrics are style indicators, not a measure of quality or provenance." (FR-5.4 of the parent
  PRD.)
- The JSON keys (`metrics`, `metricsBefore`, `summary.metrics`) and MCP keys (`metrics`, `metricsBefore`,
  `metricsAfter`), and that `schemaVersion` stays 1 because the change is additive.

---

## 15. `ParamSpec` bounds, config check, `explain`

**File:** `src/rules/mod.rs` (changed at lines 60-66), `src/rules/builtin/slp005_rule_of_three.rs:41-45`, `slp010_bullet_spam.rs:43-52`, `slp011_over_bolding.rs:39-43`, `slp014_em_dash_density.rs:38-42`, `src/config/mod.rs` (changed at lines 512-523), `src/cli/mod.rs` (changed at lines 317-321), `src/mcp/mod.rs` (changed at lines 257-259)
**Component:** §4.13 Config bounds (ADR-09)
**Implements:** NFR-4, FR-4.2, FR-4.3

```rust
/// One tunable threshold; config validates names, types and `[min, max]` against these (FR-7.6, ADR-09).
#[derive(Clone, Copy, Debug)]
pub struct ParamSpec {
    pub name: &'static str,
    pub default: f64,
    /// Inclusive bounds. `f64::MAX` as `max` means unbounded above.
    pub min: f64,
    pub max: f64,
    pub help: &'static str,
}
```

Each of the five existing literals gains `min: 0.0, max: f64::MAX`. That is the widest sane range for counts and
rates (ADR-09), so no config that loads today is rejected. In `rule_tables`, after `n` is computed (line 523) and
before `o.params.insert`:

```rust
            if n.is_nan() {
                return Err(config_err(file, &k, &format!("expected a number, found NaN for {c}")));
            }
            if n < spec.min {
                return Err(config_err(file, &k, &format!("{n} is below the minimum {} for {c}", spec.min)));
            }
            if n > spec.max {
                return Err(config_err(file, &k, &format!("{n} is above the maximum {} for {c}", spec.max)));
            }
```

`config_err(file, key, reason)` (`src/config/file.rs:148`) puts the key `rules.SLPxxx.<name>` in the message, so
rule and param are both named (NFR-4).

`explain` (`src/cli/mod.rs:319-321`) prints each parameter as `  {name} = {default}  {help}  (min {min})` when `max`
is `f64::MAX`, and as `(min {min}, max {max})` otherwise. MCP `explain_rule` (`src/mcp/mod.rs:257-259`) adds
`"min": p.min` and `"max"`, which is `null` when the value is `f64::MAX`.

---

## 16. `Family` additions and metric-rule helpers

**File:** `src/rules/mod.rs` (changed at lines 46-58), `src/rules/helpers.rs` (new items after line 211)
**Component:** §4.9 Rule integration
**Implements:** FR-4.1, FR-4.2, FR-4.3

`Family` gains `Readability`, `Wordiness` and `Redundancy`, serialised `readability`, `wordiness` and `redundancy`
(kebab-case, additive). The helpers that every metric rule shares:

```rust
use crate::doc::BlockKind;
use crate::rules::{ParamSpec, RuleCtx};
use crate::text::Metrics;
use crate::text::metrics::{MIN_SENTENCES, MIN_WORDS};

/// The `min_sentences` param every metric rule declares. Its lower bound is the report floor, so a rule can raise
/// the floor but never fire on a metric the report shows as unavailable (FR-4.3, ADR-09).
pub const MIN_SENTENCES_PARAM: ParamSpec = ParamSpec {
    name: "min_sentences",
    default: MIN_SENTENCES as f64,
    min: MIN_SENTENCES as f64,
    max: f64::MAX,
    help: "Fewest paragraph sentences before the rule can fire (at least the report floor, 5)",
};

pub const MIN_WORDS_PARAM: ParamSpec = ParamSpec {
    name: "min_words",
    default: MIN_WORDS as f64,
    min: MIN_WORDS as f64,
    max: f64::MAX,
    help: "Fewest paragraph words before the rule can fire (at least the report floor, 100)",
};

/// The document's metrics for a metric rule, or `None` below the rule's own minimums or the report floors.
pub fn document_metrics(ctx: &RuleCtx<'_>) -> Option<Metrics> {
    let c = ctx.prose.counts(ctx.doc);
    if (c.sentences as f64) < ctx.param(MIN_SENTENCES_PARAM.name) || (c.words as f64) < ctx.param(MIN_WORDS_PARAM.name)
    {
        return None;
    }
    let m = c.derive();
    m.avg_sentence_length.is_some().then_some(m)
}

/// Where a document-level finding reports (FR-4.1): the first prose run of the first paragraph.
pub fn metric_anchor(doc: &Document) -> Option<Range<usize>> {
    doc.blocks.iter().filter(|b| b.kind == BlockKind::Paragraph).find_map(|b| first_prose(doc, &b.range))
}
```

---

## 17. SLP018–SLP021 metric rules

**File:** `src/rules/builtin/slp018_grade_level.rs`, `slp019_sentence_length_variation.rs`, `slp020_long_sentences.rs`, `slp021_lexical_diversity.rs` (new); `src/rules/builtin/mod.rs` (lines 5-21, 23-43); `fixtures/rules/SLP018/` … `SLP021/` (new); `tests/rules_fixtures.rs` (macro list, lines 135-198)
**Component:** §4.9 Rule integration (rules table rows SLP018–SLP021)
**Implements:** FR-4.1, FR-4.2, FR-4.3, FR-4.4, FR-4.5, FR-4.6, FR-4.7, FR-11.2

SLP018 in full. The other three differ only in the rows of the table below.

```rust
//! SLP018 `grade-level` (classical-nlp SPEC §17): the paragraphs' Flesch-Kincaid grade is above a threshold.
//! Default-off, Suggestion, no fix (D-4); the threshold is provisional until calibrated (FR-4.5).

use crate::rules::helpers::{MIN_SENTENCES_PARAM, MIN_WORDS_PARAM, document_metrics, metric_anchor};
use crate::rules::{Family, Finding, FixClass, ParamSpec, Rule, RuleCode, RuleCtx, RuleMeta, Severity};

const MAX_GRADE: &str = "max_grade";

struct GradeLevel {
    meta: RuleMeta,
}

pub(super) fn rule() -> Box<dyn Rule> {
    Box::new(GradeLevel {
        meta: RuleMeta {
            code: RuleCode(18),
            name: "grade-level",
            family: Family::Readability,
            severity: Severity::Suggestion,
            default_on: false,
            fix: FixClass::None,
            explain: "Reports a document whose paragraphs read above a Flesch-Kincaid grade threshold, computed over \
                      paragraph sentences with an approximate syllable count. It needs at least 5 sentences and 100 \
                      words. The threshold is provisional until calibrated against human prose. Detection only: \
                      no rewrite is attached and no model window is selected.",
            example: (
                "A paragraph of long sentences built from polysyllabic, abstract vocabulary throughout.",
                "Short sentences. Plain words.",
            ),
            params: &[
                ParamSpec {
                    name: MAX_GRADE,
                    default: 12.0,
                    min: 0.0,
                    max: 30.0,
                    help: "Grade level above which the document is reported (provisional)",
                },
                MIN_SENTENCES_PARAM,
                MIN_WORDS_PARAM,
            ],
            since: "0.2.0",
        },
    })
}

impl Rule for GradeLevel {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        let Some(grade) = document_metrics(ctx).and_then(|m| m.grade_level) else {
            return;
        };
        let max = ctx.param(MAX_GRADE);
        if grade <= max {
            return;
        }
        let Some(range) = metric_anchor(ctx.doc) else {
            return;
        };
        let message = format!("grade level {grade:.2} is above {max} (provisional threshold)");
        out.push(ctx.finding(&self.meta, range, message, None));
    }
}
```

| Code | Name / file | Param (default, min, max) | Fires when | Message |
|------|-------------|---------------------------|-----------|---------|
| SLP018 | `grade-level` | `max_grade` (12, 0, 30) | `grade_level > max_grade` | `grade level {g:.2} is above {max} (provisional threshold)` |
| SLP019 | `sentence-length-variation` | `min_cv` (0.35, 0, 5) | `sentence_length_cv < min_cv` | `sentence-length variation is low: cv {cv:.2} is below {min} (provisional threshold)` |
| SLP020 | `long-sentences` | `max_avg_words` (25, 1, 1000) | `avg_sentence_length > max_avg_words` | `average sentence length {a:.2} words is above {max} (provisional threshold)` |
| SLP021 | `lexical-diversity` | `min_ttr` (0.4, 0, 1) | `type_token_ratio < min_ttr` | `type/token ratio {t:.2} is below {min} (provisional threshold)` |

All four rules are `Family::Readability`, `Severity::Suggestion`, `default_on: false`, `FixClass::None` and
`since: "0.2.0"`. They declare `MIN_SENTENCES_PARAM` and `MIN_WORDS_PARAM` after their threshold, and their
`explain` ends with the provisional-threshold sentence (FR-4.5). SLP019's `explain` also says that the measure is
sentence-length variation and not perplexity-based burstiness (D-3). Comparisons use the rounded values, so the
output is byte-stable (NFR-3).

`builtin/mod.rs` gains the four `mod` lines and `rule()` calls, in code order. Each rule gets `positive.md` (a
paragraph run that passes the floors and crosses the default threshold) and `negative.md`. The negative holds a
paragraph run under the threshold, plus a fenced block, inline code and a link, as `check_negative` requires
(`tests/rules_fixtures.rs:86-98`). The macro lists them under `model(…)`, the arm for every non-safe class. Its
first assertion is `assert_ne!(fix_class, Safe)` (`tests/rules_fixtures.rs:123-132`).

**Design notes:**

- A metric finding has `FixClass::None`, so it never selects a window (`src/engine/window.rs:117`; FR-4.4), and it
  enters the score only through the formula as weight 1 (FR-4.6).
- Default thresholds are provisional values, not calibrated ones (D-4). §19 records the calibration values beside
  them, and changing a default is a separate change.

---

## 18. Rule-set version, CHANGELOG, catalogue

**File:** `src/rules/mod.rs:146`, `CHANGELOG.md` (new top section), `docs/REFERENCE.md` (lines 174-219), `docs/SPEC.md` (§10 table, lines 1013-1041), `README.md` (rule count, if it states one)
**Component:** §4.9 Rule integration; §12 phases 1d, 2, 5, 6
**Implements:** FR-11.1, FR-11.2, FR-11.3

`RULE_SET_VERSION` becomes `"0.2.0"` in Phase 1d, and new rules carry `since: "0.2.0"`. `new_rules_start_off`
(`tests/rules_registry.rs:131-146`) then enforces default-off for every one of them. `default_set_is_explicit`
(`:121-128`) is unchanged, because no default changes. `CHANGELOG.md` gains `## 0.2.0 (unreleased)` at the top, and
each phase adds its bullets: rules added (with code, name, severity, class and default), SLP001 rows, the verifier
invariants, the SLP003/SLP004 class change with its JSON consequence (§31), the `ParamSpec` bounds, the
`metrics` keys, and the release-binary size before and after (Phases 1c and 6; NFR-5). The REFERENCE catalogue paragraph (line 174) and its per-rule list, and the `docs/SPEC.md` §10 table,
gain a row per new rule with its params. They change SLP001's, SLP003's, SLP004's and SLP006's rows where the phase
changes them.

**Design notes:**

- A phase that adds or reclassifies rules bumps `RULE_SET_VERSION` only if the version at the top of the CHANGELOG
  is already released. While `0.2.0 (unreleased)` is on top, changes accumulate under it. See DQ-3 and the
  Architecture Deviation Notes.

---

## 19. Calibration corpus

**File:** `eval/calibration/*.md` (new), `eval/calibration/README.md` (new), `tests/calibration.rs` (new)
**Component:** §12 phase 1e (architecture); PRD D-4 follow-up
**Implements:** D-4 follow-up (PRD §7 Phase 1 step 6), supports FR-4.5

The corpus is human prose under a clean licence, used to calibrate the provisional thresholds. It sits in the repo,
not in the binary.

- **Content:** 1–3 MB of English prose in Markdown files of different lengths, at least 12 files. Each file is
  paragraphs only, with headings and lists removed, so all of it counts.
- **Licence policy:** public-domain sources only (DQ-16). Examples are works of the US federal government (17 U.S.C.
  §105) and texts first published before 1929, with any distributor's licence header stripped. Nothing under
  CC BY-SA or any other share-alike or attribution licence goes in.
- **`README.md`:** a table `| file | source (URL) | licence | words |` with one row per file. Below it, a second table
  records the value of each metric per file and over the whole corpus, beside the provisional thresholds of §17
  (FR-4.5, D-4). No default changes in this phase.

```rust
//! Calibration corpus guard (classical-nlp SPEC §19): every file is listed with a public-domain licence, and every
//! file is long enough for metrics. Prints each file's metrics so the README table can be updated by hand.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::sync::Arc;

use sloperator::doc::{self, Format, ProtectSettings};
use sloperator::text::Prose;

#[test]
fn calibration_corpus_is_licensed_and_measurable() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("eval/calibration");
    let readme = std::fs::read_to_string(dir.join("README.md")).unwrap();
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "md") && !p.ends_with("README.md"))
        .collect();
    files.sort();
    assert!(files.len() >= 12, "the corpus has {} files", files.len());
    for path in files {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let row = readme.lines().find(|l| l.starts_with(&format!("| {name} |"))).expect("file not in README");
        assert!(row.contains("| public domain |"), "{name}: licence is not public domain");
        let text = std::fs::read_to_string(&path).unwrap();
        let d = doc::parse(Arc::from(text), Format::Markdown, &ProtectSettings::default());
        let m = Prose::new(&d).counts(&d).derive();
        assert!(m.avg_sentence_length.is_some(), "{name} is below the metric floors");
        println!("{name}: {}", serde_json::to_string(&m).unwrap());
    }
}
```

---

## 20. SLP001 wordiness and nominalisation rows

**File:** `src/rules/builtin/slp001_ai_vocabulary_replaceable.rs` (changed at lines 1-4, 9-27, 43-49), `fixtures/rules/SLP001/positive.md`, `fixed.md`
**Component:** §4.9 Rule integration, row SLP001 (ADR-08, **Proposed**, see OD-1)
**Implements:** FR-5.1, FR-5.2, FR-5.3, FR-5.5, FR-11.2

Written for OD-1 option (a), the ADR-08 recommendation. If Brad picks (b), the rows below move unchanged into a new
default-off Safe rule (see OD-1).

Rows are appended to `ENTRIES` (line 9), each inflection explicit (D-9). Every row's replacement passes the Phase 3
invariants: no row adds or removes a modal or a negation, so FR-6.6 cannot fail on them (DQ-17). No new `None` row is
added, so the table adds no model-eligible findings (FR-10.3, DQ-17). FR-5.3 still holds as a rule for rows: a row
whose replacement is not certain must be `None` or stay out.

| Phrase rows (→ replacement) |
|-----------------------------|
| `make a decision` → `decide`, `makes a decision` → `decides`, `made a decision` → `decided`, `making a decision` → `deciding` |
| `reach a conclusion` → `conclude`, `reaches a conclusion` → `concludes`, `reached a conclusion` → `concluded`, `reaching a conclusion` → `concluding` |
| `take into consideration` → `consider`, `takes into consideration` → `considers`, `took into consideration` → `considered`, `taking into consideration` → `considering`, `taken into consideration` → `considered` |
| `take into account` → `consider`, `takes into account` → `considers`, `took into account` → `considered`, `taking into account` → `considering`, `taken into account` → `considered` |
| `give consideration to` → `consider`, `gives consideration to` → `considers`, `gave consideration to` → `considered`, `giving consideration to` → `considering`, `given consideration to` → `considered` |
| `conduct an analysis of` → `analyze`, `conducts an analysis of` → `analyzes`, `conducted an analysis of` → `analyzed`, `conducting an analysis of` → `analyzing` |
| `perform an analysis of` → `analyze`, `performs an analysis of` → `analyzes`, `performed an analysis of` → `analyzed`, `performing an analysis of` → `analyzing` |
| `conduct an investigation into` → `investigate`, `conducts an investigation into` → `investigates`, `conducted an investigation into` → `investigated`, `conducting an investigation into` → `investigating` |
| `provide assistance to` → `help`, `provides assistance to` → `helps`, `provided assistance to` → `helped`, `providing assistance to` → `helping` |
| `make an attempt to` → `try to`, `makes an attempt to` → `tries to`, `made an attempt to` → `tried to`, `making an attempt to` → `trying to` |
| `has a tendency to` → `tends to`, `have a tendency to` → `tend to`, `had a tendency to` → `tended to` |
| `is in agreement with` → `agrees with`, `are in agreement with` → `agree with`, `was in agreement with` → `agreed with`, `were in agreement with` → `agreed with` |
| `due to the fact that` → `because`, `owing to the fact that` → `because`, `in light of the fact that` → `because` |
| `despite the fact that` → `although`, `in spite of the fact that` → `although` |
| `at this point in time` → `now`, `at the present time` → `now`, `in the near future` → `soon` |
| `in the event that` → `if`, `until such time as` → `until`, `prior to` → `before` |
| `for the purpose of` → `for`, `with the exception of` → `except`, `in close proximity to` → `near` |
| `a sufficient number of` → `enough`, `on a daily basis` → `daily`, `on a regular basis` → `regularly` |

The comment (lines 1-4) and `explain` (lines 43-45) widen to say the rule covers padded verbs, nominalisations
("make a decision") and padded phrases ("due to the fact that"), each inflection fixed separately. `positive.md`
gains a paragraph that uses every new phrase once in a grammatical sentence. `fixed.md` gains the same paragraph
with the replacements, and that text is grammatical (FR-5.2). `fix_is_idempotent_over_all_fixtures`
(`tests/rules_fixtures.rs:526`) covers FR-5.5. No replacement contains a phrase of the table.

**Design notes:**

- Rejected candidates (FR-5.3) and why: `the majority of` → `most` breaks on "the majority of the time". `have the
  ability to` → `can` breaks after "to" and adds a modal. `is able to` → `can` adds a modal. `in the process of`
  needs a deletion with its trailing space, which `WordList`'s whole-word match cannot express
  (`src/rules/helpers.rs:71-75`).

---

## 21. SLP022 `weak-intensifier`

**File:** `src/rules/builtin/slp022_weak_intensifier.rs` (new), `src/rules/builtin/mod.rs`, `fixtures/rules/SLP022/` (new), `tests/rules_fixtures.rs`
**Component:** §4.9 Rule integration, row SLP022 (ADR-08, see OD-1)
**Implements:** FR-5.4, FR-11.2, D-10

This is a `WordList` (`src/rules/helpers.rs:14-65`) whose every entry is `None`. `FixClass::None`, so it never
selects a window. `Family::Wordiness`, `Severity::Suggestion`, `default_on: false`, `since: "0.2.0"`, no params.

```rust
static ENTRIES: &[(&str, Option<&str>)] = &[
    ("absolutely", None), ("actually", None), ("basically", None), ("extremely", None), ("highly", None),
    ("incredibly", None), ("literally", None), ("quite", None), ("really", None), ("totally", None),
    ("truly", None), ("very", None),
];
```

`check` mirrors SLP001's loop (`slp001…rs:62-79`), with the message `` `{word}` adds emphasis, not meaning; consider
deleting it ``. `explain` says it is detection only, because deleting an intensifier changes strength and the
verifier does not check strength (D-10). The fixtures follow §17's macro arm.

---

## 22. `text::lexicon`

**File:** `src/text/lexicon.rs` (new); `src/rules/builtin/slp006_copula_avoidance.rs` (lines 29-32 deleted, line 120-123 changed)
**Component:** §4.2 Closed-class lexicon (ADR-07)
**Implements:** FR-6.1, FR-6.2, FR-6.3, FR-9.1, NFR-4

```rust
//! Closed word lists shared by the verifier, the tagger, SLP006 and redundancy (ADR-07: compile-time tables, each
//! sorted and unique, checked by unit tests).

/// FR-6.1's closed list. `n't` forms count through [`negation_count`].
pub const NEGATIONS: &[&str] =
    &["cannot", "neither", "never", "no", "nobody", "none", "nor", "not", "nothing", "nowhere"];

/// FR-6.3. (SLP006's local copy, which also held "to", is deleted; SLP006 tests "to" itself.)
pub const MODALS: &[&str] = &["can", "could", "may", "might", "must", "shall", "should", "will", "would"];

/// Hedging adverbs: capitalised in title case, never names (the proper-noun proxy skips them).
pub const HEDGES: &[&str] = &["arguably", "perhaps", "possibly", "potentially", "presumably", "probably"];

/// Capitalised tokens that are never names.
pub const NOT_NAMES: &[&str] =
    &["I", "I'd", "I'll", "I'm", "I've", "I\u{2019}d", "I\u{2019}ll", "I\u{2019}m", "I\u{2019}ve"];

/// Function words: removed before redundancy comparison (FR-9.1), and never names. Negations and modals are not
/// in it.
pub const STOP_WORDS: &[&str] = &[
    "a", "about", "above", "after", "again", "against", "all", "also", "am", "an", "and", "any", "are", "as", "at",
    "be", "because", "been", "before", "being", "below", "between", "both", "but", "by", "did", "do", "does",
    "doing", "down", "during", "each", "few", "for", "from", "further", "had", "has", "have", "having", "he", "her",
    "here", "hers", "herself", "him", "himself", "his", "how", "i", "if", "in", "into", "is", "it", "its", "itself",
    "just", "me", "more", "most", "my", "myself", "of", "off", "on", "once", "only", "or", "other", "our", "ours",
    "ourselves", "out", "over", "own", "same", "she", "so", "some", "such", "than", "that", "the", "their",
    "theirs", "them", "themselves", "then", "there", "these", "they", "this", "those", "through", "to", "too",
    "under", "until", "up", "very", "was", "we", "were", "what", "when", "where", "which", "while", "who", "whom",
    "why", "with", "you", "your", "yours", "yourself", "yourselves",
];

fn folded(word: &str) -> String {
    word.to_lowercase().replace('\u{2019}', "'")
}

/// 1 for a word of [`NEGATIONS`] or an `n't` form (ASCII or ’ apostrophe), else 0.
pub fn negation_count(word: &str) -> u32 {
    let w = folded(word);
    u32::from(NEGATIONS.binary_search(&w.as_str()).is_ok() || w.ends_with("n't"))
}

/// The modal a word carries: `won't` → will, `can't`/`cannot` → can, `shan't` → shall, `X'll` → will, `Xn't` → X.
/// `'d` is not mapped (would or had).
pub fn modal_of(word: &str) -> Option<&'static str> {
    let w = folded(word);
    if w.ends_with("'ll") {
        return Some("will");
    }
    let base = match w.as_str() {
        "won't" => "will",
        "can't" | "cannot" => "can",
        "shan't" => "shall",
        other => other.strip_suffix("n't").unwrap_or(other),
    };
    MODALS.iter().copied().find(|m| *m == base)
}

pub fn is_stop_word(lower: &str) -> bool {
    STOP_WORDS.binary_search(&lower).is_ok()
}

/// A capitalised word that is not a name candidate: [`NOT_NAMES`], or a stop word, modal or hedge in any case.
pub fn is_common_capitalised(word: &str) -> bool {
    if NOT_NAMES.contains(&word) {
        return true;
    }
    let w = folded(word);
    is_stop_word(&w) || MODALS.binary_search(&w.as_str()).is_ok() || HEDGES.binary_search(&w.as_str()).is_ok()
}
```

SLP006 deletes its local `MODALS` (lines 29-32) and tests `prev == "to" || lexicon::MODALS.contains(&prev)` where it
tested `MODALS.contains` (lines 120, 123). Its behaviour is unchanged.

---

## 23. `verify::meaning`, new `Check` variants, `verify::edit(resolves)`

**File:** `src/verify/meaning.rs` (new), `src/verify/mod.rs` (changed at lines 1-15, 35-51, 60-105)
**Component:** §4.10 Verifier invariants (ADR-06, D-8)
**Implements:** FR-6.1, FR-6.2, FR-6.3, FR-6.4, FR-8.5, NFR-6

```rust
//! Meaning invariants (classical-nlp SPEC §23, ADR-06): negations, a proper-noun proxy and modal verbs of the
//! replaced bytes and the replacement, each a count or multiset.

use std::collections::BTreeMap;

use unicode_segmentation::UnicodeSegmentation;

use crate::text::lexicon;
use crate::types::{ProposalOrigin, RuleCode};

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Meaning {
    pub(super) negations: u32,
    /// Capitalised, non-sentence-initial words that are not common words (FR-6.2's proxy).
    pub(super) proper: BTreeMap<String, u32>,
    pub(super) modals: BTreeMap<String, u32>,
}

/// D-8: an edit that resolves these may drop one negation per finding.
const NEGATION_EXEMPT: [RuleCode; 2] = [RuleCode(3), RuleCode(4)];

fn bump(map: &mut BTreeMap<String, u32>, key: &str) {
    let slot = map.entry(key.to_owned()).or_insert(0);
    *slot = slot.saturating_add(1);
}

/// `initial_at_start`: whether the text's first word starts a sentence in the document. Inside the text a word is
/// sentence-initial when the nearest non-space text before it (closing quotes and brackets skipped) ends in `.`,
/// `!` or `?`.
pub(super) fn meaning(text: &str, initial_at_start: bool) -> Meaning {
    let mut m = Meaning::default();
    let mut prev_end: Option<usize> = None;
    for (at, word) in text.unicode_word_indices() {
        let gap = text[prev_end.unwrap_or(0)..at]
            .trim()
            .trim_end_matches(['"', '\'', '\u{201D}', '\u{2019}', ')', ']']);
        let initial = if gap.is_empty() { prev_end.is_none() && initial_at_start } else { gap.ends_with(['.', '!', '?']) };
        m.negations = m.negations.saturating_add(lexicon::negation_count(word));
        if let Some(modal) = lexicon::modal_of(word) {
            bump(&mut m.modals, modal);
        }
        if !initial && word.chars().next().is_some_and(char::is_uppercase) && !lexicon::is_common_capitalised(word) {
            bump(&mut m.proper, word);
        }
        prev_end = Some(at + word.len());
    }
    m
}

/// How many negations the edit may drop (D-8): 1 for a safe fix of SLP003/SLP004; for a model rewrite, one per
/// SLP003/SLP004 finding in its window (`resolves`, from `Window.codes`).
pub(super) fn negation_allowance(origin: ProposalOrigin, resolves: &[RuleCode]) -> u32 {
    match origin {
        ProposalOrigin::Rule(code) => u32::from(NEGATION_EXEMPT.contains(&code)),
        ProposalOrigin::Model => {
            u32::try_from(resolves.iter().filter(|c| NEGATION_EXEMPT.contains(c)).count()).unwrap_or(u32::MAX)
        }
    }
}
```

```rust
// verify/mod.rs
mod meaning;

use crate::text::segment;
use crate::types::{Proposal, RuleCode};

pub enum Check {
    // … Numbers … Document unchanged …
    /// The edit drops more negations than its allowance (D-8).
    Negation,
    /// The edit removes, adds or changes a capitalised, non-sentence-initial word.
    ProperNouns,
    /// The edit changes the multiset of modal verbs.
    Modals,
}

/// … (existing doc), plus: drops a negation beyond the D-8 allowance, changes a capitalised non-initial word, or
/// changes the modal verbs. `resolves`: the non-system codes of the findings a model rewrite is meant to resolve
/// (`Window.codes`); empty for a safe fix, whose origin code decides (ADR-06).
pub fn edit(
    doc: &Document,
    p: Proposal,
    frozen: &[Range<usize>],
    protect: &ProtectSettings,
    resolves: &[RuleCode],
) -> Result<Verified<Proposal>, Rejection> {
    // … existing checks through Terms (lines 71-105) …
    let initial = starts_at_sentence(doc, r.start);
    let (mb, ma) = (meaning::meaning(original, initial), meaning::meaning(&p.replacement, initial));
    let allowance = meaning::negation_allowance(p.origin, resolves);
    if ma.negations.saturating_add(allowance) < mb.negations {
        return Err(reject(
            Check::Negation,
            format!("negations {} -> {} (allowance {allowance})", mb.negations, ma.negations),
        ));
    }
    if mb.proper != ma.proper {
        return Err(reject(Check::ProperNouns, tokens::diff(&mb.proper, &ma.proper)));
    }
    if mb.modals != ma.modals {
        return Err(reject(Check::Modals, tokens::diff(&mb.modals, &ma.modals)));
    }
    // … reparse and protected-bytes checks (lines 106-143) unchanged …
}

/// Whether source byte `at` starts a sentence: `text::segment::starts_sentence` on its block's text (ADR-03),
/// which skips line-start markup such as `# `. False outside every block or inside a protected run.
fn starts_at_sentence(doc: &Document, at: usize) -> bool {
    let Some(block) = doc.blocks.iter().find(|b| b.range.start <= at && at < b.range.end) else {
        return false;
    };
    let bt = doc.block_text(block.id);
    bt.to_text(at).is_some_and(|t| segment::starts_sentence(&bt.text, t))
}
```

The checks run after the term check and before the reparse, because they are cheaper (architecture §4.10). Every
`verify::edit` call site passes `resolves`: `&[]` at `src/pipeline/mod.rs:118`, `&w.codes` at `:273` (§24), and `&[]`
in `tests/verify_edits.rs:29, 231, 246`, `tests/rules_fixtures.rs:73` and `tests/edit_apply.rs:52, 206`.

**Design notes:**

- The checks compare multisets, not positions, so a reordering rewrite passes. Negations may rise (FR-6.1 rejects
  reductions only).
- Walking the existing safe fixes through the checks:
  - SLP008/SLP016 lead-in deletion (`It is worth noting that t` → `T`): the first word on each side is
    sentence-initial, so the proxy ignores it (FR-6.2).
  - SLP009 `May Potentially` → `May` (`tests/rules_fixtures.rs:271-277`): "May" is a modal and "Potentially" a hedge,
    so both are common words and the proxy multisets are equal. The modal multiset is `{may}` on both sides.
  - SLP017 emoji deletions contain no word.
  - SLP001 rows replace lower-case words. A mid-sentence all-caps row (`We UTILIZE it`) is now rejected as SLP901,
    loudly (DQ-10).
- The FR-6.6 regression (§36.6) checks every Safe fixture mechanically.

---

## 24. `Window.codes`, pipeline call sites, check names

**File:** `src/engine/window.rs` (changed at lines 17-30, 194-206), `src/pipeline/mod.rs` (changed at lines 118, 273, 331-344), `tests/eval.rs` (lines 46-57)
**Component:** §4.11 Pipeline and windows (ADR-06)
**Implements:** FR-6.4, FR-6.5, D-8, NFR-9

```rust
pub struct Window {
    // … block, range, masked, slots, wrap, weight_before unchanged …
    /// The code of every non-system finding overlapping `range`, one entry per finding (duplicates kept), in
    /// finding order: the D-8 allowance counts SLP003/SLP004 findings (ADR-06).
    pub codes: Vec<RuleCode>,
}

// in plan(), replacing lines 194-198:
    let overlapping = || {
        findings.iter().filter(|f| !f.code.is_system() && f.range.start < range.end && range.start < f.range.end)
    };
    let weight_before = overlapping().map(|f| crate::score::weight(f.severity)).sum();
    let codes = overlapping().map(|f| f.code).collect();
```

`check_name` (`src/pipeline/mod.rs:331-344`) gains `Check::Negation => "negation"`, `Check::ProperNouns =>
"properNouns"` and `Check::Modals => "modals"`. The match is exhaustive, so a missing arm fails the build
(architecture §5). `rejection()` (`:320-329`) already prints the name, so an SLP901 message reads `rewrite rejected
(negation): negations 2 -> 0 (allowance 1)` (FR-6.4, FR-6.5). `tests/eval.rs:46-57` `CHECKS` becomes
`[Check; 13]` with the three new variants, so the eval report lists them.

---

## 25. Verification-limits sentence in three places

**File:** `src/verify/mod.rs:4-5`, `docs/REFERENCE.md:443` (and its "Edit gate" section, line 128), `tests/score.rs:107-111`, `docs/SPEC.md:238`
**Component:** §4.10 Verifier invariants
**Implements:** FR-6.7, NFR-9

The old sentence, "Verification covers numbers, URLs, configured terms, placeholders and block structure, not names
or claim polarity.", is replaced in the same change at `docs/REFERENCE.md:443`, in the `tests/score.rs:107-111`
assertion, and in the module header (`src/verify/mod.rs:5`, which has its own shorter wording). The new text is:

> Verification covers numbers, URLs, configured terms, placeholders, block structure, negations, modal verbs and
> capitalised words as a proxy for names. It does not detect every change of meaning: a reworded claim with the same
> negations, modals and capitalised words passes.

`docs/SPEC.md:238` gets the same text. After the change, a grep for "not names or claim polarity" returns nothing
(NFR-9).

---

## 26. `text::tagger`

**File:** `src/text/tagger/mod.rs`, `src/text/tagger/lexicon_data.rs`, `src/text/tagger/rules_data.rs` (new)
**Component:** §4.5 Tagger (ADR-07, D-7)
**Implements:** FR-7.1, FR-7.2, NFR-4, NFR-10

```rust
// tagger/mod.rs
//! A Brill-style tagger (D-7): lexicon tags, suffix guesses for unknown words, then contextual rules in table
//! order. It is used only as a gate for safe fixes; `certain` is the only property a gate may rely on.

use crate::doc::Document;

use super::segment::Segments;

mod lexicon_data;
mod rules_data;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tag {
    Det, Pron, Noun, ProperNoun, Verb, VerbPast, VerbPart, Verb3s, Modal, Adj, Adv, Prep, Conj, Num, Other,
}

/// One word's tag. `certain`: the lexicon gives exactly one tag and no rule changed it. `unknown`: not in the
/// lexicon; the tag is a suffix guess (FR-7.1). An unknown word is never certain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tagged {
    pub tag: Tag,
    pub certain: bool,
    pub unknown: bool,
}

/// When a contextual rule applies; positions are within one sentence.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Context {
    PrevTag(Tag),
    NextTag(Tag),
    PrevWord(&'static str),
    NextWord(&'static str),
}

/// Change `from` to `to` where `when` holds, only on a word whose lexicon tags include `to`, or an unknown word.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ContextRule {
    pub(crate) from: Tag,
    pub(crate) to: Tag,
    pub(crate) when: Context,
}

fn lookup(lower: &str) -> Option<&'static [Tag]> {
    let lex = lexicon_data::LEXICON;
    lex.binary_search_by(|(w, _)| (*w).cmp(lower)).ok().map(|i| lex[i].1)
}

fn guess(word: &str, sentence_initial: bool) -> Tag {
    let lower = word.to_lowercase();
    if word.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',') {
        Tag::Num
    } else if !sentence_initial && word.chars().next().is_some_and(char::is_uppercase) {
        Tag::ProperNoun
    } else if lower.ends_with("ly") {
        Tag::Adv
    } else if lower.ends_with("ing") {
        Tag::VerbPart
    } else if lower.ends_with("ed") {
        Tag::VerbPast
    } else {
        Tag::Noun
    }
}

/// One tag per entry of `seg.words`, in order (FR-7.1, FR-7.2).
pub fn tag(doc: &Document, seg: &Segments) -> Vec<Tagged> {
    let mut out = Vec::with_capacity(seg.words.len());
    let mut options: Vec<&'static [Tag]> = Vec::with_capacity(seg.words.len());
    for w in &seg.words {
        let text = &doc.src[w.range.clone()];
        match lookup(&text.to_lowercase()) {
            Some(tags) if !tags.is_empty() => {
                out.push(Tagged { tag: tags[0], certain: tags.len() == 1, unknown: false });
                options.push(tags);
            }
            _ => {
                out.push(Tagged { tag: guess(text, w.sentence_initial), certain: false, unknown: true });
                options.push(&[]);
            }
        }
    }
    let word = |i: usize| &doc.src[seg.words[i].range.clone()];
    for s in &seg.sentences {
        let r = s.words.start as usize..s.words.end as usize;
        for rule in rules_data::RULES {
            for i in r.clone() {
                if out[i].tag != rule.from || !(options[i].is_empty() || options[i].contains(&rule.to)) {
                    continue;
                }
                let hit = match rule.when {
                    Context::PrevTag(t) => i > r.start && out[i - 1].tag == t,
                    Context::NextTag(t) => i + 1 < r.end && out[i + 1].tag == t,
                    Context::PrevWord(x) => i > r.start && word(i - 1).eq_ignore_ascii_case(x),
                    Context::NextWord(x) => i + 1 < r.end && word(i + 1).eq_ignore_ascii_case(x),
                };
                if hit {
                    out[i] = Tagged { tag: rule.to, certain: false, unknown: out[i].unknown };
                }
            }
        }
    }
    out
}
```

```rust
// tagger/lexicon_data.rs — sorted by word, lower-case, every word with its common tags, first = most frequent.
use super::Tag::{self, *};
pub(super) static LEXICON: &[(&str, &[Tag])] = &[
    ("a", &[Det]), ("an", &[Det]), ("another", &[Det]), ("are", &[Verb]), ("careful", &[Adj]), ("cheap", &[Adj]),
    ("each", &[Det]), ("every", &[Det]), ("fast", &[Adj, Adv]), ("guide", &[Noun, Verb]), ("her", &[Det, Pron]),
    ("his", &[Det, Pron]), ("its", &[Det]), ("library", &[Noun]), ("main", &[Adj]), ("my", &[Det]),
    ("our", &[Det]), ("quick", &[Adj]), ("ready", &[Adj]), ("service", &[Noun, Verb]), ("simple", &[Adj]),
    ("that", &[Det, Pron, Conj]), ("the", &[Det]), ("their", &[Det]), ("these", &[Det, Pron]),
    ("this", &[Det, Pron]), ("tool", &[Noun, Verb]), ("your", &[Det]),
    // … the full table: closed classes (determiners, pronouns, prepositions, conjunctions, modals, auxiliaries,
    // frequent adverbs) plus frequent nouns, verbs and adjectives, at most 3,000 entries …
];

// tagger/rules_data.rs — applied in this order; at most a few hundred rules (D-7).
use super::{Context, ContextRule, Tag::*};
pub(super) static RULES: &[ContextRule] = &[
    ContextRule { from: Verb, to: Noun, when: Context::PrevTag(Det) },
    ContextRule { from: Noun, to: Verb, when: Context::PrevTag(Modal) },
    ContextRule { from: Noun, to: Verb, when: Context::PrevWord("to") },
    ContextRule { from: Adj, to: Adv, when: Context::NextTag(Adj) },
    // …
];
```

**Lexicon policy.** A word lists every tag it commonly takes in modern technical prose; rare or archaic uses are
left out. The entries shown above are **required**, because the fixtures of §28 and §29 depend on them. `tool`,
`service` and `guide` are noun/verb ambiguous, `fast` is adjective/adverb, and `these`/`this`/`that` are
determiner/pronoun. That is what keeps "The tool serves as a bridge." and "Please export these functions as a
module." unfixed (FR-7.7). A unit test pins these entries (§36.8). The lexicon has no `ProperNoun` entries: a
capitalised word is a proper noun only by guess, and so never certain.

**Design notes:**

- Rules never make a word certain. A rule changes only the best-guess tag, and `certain` comes from the lexicon
  alone (architecture §4.5).
- No third-party tagger or lexicon (D-7). The tables are `static` Rust, so a typo in a tag fails compilation, and
  order, uniqueness and non-emptiness are unit tests (ADR-07, NFR-4).

---

## 27. `Prose::tags` (lazy, once per pass)

**File:** `src/text/mod.rs` (changed: §9's struct), `src/rules/mod.rs` (new `#[cfg(test)] mod tests` at the end)
**Component:** §4.8 Per-document context (ADR-02)
**Implements:** FR-7.3, FR-7.4, NFR-2

```rust
#[cfg(test)]
use std::sync::atomic::{AtomicU32, Ordering};

pub mod tagger;
pub use tagger::{Tag, Tagged};

pub struct Prose {
    segments: Segments,
    tags: OnceLock<Vec<Tagged>>,
    counts: OnceLock<Counts>,
    src_len: usize,
    /// How many times this pass ran the tagger (NFR-2 test hook).
    #[cfg(test)]
    pub(crate) tag_runs: AtomicU32,
}

impl Prose {
    // new(): adds `tags: OnceLock::new()` and `#[cfg(test)] tag_runs: AtomicU32::new(0)`.

    /// One tag per word of `segments().words`, computed on the first call only (FR-7.3, FR-7.4).
    pub fn tags(&self, doc: &Document) -> &[Tagged] {
        debug_assert_eq!(doc.src.len(), self.src_len, "a Prose is used with another Document");
        self.tags.get_or_init(|| {
            #[cfg(test)]
            self.tag_runs.fetch_add(1, Ordering::Relaxed);
            tagger::tag(doc, &self.segments)
        })
    }
}
```

The single-tagging test is an in-crate unit test, because `#[cfg(test)]` items do not exist when `tests/` compiles
the library:

```rust
// src/rules/mod.rs
#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;

    use super::registry::Registry;
    use super::*;
    use crate::directives::Suppressions;
    use crate::doc::{self, Format, ProtectSettings};
    use crate::text::Prose;

    fn enabled(code: u16) -> Enabled<'static> {
        let rule = Registry::builtin().get(RuleCode(code)).unwrap_or_else(|| panic!("SLP{code:03}"));
        Enabled { rule, severity: rule.meta().severity, params: rule.meta().params.iter().map(|p| (p.name, p.default)).collect() }
    }

    fn runs(src: &str) -> u32 {
        let d = doc::parse(Arc::from(src), Format::Markdown, &ProtectSettings::default());
        let prose = Prose::new(&d);
        let _ = detect(&d, &prose, &Suppressions::default(), &[enabled(4), enabled(6)]);
        prose.tag_runs.load(Ordering::Relaxed)
    }

    /// NFR-2 / FR-7.3: SLP006 and SLP004 both read tags in one pass; the tagger runs once.
    #[test]
    fn tagging_runs_once_per_pass() {
        assert_eq!(runs("The library serves as a guide. It is not only cheap but also quick.\n"), 1);
    }

    /// FR-7.4: a document with no candidate is never tagged.
    #[test]
    fn no_candidate_no_tagging() {
        assert_eq!(runs("Plain prose with nothing to fix.\n"), 0);
    }
}
```

With §6's two structural tests, `tagger::tag(` only inside `Prose::tags` and `Prose::new(` only in `detect_in` plus
in-crate tests, this shows one tagging per document pass across a multi-window fix (NFR-2). See the PRD Deviation
Notes.

---

## 28. SLP006 tag-gated subjects

**File:** `src/rules/builtin/slp006_copula_avoidance.rs` (changed at lines 1-10, 12-13, 49-67 `explain`, 111-127, 134-148), `fixtures/rules/SLP006/positive.md`, `fixed.md`, `tests/rules_fixtures.rs` (`safe_fix_contexts_grammatical`, lines 303-356)
**Component:** §4.9 Rule integration, row SLP006
**Implements:** FR-7.5, FR-7.6, FR-7.7, FR-7.8

The existing branches stay exactly as they are. A new branch runs only for the third-person form ("serves",
"stands", "functions"), and only when the listed-subject branch found nothing and the complement guard (the word
after "as" is in `AS_COMPLEMENT`, lines 25-27) passes. It then asks the tagger about the subject. Two subject classes
are covered:

- **Class A (`Det N`):** a sentence-initial singular determiner and a noun. "The library serves as a guide." → "The
  library is a guide."
- **Class B (`Det Adj N`):** the same with one adjective. "The main library serves as a guide." → "The main library
  is a guide."

```rust
use crate::doc::Document;
use crate::text::{Prose, Tag, lexicon};

/// Determiners that can open a singular subject of "serves/stands/functions as" (class A and B).
const SINGULAR_DET: &[&str] =
    &["a", "an", "another", "each", "every", "her", "his", "its", "my", "our", "that", "the", "their", "this", "your"];

/// Two words after "as" that make an idiom, not a noun-phrase complement. Only the tag-gated branch checks them,
/// so the listed-subject branch behaves exactly as before.
const AS_IDIOMS: &[&str] = &["a matter", "a result", "a rule", "a whole"];

/// FR-7.5: the subject before the verb at source byte `verb` is a sentence-initial `Det N` or `Det Adj N` whose
/// words the tagger is certain of, joined by whitespace only, and the clause after "as" holds no verb or modal.
/// Tags are read only here, after every cheaper guard has passed (FR-7.4).
fn tag_certain_subject(doc: &Document, prose: &Prose, verb: usize) -> bool {
    let seg = prose.segments();
    let Some(v) = seg.word_at(verb) else { return false };
    let Some(noun) = v.checked_sub(1) else { return false };
    let lower = |i: usize| doc.src[seg.words[i].range.clone()].to_lowercase();
    if seg.words.get(v + 1).is_none() || lower(v + 1) != "as" {
        return false;
    }
    let tags = prose.tags(doc);
    let certain = |i: usize, t: Tag| tags[i].certain && tags[i].tag == t;
    // Words `a` and `a + 1` are separated by whitespace only, with no protected byte between.
    let joined = |a: usize| {
        let gap = seg.words[a].range.end..seg.words[a + 1].range.start;
        doc.src[gap.clone()].trim().is_empty() && !doc.is_protected(&gap)
    };
    if !certain(noun, Tag::Noun) || !joined(noun) {
        return false;
    }
    let det = match noun.checked_sub(1) {
        Some(d) if certain(d, Tag::Det) => d,
        Some(a) if certain(a, Tag::Adj) && joined(a) => match a.checked_sub(1) {
            Some(d) if certain(d, Tag::Det) => d,
            _ => return false,
        },
        _ => return false,
    };
    if !joined(det) || !seg.words[det].sentence_initial || !SINGULAR_DET.contains(&lower(det).as_str()) {
        return false;
    }
    let Some(s) = seg.sentence_of(v) else { return false };
    let end = seg.sentences[s].words.end as usize;
    let first = v + 2;
    if first + 1 < end && AS_IDIOMS.contains(&format!("{} {}", lower(first), lower(first + 1)).as_str()) {
        return false;
    }
    for i in first..end {
        if i > first {
            let gap = &doc.src[seg.words[i - 1].range.end..seg.words[i].range.start];
            if gap.contains([',', ';', ':', '(', '\u{2014}', '\u{2013}']) {
                break;
            }
        }
        if matches!(tags[i].tag, Tag::Verb | Tag::Verb3s | Tag::VerbPast | Tag::Modal) {
            return false;
        }
    }
    true
}
```

`safe_replacement` (lines 111-127) splits out its first two lines as `fn complement_ok(text: &str, at:
&Range<usize>) -> bool`, which it then calls itself. `check` (lines 143-144) becomes:

```rust
                let fix = safe_replacement(&bt.text, &m, form)
                    .or_else(|| {
                        (form == Form::ThirdPerson
                            && complement_ok(&bt.text, &m)
                            && tag_certain_subject(ctx.doc, ctx.prose, range.start))
                        .then_some("is")
                    })
                    .map(|r| (range.clone(), r.to_string()));
```

`explain` and the module comment add: "or after a sentence-initial determiner and noun (optionally with one
adjective) that the tagger is certain of, when no verb follows in the clause". The rule's class, default and
severity do not change.

**Fixtures and tests** (FR-7.7, FR-7.8):

- `positive.md` and `fixed.md` keep their lines byte-for-byte. Each gains one paragraph: "The library serves as a
  guide. The main library serves as a guide." becomes "The library is a guide. The main library is a guide."
- `safe_fix_contexts_grammatical`: `fixed` gains `(6, "The library serves as a guide.\n", "The library is a
  guide.\n")` and `(6, "The main library serves as a guide.\n", "The main library is a guide.\n")`. `unchanged`
  gains one decline per class and guard:
  - `"Export the library functions as a module.\n"` (determiner not sentence-initial);
  - `"The library functions as a module are deprecated.\n"` (verb in the clause);
  - `"The library serves as a whole.\n"` (idiom);
  - `"The fast library serves as a guide.\n"` (class B with an ambiguous adjective).
  
  The existing `"The tool serves as a bridge.\n"` stays in `unchanged`, because `tool` is ambiguous.

**Phase 4 spike gate** (architecture §12): the tagger must mark at least one class certain on these positives and
decline on every negative. If it cannot, this section is dropped and Phase 4 ships the tagger for Phase 5 only.

---

## 29. SLP004 template `X and Y`

**File:** `src/rules/builtin/slp004_not_just_but_also.rs` (changed at lines 1-2, 4-5, 14-17, 19-39, 46-56), `fixtures/rules/SLP004/positive.md` (unchanged), `fixed.md` (new)
**Component:** §4.9 Rule integration, row SLP004
**Implements:** FR-8.1, FR-8.3, FR-8.4, FR-8.5, FR-8.7

The class becomes `FixClass::Safe`. Detection is unchanged (`PATTERNS`, lines 9-12). For each detection match a
template is tried at the same start. If any guard fails, the finding carries no fix and still selects a model window
(`src/engine/window.rs:116`; FR-8.3).

```rust
use std::ops::Range;

use crate::doc::BlockText;
use crate::rules::helpers::PatternRule;
use crate::rules::{Family, Finding, FixClass, Rule, RuleCode, RuleCtx, RuleMeta, Severity};
use crate::text::Tag;

/// FR-8.1's shape: one word X, an optional comma, "but" with an optional "also", one word Y. Captures 1, 2 = X, Y.
const TEMPLATE: &str = r"(?i)\bnot (?:just|only) (\w+),? but(?: also)? (\w+)\b";

/// Tags X and Y may share; closed classes never join as `X and Y`.
const CONTENT: &[Tag] = &[Tag::Adj, Tag::Adv, Tag::Noun, Tag::Verb, Tag::VerbPast, Tag::VerbPart, Tag::Verb3s];

struct NotJustButAlso {
    meta: RuleMeta,
    patterns: PatternRule,
    template: regex::Regex,
}

// rule(): `fix: FixClass::Safe`; `template: match regex::Regex::new(TEMPLATE) { Ok(r) => r, Err(e) => panic!("rule
// pattern does not compile: {e}") }` (static rule data, as `PatternRule::new` does, src/rules/helpers.rs:103-114);
// explain: "Flags \"not just X, but (also) Y\" and \"not only X but also Y\". The frame pads a plain addition with a
// contrast. When X and Y are single words of the same part of speech that the tagger is certain of, Y ends the
// clause and all of it is one sentence, the fix is \"X and Y\"; otherwise no fix is attached and the model engine
// joins them."

impl NotJustButAlso {
    /// `X and Y`, or `None` when any guard fails (FR-8.1, FR-8.3).
    fn template_fix(&self, ctx: &RuleCtx<'_>, bt: &BlockText, at: usize) -> Option<(Range<usize>, String)> {
        let c = self.template.captures_at(&bt.text, at).filter(|c| c.get(0).is_some_and(|g| g.start() == at))?;
        let (whole, x, y) = (c.get(0)?, c.get(1)?, c.get(2)?);
        if !bt.text[whole.end()..].chars().next().is_none_or(|ch| ".,;:!?".contains(ch)) {
            return None; // Y must end the clause: "but also simple tools" is not joined
        }
        let range = bt.src_range(whole.range())?;
        let (xs, ys) = (bt.src_range(x.range())?, bt.src_range(y.range())?);
        let seg = ctx.prose.segments();
        let (xi, yi) = (seg.word_at(xs.start)?, seg.word_at(ys.start)?);
        if seg.words[xi].range != xs || seg.words[yi].range != ys || seg.sentence_of(xi)? != seg.sentence_of(yi)? {
            return None;
        }
        let tags = ctx.prose.tags(ctx.doc);
        let (tx, ty) = (tags[xi], tags[yi]);
        if !(tx.certain && ty.certain && tx.tag == ty.tag && CONTENT.contains(&tx.tag)) {
            return None;
        }
        let first = if bt.text[at..].starts_with('N') { capitalise(x.as_str()) } else { x.as_str().to_string() };
        Some((range, format!("{first} and {}", y.as_str())))
    }
}

fn capitalise(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

impl Rule for NotJustButAlso {
    fn meta(&self) -> &RuleMeta {
        &self.meta
    }

    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        for bt in ctx.blocks() {
            for (_, m) in self.patterns.find_text(&bt.text) {
                let Some(range) = bt.src_range(m.clone()) else { continue };
                let message = format!("`{}` is a negative parallelism", &ctx.doc.src[range.clone()]);
                let fix = self.template_fix(ctx, &bt, m.start);
                out.push(ctx.finding(&self.meta, range, message, fix));
            }
        }
    }
}
```

The fix range runs from "not" to the end of Y, so it is wider than the finding range, which ends at "but (also)". A
`Finding`'s fix range is independent of its range (`src/rules/mod.rs:107-128`).

`fixed.md` (with the lexicon of §26):

```markdown
The tool is not just fast, but also simple.

It is cheap and quick.

We are careful and ready.
```

Line 1 keeps its finding with no fix (`fast` is ambiguous), so it also serves as the in-fixture decline.
`check_fix` asserts that `fixed.md` has no fixable finding left (`tests/rules_fixtures.rs:100-109`). The verifier
accepts lines 2 and 3 through the D-8 allowance only, because one `not` is dropped. They have no capitalised word
and no modal (FR-8.5).

---

## 30. SLP003 template `it's Y`

**File:** `src/rules/builtin/slp003_not_x_but_y.rs` (changed at lines 1-2, 4-5, 13-16, 18-36, 43-53), `fixtures/rules/SLP003/positive.md` (one line added), `fixed.md` (new)
**Component:** §4.9 Rule integration, row SLP003
**Implements:** FR-8.2, FR-8.3, FR-8.4, FR-8.5, FR-8.7

The class becomes `FixClass::Safe`, with one param:
`ParamSpec { name: "max_words", default: 3.0, min: 1.0, max: 10.0, help: "Most words X and Y may each have for the fix to apply" }`.
The fix deletes the denied clause up to the repeated subject, so the second clause stands as written.

```rust
use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

use crate::doc::{BlockText, OBJ};
use crate::rules::helpers::{PatternRule, delete_lead_in, starts_sentence};
use crate::rules::{Family, Finding, FixClass, ParamSpec, Rule, RuleCode, RuleCtx, RuleMeta, Severity};

const MAX_WORDS: &str = "max_words";

/// FR-8.2's shape inside one sentence: subject + negated copula, X, a clause break that is not a sentence end, an
/// optional "but", the same subject + copula. Captures 1, 2, 3 = first subject, X, second subject.
const TEMPLATE: &str = r"(?i)\b(it|this|that)(?:['’]s not| is not| isn['’]t)\s+([^.!?\n,;:—–]{1,80}?)\s*(?:[,;:—–]|\s--?\s)\s*(?:but\s+)?(it|this|that)(?:['’]s| is)\b";

impl NotXButY {
    /// Deletes "it's not X, " (capitalising the next letter at a sentence start), or `None` when any guard fails.
    fn template_fix(&self, ctx: &RuleCtx<'_>, bt: &BlockText, at: usize) -> Option<(Range<usize>, String)> {
        let c = self.template.captures_at(&bt.text, at).filter(|c| c.get(0).is_some_and(|g| g.start() == at))?;
        let (whole, s1, x, s2) = (c.get(0)?, c.get(1)?, c.get(2)?, c.get(3)?);
        if !s1.as_str().eq_ignore_ascii_case(s2.as_str()) {
            return None; // "This isn't X — it's Y" changes the subject
        }
        let max = ctx.param(MAX_WORDS);
        let x_words = x.as_str().unicode_words().count();
        // A capital, a digit or a protected run in X would be lost; the verifier would reject it, so decline here.
        if x_words == 0
            || x_words as f64 > max
            || x.as_str().chars().any(|ch| ch.is_uppercase() || ch.is_ascii_digit() || ch == OBJ)
        {
            return None;
        }
        let rest = &bt.text[whole.end()..];
        let y = &rest[..rest.find(['.', '!', '?', ',', ';', ':', '\n']).unwrap_or(rest.len())];
        let y_words = y.unicode_words().count();
        if y_words == 0 || y_words as f64 > max || y.contains(OBJ) {
            return None;
        }
        let seg = ctx.prose.segments();
        let first = seg.word_at(bt.src_range(s1.range())?.start)?;
        let second = seg.word_at(bt.src_range(s2.range())?.start)?;
        if seg.sentence_of(first)? != seg.sentence_of(second)? {
            return None;
        }
        let lead = at..s2.start();
        if starts_sentence(&bt.text, at) {
            delete_lead_in(bt, lead)
        } else {
            bt.src_range(lead).map(|r| (r, String::new()))
        }
    }
}
```

The struct gains `template: regex::Regex`, built like §29's. `check` mirrors §29's loop. `explain` changes to: "Flags
"it's not X, it's Y" and "this isn't X — it's Y", in one sentence or in two adjacent sentences. The construction
denies a claim nobody made before it states the real one. In one sentence, with the same subject twice and X and Y
of at most `max_words` words each, the fix deletes the denied clause; otherwise no fix is attached and the model
engine states Y directly." The example becomes `("It's not a bug, it's a feature.", "It's a feature.")`.

`positive.md` gains the line "It's not a bug that we can fix in an hour, it's a feature." (X has 9 words, so the
template declines). The fixed lines read:

```markdown
It's a feature.

This isn't a framework — it's a mindset. That is a hope.

It's not a rule. It's a habit.

It's not a bug that we can fix in an hour, it's a feature.
```

The D-8 allowance accepts each fix (`isn't`/`not` dropped, allowance 1). The replaced bytes `It's not a bug, i` and
the replacement `I` both start a sentence, so the proper-noun proxy sees no word, and no modal changes (FR-8.5).

---

## 31. Class change fallout

**File:** `tests/rules_fixtures.rs` (lines 143-144), `src/report/json.rs` (lines 88-96, comment only), `docs/REFERENCE.md`, `docs/SPEC.md` §10, `CHANGELOG.md`
**Component:** §4.9 Rule integration; §8 persistence (NFR-9)
**Implements:** FR-8.4, FR-8.6, NFR-9

- `rule_fixtures!` moves 3 and 4 to the safe arm: `safe(3, slp003_positive_fires, slp003_negative_silent,
  slp003_fix_matches)` and `safe(4, …, slp004_fix_matches)`. These are the documented edits to existing tests that
  FR-8.4 makes necessary (NFR-1).
- JSON `fix.available` for SLP003/SLP004 becomes "a template fix was attached" (`src/report/json.rs:90-91`), so it is
  `false` for an unfixed match even though the model may still rewrite it (architecture §14 Risks). The code does not
  change. The comment at line 88 adds: "Safe-class findings without a fix may still be rewritten by the model."
  REFERENCE and the CHANGELOG state the change.
- SLP007 and SLP015 are not touched (FR-8.6). §36.9 asserts their class and `explain`, and the story's diff shows
  no change under `slp007…rs` and `slp015…rs`.

---

## 32. `text::stem`

**File:** `src/text/stem.rs` (new); `Cargo.toml` and `deny.toml` (only if the crate is chosen)
**Component:** §4.6 Stemmer facade (D-9)
**Implements:** FR-9.1, D-9, NFR-5

The Phase 6 story runs `cargo deny check` with `rust-stemmers = "=1.2.0"` added (`deny.toml:2-14`: MIT and
BSD-3-Clause are allowed, and yanked crates are denied). It records the result in `CHANGELOG.md` and keeps exactly
one of the two bodies below behind the same signature.

```rust
//! One stemming function for redundancy (D-9). Input is lower case.
use std::borrow::Cow;

// (a) If `cargo deny check` passes with `rust-stemmers = "=1.2.0"`:
use std::sync::LazyLock;
static STEMMER: LazyLock<rust_stemmers::Stemmer> =
    LazyLock::new(|| rust_stemmers::Stemmer::create(rust_stemmers::Algorithm::English));
pub fn stem(word_lower: &str) -> Cow<'_, str> {
    STEMMER.stem(word_lower)
}

// (b) Otherwise, the in-tree suffix stripper: the first matching rule applies, then a final `e` is dropped when
// at least 4 letters remain, so "make", "makes" and "making" meet at "mak".
const SUFFIXES: &[(&str, &str, usize)] = &[
    ("sses", "ss", 2), ("ies", "y", 2), ("ing", "", 3), ("ed", "", 3), ("ly", "", 3), ("s", "", 3),
];
pub fn stem(word_lower: &str) -> Cow<'_, str> {
    let mut w: Cow<'_, str> = Cow::Borrowed(word_lower);
    if !word_lower.ends_with("ss") {
        for (suffix, with, keep) in SUFFIXES {
            if let Some(base) = word_lower.strip_suffix(suffix)
                && base.chars().count() >= *keep
            {
                w = Cow::Owned(format!("{base}{with}"));
                break;
            }
        }
    }
    match w.strip_suffix('e') {
        Some(base) if base.chars().count() >= 4 => Cow::Owned(base.to_string()),
        _ => w,
    }
}
```

**Design notes:**

- Choosing (a) adds `rust-stemmers` to `Cargo.toml` with an exact pin, the house style for every dependency
  (`Cargo.toml` `[dependencies]`), and adds `"rust_stemmers"` to an `only_in` test that permits it only in
  `src/text/stem.rs`.

---

## 33. `text::similarity`

**File:** `src/text/similarity.rs` (new)
**Component:** §4.7 Similarity
**Implements:** FR-9.1, FR-9.5

The measure is weighted Jaccard over stemmed content-word counts: Σ min / Σ max over the union. FR-9.1 allows Jaccard
or cosine over term counts, and this is Jaccard over term counts. It needs no square root, and its value comes from
integers (DQ-14).

```rust
use std::collections::BTreeMap;

use crate::doc::Document;

use super::lexicon;
use super::segment::{Segments, Sentence};
use super::stem::stem;

/// Stemmed content words of one sentence with their counts; stop words and words with no letter removed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Bag(BTreeMap<Box<str>, u32>);

impl Bag {
    /// The number of content words (with repeats).
    pub fn size(&self) -> u32 {
        self.0.values().sum()
    }
}

pub fn bag(doc: &Document, seg: &Segments, s: &Sentence) -> Bag {
    let mut map: BTreeMap<Box<str>, u32> = BTreeMap::new();
    for w in &seg.words[s.words.start as usize..s.words.end as usize] {
        let lower = doc.src[w.range.clone()].to_lowercase();
        if lexicon::is_stop_word(&lower) || !lower.chars().any(char::is_alphabetic) {
            continue;
        }
        *map.entry(stem(&lower).into_owned().into_boxed_str()).or_insert(0) += 1;
    }
    Bag(map)
}

/// Weighted Jaccard in [0, 1]; 0 when both bags are empty.
pub fn jaccard(a: &Bag, b: &Bag) -> f64 {
    let (mut min, mut max) = (0u64, 0u64);
    let mut keys: Vec<&Box<str>> = a.0.keys().chain(b.0.keys()).collect();
    keys.sort();
    keys.dedup();
    for k in keys {
        let (x, y) = (a.0.get(k).copied().unwrap_or(0), b.0.get(k).copied().unwrap_or(0));
        min += u64::from(x.min(y));
        max += u64::from(x.max(y));
    }
    if max == 0 { 0.0 } else { min as f64 / max as f64 }
}
```

---

## 34. SLP023 `redundant-sentence`

**File:** `src/rules/builtin/slp023_redundant_sentence.rs` (new), `src/rules/builtin/mod.rs`, `fixtures/rules/SLP023/` (new), `tests/rules_fixtures.rs`
**Component:** §4.9 Rule integration, row SLP023
**Implements:** FR-9.1, FR-9.2, FR-9.3, FR-9.4, FR-9.5, FR-9.6, FR-11.2

`Family::Redundancy`, `Severity::Suggestion`, `default_on: false`, `FixClass::None`, `since: "0.2.0"`. Params:

| Param | Default | Min | Max | Meaning |
|-------|---------|-----|-----|---------|
| `min_overlap` | 0.6 | 0.1 | 1.0 | weighted Jaccard at or above which the later sentence is reported (FR-9.2) |
| `min_content_words` | 4 | 1 | 100 | sentences with fewer content words are skipped (FR-9.4) |
| `min_sentences` | 3 | 2 | f64::MAX | documents with fewer paragraph sentences are skipped (FR-9.4) |

```rust
    fn check(&self, ctx: &RuleCtx<'_>, out: &mut Vec<Finding>) {
        let seg = ctx.prose.segments();
        if (ctx.prose.counts(ctx.doc).sentences as f64) < ctx.param(MIN_SENTENCES) {
            return;
        }
        let (min_overlap, min_words) = (ctx.param(MIN_OVERLAP), ctx.param(MIN_CONTENT_WORDS));
        // Sentences are in block order, so each paragraph's sentences are one contiguous run (FR-9.5).
        for run in seg.sentences.chunk_by(|a, b| a.block == b.block) {
            let paragraph = ctx.doc.blocks.get(run[0].block.0 as usize).is_some_and(|b| b.kind == BlockKind::Paragraph);
            if !paragraph {
                continue;
            }
            let bags: Vec<Bag> = run.iter().map(|s| similarity::bag(ctx.doc, seg, s)).collect();
            for j in 1..run.len() {
                if f64::from(bags[j].size()) < min_words {
                    continue;
                }
                let earlier = (0..j).find(|&i| {
                    f64::from(bags[i].size()) >= min_words && similarity::jaccard(&bags[i], &bags[j]) >= min_overlap
                });
                let Some(i) = earlier else { continue };
                let Some(range) = first_prose(ctx.doc, &run[j].range) else { continue };
                let (line, _) = ctx.doc.line_col(run[i].range.start);
                let message = format!(
                    "this sentence repeats {:.0}% of the content words of the sentence on line {line}",
                    100.0 * similarity::jaccard(&bags[i], &bags[j])
                );
                out.push(ctx.finding(&self.meta, range, message, None));
            }
        }
    }
```

`explain` says it is detection only, because deleting a sentence cannot be verified (FR-9.6), and that the
thresholds are provisional. The cost is pairwise within one paragraph, O(Σ k²) over paragraphs of k sentences
(FR-9.5). The finding is on the later sentence's first prose run, so `detect` never drops it for overlapping inline
code (`src/rules/mod.rs:172-174`).

---

## 35. Performance test extension

**File:** `tests/perf.rs` (changed at lines 105-112, the written `sloperator.toml`)
**Component:** §8 Performance budgets; §11 Test Architecture (Performance row)
**Implements:** NFR-2, FR-9.5

The written config gains `extend_select` (a top-level key, `src/config/file.rs:18`). Each phase that adds a rule
extends the list: `["SLP018", "SLP019", "SLP020", "SLP021"]` in Phase 1d, `"SLP022"` in Phase 2, `"SLP023"` in
Phase 6. SLP003, SLP004 and SLP006 are on by default, so their template and tag paths run too. The perf corpus is
built from every rule's `positive.md` and `negative.md` (`tests/perf.rs:31-61`), so it includes the new fixtures and
their candidate hits.

```rust
        format!(
            "respect_gitignore = false\nextend_select = [\"SLP018\", \"SLP019\", \"SLP020\", \"SLP021\", \"SLP022\", \
             \"SLP023\"]\n\n[model]\ncache_dir = {}\n",
            toml_path(&cache)
        ),
```

The 1-second budget (`tests/perf.rs:16`) is unchanged (NFR-2).

---

## 36. Testing Strategy

**Implements:** NFR-10 (and the "Verified by" of NFR-1 to NFR-9)

Run each group with `make test TESTS='<selector>'`. Unit tests live in `#[cfg(test)] mod tests` of the named source
file. Gates for the slice stage follow the phase order of §1–§35.

### 36.1 Segmenter

**File:** `src/text/segment.rs` (unit) and `tests/text_segment.rs` (new)

| Test | Verifies |
|------|----------|
| `segment::tests::two_sentences_words_and_initials` | FR-1.1 — "It is. We go." gives 2 sentences, 4 words, `sentence_initial` on "It" and "We" |
| `segment::tests::protected_span_is_not_a_word_or_bound` | FR-1.2 — "Use `a. b` now." is 1 sentence; the code span gives no word |
| `segment::tests::word_lookup_helpers` | `word_before`, `word_at`, `sentence_of` on a two-sentence block, including an empty-word sentence |
| `text_segment::word_count_equals_prose_words_on_every_fixture` | FR-1.3 — every `.md` under `fixtures/` and `eval/paragraphs/` |
| `text_segment::random_input_never_panics` | architecture §8 errors — 500 xorshift-generated 200-char strings (Markdown punctuation, `OBJ`, `é`, newlines); word count still equals `prose_words` |
| `text_segment::sentence_starts_match_window_sentences` | FR-1.4 — for each fixture paragraph, `sentence_starts` equals the bounds the sentence-mode window uses (through `windows` with `model.window = "sentence"` on a Model-class finding) |

Existing window and mask tests (`tests/engine.rs`) pass unmodified: the phase-1b gate (ADR-03).

### 36.2 Syllables and metrics

**File:** `src/text/syllable.rs`, `src/text/metrics.rs` (unit), `tests/text_metrics.rs` (new)

| Test | Verifies |
|------|----------|
| `syllable::tests::fixed_list` | FR-2.6, NFR-10 — the, make, table, free, jumped, wanted, played, freed, makes, boxes, matches, uses, goes, people, idea, every, café, readability, sentence, cat, 42, I, simple, sloperator = 1,1,2,1,1,2,1,1,1,2,2,2,1,2,3,2,2,5,2,1,0,1,2,4 |
| `syllable::tests::exceptions_sorted_unique` | ADR-07, NFR-4 |
| `metrics::tests::hand_computed_values` | FR-2.2–2.5 — words 100, sentences 5, syllables 150, Σx² 2200, 52 types → avg 20.0, grade 9.91, SD 6.32, CV 0.32, TTR 0.52 |
| `metrics::tests::below_floor_is_unavailable` | FR-2.7 — 4 sentences gives every value `None`; JSON shows `null` |
| `metrics::tests::merge_unions_types` | FR-2.8, ADR-04 — merging a copy with itself keeps 52 types; TTR halves to 0.26 |
| `metrics::tests::paragraphs_only` | FR-2.10, D-5 — a document whose heading, list item and table cell add no count |
| `text_metrics::json_identical_across_runs` | NFR-3 — two CLI runs of `check --output-format json` on a metrics-heavy document give identical bytes (as `tests/score.rs:82-99` does for the score) |
| `text_metrics::fix_reports_before_and_after` | FR-3.4 — a rules-only fix of an SLP001-heavy document: `metrics_before` is Some and its word count differs from `metrics` |

### 36.3 Reports and MCP

**File:** `tests/report.rs`, `tests/mcp.rs`

| Test | Verifies |
|------|----------|
| `text_shows_path_line_col_code_severity_message_source_line` (updated) | FR-3.1 — metrics line after the score line; overall line after the summary |
| `text_metrics_line_lists_values` (new) | FR-3.1 — with available counts, the line reads `metrics avg sentence … type/token …` |
| `text_fix_prints_metrics_before` (new) | FR-3.4 — `metrics before` line when `metrics_before` is Some |
| `json_file_has_metrics_and_metrics_before` (new) | FR-3.2 — `files[0].metrics` and `metricsBefore` present; `score` shape unchanged |
| summary JSON assertion (updated, `:166-168`) | FR-3.2 — `summary.metrics` with `null` values |
| `concise_and_github_print_no_metrics` (new) | FR-3.5 — no `metrics` text in either format |
| `mcp::lint_has_metrics` (new) | FR-3.3 |
| `mcp::fix_has_metrics_before_and_after` (new) | FR-3.3 |

### 36.4 Config, registry and metric rules

**File:** `tests/config.rs`, `tests/rules_registry.rs`, `tests/rules_fixtures.rs`

| Test | Verifies |
|------|----------|
| `config::rejects_param_below_minimum` | NFR-4 — `[rules.SLP018]\nmin_sentences = 0` is `Error::Config` naming `rules.SLP018.min_sentences` and `SLP018` |
| `config::rejects_negative_threshold` | NFR-4 — `max_grade = -1` rejected |
| `config::rejects_param_above_maximum` | NFR-4 — `min_ttr = 2` rejected |
| `config::existing_params_still_load` | ADR-09 — `max_per_1000_words = 1.0` (`tests/config.rs:260`) loads |
| `rules_registry::param_defaults_within_bounds` | ADR-09 — every rule's every `ParamSpec` has `min <= default <= max` |
| `rules_registry::metric_rules_explain_provisional` | FR-4.5 — SLP018–SLP021 and SLP023 `explain` contains "provisional" |
| `slp018_positive_fires` … `slp021_negative_silent` (macro) | FR-4.7 |
| `every_rule_has_fixtures` (existing) | FR-4.7, FR-11.3 |
| `metric_rule_silent_when_unavailable` (new) | FR-4.3 — a 3-sentence document with a high grade gives no SLP018 finding |
| `metric_finding_never_selects_window` (new, `tests/engine.rs`) | FR-4.4 — an SLP018 finding gives no window |
| `enabling_metric_rule_adds_suggestion_weight` (new, `tests/pipeline.rs`) | FR-4.6 — with SLP018 enabled the score's `weighted` grows by exactly 1 |

### 36.5 Wordiness

**File:** `tests/rules_fixtures.rs`

| Test | Verifies |
|------|----------|
| `slp001_fix_matches` (existing, fixtures extended) | FR-5.1, FR-5.2 |
| `slp001_every_row_in_positive` (new) | FR-5.2 — each `ENTRIES` phrase occurs in `positive.md`; reads the table through findings on `positive.md`, one per row |
| `fix_is_idempotent_over_all_fixtures` (existing) | FR-5.5 |
| `slp022_positive_fires`, `slp022_negative_silent` (macro) | FR-5.4 |
| `slp022_never_attaches_a_fix` (covered by `only_safe_rules_attach_fixes`) | FR-5.4, D-10 |

### 36.6 Verifier

**File:** `tests/verify_edits.rs`, `tests/pipeline.rs`, `tests/rules_fixtures.rs`, `src/text/lexicon.rs` (unit)

| Test | Verifies |
|------|----------|
| `rejects_dropped_negation` | FR-6.1, FR-6.8 — Model edit "It is not ready." → "It is ready." fails `Check::Negation` |
| `accepts_kept_negation` | FR-6.8 — "It isn't ready." → "It is not ready." passes |
| `slp003_origin_may_drop_one_negation` | D-8 — `ProposalOrigin::Rule(RuleCode(3))` drop of one `not` passes; dropping two fails |
| `model_allowance_counts_window_findings` | D-8 — Model edit with `resolves = [SLP003]` may drop one, not two; `resolves = []` none |
| `rejects_removed_proper_noun` | FR-6.2, FR-6.8 — "We use Redis here." → "We use it here." fails `Check::ProperNouns` |
| `accepts_sentence_initial_capital_change` | FR-6.2 — SLP008-style "It is worth noting that t" → "T" passes |
| `accepts_title_case_hedge_removal` | FR-6.2 — "May Potentially" → "May" passes |
| `rejects_changed_modal` | FR-6.3, FR-6.8 — "It must ship." → "It should ship." fails `Check::Modals` |
| `accepts_modal_contraction` | FR-6.3 — "It will not ship." → "It won't ship." passes |
| `rejected_new_check_is_slp901_with_name` (`tests/pipeline.rs`) | FR-6.4, FR-6.5, NFR-6 — forge an SLP001 fix that drops "not"; the result has SLP901 `rewrite rejected (negation)` and the original bytes |
| `no_safe_fixture_fix_newly_rejected` (`tests/rules_fixtures.rs`) | FR-6.6 — for every Safe rule and every fixture, every planned fix passes `verify::edit(…, &[])` (the Phase 3 gate: fails on any) |
| `reference_states_limits` (`tests/score.rs`, updated) | FR-6.7 |
| `lexicon::tests::tables_sorted_unique`, `contractions` | NFR-4, FR-6.1, FR-6.3 |

`apply_fixes` in `tests/rules_fixtures.rs` already unwraps every `verify::edit`, so after Phase 3 every existing
fixture test also exercises the new checks.

### 36.7 Pipeline and windows

**File:** `tests/engine.rs`, `tests/pipeline.rs`

| Test | Verifies |
|------|----------|
| `window_codes_one_per_finding` | D-8, ADR-06 — two SLP003 findings in one paragraph give `codes` with two SLP003 entries |
| existing `fix_model` cases | NFR-6 — model rewrites still verified, now with `&w.codes` |
| `failing_template_fix_is_slp901` | NFR-6 — forge an SLP004 fix whose replacement drops Y's capitalised word; SLP901 |

### 36.8 Tagger and SLP006

**File:** `src/text/tagger/mod.rs` (unit), `tests/text_tagger.rs` (new), `src/rules/mod.rs` (unit), `tests/rules_fixtures.rs`, `tests/layering.rs`

| Test | Verifies |
|------|----------|
| `tagger::tests::lexicon_sorted_unique_lowercase_nonempty` | ADR-07, NFR-4 |
| `tagger::tests::required_entries` | §26 — `tool`, `service`, `guide` are `[Noun, Verb]`; `fast` `[Adj, Adv]`; `library` `[Noun]`; `main`, `careful`, `ready`, `cheap`, `quick`, `simple` `[Adj]`; `these`, `this` include `Det` and `Pron` |
| `text_tagger::known_word_is_certain` | FR-7.1, NFR-10 |
| `text_tagger::unknown_word_flagged_and_guessed` | FR-7.1 — "zorbly" → `Adv`, `unknown`, not certain |
| `text_tagger::ambiguous_word_not_certain` | FR-7.1 — "tool" not certain |
| `text_tagger::rule_changes_tag_but_never_certainty` | FR-7.1 — "the guide": `guide` is `Noun`, not certain |
| `rules::tests::tagging_runs_once_per_pass` | FR-7.3, NFR-2 |
| `rules::tests::no_candidate_no_tagging` | FR-7.4 |
| `layering::tagger_runs_only_inside_prose`, `prose_built_only_per_pass` | NFR-2 (structural) |
| `layering::forbidden_uses`, `pure_modules_have_no_io` (rows added) | NFR-5, NFR-7 |
| `slp006_fix_matches` (existing, fixture extended) | FR-7.7 |
| `safe_fix_contexts_grammatical` (extended) | FR-7.5, FR-7.6, FR-7.8 |
| `text_prose_is_send_sync` | NFR-8 — compile-time assertion in `src/text/mod.rs` (fails the build) |

### 36.9 Templates

**File:** `tests/rules_fixtures.rs`, `tests/rules_registry.rs`

| Test | Verifies |
|------|----------|
| `slp003_fix_matches`, `slp004_fix_matches` (macro, safe arm) | FR-8.1, FR-8.2, FR-8.7 |
| `slp004_declines_when_y_continues` | FR-8.3 — "It is not only cheap but also quick tools." has no fix (both words certain; Y does not end the clause) |
| `slp004_declines_across_sentences` | FR-8.1 — no fix when X and Y are in different sentences |
| `slp003_declines_on_long_x_or_other_subject` | FR-8.2, FR-8.3 |
| `slp003_mid_sentence_keeps_case` | FR-8.2 — "I think it's not a bug, it's a feature." → "I think it's a feature." |
| `fix_is_idempotent_over_all_fixtures` (existing) | FR-8.7 |
| `slp007_slp015_unchanged` (`tests/rules_registry.rs`) | FR-8.6 — both are `FixClass::Model` and `explain` equals the literal text of `slp007…rs:28-30`, `slp015…rs:26-29` |

### 36.10 Redundancy

**File:** `src/text/stem.rs`, `src/text/similarity.rs` (unit), `tests/rules_fixtures.rs`

| Test | Verifies |
|------|----------|
| `stem::tests::inflections_meet` | FR-9.1 — make/makes/making share a stem; caches/cached/caching share a stem |
| `similarity::tests::jaccard_hand_values` | FR-9.1 — {a:2,b:1} vs {a:1,c:1} → 1/4 = 0.25; empty/empty → 0 |
| `similarity::tests::stop_words_removed` | FR-9.1 |
| `slp023_positive_fires`, `slp023_negative_silent` (macro) | FR-9.2, FR-9.3 |
| `slp023_skips_short_sentences_and_documents` | FR-9.4 |
| `slp023_compares_within_paragraph_only` | FR-9.1, FR-9.5 — the same sentence in two paragraphs is not reported |

### 36.11 Performance and model reliance

| Test | Verifies |
|------|----------|
| `perf::rules_only_100kb_under_1s` (config extended) | NFR-2, FR-9.5 |
| `model_reliance::residual_does_not_exceed_baseline` | FR-10.1–10.3 |
| `calibration::calibration_corpus_is_licensed_and_measurable` | D-4 follow-up |
| `standalone::no_host_references` (existing; green after §1) | NFR-1 |

**Test infrastructure:** fixtures under `fixtures/rules/SLP018/` … `SLP023/`, plus `fixed.md` for SLP003/SLP004.
`tempfile` dirs for pipeline sources (`tests/pipeline.rs:25-37`). The xorshift generator is inline in
`tests/text_segment.rs`, with no new dev-dependency. `common::FakeFactory` for model-path tests
(`tests/common/mod.rs`). `eval/model-reliance.md` and `eval/calibration/`.

---

## 37. Resolved Design Questions

| # | Question | Resolution |
|---|----------|------------|
| DQ-1 | Rule codes | SLP018–SLP023 as the architecture's table proposed, assigned in delivery order (§17, §21, §34). |
| DQ-2 | Param names and defaults | snake_case, as the shipped params (architecture §8): `max_grade` 12, `min_cv` 0.35, `max_avg_words` 25, `min_ttr` 0.4, `min_sentences`/`min_words` 5/100, `max_words` 3 (SLP003), `min_overlap` 0.6, `min_content_words` 4, `min_sentences` 3 (SLP023). All provisional (D-4). |
| DQ-3 | `since` and `RULE_SET_VERSION` | `"0.2.0"` for every new rule. `RULE_SET_VERSION` goes to `"0.2.0"` once in Phase 1d, and later phases add to the unreleased `0.2.0` CHANGELOG section. A per-phase bump while unreleased would mint versions no release carries. |
| DQ-4 | Metric thresholds | Provisional values in DQ-2. Raw metrics are always reported, so a wrong threshold costs nothing at defaults (rules off). |
| DQ-5 | Exact TTR (PRD Risk 3) | Distinct `to_lowercase()` words over words, over paragraph sentences. No windowed variant, because FR-2.5 fixes the formula and ADR-04 Option 3 is a later, explicit change. |
| DQ-6 | Words with no letters | Count as words (the score's definition) with 0 syllables. Numbers lower the grade slightly, which is documented as approximate. |
| DQ-7 | Sentences with no prose word | Not counted by metrics (a code-only "sentence" would add a 0-length sentence and distort SD). |
| DQ-8 | The negation list | `NEGATIONS` of §22 plus every `n't` form. `without`, `hardly`, `few` are excluded, being scalar rather than polar. |
| DQ-9 | Modal contractions | `won't`, `can't`, `cannot`, `shan't`, `Xn't` and `'ll` map to their modal. `'d` is not mapped (would or had), so "I'd" → "I would" is rejected as SLP901. |
| DQ-10 | Proper-noun proxy scope | Capitalised, not sentence-initial, and not a common word (`NOT_NAMES`, stop words, modals, hedges). All-caps words count, so a mid-sentence all-caps SLP001 replacement and title-case content words in headings become SLP901: loud, never silent, and not present in any fixture (FR-6.6). |
| DQ-11 | SLP006 subject classes | Class A `Det N`, class B `Det Adj N`, sentence-initial, singular-compatible determiner, every word certain, no verb in the clause after "as", and not an `AS_IDIOMS` phrase. `AS_IDIOMS` applies to the new branch only, to keep FR-7.7 and NFR-1. |
| DQ-12 | SLP004 guards | Y must end the clause (followed by `.,;:!?` or the block end). X and Y must be certain, share one content tag and lie in one sentence. A leading capital "Not" capitalises X. |
| DQ-13 | SLP003 guards | Same subject twice. Separators `, ; : — –` and spaced hyphens, never a sentence end. X has no capital, digit or protected run. X and Y each have 1..=`max_words` words. Sentence-initial matches capitalise through `delete_lead_in`. |
| DQ-14 | Jaccard or cosine | Weighted Jaccard over counts: integers only, no square root, and FR-9.1 allows it. |
| DQ-15 | Redundancy reporting | One finding per later sentence, on its first prose run. The message names the earlier sentence's line. |
| DQ-16 | Calibration licence policy | Public domain only (US federal works; texts published before 1929). No CC BY-SA or other attribution or share-alike licence, so the MIT repo carries no licence obligations. |
| DQ-17 | Which wordiness rows | No row may add or remove a modal or negation (they would fail Phase 3's FR-6.6 gate). No new `None` row (it would raise FR-10's residual). |
| DQ-18 | Text-report layout | Metrics line after the score line; `metrics before` line on Fix/Diff; overall metrics on a line after the summary line. |
| DQ-19 | `explain` shows bounds | Yes: `(min …)` or `(min …, max …)` after the help text. MCP `explain_rule` gains `min`/`max` (`null` when unbounded). |
| DQ-20 | Lexicon policy | Common tags only, first tag most frequent. Required entries pinned by a test (§26). At most 3,000 entries and a few hundred rules (D-7). |
| DQ-21 | Model-reliance settings | Default settings, sources as temp-dir paths, so no checkout config applies (§2). |
| PRD assumptions adopted | PRD §9 records Brad's answers (Q1–Q7) as D-2, D-3, D-4, D-10, D-11; there are no open `Assumption:` items. | Adopted as decided. |

---

## Open Decisions

| # | Question | Options (consequence each) | Recommendation |
|---|----------|----------------------------|----------------|
| OD-1 | ADR-08 is **Proposed**. Where do the wordiness rows and the detection-only intensifiers go? The PRD's "existing wordiness rule" (FR-11.2) does not exist; the Safe phrase table is SLP001 `ai-vocabulary-replaceable`. | **(a)** Wordiness and nominalisation rows extend SLP001 (default-on, so no-model `--fix` gains them by default); intensifiers go to a new default-off `FixClass::None` rule SLP022. Default users see new SLP001 findings and scores on padded phrasing; SLP001's name and `ai-vocabulary` family now also cover generic wordiness. **(b)** A new default-off Safe rule `wordiness` (it would take SLP022, family Wordiness) holds every new row, and intensifiers go to SLP023 `weak-intensifier` (`FixClass::None`), so redundancy becomes SLP024. Default users get none of the new fixes until they enable it; no default score changes. **(c)** Everything in SLP001 with intensifiers as `None` rows. Intensifiers become default-on findings that select model windows, against D-10 and FR-10.3. | **(a)**, because it is the only option where the PRD's goal "does more with no model" reaches default users, and it keeps intensifiers away from the model. §20–§21 are written for (a). Under (b), §20's rows move unchanged into the new rule's file, SLP001 is untouched, §21 and §34 renumber, and §35's `extend_select` adds the wordiness rule. |

---

## PRD Deviation Notes

- **FR-2.1 / D-6:** the PRD says "the list of per-sentence word lengths" and "summed distinct types". The SPEC keeps
  `sentence_len_sq` (with `words` and `sentences`) and a merged type set (ADR-04, architecture PRD Deviation Note 1).
  Why it is equivalent or better: the population SD is exact from n, Σx and Σx². Overall TTR is exact only with set
  union, because summed distinct counts double-count shared words.
- **D-5 "configurable as params":** report floors are constants, and the rule minimums are params bounded below by
  them (architecture PRD Deviation Note 3; §16). Why: params belong to rules, and metrics are reported with every
  rule off.
- **FR-6.2:** the PRD says reject an edit that "removes or changes a capitalised token that is not sentence-initial".
  The SPEC also skips capitalised common words (`NOT_NAMES`, stop words, modals, hedges; §22, §23). Why: without it,
  the existing `slp009_keeps_the_modal_case` test ("It May Potentially fail." → "It May fail.") would be rejected,
  and so would every title-case function word. Neither is a name, and NFR-1 requires existing tests to pass. The
  literal reading is kept for every other capitalised word, all-caps included (DQ-10).
- **FR-6.3:** the PRD says "changes the multiset of modal verbs". The SPEC maps contractions to their modal (DQ-9).
  Why: "will not" → "won't" keeps the modal, and a literal token compare would reject it.
- **NFR-2:** the PRD's verification is "a test that counts tagger invocations across a multi-window fix". The SPEC
  proves it in two parts. The structural tests (`tagger::tag(` only in `Prose::tags`; `Prose::new(` only in
  `detect_in` and in-crate tests) bound it to one tagging per `Prose` and one `Prose` per document pass. The in-crate
  `tagging_runs_once_per_pass` shows the per-`Prose` bound. Why: the `#[cfg(test)]` counter the architecture chose
  does not exist in integration builds, and a global counter would be the mutable global state NFR-8 forbids.
- **NFR-5:** the PRD says "the existing no-network test pattern run over a lint and a fix of a metrics-enabled
  document". No such generic test exists. The SPEC relies on `pure_modules_have_no_io` (now over `src/text/`) and
  `network_only_in_pull` (`tests/layering.rs:96-104, 214-217`), and records the release-binary size before and after
  in `CHANGELOG.md`'s 0.2.0 section, in Phases 1c and 6. Why it is equivalent: no file in `src/text/` can name
  `std::net` or `ureq`, and the pipeline paths are unchanged.
- **FR-5.3:** satisfied as a rule on rows (an uncertain row carries no fix or stays out). The SPEC adds no new `None`
  row (DQ-17), to protect FR-10.3.

---

## Architecture Deviation Notes

- **§4.1 Segmenter / ADR-03:** the architecture says `sentence_range` keeps only range arithmetic over
  `sentence_starts`. The code forces `pub(crate) blanked` and `scan_starts` beside `sentence_starts`, because today's
  `sentence_range` trims on the blanked copy (`src/engine/window.rs:271-277`). Trimming the raw source would keep a
  continuation prefix (`> `) and change window ranges, against the phase-1b gate. The architecture needs: name the
  two `pub(crate)` helpers in §4.1.
- **§4.10 Verifier / ADR-03:** the architecture says sentence-initial status of the edit's first token comes from
  `sentence_starts` on the edit's block. The code forces `starts_sentence` on the block's text (with the new
  `BlockText::to_text`), because a block's range includes line-start markup. For `# It is worth noting that …`
  (`tests/rules_fixtures.rs:266-269`) the raw sentence start is the `#`, so "It" would count as a name and the
  SLP008 fix would be rejected. ADR-03 already names `starts_sentence` for the verifier. The architecture needs: §4.10
  to say `starts_sentence` plus `BlockText::to_text`.
- **§4.2 Lexicon:** the architecture lists negations, modals, stop words, determiners and pronouns. The SPEC adds
  `HEDGES`, `NOT_NAMES` and `is_common_capitalised` for the proper-noun proxy (PRD Deviation Note on FR-6.2).
  Determiners and pronouns live in the tagger lexicon (§26), not in `text::lexicon`. The architecture needs: the
  list in §4.2.
- **§4.5 Tagger:** `Tagged` gains `unknown: bool`, because FR-7.1 requires an "unknown" flag, and `certain` alone
  conflates unknown with ambiguous. The architecture needs: the field in §4.5.
- **§4.8 Prose / §11 tests:** the NFR-2 single-tagging test is an in-crate unit test plus two structural layering
  tests, not a pipeline test (PRD Deviation Note on NFR-2). The architecture needs: §11's Pipeline row to say so.
- **§4.11 Pipeline:** `detect_in` returns a private type alias `Detected` for the 4-tuple, because
  `clippy::type_complexity` under `-D warnings` (`Makefile:93`) is likely to reject the bare type.
- **ADR-06 / §4.11:** `Window.codes` keeps one entry per overlapping finding, duplicates included. The allowance
  counts SLP003/SLP004 findings, and a deduplicated list could not count two.
- **§4.12 Report:** the architecture says "the summary line gains the overall metrics". The SPEC prints them on a
  new line after it, so `summary_line` stays byte-identical (DQ-18).
- **§4.9 / FR-11.1:** the architecture says `RULE_SET_VERSION` is "bumped per phase that adds a rule". The SPEC bumps
  it once per unreleased version (DQ-3).
- **§11 Test Architecture, line 563, and PRD line 298:** both name the host build contract, which fails
  `tests/standalone.rs::no_host_references`. §1 rewrites both. The architecture needs: that row's "Runs as" column in
  the repo's `make test TESTS='…'` form.

---

## Self-Audit

- **FR coverage:**
  - FR-1: FR-1.1 → §4. FR-1.2 → §4. FR-1.3 → §4, §36.1. FR-1.4 → §3, §5. FR-1.5 → §4, §9.
  - FR-2: FR-2.1 → §8. FR-2.2 → §8. FR-2.3 → §7, §8. FR-2.4 → §8. FR-2.5 → §8. FR-2.6 → §7, §14. FR-2.7 → §8.
    FR-2.8 → §8, §12. FR-2.9 → §8. FR-2.10 → §4, §8.
  - FR-3: FR-3.1 → §12. FR-3.2 → §12. FR-3.3 → §13. FR-3.4 → §11, §12. FR-3.5 → §12. FR-3.6 → §14.
  - FR-4: FR-4.1 → §16, §17. FR-4.2 → §15, §16, §17. FR-4.3 → §16. FR-4.4 → §17. FR-4.5 → §17. FR-4.6 → §17.
    FR-4.7 → §17.
  - FR-5: FR-5.1 → §20. FR-5.2 → §20. FR-5.3 → §20. FR-5.4 → §21. FR-5.5 → §20.
  - FR-6: FR-6.1 → §22, §23. FR-6.2 → §3, §22, §23. FR-6.3 → §22, §23. FR-6.4 → §23, §24. FR-6.5 → §24.
    FR-6.6 → §36.6. FR-6.7 → §25. FR-6.8 → §36.6.
  - FR-7: FR-7.1 → §26. FR-7.2 → §26. FR-7.3 → §10, §27. FR-7.4 → §27, §28. FR-7.5 → §28. FR-7.6 → §28.
    FR-7.7 → §28. FR-7.8 → §28.
  - FR-8: FR-8.1 → §29. FR-8.2 → §30. FR-8.3 → §29, §30. FR-8.4 → §29, §30, §31. FR-8.5 → §23, §29, §30.
    FR-8.6 → §31. FR-8.7 → §29, §30.
  - FR-9: FR-9.1 → §32, §33, §34. FR-9.2 → §34. FR-9.3 → §34. FR-9.4 → §34. FR-9.5 → §33, §34, §35. FR-9.6 → §34.
  - FR-10: FR-10.1 → §2. FR-10.2 → §2. FR-10.3 → §2.
  - FR-11: FR-11.1 → §18. FR-11.2 → §17, §20, §21, §34. FR-11.3 → §15, §18.
- **NFR coverage:**
  - NFR-1 → §1, §12, §28, §31, §36 (only the listed test edits).
  - NFR-2 → §6, §9, §11, §27, §35.
  - NFR-3 → §8, §12, §36.2.
  - NFR-4 → §7, §15, §22, §26.
  - NFR-5 → §6, §32, PRD Deviation Notes.
  - NFR-6 → §23, §24, §36.6–36.7.
  - NFR-7 → §4, §6.
  - NFR-8 → §9.
  - NFR-9 → §3, §5, §23, §25, §31.
  - NFR-10 → §36.
- **Component coverage:**
  - §4.1 Segmenter → §3, §4, §5. §4.2 Lexicon → §22. §4.3 Syllable → §7. §4.4 Metrics → §8, §14.
  - §4.5 Tagger → §26. §4.6 Stem → §32. §4.7 Similarity → §33. §4.8 Prose → §6, §9, §27.
  - §4.9 Rule integration → §10, §16, §17, §18, §20, §21, §28, §29, §30, §31, §34.
  - §4.10 Verifier → §3, §23, §25. §4.11 Pipeline and windows → §5, §11, §24. §4.12 Report and MCP → §12, §13, §14.
  - §4.13 Config bounds → §15. §4.14 Model-reliance harness → §2.
  - §12 phases 1a–6 → section order. ADR-01 → §6. ADR-02 → §9, §10, §11, §27. ADR-03 → §3, §4, §5.
  - ADR-04 → §8. ADR-05 → §12, §13. ADR-06 → §23, §24. ADR-07 → §7, §22, §26. ADR-08 → §20, §21 (OD-1).
    ADR-09 → §15, §16.
- **Deviations:** 7 PRD Deviation Notes and 10 Architecture Deviation Notes, all listed above. One Open Decision
  (OD-1).
- **Self-review:** ran passes 0–5 over the draft and fixed what they found.
  - *Pass 0 (grounding):* every cited line was re-read in this session. `ParamSpec` call sites are `slp005:41`,
    `slp010:43,48`, `slp011:39` and `slp014:38`. `verify::edit` call sites are pipeline `:118, :273`,
    `verify_edits:29, 231, 246`, `rules_fixtures:73` and `edit_apply:52, 206`. `detect` call sites are
    `rules_core:80, 98, 103, 119`, `rules_fixtures:53` and pipeline `:307`.
  - *Pass 1 (consistency):*
    - The first draft gave SLP006 a "Det N" class that would have fixed "The tool serves as a bridge." in
      `fixed.md` (FR-7.7). Fixed by pinning `tool` as ambiguous in §26 and testing it.
    - SLP004's `fixed.md` first assumed `fast` was certain. Fixed: `fast` is `[Adj, Adv]`, and line 1 stays unfixed.
    - The verifier's sentence-initial test first used `sentence_starts`. That broke the SLP008 heading test, so it
      was switched to `starts_sentence` (deviation note).
    - The proper-noun proxy first broke `slp009_keeps_the_modal_case`. Fixed with the common-word exemption.
    - Names checked across sections: `Window.codes`, `Counts`/`Metrics`, `MIN_SENTENCES_PARAM`, `metric_anchor`
      and `Detected`.
  - *Pass 2 (completeness):* every FR and NFR is mapped above, and every component has a section.
  - *Pass 3 (testability):* each test names a file and what it asserts. The FR-6.6 gate is
    `no_safe_fixture_fix_newly_rejected`.
  - *Pass 4 (scope):* nothing beyond the PRD, apart from §1's doc fix, which a gate needs.
  - *Pass 5 (decisions):* ADR-08 is routed to Brad as OD-1, not decided here. Everything else is a DQ inside the
    PRD's and architecture's decisions.
  - *Compile:* see the note under the title.
- **For the adversarial review:**
  1. OD-1 (ADR-08).
  2. DQ-10: the proper-noun proxy's exemptions, and the loud SLP901 on mid-sentence all-caps or title-case-heading
     SLP001 fixes.
  3. The SLP003/SLP004 `fix.available` change (§31).
  4. §28's clause guard, which reads best-guess tags rather than certain ones (any verb-like guess declines, so it
     errs toward no fix).
  5. The NFR-2 proof form (PRD Deviation Note).
  6. §1: the PRD and architecture currently fail `no_host_references`.

---

## Revision Log

(first pass; no revisions yet)
