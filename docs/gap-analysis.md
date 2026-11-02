# Gap Analysis: sloperator — a standalone CLI that makes LLM-generated text read as human

> Historical record: the one-time repository text scan and its planted-reference test have since been removed. Current standalone checks cover path dependencies and extracted builds (SPEC §5).
**Source:** [PRD](PRD.md) | [Architecture](ARCHITECTURE.md) (ADR-01…09 in `docs/adr/`) | [SPEC](SPEC.md) | UX: none (architecture records no UX scope)
**Reviewer:** reviewer
**Last round:** 3 (2026-10-31)

---

## Executive Summary

**Round 3 (delta).** All 4 FIX items open after round 2 are now resolved: Gaps 4, 9, 14 and 15. This round adds 1 new gap on the changed SLP006 text: Gap 16 (MEDIUM, FIX-SPEC).

**Open now, by severity:** BLOCKING 0, HIGH 0, MEDIUM 1, LOW 0, plus 1 DECIDE.

**Open now, by disposition:**
- FIX-SPEC: 1 (Gap 16 MEDIUM)
- DECIDE: 1 (Gap 11)
- Closed: DEFER 1 (Gap 12) and ACCEPT 1 (Gap 13)

**The gap that matters most is Gap 16.** The new SLP006 guard checks the word *before* "serves/functions as", but nothing checks the word *after* it. So "It functions as follows" becomes "It is follows", and "It serves as well as X" becomes "It is well as X". The modal form has the same problem: "can serve as well as" becomes "can be well as". This is the Gap 2/14 failure again, but much rarer.

**Recommendation: proceed.** No BLOCKING or HIGH gap is open. Gap 16 is a MEDIUM FIX-SPEC item, and Gap 11 (OD-1, which distribution binaries ship) still goes to Brad.

---

## Triage (open gaps)

| Gap | Severity | Origin | Disposition | Rationale / what must change |
|-----|----------|--------|-------------|------------------------------|
| Gap 16 | MEDIUM | missed-earlier (on text rewritten in round 3) | FIX-SPEC | The SLP006 "→ is" / "→ be" fixes have no guard on the word that follows "as". Fix only when the next token is in a fixed list (e.g. `a`, `an`, `the`), or deny `follows`/`well`/`long`/`soon`/`much`/`far`. Add "It functions as follows" and "It serves as well as the old one" to the fires-with-no-fix fixtures. |
| Gap 11 | — | decision | DECIDE | OD-1: which distribution binaries are built, and where. Recommend (a), a per-target CI matrix after extraction. |

---

## Round 1

### Gap 1: Missing-tool handling for every make target

**Severity:** HIGH
**Origin:** first-draft
**Pass:** literal-vs-actual / feasibility
**PRD Ref:** FR-1.9; Brad Q7 (a)
**Architecture Ref:** §4.15; ADR-07
**SPEC Ref:** §3 (`need`), §23.2

**Expected:** Every target that calls `cargo`, `cargo-nextest` or `cargo-deny` prints a one-line stderr banner naming the missing tool and its install step, and exits non-zero.

**Resolved:** the `Makefile`'s `need` macro guards `build`, `test`, `test-integration` and `check` for each tool they call, prints "sloperator: '<tool>' not found on PATH — <install hint>" and exits 1; §23.2 checks it with the tools removed.

### Gap 2: Some "safe" fixes change grammar or meaning

**Severity:** HIGH
**Origin:** first-draft
**Pass:** literal-vs-actual
**PRD Ref:** D-4 ("deterministic substitutions that keep meaning"); FR-4.7; Brad Q3 (fixes written by `--fix` by default)
**Architecture Ref:** ADR-06
**SPEC Ref:** §10 (SLP001, SLP006 rows), §9 `WordList::find`

**Expected:** A safe fix keeps meaning and is written without model verification.

**Actual:**
- **SLP006** maps "serve as" → "are". After a modal or "to", the result is ungrammatical: "These files can serve as templates" → "These files can are templates", and "designed to serve as" → "designed to are". The SPEC also gives no form for "serving as" or "functions/stands" in their inflections.
- **SLP001** lists "leverage" with the note "(verb)". But `WordList` is an aho-corasick whole-word matcher with no part-of-speech signal, so the noun is rewritten too: "we have leverage over suppliers" → "we have use over suppliers".
- `verify::edit` checks numbers, URLs, terms and structure. It does not check grammar, so these edits are accepted and written in place.
- The §10 fixtures are written by the author and only prove the cases the author thought of.

**Impact:** `--fix`, the default write path that Brad approved because safe fixes keep meaning, writes broken English into user files. Users of a tool sold as making text read human see exactly the opposite. No gate checks these contexts.

**Disposition:** FIX-SPEC.
- For SLP006, either map by inflection with a preceding-word guard (modal, "to" or "will" + "serve as" → "be"), or class the base form "serve as" as `model`.
- For SLP001, drop "leverage" from the safe list. Either make it model-class, or match only inflected verb forms ("leverages", "leveraged", "leveraging") with their replacements.
- Add each failing context above to the rule's `negative.md` or `fixed.md`.
- State in §10 the rule a safe fix must meet: no part-of-speech ambiguity.

### Gap 3: The SPEC's code breaks its own layering table

**Severity:** MEDIUM
**Origin:** first-draft
**Pass:** alignment / feasibility
**Architecture Ref:** §4.8 ("Must never depend on: engine, rules, I/O"), §5
**SPEC Ref:** §5 `FORBIDDEN_USES`; §7 (`use crate::rules::RuleCode;`); §9 (`use crate::edit::Proposal;`)

**Expected:** `tests/layering.rs::forbidden_uses` passes on the code the SPEC specifies.

**Actual:**
- The `src/rules/` row forbids `crate::edit`, but `rules/mod.rs` imports `crate::edit::Proposal` because `Finding.fix` is `Option<Proposal>`.
- The `src/edit/` row forbids `crate::rules`, but `edit/mod.rs` imports `RuleCode` for `ProposalOrigin::Rule`.
- Both rows match the architecture, and the code contradicts both.

**Impact:** The Phase 2 test fails on the first compile. The implementer then has to pick a type home without guidance. A failing test catches it, so it is not blocking.

**Disposition:** FIX-SPEC. Put the shared types where both may reach them, for example `RuleCode` in `doc` or a leaf `types` module, or invert the dependency. Then update the table, the code and the §5 flowchart row. The architecture wording is Gap 10.

### Gap 4: The standalone test fails on its own marker list

**Severity:** MEDIUM
**Origin:** first-draft
**Pass:** feasibility
**PRD Ref:** FR-1.8, NFR-2
**SPEC Ref:** §5 (the marker constant and the reference-scan test)

**Expected:** The test fails only when a real reference to the host project exists.

**Actual:**
- The scan covers every file in the crate except `target/`.
- `tests/standalone.rs` contains the host-project name and its build-plumbing markers as literals.
- The test therefore matches itself on every run.

**Impact:** Red from the first run. An implementer may "fix" it by weakening the scan.

**Disposition:** FIX-SPEC. Exempt exactly `tests/standalone.rs`, or build the markers so they never appear literally, for example by concatenation. Add a test that a planted reference in a temp copy is caught, so the exemption cannot hide a real one.

### Gap 5: The walker's defaults are not pinned, so FR-2.4 and FR-2.2 do not hold

**Severity:** MEDIUM
**Origin:** first-draft
**Pass:** literal-vs-actual / edge case
**PRD Ref:** FR-2.2, FR-2.4
**SPEC Ref:** §14 (Walking)

**Expected:**
- Directories are walked for `.md`, `.markdown` and `.txt`.
- Only `.gitignore` and `exclude` cause skips.
- `respect_gitignore = false` turns the gitignore skip off.

**Actual:**
- The SPEC sets only `.git_ignore(s.respect_gitignore)` and `.sort_by_file_path`.
- `ignore::WalkBuilder`'s other defaults stay on: skip hidden files and directories, honour `.ignore` files, the global gitignore and `.git/info/exclude`, read parent ignore files, and apply `.gitignore` only inside a git repo (`require_git`). This is from the crate's documented defaults; I did not run the crate in this session.

**Impact:**
- With `respect_gitignore = false`, files listed in a user's global gitignore or `.ignore` are still skipped silently.
- `.github/*.md` (hidden directory) is never checked.
- Outside a git repo, `.gitignore` is ignored.
- Each is a file passing as "clean" without being checked.

**Disposition:** FIX-SPEC. Set every relevant builder flag explicitly: `hidden`, `ignore`, `git_global`, `git_exclude`, `parents` and `require_git`, all tied to `respect_gitignore` or a stated default. Extend `gitignore_and_exclude_skipped_respect_gitignore_false` to cover a hidden directory and the case with no git repo.

### Gap 6: Directive-shaped lines inside code fences are taken as directives

**Severity:** MEDIUM
**Origin:** first-draft
**Pass:** edge case
**PRD Ref:** FR-6.1–6.5, FR-8.1
**SPEC Ref:** §12 (`MD_DIRECTIVE` is a whole-line regex; `extract` "re-tags each directive line's span")

**Expected:** Text inside a code block is protected and has no effect, which is the spirit of FR-8.1 and FR-4.4.

**Actual:**
- `extract` is specified as matching whole lines against `MD_DIRECTIVE`.
- It does not say that only `Html` spans count.
- §6 classifies fenced content as `Protected(CodeBlock)`, not `Html`, but nothing in §12 restricts extraction to `Html`.

**Impact:** sloperator's own README (§2) documents the directives, presumably in fences. Running `sloperator check` on it, or on any doc that shows `<!-- sloperator-disable SLP001 -->` in a code sample, opens an unclosed region and exits 2 (FR-6.3). Or it silently suppresses findings.

**Disposition:** FIX-SPEC. Markdown directives are recognised only in `Html` or `HtmlBlock` spans that cover the whole line. Plain-text directives are recognised by line. Add `directive_in_code_fence_ignored` to §23.8.

### Gap 7: Container prefixes in hard-wrapped windows

**Severity:** MEDIUM
**Origin:** first-draft
**Pass:** edge case
**PRD Ref:** FR-8.2, FR-9.5
**Architecture Ref:** ADR-04
**SPEC Ref:** §16 (`Window.wrap`, the mask list, the re-wrap note), §6 (continuation bytes are `Protected(Markup)`)

**Expected:** A wrapped paragraph inside a block quote or list item can be rewritten without changing structure.

**Actual:**
- The continuation prefixes (`> `, list indentation) fall inside the window range and are `Protected(Markup)`.
- §16's mask list names "emphasis/hard-break markup", not container prefixes.
- Re-wrapping "reuses its first break text (newline plus indentation)", which is not defined when the prefix is `> `.
- If the prefixes become placeholders, the model can move them mid-line, where they parse as prose. `protected_strings` then differs, so the edit is rejected.
- If they are dropped, lazy continuation keeps the skeleton, but the bytes change.

I am about 75% sure every such rewrite is rejected. I am confident the SPEC does not say what happens.

**Impact:** Model rewrites inside wrapped quotes or list items are either always rejected (SLP901 noise, with no fix) or they change markup bytes.

**Disposition:** FIX-SPEC.
- Define how the window excludes container prefixes: strip them before masking and re-insert them on every re-wrapped line.
- Have `wrap` capture the per-line prefix.
- Add `wrapped_blockquote_and_list_item_rewrapped_with_prefixes` to §23.12.

### Gap 8: The JSON finding has no `path`

**Severity:** LOW
**Origin:** first-draft
**Pass:** literal-vs-actual
**PRD Ref:** FR-3.3 ("per finding: path, rule code, …")
**SPEC Ref:** §18 (JSON example: `path` is on the file object only)

**Expected:** Each finding carries `path`.

**Actual:** Only the file object carries `path`.

**Impact:** Small. Consumers that flatten findings must join on the file.

**Disposition:** FIX-SPEC. Add `path` to each finding, or record a PRD Deviation Note for the nesting.

### Gap 9: Targeted test runs pay for a release build

**Severity:** LOW
**Origin:** first-draft
**Pass:** feasibility
**PRD Ref:** FR-1.5
**SPEC Ref:** §3 `test` target (the second `cargo nextest run --cargo-profile release`)

**Expected:** `TESTS='<one test>'` is a quick, targeted run.

**Actual:** nextest builds every test binary before it filters. So each targeted run also compiles the whole test set in release, even when the filter `binary(=perf) and (…)` cannot match.

**Impact:** Story gates run targeted tests over and over, and each run can cost minutes. It works, but it is slow.

**Disposition:** FIX-SPEC. Skip the perf run when `TESTS` is non-empty and contains no `perf`. The `count` check already covers the "nothing ran" case.

### Gap 10: The architecture lags the SPEC

**Severity:** LOW
**Origin:** drift
**Pass:** alignment
**Architecture Ref:** §4.8, §4.9, §5 flowchart; §4.15 and ADR-07 (selector form)
**SPEC Ref:** §5, §8, §3; Architecture Deviation Notes

**Actual:**
- Architecture §4.9 still shows `edit(doc, p, s: &Settings)`, `Tokens.placeholders`, and a `Check` enum without `Protected` or `Document`.
- §5 still draws `verify → rules`, but the SPEC's `improvement` takes counts and `pipeline` detects.
- §4.8 says `edit` must never depend on `rules`; see Gap 3.
- §4.15 and ADR-07 say each word becomes `test(~word)`, while the SPEC also maps `binary_id#test` exactly.

**Disposition:** FIX-ARCHITECTURE (non-blocking). The architect updates these sections in the next SPEC pass so they match the SPEC, including the type home Gap 3 chooses and the missing-tool guard (Gap 1).

### Gap 11: Distribution binaries (OD-1, Brad's follow-up to Q7)

**Severity:** — (DECIDE)
**Origin:** decision
**Pass:** coverage
**PRD Ref:** none. The PRD is silent on release targets; Brad raised it in decisions.md ("we should be able to build binaries for distribution on any machine, correct?").
**SPEC Ref:** Open Decisions OD-1

**The answer to Brad's question:**
- Yes. Q7 governs only the build targets on a machine without Rust. A built binary needs no Rust at run time.
- What remains open is which targets ship and how they are built.

**Question:** How are distribution binaries produced?

**Options:**
- (a) A per-target native CI matrix (macOS arm64 with Metal, Linux x86_64/arm64, Windows x86_64) in GitHub Actions. Reliable and GPU-capable on macOS; costs CI minutes and lands outside these stories.
- (b) Cross-compile every target from one machine (`cargo-zigbuild`/`cross`). One build machine, but llama.cpp's cmake cross-builds are fragile, and Metal cannot be built off macOS.
- (c) Source only (`cargo install`). No release work, but every user needs Rust, clang and cmake.
- (d) Like (a), but add a `make package` target now, for local release archives. One extra story.

**Recommendation:** (a). It is the only option that ships a Metal macOS binary plus CPU binaries everywhere without a fragile cross toolchain, and v1 needs only the local `make build` already specified.

### Gap 12: NFR-3's "never reads the model cache" has no mechanism

**Severity:** LOW
**Origin:** first-draft
**Pass:** tests
**PRD Ref:** NFR-3
**SPEC Ref:** §23.18 (`rules_only_100kb_under_1s` "also asserts that the model cache dir was never read")

**Actual:** The SPEC gives no mechanism for the assertion: no access-time check, no unreadable directory, no hook.

**Disposition:** DEFER → `deferred.md` item 5. The story that carries §23.18 fixes the mechanism, for example pointing `model.cache_dir` at a mode-000 directory and asserting the run succeeds.

### Gap 13: Self-audit questions 4-6 (`Box::leak`, the `count` check, `max_window_chars`)

**Severity:** LOW
**Origin:** first-draft
**Pass:** edge case
**SPEC Ref:** §21, §3, §13 / DQ-5

**Disposition:** ACCEPT.
- **`Box::leak`:** one model per process, held until exit. The OS reclaims it, and nothing reloads in the same process (FR-9.4).
- **The `count` check:** failing a selector that matches nothing is the fail-loudly convention applied correctly.
- **`max_window_chars = 4000`:** a conservative value with a context-overflow backstop. Over-long windows are reported as SLP902, never dropped silently, and the spike's Decision section can retune it.

The remaining self-audit questions are settled in the passes below: DQ-3 (strictly fewer) and DQ-4 (SLP901 bypasses `--min-severity`) both follow the gate's DEFER intent, and the bullet-spam class of `none` is honestly recorded as a PRD Deviation Note.

### Coverage

| Requirement / component | Status | Architecture component | SPEC section / Gap |
|-------------------------|--------|------------------------|--------------------|
| FR-1.1 | COMPLETE | §4.15 | §2 |
| FR-1.2 | COMPLETE | §4.15 | §3 (`build`, `test`, `test-integration`, `check`, `clean`, `help`) |
| FR-1.3 | COMPLETE | §4.15 | §3 (`rc` kept, so the JUnit report is checked even on failure) |
| FR-1.4 | COMPLETE (deviation note) | ADR-07 | §3, DQ-1 |
| FR-1.5 | COMPLETE | ADR-07 | §3, DQ-2; Gap 9 (speed) |
| FR-1.6 | COMPLETE | §4.15 | §3 (fmt, clippy `-D warnings` ×2) |
| FR-1.7 | COMPLETE | §11 | §3 (`--no-default-features` unit tier; `--run-ignored only` integration) |
| FR-1.8 | PARTIAL | §5 | §5; Gap 4 |
| FR-1.9 | COMPLETE | ADR-07 | §3; Gap 1 (resolved) |
| FR-2.1 | COMPLETE | §4.1, §4.11 | §17, §20 |
| FR-2.2 | PARTIAL | §4.3 | §14; Gap 5 |
| FR-2.3 | COMPLETE | §4.3 | §14 |
| FR-2.4 | PARTIAL | §4.3 | §14; Gap 5 |
| FR-2.5 | COMPLETE | §4.8 | §7, §20 |
| FR-2.6 | COMPLETE | §4.13 | §19, §20 |
| FR-2.7 | COMPLETE | §4.1 | §20 step 10 |
| FR-2.8 | COMPLETE | §4.1 | §20 |
| FR-2.9 | COMPLETE | §4.1 | §20, §11 |
| FR-2.10 | COMPLETE | §4.1 | §20 (`conflicts_with`) |
| FR-2.11 | COMPLETE | §4.2 | §13 |
| FR-2.12 | COMPLETE | §4.6 | §9, §20 |
| FR-2.13 | COMPLETE | §4.2 | §13, DQ-9 |
| FR-2.14 | COMPLETE | §4.14 | §15 |
| FR-2.15 | COMPLETE | §4.1 | §4, §13, §20 (clap error → 2) |
| FR-2.16 | COMPLETE | §4.1 | §20 `main` |
| FR-3.1, 3.2, 3.4, 3.5, 3.6 | COMPLETE | §4.12 | §18, §20 |
| FR-3.3 | PARTIAL | §4.12 | §18; Gap 8 |
| FR-4.1–4.6 | COMPLETE | §4.6 | §9, §10 |
| FR-4.7 | COMPLETE (literal); meaning-keeping broken for two rules | §4.6 | §9, §10; Gap 2 |
| FR-5.1–5.4 | COMPLETE | §4.7 | §11, §2 |
| FR-6.1–6.5 | PARTIAL | §4.5 | §12; Gap 6 |
| FR-6.6, 6.7 | COMPLETE | §4.5 | §7, §8, §12, §6 |
| FR-7.1–7.6 | COMPLETE | §4.2 | §13 |
| FR-8.1, 8.3–8.9 | COMPLETE | §4.4, §4.9 | §6, §7, §8, §14 |
| FR-8.2 | PARTIAL | §4.4 | §6, §16; Gap 7 |
| FR-9.1–9.10 | COMPLETE | §4.10, §4.14 | §15, §16, §17, §21, §1 (9.5 stricter: deviation note, DQ-3) |
| FR-10.1, 10.2 | COMPLETE | §11 | §22 |
| NFR-1 | COMPLETE | §5 | §23.2 |
| NFR-2 | COMPLETE | §5 | §5 (`builds_when_extracted`) |
| NFR-3 | COMPLETE (mechanism deferred) | §11 | §3, §23.18, §22; Gap 12 |
| NFR-4 | COMPLETE | §8 | §4, §12, §13, §14, §16 |
| NFR-5 | COMPLETE (stricter, DQ-10) | §4.13 | §19, §20 |
| NFR-6 | COMPLETE | §6 | §11, §14, §23.17 |
| NFR-7 | COMPLETE | §9 | §2 (exact pins, `deny.toml`), §3 |
| NFR-8 | COMPLETE | §11 | §23 |
| NFR-9 | COMPLETE | — | §18 `schemaVersion`; no shims |
| §4.1–§4.15 components | COMPLETE | — | Self-Audit component map checked: every component has a section |
| ADR-01…ADR-09 | HONORED (ADR-05 and ADR-07 with recorded deviations) | — | Architecture Deviation Notes; Gap 10 drift |

### Scope creep

| SPEC concept | In PRD? | Verdict |
|--------------|---------|---------|
| SLP902 `window-skipped` | No | Justified: FR-9.7 forbids skipping a window silently |
| `model.path` + `model.profile` | No | Justified: ADR-08 follow-up (DQ-6); no network |
| `metal` / `cuda` features, `FEATURES` knob | No | Justified: D-6 GPU path; `make test-integration FEATURES=metal` selects it |
| `--seed` / `model.seed` | Yes (FR-9.8) | — |
| `-v` / `-q` | No | Justified: architecture §8 diagnostics; small |
| `ci-perf` second nextest run | No | Justified: NFR-3 benchmark; cost is Gap 9 |
| `cargo-deny` | NFR-7 lets the SPEC choose | Justified: DQ-15 |

### Edge cases walked

| FR group / class | Boundaries checked | Unaddressed |
|------------------|--------------------|-------------|
| FR-1 crate shell | missing cargo, cargo-nextest or cargo-deny for each target; empty `TESTS`; selector matching nothing; failing test plus JUnit report | none (Gap 1 resolved) |
| FR-2 / §14 input | no path, missing path, glob matching nothing, stdin with and without a name, excluded explicit file, hidden dirs, no git repo | Gap 5 |
| FR-6 directives | unclosed, unknown code, system code, directive in a fence, ignore across blank lines | Gap 6 |
| FR-8 / §6–§8 verify | CRLF, trailing newline, placeholder reorder, edit into frozen or protected bytes, forged edit pair | Gap 7 (container prefixes) |
| FR-9 / §16–§21 model | missing model, model-less build, no window means no load, runtime error, overflow, `⟦` in input, truncation at `max_tokens` | none |
| NFR-5 / §19 write | permissions, partial failure across files, unchanged file, stdin | none |
| §10 safe fixes | inflection, part of speech, modal context, idempotence | Gap 2 |

### Tests

| FR / NFR | Test file | Test method | Status |
|----------|-----------|-------------|--------|
| FR-1.8 | `tests/standalone.rs` | the reference-scan test | VAGUE: Gap 4 (fails on itself) |
| FR-1.9 | §23.2 build checks | each tool removed: `make test`, `make build`, `make check` | PRESENT (Gap 1 resolved) |
| FR-2.4 | `tests/input.rs` | `gitignore_and_exclude_skipped_respect_gitignore_false` | VAGUE: Gap 5 |
| FR-4.7 / D-4 | `tests/rules_fixtures.rs` | `slp001_*`, `slp006_*` | VAGUE: Gap 2 (no failing-context fixtures) |
| FR-6.x | `tests/directives.rs` | — | MISSING `directive_in_code_fence_ignored`: Gap 6 |
| FR-8.2 / FR-9.5 | `tests/engine.rs` | `hard_wrapped_window_rewrapped` | VAGUE: Gap 7 |
| NFR-3 | `tests/perf.rs` | `rules_only_100kb_under_1s` | VAGUE: Gap 12 (deferred) |
| All other FR/NFR rows | §23.3–§23.19 | as listed | PRESENT; each named test would fail if its requirement regressed |

---

## Round 2

**Changed since round 1:** SPEC v0.1 → v0.2 (202 lines: §3, §4, §5, §6, §7, §8, §9, §10, §12, §14, §16, §18, §23.8/10/12, Deviation Notes, Revision Log). Architecture §4.6, §4.8, §4.9, §4.10, §4.15, §5 (47 lines). ADR-07 (12 lines).

### Earlier FIX items re-checked

| Gap | Status | Evidence |
|-----|--------|----------|
| 1 (HIGH) | RESOLVED (round 2) | SPEC §3: the `need` macro prints the banner and exits non-zero for each missing tool. ADR-07 and architecture §4.15 are updated. The missing §23.2 row is Gap 15. |
| 2 (HIGH) | RESOLVED (round 2), for the cases raised | SPEC §10:1055, 1060, 1092-1107. Bare "leverage" now has no fix, and "can serve as" now becomes "can be". The new SLP006 row brought the same class of bug back for "functions as": see Gap 14. |
| 3 (MEDIUM) | RESOLVED (round 2) | `src/types.rs` is a leaf (§9). `rules` and `edit` re-export from it (§7, §9). Layering rows and `PURE_DIRS` are updated (§5). The architecture §5 graph matches. |
| 4 (MEDIUM) | **STILL OPEN** | §5:558-572. The marker literals are now `concat!`. But the scan "match[es] case-insensitively" (§5:571), and the same file's marker constant, test name and a doc comment (§5:561, §5:565) each contain the host-project name ignoring case, so the scan still matches its own file. the retired planted-reference check asserts that the unmodified `standalone.rs` copy "reports nothing", so it fails as well. Fix: rename those identifiers (e.g. `HOST_MARKERS`, the retired repository text check), or exempt the file and rely on the planted test. |
| 5 (MEDIUM) | RESOLVED (round 2) | §14: `standard_filters(false)` with every flag set explicitly. Two new tests in §23.10. |
| 6 (MEDIUM) | RESOLVED (round 2) | §12: only a line that one `Html`/`HtmlBlock` span fully covers counts. Plain text uses `Protected(Directive)`. New tests in §23.8. |
| 7 (MEDIUM) | RESOLVED (round 2) | §16: `Wrap { width, prefix, newline }`. Prefixes are stripped, then re-inserted on every line. An inconsistent prefix gives SLP902. `protected_strings` counts prefix spans once per block (§6). New tests in §23.12. |
| 8 (LOW) | RESOLVED (round 2) | §18: `path` is on every finding, with a design note. |
| 9 (LOW) | **STILL OPEN** | `RUN_PERF` is defined in §3 and was probed. But the `test` recipe never reads it: it runs the `--cargo-profile release` perf pass unconditionally. The fix is declared but not wired in. Guard the perf run with `if [ -n '$(RUN_PERF)' ]; then …; fi`. |
| 10 (LOW, arch) | RESOLVED (round 2) | Architecture §4.6, §4.8, §4.9, §4.10, §4.15 and §5, plus ADR-07, now match the SPEC: types home, `edit` signature, `Check`, `Wrap`, selector form, missing-tool guard. |

Gap 11 (DECIDE) is still open for Brad. Gap 12 (DEFER) and Gap 13 (ACCEPT) need nothing more.

### Gap 14: SLP006 "functions as" → "is" rewrites a plural noun

**Severity:** HIGH
**Origin:** fix-regression
**Pass:** literal-vs-actual
**PRD Ref:** D-4 ("deterministic substitutions that keep meaning"); FR-4.7; Brad Q3 (a)
**SPEC Ref:** §10:1060 (SLP006 row); §10:1092-1100 (the safe-fix rule, item 2); §9:1018-1027 (`WordList`, case-insensitive whole-word matching with no part-of-speech signal)

**Expected:** The SPEC's own rule: "a form that is also a noun … gets `None` from `WordList` and is left to the model."

**Actual:**
- The row maps "serves/stands/functions as" → "is" in every context.
- "functions" is very often a plural noun followed by "as", especially in the technical prose this tool targets: "export these functions as a module" → "export these is a module"; "we expose helper functions as a convenience" → "we expose helper is a convenience".
- "stands" (a plural noun, e.g. "the market stands as …") is rarer but has the same problem.
- `verify` checks numbers, URLs, terms and structure, not grammar, so `--fix` writes the result.
- The §10 fixtures test only "The tool serves as a bridge", so no test would catch this.

**Impact:** The default write path produces broken English in user files. This is the failure Gap 2 was raised for, brought back by the fix. No later gate would find it, because the fixtures cover only the cases the author chose.

**Disposition:** FIX-SPEC. Either:
- make "functions as" (and "stands as") no-fix, leaving them to the model; or
- fix them only behind a fixed singular-subject guard on the preceding token (e.g. `it`, `this`, `that`, `which`).

Add "export these functions as a module" to the SLP006 fixtures as fires-with-no-fix, and assert it in `safe_fix_contexts_grammatical`.

### Gap 15: §23.2 lacks the checks the Revision Log says were added

**Severity:** MEDIUM
**Origin:** fix-regression
**Pass:** tests
**PRD Ref:** FR-1.9, FR-1.5
**SPEC Ref:** Revision Log rows 1 and 9 (§Revision Log:2532, 2540); §23.2:2172-2182

**Actual:**
- Row 1 says "§23.2 gains the check 'no cargo: every target prints the banner'".
- Row 9 says "Added a §23.2 check".
- §23.2 still holds only the round-1 rows. The FR-1.9 row names `make test` only, with no `build` or `check`, and no row checks that the perf pass is skipped for a targeted selector.
- So the Phase 2 gate would not exercise Gap 1's fix. It would also not catch Gap 9's unwired variable.

**Impact:** The gate evidence for FR-1.9 (`build`, `check`) and FR-1.5 speed is missing. The `need` guard works today, so the result is not wrong as built. That is why this is MEDIUM.

**Disposition:** FIX-SPEC. Add two §23.2 rows:
- "With `cargo` removed: `make build`, `make test` and `make check` print the banner and exit non-zero."
- "`make test TESTS='verify'` starts no `--cargo-profile release` build (the log has no `ci-perf` run)."

### New text with no gap

| Changed area | Checked | Result |
|--------------|---------|--------|
| §5 `types.rs` row and `PURE_DIRS` file entry | `forbidden_uses` matching a file row | OK. The table note states that a row may name one file. |
| §12 directive candidates | an HTML comment inside a list item or quote (`Html` span covers the line's non-whitespace) | OK |
| §14 walker | `.git` skipped by `filter_entry`; `parents(g)`; `require_git(false)` | OK. FR-2.4 is met literally. |
| §16 `Wrap` / `protected_strings` once per block | lazy continuation → SLP902; first-line marker outside the range | OK |
| §10 Safe finding with `fix: None` selects a window | §16:1620-1621 says "fix class is Safe or Model" | OK, consistent |
| Architecture §4.9 `verify` deps / §5 graph | `edit → verify`, `verify → doc & types` | OK. No cycle, because `verify` no longer needs `edit` for `Proposal`. |

### Coverage rows updated

| Requirement | Status now |
|-------------|------------|
| FR-1.8 | PARTIAL (Gap 4, still open) |
| FR-1.9 | COMPLETE (mechanism); gate check missing (Gap 15) |
| FR-1.5 | COMPLETE; speed fix not wired (Gap 9) |
| FR-2.2, FR-2.4 | COMPLETE |
| FR-3.3 | COMPLETE |
| FR-4.7 / D-4 | PARTIAL (Gap 14) |
| FR-6.1–6.5 | COMPLETE |
| FR-8.2 | COMPLETE |

**Round 2 recommendation: fix_spec.** Gap 14 is HIGH and open.


---

## Round 3

**Changed since round 2:** SPEC v0.2 → v0.3 (55 lines: §3 `test` recipe and design note, §5 `standalone.rs` block and table, §10 SLP006 row and safe-fix design note, §23.2 two rows, Revision Log "Revision 2"). Architecture §5 module-rules table (one row).

### Earlier FIX items re-checked

| Gap | Status | Evidence |
|-----|--------|----------|
| 4 (MEDIUM) | RESOLVED (round 3) | SPEC §5: the constant is `HOST_MARKERS` and the test is the retired repository text check. The doc comment names the retired repository text check, and the file comment says no text in the file may hold the host project's name in any case. A case-insensitive grep of the §5 Rust block (SPEC lines 560-590) finds the word only in the prose bullet *outside* the code block, not in the code. The architecture §5 rule table is renamed to match. So the retired planted-reference check's "clean copy reports nothing" now holds. |
| 9 (LOW) | RESOLVED (round 3) | SPEC §3. The perf nextest run is inside `if [ -n '$(RUN_PERF)' ]; then … else echo "…perf pass skipped…" >&2; fi`. `RUN_PERF` is empty for a selector with no `perf`, and `all` when `TESTS` is unset. `rc` is kept from the unit run, and the `count` check still counts the unit XML. |
| 14 (HIGH) | RESOLVED (round 3), for the case raised | SPEC §10:1071 and §10:1113-1120. "serves/stands/functions as" → "is" fixes only when the previous token is in `SINGULAR_SUBJECT` (`it`, `this`, `that`, `which`, `who`, `he`, `she`). "export these functions as a module", "we expose helper functions as a convenience" and "The tool serves as a bridge" are fires-with-no-fix fixtures asserted by `safe_fix_contexts_grammatical`. One related context is still open: see Gap 16. |
| 15 (MEDIUM) | RESOLVED (round 3) | SPEC §23.2 has both new rows: no cargo → `make build`/`test`/`check` print the banner and exit non-zero; and `TESTS='verify'` → the "perf pass skipped" banner, no `ci-perf` run and no `target/nextest/ci-perf/junit.xml`, while `TESTS='perf'` runs the pass. |

Gap 11 (DECIDE) is still open for Brad. Gap 12 (DEFER) and Gap 13 (ACCEPT) need nothing more.

### Gap 16: SLP006 fixes ignore the word after "as"

**Severity:** MEDIUM
**Origin:** missed-earlier (found on the SLP006 row and design note that this round rewrote)
**Pass:** literal-vs-actual
**PRD Ref:** D-4 ("deterministic substitutions that keep meaning"); FR-4.7
**SPEC Ref:** §10:1071 (SLP006 row); §10:1103-1110 (the safe-fix rule: a fix must be grammatical "in *every* context the matcher can see")

**Expected:** The SPEC's own rule. A safe fix is grammatical in every context the matcher can see.

**Actual:**
- The guard looks only at the token *before* the verb. "as" also starts fixed phrases where the verb is not a copula stand-in:
  - "It functions as follows:" → "It is follows:"
  - "It serves as well as the old one" → "It is well as the old one"
  - "This can serve as well as any" → "This can be well as any" (the modal → "be" form, too)
  - "as long as", "as soon as" and "as much as" have the same problem.
- "functions as follows" is common in technical prose, the text this tool targets.
- `verify` checks numbers, URLs, terms and structure, not grammar, so `--fix` writes the result.

**Why MEDIUM, not HIGH:** the guarded "it/this functions as follows" context is far narrower than Gap 14's bare plural noun. The fixture-driven gate still would not catch it, so it needs a SPEC fix, but it does not stop the design.

**Disposition:** FIX-SPEC. Add a following-token check to safe-fix rule item 3. Either:
- fix only when the token after "as" is in a fixed list (`a`, `an`, `the`); or
- give no fix when the token after "as" is in a fixed deny list (`follows`, `well`, `long`, `soon`, `much`, `far`, `such`).

Add "It functions as follows" and "This can serve as well as any" as fires-with-no-fix fixtures, asserted in `safe_fix_contexts_grammatical`.

### New text with no gap

| Changed area | Checked | Result |
|--------------|---------|--------|
| §3 perf guard under `.ONESHELL` + `-eu -o pipefail` | The `if` sits on its own lines inside the recipe. `|| rc=$$?` keeps `-e` from aborting. The `else` banner goes to stderr. | OK. The skip is loud, not silent. |
| §23.2 new rows vs §3 | The rows' expected banners match the `need` and perf-skip text in §3 | OK |
| §5 rename vs architecture §5 | Both name the retired repository text check and `no_external_path_deps` | OK |
| §10 `SINGULAR_SUBJECT` vs §9 `WordList` | The guard is a preceding-token word list (rule item 3). "which"/"that" cover relative clauses. | OK, apart from Gap 16 |
| Revision Log "Revision 2" | Every claimed change is present in the text | OK. The round-2 Gap 15 failure (claims without text) did not repeat. |

### Coverage rows updated

| Requirement | Status now |
|-------------|------------|
| FR-1.5 | COMPLETE (Gap 9 resolved; gate row in §23.2) |
| FR-1.8 | COMPLETE (Gap 4 resolved) |
| FR-1.9 | COMPLETE (Gap 15 resolved) |
| FR-4.7 / D-4 | COMPLETE for Gap 14; one narrow context open (Gap 16) |

### Tests

| FR / NFR | Test file | Test method | Status |
|----------|-----------|-------------|--------|
| FR-1.8 | `tests/standalone.rs` | the retired repository text check, the retired planted-reference check | PRESENT |
| FR-1.9 / FR-1.5 | §23.2 build checks | no-cargo banner per target; perf-skip for `TESTS='verify'` | PRESENT |
| FR-4.7 / D-4 | `tests/rules_fixtures.rs` | `safe_fix_contexts_grammatical` | VAGUE: Gap 16 (no "as follows" / "as well as" fixture) |

**Round 3 recommendation: proceed.** No BLOCKING or HIGH gap is open. One MEDIUM FIX-SPEC item (Gap 16) and one DECIDE (Gap 11) remain.
