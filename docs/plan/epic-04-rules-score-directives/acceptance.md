# Acceptance: epic-04-rules-score-directives

**Verdict:** met
**Base:** 15a94cbe51c97a17a2fcce234363194b65bb308d
**Reviewer:** reviewer
**Round:** 2. This round re-checks G1 from round 1.

## Epic criteria

| Criterion | Status | Evidence |
|---|---|---|
| Each of the 16 rules has fixtures, and its positive, negative and (safe rules) fix tests pass | met | `src/rules/builtin/mod.rs` registers SLP001-SLP016. The `rule_fixtures!` call at `tests/rules_fixtures.rs:135-192` makes the tests for each rule: positive and negative tests for all 16, plus fix tests for the safe rules 1, 6, 8, 9, 12, 13, 14 and 16. `check_fix` (`:100-109`) runs the fixes through the real plan → verify → apply path and compares the result with `fixed.md`. The `model` arm (`:123-130`) asserts that the rule is not safe and attaches no fix. `check_negative` (`:86-98`) requires the trigger to appear in a fence, in inline code and in a link (FR-4.6). `every_rule_has_fixtures` enforces the fixture contract. tests_output: PASS |
| FR-8.9 idempotence over all fixtures | met | `tests/rules_fixtures.rs:455-468` `fix_is_idempotent_over_all_fixtures` applies every safe rule twice over every fixture and asserts that the second pass changes nothing |
| `check` with a warning exits 1 and prints one line per finding; suggestions only exits 0 (FR-2.7, D-8) | met | In `src/cli/mod.rs:55-91`, `check` prints `path:line:col: CODE [severity] message` for each finding and returns `Findings` when any finding is `>= Severity::Warning`, `Clean` otherwise. `lint` (`:113-132`) calls `directives::extract` and then `rules::detect` with `default_rules` (`:94-110`). Tests: `tests/cli_exit.rs:43` `exit_one_on_warning` checks one line, `SLP003 [warning]` and `ExitCode::Findings`. `:59` `exit_zero_on_suggestion_only` checks `Clean` and that `SLP002 [suggestion]` is printed. `:77` `exit_two_on_directive_error` checks exit 2 on a directive error (FR-6.5) |
| The tests named in every story pass together | met | tests_output: PASS. discriminate_output: PASS, so the new tests fail when their requirement regresses |

## Reachability

- The rules, registry, `detect` and `directives::extract` are reached from the shipped `check` command (`src/cli/mod.rs:56-57`, `:125-126`).
- `score::score` and `score::overall` (`src/score/mod.rs:26`, `:40`) are still called only from `tests/score.rs`: a grep of `src/**` finds only the definitions. The epic file's Notes (`epic-04-rules-score-directives.md:31`) now name the later epic that wires them, epic-05-linter-surface story-04-pipeline (step 4, FR-5.2). That story exists, and `docs/plan/epic-05-linter-surface/story-04-pipeline.md:11` and `:26` plan the wiring. The review contract's exception applies, so this is not blocking.

## Story criteria (spot-checked)

- **Story 01:** `detect` skips protected and suppressed spans and never suppresses system codes (`rules_core` tests). `rules_registry` `system_codes_not_in_registry` passes. Met.
- **Story 03:** the nine `tests/directives.rs` tests cover ignore, regions, an unclosed disable, ignore-file, unknown and system codes, and directives inside a fence or inline code. `extract` is wired into `check`. Met.
- **Story 04:** `tests/score.rs` checks the README formula (46.67), the word-weighted overall score, the exclusion of SLP9xx findings, determinism, and the FR-5.4 wording. Met. The score is reachable later (see Reachability).
- **Story 08:** the catalogue has 16 rules in 9 families, there is an explicit default set, new rules start off, and `only_safe_rules_attach_fixes` (FR-4.7) passes. `check` is wired. Met.

## Gaps

### G1 (round 1): `score` is not reachable as shipped. RESOLVED
The epic Notes now name epic-05-linter-surface story-04-pipeline as the wiring epic (`epic-04-rules-score-directives.md:31`), and that story exists. No code change was needed.

### G2: no-op metadata assertion. Non-blocking, DEFERRED
- **What:** `tests/rules_registry.rs:66` has `let _ = (m.severity, m.default_on, m.fix);`, which checks nothing at runtime. The type system already requires these fields, so FR-4.1 holds and no result is wrong.
- **Severity:** minor.
- **Disposition:** DEFER. Filed as `docs/backlog/epic-04-registry-noop-assert.md`.
