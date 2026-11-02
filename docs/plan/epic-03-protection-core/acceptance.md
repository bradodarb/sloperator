# Acceptance: epic-03-protection-core

> Historical record: the one-time repository text scan and its planted-reference test have since been removed. Current standalone checks cover path dependencies and extracted builds (SPEC §5).
**Verdict:** gaps
**Base:** f6b6076dcc71cea5abad489afa9bed00ddfdb2f6
**Reviewer:** reviewer, round 2 (after story-04)

## Criteria

| Criterion | Status | Evidence |
|---|---|---|
| All `fixtures/protect/*` round-trip byte-identical under a no-op plan | met | `tests/edit_apply.rs:63` `noop_plan_round_trips_every_protect_fixture` compares bytes for the 10 fixtures under `fixtures/protect/`; passes |
| Forged edits that drop a number or add a heading are rejected | met | `tests/verify_edits.rs` `rejects_lost_number`, `rejects_new_heading_marker` (line ~135); both pass |
| `sloperator check` parses every input with `doc::parse` and exits 0 on a clean file | met | `src/cli/mod.rs:56` calls `doc::parse` for each path; `src/cli/mod.rs:68` returns `ExitCode::Clean` when every read succeeds |
| Epic-01 layering tests still pass | met | the whole-unit run (`make test`) passed 44 of 44, including `sloperator::layering` `forbidden_uses`, `pure_modules_have_no_io`, `single_exit_point` and the others |
| The tests named in every story pass together on the merged epic | met | targeted run of `verify_edits`, `edit_apply`, `doc_spans` and the retired repository text check passed 33 of 33. The whole unit passed 44 of 44. Gates tests and discriminate both report PASS |

## Round-1 FIX items

- **G1 (host name in `rejects_lost_term`): closed.** `tests/verify_edits.rs:119-140` now uses `Acme`, and every assertion is unchanged (`Check::Terms`, `Protected | Terms`, `terms.get("Acme") == Some(&2)`). A case-insensitive grep for the host name under `src`, `tests` and `fixtures` finds nothing, and the retired repository text check passes.

## Gaps

### G2: `verify` and `edit` are not reachable as shipped, and the epic does not name the epic that wires them. BLOCKING
- **Where:** No file in `src` outside `src/verify/` and `src/edit/` calls `verify::` or `edit::`. A grep for `verify::|edit::|use crate::(verify|edit)` over src, excluding those two directories, returns nothing. They are reached only from `tests/verify_edits.rs` and `tests/edit_apply.rs`. `doc::parse` is the only part that is wired (`src/cli/mod.rs:56`).
- **Why blocking:** Under the review contract, a part that only tests can reach is blocking unless this epic's file names the later epic that wires it. `docs/plan/epic-03-protection-core.md` (Notes, lines 28-30) names no later epic. The wiring is planned elsewhere: `epic-05-linter-surface/story-04-pipeline.md:11` (fix step 1: plan → verify → apply), `epic-04-rules-score-directives/story-08-catalogue-and-check-wiring.md:47`, and `epic-06-model-fix-path/story-03-model-window-pipeline.md:11` (`verify::edit`, `verify::improvement`). Those epics exist, but this epic file does not point to them.
- **Fix:** Add a sentence to the Notes of `docs/plan/epic-03-protection-core.md`: `verify::edit`, `edit::plan` and `edit::apply` are wired into the shipped path by epic-05-linter-surface story-04-pipeline (fix step 1). `verify::improvement` and the model path are wired by epic-06-model-fix-path story-03. No code change is needed.
- **Severity:** blocking (process/traceability; the code is correct). **Disposition:** FIX (epic file Notes).

No other gaps found.
