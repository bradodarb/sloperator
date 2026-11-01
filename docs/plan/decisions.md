# Decisions — sloperator

Brad’s answers to the questions the reviews could not settle. Authors build exactly these; reviewers do not reopen them.

## prd gate

**Open:**

VERDICT: approve on substance; escalate=yes because the PRD runs on assumptions that only Brad can confirm (new public surface, new external dependencies, a departure from a convention, a licence).

CHECKLIST: no PRD checklist exists, so I judged against the review rules: traceability, testability, scope honesty and conventions.

WHAT I VERIFIED:
- Build claims hold: D-2 and D-11 describe a crate that builds and tests on its own through make targets over cargo and nextest.
- Traceability and testability are strong. Each FR is a SHALL with an observable outcome (exit codes, files, JSON fields), and NFRs 1-9 each carry a 'Verified by' line. Phases 1-7 map to the FR groups: the spike covers FR-9.9, the shell FR-1, protection FR-8, rules FR-4/5/6, CLI FR-2/3/7, model FR-9, eval FR-10.
- Scope is honest. It does not claim fact preservation beyond numbers, URLs and terms (D-7, Risk 1). It makes no detection-evasion claim (FR-5.4). Out-of-scope items are explicit.
- Conventions: fail loudly (D-5, NFR-4, FR-9.6/9.7) and clean breaking changes (NFR-9) are honored. The camelCase config-key rule is departed from on purpose and openly (D-9); that is a DECIDE below, not a defect.

FIX items: none blocking. Nothing here would be built wrong in a way that the SPEC review or the story gates would not catch.

DEFER (written to docs/plan/deferred.md, 4 items):
1. The integration-tier JUnit report must land in its own location, not the unit tier's. FR-1.4 names only the unit tier.
2. TESTS selectors must reach nextest as one filterset, one selector giving one testcase. Phase 1 task 4 already plans to check this.
3. FR-9.5 ('no more' weighted findings) disagrees with Risk 2 ('does not lower'). The SPEC must pick one.
4. FR-8.7: the PRD does not say the severity of a verification-failure finding, or how it interacts with --exit-zero and --min-severity.

ACCEPT:
- Unmeasured performance: NFR-3 sets the budget and the spike measures it.

DECIDE — for Brad (the PRD's own Follow-up Questions; it proceeds on the assumption in [brackets]):
Q1. Where does the crate live? (a) a self-contained crate with its own Makefile, CI and repository [assumed, recommended]. Recommend (a): nothing outside the crate is needed to build or test it.
Q2. New network and external dependency: model weights (1.8-4.3 GB) from Hugging Face. (a) explicit `model pull`, exit 2 when the model is missing [assumed, recommended]; (b) auto-download on first --fix; (c) silent fallback to rules-only. Recommend (a): (c) breaks fail-loudly, and (b) makes hidden network calls.
Q3. Are verified model rewrites written in place by --fix by default? (a) yes, when verification passes [assumed]; (b) only behind --unsafe-fixes, with --diff otherwise; (c) never. Recommend (a), with --diff available. Caveat: verification cannot catch a changed name or a flipped claim.
Q4. Default model and licence. (a) Qwen3-4B Q4_K_M after the spike [assumed], with Phi-4-mini (MIT) as fallback; (b) Qwen3-1.7B for speed; (c) Phi-4-mini outright. Llama 3.2 is excluded because its AUP bars passing output off as human-written. Recommend (a), provided the Phase 1 licence read clears Qwen.
Q5. Config keys depart from the camelCase config-key convention. (a) sloperator.toml with snake_case keys, because the tool leaves the repo and follows Rust idiom [assumed, recommended]; (b) camelCase keys in TOML; (c) YAML. Recommend (a), but only Brad can waive a non-negotiable.
Q6. New CLI surface for the public tool: check/--fix/--diff, rules, config, model pull|list|path; output formats text|concise|json|github; exit codes 0/1/2; exit 1 at warning or above (Q6/Q7). Recommend accepting it as written.
Q7. Machines without Rust (FR-1.9). (a) every make target fails with a one-line banner naming the missing tool and its install step [recommended]. Recommend (a): it fails loudly.
Q8. Code licence: (a) MIT OR Apache-2.0 [assumed, recommended]; (b) MIT; (c) proprietary until release.
Other questions with sound defaults (Q8 fine-tuning deferred, Q11 in-process runtime only): accept the assumptions unless Brad objects.

**Brad:** all assumed/recommended options look solid, but I do want to confirm that Q7 is really around building the project, right? we should be able to build binaries for distribution on any machine, correct?

## spec gate

**Open:**

Gap 11: How are distribution binaries produced (OD-1)? Yes, Q7 covers only building the crate; a built binary needs no Rust. (a) a per-target native CI matrix (macOS arm64 + Metal, Linux x86_64/arm64, Windows x86_64) after extraction; (b) cross-compile every target from one host with cargo-zigbuild/cross: fragile with llama.cpp cmake, and no Metal; (c) source only via cargo install: every user needs Rust, clang and cmake; (d) as (a), plus a sloperator.package target now: one extra story. Recommend (a): it is the only option that ships a Metal macOS binary and CPU binaries everywhere without a fragile cross toolchain.

**Brad:** we'll eventually have a CI/CD process that builds and processes these, but that is out of scope for now.

## slice gate

**Open:**

The gate notes were flagged for your call; the reviewer said approve without escalating. Its notes: VERDICT: approve, escalate=no. No blocking item.

CHECKLIST: no slice checklist exists, so I judged against the review rules: traceability, testability, scope honesty and conventions.

WHAT I VERIFIED:
- Traceability to SPEC §23. I extracted every backticked identifier of 6+ characters from SPEC §23 onward and diffed it against every story file. Every §23 test name appears in a story. The only identifiers missing from the stories are prose words and config words (another, leveraged, profile, frozen, respect_gitignore, now_seed and similar), not test methods. The Makefile's tool guard and selector helpers are covered in e01 s01's Makefile row.
- All 5 deferred items are concrete, checkable acceptance criteria:
  - #1 (the integration tier's own JUnit report) is in e01 s02.
  - #2 (one selector gives exactly one testcase; `no_such_test` fails loudly) is in e01 s01.
  - #3 (strictly fewer weighted findings) is in e03 s02 as the unit-level `improvement_requires_strictly_fewer` and in e06 s03 as the end-to-end `equal_score_rewrite_rejected`.
  - #4 (SLP901 under --min-severity error and --exit-zero) is in e06 s03 test 13.
  - #5 (cache_dir set to mode 000) is in e05 s08. The story also handles uid 0: the test skips loudly and never passes silently, which honors fail-loudly.
- Dependencies match the architecture. Epic 07 needs both the spike epic and the spike-results.md file, which is the Phase 1 gate. The spike runs in parallel with epics 03-06, as architecture §12 allows. Epic 08 checks that MODEL_WINDOW_BUDGET_S comes from the spike record.
- Scope is honest. OD-1 (no packaging story) and the Q1-Q8 assumptions are applied as decisions.md records them; I did not reopen them. Stories that touch Cargo.toml for mistralrs are conditional on the spike, and SPEC §2 says so.
- Conventions are met. Every story Gate goes through `make test` with exact `binary#test` selectors. Fail-loudly holds: an empty selector, a missing model or a runtime error each exit non-zero. NFR-1 (the crate is self-contained) is an AC in e01 s01.

ACCEPT:
- Six test methods not named in SPEC §23. Each one closes a deferred item or guards a corpus, and each is explained in its story's Notes. This is additive coverage, not scope creep.
- No Gate has been run yet because the toolchain is not installed. This is an implementation-stage risk, and epic 01's Gate is the first place it can surface.
- Epic 04's rule stories run in sequence, a throughput cost only. They share builtin/mod.rs.
- The e01 s02 integration leg compiles llama.cpp through the default `model` feature, so clang and cmake (Phase 0) are needed from epic 01 on. The story's Notes state this.

FIX: none. DEFER: none new.

**Brad:** (no answer — approved: build the recommended option of each open decision above)