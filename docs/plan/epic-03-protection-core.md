# Epic 03: Protection core: span map, verifier, source-range edits

**Goal:** Every Markdown and plain-text file parses into a byte-exact span map; edits are byte ranges that only a verifier can approve; a no-op plan round-trips every protect fixture byte-identical.
**PRD Refs:** FR-8.1–FR-8.6, FR-8.9 (apply side), FR-6.6 (frozen), FR-9.5 (improvement), FR-2.5 (diff), D-7, D-10
**SPEC Refs:** §6 (`doc`), §7 (`edit`), §8 (`verify`), §9 (types.rs), §23.3–§23.5
**Modules:** `sloperator`
**Status:** Delivered. All four stories merged: `doc` span map and `types.rs` (story 01), `verify` gate (story 02), `edit::plan/apply` (story 03), and the neutral-term fixture fix G1 (story 04). Tests: `tests/doc_spans.rs`, `tests/verify_edits.rs`, `tests/edit_apply.rs`; fixtures in `fixtures/protect/`.

## Waves

Copied from this epic's line in `index.md`. Stories in one wave run in parallel; each wave waits for the one before.

| Wave | Stories | Why this order |
|------|---------|----------------|
| 1 | [story-01-doc-span-map](./epic-03-protection-core/story-01-doc-span-map.md) | `doc` and the leaf `types.rs` come first; everything else reads the span map. |
| 2 | [story-02-verify-gate](./epic-03-protection-core/story-02-verify-gate.md) | `verify` defines `Verified<T>` and checks edits against a `Document`. |
| 3 | [story-03-edit-plan-apply](./epic-03-protection-core/story-03-edit-plan-apply.md) | `edit::apply` accepts only `Verified<Proposal>`, so it follows verify. |
| 4 | [story-04-neutral-term-fixture](./epic-03-protection-core/story-04-neutral-term-fixture.md) | Acceptance fix G1: removes the host name from `rejects_lost_term` so the standalone scan passes. |

## Acceptance Criteria

- [ ] All `fixtures/protect/*` round-trip byte-identical under a no-op plan (architecture Phase 3 gate).
- [ ] Forged edits that drop a number or add a heading are rejected (Phase 3 gate).
- [ ] `sloperator check` parses every input with `doc::parse` and still exits 0 on a clean file (wired into the skeleton).
- [ ] The layering tests from epic 01 still pass (doc and types.rs use no `crate::` module).
- [ ] The tests named in every story's Test Cases pass together on the merged epic.

## Notes

Architecture deviation (SPEC Architecture Deviation Notes): `Verified` lives in `verify` with a private constructor (E0742), and `RuleCode`, `Proposal` and `ProposalOrigin` live in the leaf `src/types.rs` (Gap 3).
