# Story 02: Layering and standalone scans; integration-tier bridge

**Epic:** [epic-01-walking-skeleton](../epic-01-walking-skeleton.md)
**SPEC Refs:** §5, §23.1, §3 (integration tier), DQ-1
**PRD Refs:** FR-1.7, FR-1.8, NFR-2, FR-9.1
**Component / ADR:** architecture §5 module rules; ADR-01, ADR-07
**Wave:** 2

## Scope

Add `tests/layering.rs`, whose `FORBIDDEN_USES`, `PURE_DIRS` and `IO_PATHS` tables are verbatim from §5, and `tests/standalone.rs`, which holds `HOST_MARKERS` built with `concat!`, `find_markers`, and the integration-tier `builds_when_extracted`. These tests pass on the stub tree from story 01 and keep guarding every later epic. No production code changes.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `tests/layering.rs` | new | tests below |
| `tests/standalone.rs` | new | tests below |

## Acceptance Criteria

- [ ] Every layering test passes on the current tree, and a planted `use crate::engine` in `src/doc/mod.rs` makes `forbidden_uses` fail (checked locally, not committed) (FR-1.8).
- [ ] `planted_reference_is_caught` proves the concat marker form cannot hide a real reference (Gap 4).
- [ ] `tests/standalone.rs` contains the banned host word in no case form (Gap 4, round 2).
- [ ] Deferred item 1 (DQ-1): after the Gate's integration leg, `target/nextest/it/junit.xml` holds the `builds_when_extracted` testcase (FR-1.4, FR-1.7).
- [ ] `builds_when_extracted` builds a copy of the crate without `.git` or `target/`, offline, and passes (NFR-2).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::layering` | `forbidden_uses` | no module uses a forbidden `crate::` path | FR-1.8 |
| 2 | `sloperator::layering` | `pure_modules_have_no_io` | no std::fs/net/process/env in pure modules | FR-1.8 |
| 3 | `sloperator::layering` | `runtime_crate_only_in_adapter` | llama_cpp_2/mistralrs appear only in engine/runtime.rs | ADR-02 |
| 4 | `sloperator::layering` | `network_only_in_pull` | ureq appears only in model/pull.rs | FR-9.1 |
| 5 | `sloperator::layering` | `single_exit_point` | std::process::exit only in main.rs | NFR-4 |
| 6 | `sloperator::standalone` | `no_host_references` | no host-project marker in the crate | FR-1.8 |
| 7 | `sloperator::standalone` | `planted_reference_is_caught` | the scan finds a planted marker | FR-1.8 |
| 8 | `sloperator::standalone` | `no_external_path_deps` | no `path =` dependency in Cargo.toml | FR-1.8, NFR-2 |
| 9 | `sloperator::standalone` | `builds_when_extracted` | the extracted copy builds and tests offline | NFR-2, FR-1.7 |

## Notes

The integration leg compiles with `--features model` (llama.cpp), so it needs clang and cmake (Phase 0). R5 is accepted: text scans can miss re-exports.

Gate: `make test TESTS='sloperator::layering sloperator::standalone' && make test-integration TESTS='sloperator::standalone#builds_when_extracted'`
