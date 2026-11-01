# Epic 04: Rules, score and directives

**Goal:** `sloperator check` runs all 16 built-in rules, honours inline directives, and exits 1 when a warning or worse is found.
**PRD Refs:** FR-4.1–FR-4.7, FR-5.1–FR-5.4, FR-6.1–FR-6.5, FR-6.7, FR-8.9, FR-2.7, D-4, D-8
**SPEC Refs:** §9 (rules core), §10 (catalogue), §11 (score), §12 (directives), §23.6–§23.8
**Modules:** `sloperator`
**Status:** Delivered. All eight stories merged: rule core (`rules` trait, registry, helpers, `detect`), the 16 rules SLP001-SLP016 in `src/rules/builtin/`, `directives`, `score`, and the catalogue tests plus `check` wiring. Tests: `tests/rules_core.rs`, `tests/rules_fixtures.rs`, `tests/rules_registry.rs`, `tests/directives.rs`, `tests/score.rs`, `tests/cli_exit.rs`; fixtures in `fixtures/rules/` and `fixtures/cli/`.

## Waves

Copied from this epic's line in `index.md`. Stories in one wave run in parallel; each wave waits for the one before.

| Wave | Stories | Why this order |
|------|---------|----------------|
| 1 | [story-01-rule-core](./epic-04-rules-score-directives/story-01-rule-core.md) | The rule trait, registry, helpers and `detect` come first; everything else uses them. |
| 2 | [story-02-rules-vocabulary-hedging](./epic-04-rules-score-directives/story-02-rules-vocabulary-hedging.md), [story-03-directives](./epic-04-rules-score-directives/story-03-directives.md), [story-04-score](./epic-04-rules-score-directives/story-04-score.md) | Parallel: the first rule family, directives and score touch disjoint files and need only the core. |
| 3 | [story-05-rule-copula](./epic-04-rules-score-directives/story-05-rule-copula.md) | Copula rules share `builtin/mod.rs` and `rules_fixtures.rs` with wave 2, so they run after it. |
| 4 | [story-06-rules-patterns-residue](./epic-04-rules-score-directives/story-06-rules-patterns-residue.md) | The pattern rules share the same two files, so they run after wave 3. |
| 5 | [story-07-rules-density-markup](./epic-04-rules-score-directives/story-07-rules-density-markup.md) | The density and markup rules share the same two files, so they run after wave 4. |
| 6 | [story-08-catalogue-and-check-wiring](./epic-04-rules-score-directives/story-08-catalogue-and-check-wiring.md) | The registry-wide tests and the check wiring need all 16 rules. |

## Acceptance Criteria

- [ ] Each of the 16 rules has fixtures, and its positive, negative and (safe rules) fix tests pass (architecture Phase 4 gate).
- [ ] The FR-8.9 idempotence test passes over all fixtures (Phase 4 gate).
- [ ] `sloperator check` on a file with a warning finding exits 1 and prints one line per finding; a file with only suggestions exits 0 (FR-2.7, D-8).
- [ ] The tests named in every story's Test Cases pass together on the merged epic.

## Notes

Rule stories run in sequence because each one adds lines to `src/rules/builtin/mod.rs` (the `all()` list) and to the `rule_fixtures!(...)` macro call in `tests/rules_fixtures.rs`. The safe-fix rule (Gap 2) applies to every safe rule: a fix only where it is grammatical in every context the matcher sees, with allow-list guards. Gaps 14 and 16 apply to SLP006. Until epic 05 story-05 adds `report`, check prints a minimal `path:line:col: CODE [severity] message` line. `score::score` and `score::overall` (`src/score/mod.rs`) are wired into the shipped path by epic-05-linter-surface story-04-pipeline (step 4, "report and scores", FR-5.2); until then only `tests/score.rs` calls them.
