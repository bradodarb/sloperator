# Decisions — classical-nlp

Brad’s answers to the questions the reviews could not settle. Authors build exactly these; reviewers do not reopen them.

## spec gate

**Open:**

Gap 4: Should Safe wordiness extend default-on SLP001 or live in a new default-off rule? — (a) Extend SLP001 with separate detection-only intensifiers; default users gain fixes and findings (b) New default-off wordiness rule; preserve current defaults but require opt-in — recommend (a) because the PRD identifies the existing phrase table and inherits its defaults.
Gap 5: Must report availability honor configurable minimums? — (a) Configure report minimums consistently, including with metric rules off (b) Fix raw-report floors at 5/100 and configure only rule firing, reducing the PRD contract — recommend (a) because D-5 and FR-2.7 explicitly require configurability.
Gap 11: Preserve the literal capitalised-token proxy with narrow Safe compatibility, or approve a global common-word exemption? — (a) Protect model edits literally and constrain Safe exceptions; preserve names and existing fixtures (b) Exempt common words globally; titles/names may disappear and FR-6.2 narrows — recommend (a) because one Safe hedge fixture does not justify weakening model name protection.

**Brad:** (no answer — approved: build the recommended option of each open decision above)

## slice gate

**Open:**

APPROVE — no blocking FIX found under review-loop.md rules 2–4. Reviewed index.md, all 9 epic artifacts and all 34 stories against PRD FR-1–FR-11/NFR-1–NFR-10, the SPEC testing strategy and architecture delivery constraints. The optional .aegis/process/checklists/slice.md and docs/context/README.md/conventions.md are absent (read tools returned missing); docs/DEVELOPMENT.md supplies the available development conventions. This is a plan review, not a claim that its future tests already pass.

ACCEPT — Traceability and testability: each story identifies PRD/SPEC/architecture references, concrete files, observable acceptance criteria, named tests and a scoped UNIT=sloperator contract gate. The walking skeleton actually connects detect_in, RuleCtx and windows (epic-01 story-05), rather than stopping at unused helpers. Its sentence_range oracle and zero-rescan test are explicit (epic-01 story-04/05). The baseline story precedes all rule changes through the dependency graph. Metric formulas, configured availability, before/after reports, MCP output, defaults, score integration, verifier acceptance/rejection, conservative template declines, redundancy boundaries and performance all have named tests.

ACCEPT — Scope and sequencing: index.md's explicit needs preserve the locked pillar order; calibration depends on completed metrics and can run beside metric rules without shared production files. Verifier invariants precede widened fixes. Epic-07 story-01 requires a recorded passing spike and fails loudly rather than dropping SLP006 widening; epic-08 explicitly requires PASS. Gap 8 is owned by epic-09 story-01 with unchanged inflection assertions against the shipped backend; Gap 9 is owned by epic-07 story-03 with distinct candidate passes, and story-02 retains no_candidate_no_tagging. These existing deferred items need no additional deferral.

ACCEPT — Brad's decisions are preserved: SLP001 rows plus default-off detection-only SLP022; configurable report floors independent of enabled rules and bounded metric-rule minimums; rule-origin common-word exception with literal model-name protection. The slice explicitly identifies stale architecture/SPEC constants and implements the approved decision instead (epic-02 Notes; epic-03 story-02). The stemmer dependency gate is already authorized by PRD D-9, not a fresh dependency decision.

DECIDE D1 (origin: decision; confidence high) — May the tracked directives/ planning tree be excluded from the standalone host-reference scan? Trigger: every story's mandated Gate line contains host markers; tests/standalone.rs:15–28 defines those markers, :37 excludes only .git and target, and :40–58 recursively enumerates remaining content. Epic-01 story-01 Scope item 2 and its acceptance criteria provisionally add directives to NOT_SCANNED, expressly subject to Brad's answer (Notes; index.md Open question for Brad). This narrows the existing repository-wide standalone contract (tests/standalone.rs:1–6; docs/DEVELOPMENT.md:36–42), so the reviewer cannot silently approve the scope change. Options: (a) Approve excluding only top-level directives/ as workflow planning artifacts; keep scanning src/tests/docs/eval/fixtures and retain planted_reference_is_caught. (b) Retain the whole scan and require landing to remove directives/ before merge; drop story-01's exclusion change and adjust the branch-present scan criterion accordingly. Recommend (a): it accommodates mandated workflow artifacts while retaining the product-content scan and its planted-reference regression. No blocking FIX depends on this unanswered decision; carry it to the human stop before commit.

**Brad:** (no answer — approved: build the recommended option of each open decision above)