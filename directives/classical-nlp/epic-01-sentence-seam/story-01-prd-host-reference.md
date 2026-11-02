# Story 01: Make the standalone host-reference scan pass on this branch

**Epic:** [epic-01-sentence-seam](../epic-01-sentence-seam.md)
**SPEC Refs:** §1 (Standalone-scan hygiene of the initiative docs)
**PRD Refs:** NFR-1, NFR-10
**Component / ADR:** Architecture §12 phase-1a gate
**Wave:** 1

## Scope

Two things make `standalone::no_host_references` (tests/standalone.rs:83-87) fail on this branch, and this story fixes both.

1. SPEC §1: rewrite the NFR-10 'Verified by' cell at docs/classical-nlp/PRD-classical-nlp.md:298 so it says "the listed tests exist and pass under `make test`" (the repo's own form, docs/DEVELOPMENT.md:20-28). Nothing else in the PRD changes.
2. Found while slicing; the SPEC misses it. The workflow's planning tree `directives/` is tracked on this branch, and every story file in it carries the mandated gate line, which contains two of the scan's markers (the host's name and `UNIT=`, tests/standalone.rs:15-28). Add `"directives"` to `NOT_SCANNED` (tests/standalone.rs:37) and extend its doc comment to say "and workflow planning artifacts (directives/), which are not shipped content". No other marker or scan logic changes.

No `src/` change.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `docs/classical-nlp/PRD-classical-nlp.md` | 298 | the NFR-10 'Verified by' sentence, in the repo's own `make test` form |
| `tests/standalone.rs` | 36-37 | `NOT_SCANNED` gains `"directives"`; doc comment says why |

## Acceptance Criteria

- [ ] docs/classical-nlp/PRD-classical-nlp.md contains none of the scan's markers, and the rest of the PRD is byte-identical (`git diff --stat` shows a one-line change) (NFR-1)
- [ ] `NOT_SCANNED` is `[".git", "target", "directives"]`, and `host_markers` is unchanged (NFR-1)
- [ ] `standalone::no_host_references` passes with the directives tree present (NFR-1)
- [ ] `standalone::planted_reference_is_caught` still passes, so the scan still catches a planted reference in `src/` (NFR-1)
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::standalone` | `no_host_references` | no shipped repository file names the development host's build contract | NFR-1 |
| 2 | `sloperator::standalone` | `planted_reference_is_caught` | the scan still finds a planted reference | NFR-1 |

## Notes

tests/standalone.rs:15-35 lists the markers. Docs written by later stories (CHANGELOG, REFERENCE, eval/, docs/classical-nlp/spikes/) are still scanned, so they must use only the repo's `make test TESTS='…'` / `make build` form. The `directives` exclusion is this slice's provisional answer to a question only Brad can settle (index.md, "Open question for Brad"). If he picks the other option, this story drops part 2 and the landing removes `directives/` instead.

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='sloperator::standalone'`
