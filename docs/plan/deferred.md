# sloperator — deferred items

Items raised at the gates that a later stage owns. The slice turns each into an acceptance criterion or a story.

## From the PRD gate (reviewer, 2026-10-29)

1. **Integration-tier report location (FR-1.4, FR-1.7, D-11).** FR-1.4 names only the unit tier's report. The
   integration tier needs its own JUnit report so a unit run never overwrites it. Owner: SPEC → story acceptance
   criterion: after `make test-integration`, `target/nextest/it/junit.xml` holds the integration testcases.
2. **`TESTS` selectors (FR-1.5, D-11).** Each selector must reach nextest as part of one filterset. Owner: SPEC →
   acceptance criterion: `TESTS='<one test>'` produces JUnit XML with exactly that testcase.
3. **Acceptance rule for model rewrites is stated two ways.** FR-9.5 accepts a rewrite when the detectors report
   "no more weighted findings" than before (equal passes); Risk 2's mitigation says FR-9.5 "rejects a rewrite that does
   not lower" them. Owner: SPEC picks one (the reviewer reads "strictly lower" as the intent, since an equal-score
   rewrite only adds risk) → acceptance criterion with an equal-score fixture.
4. **Verification-failure findings and exit codes (FR-8.7 vs FR-2.8/FR-2.9).** FR-8.7 says a failed verification
   "SHALL NOT exit 0 silently"; the PRD does not say that finding's severity, nor whether `--exit-zero` or
   `--min-severity error` can hide it. Owner: SPEC → acceptance criterion fixing its severity and exit behaviour.

## From spec-review round 1 (reviewer, 2026-10-30)

5. **Gap 12: how `perf.rs` proves NFR-3's "SHALL NOT read the model cache".** SPEC §23.18 says
   `rules_only_100kb_under_1s` "also asserts that the model cache dir was never read", but gives no mechanism.
   Owner: the story that delivers §23.18 (the rules engine / CLI epic). Acceptance criterion: the test points
   `model.cache_dir` at a directory it cannot read (for example mode 000), and the rules-only check still exits 0 within budget.

