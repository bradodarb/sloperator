# Epic 09: Backlog: delivery follow-ups

**Status:** delivered

**Goal:** Close the non-blocking items that the acceptance audits deferred after epic 08, so the integration tier passes on any developer host.
**PRD Refs:** FR-1.7, FR-2.7–2.9
**SPEC Refs:** §5, §23.16
**Modules:** `sloperator`

## Waves

Copied from this epic's line in `index.md`. Stories in one wave run in parallel; each wave waits for the one before.

| Wave | Stories | Why this order |
|------|---------|----------------|
| 1 | [story-01-cli-exit-tmpdir-independent](./epic-09-backlog/story-01-cli-exit-tmpdir-independent.md) | One test-only story that touches only `tests/cli_exit.rs` (changes.md CH-18). |

## Acceptance Criteria

- [ ] `make test-integration` exits 0 even when `$TMPDIR/.gitignore` ignores everything (CH-18).
- [ ] The tests named in every story's Test Cases pass together on the merged epic.

## Notes

Trailing backlog epic, added by the re-plan after epic-08-eval. The item comes from the epic-08 acceptance audit, Deferred D1 (`docs/backlog/cli-exit-tmpdir-gitignore.md`). It is environmental and was already present at base, and no epic depends on it, so it is appended here and blocks nothing.
