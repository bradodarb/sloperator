# Story 02: Prove no network with an OS sandbox, not proxy variables

**Epic:** [epic-08-eval](../epic-08-eval.md)
**SPEC Refs:** §23.19
**PRD Refs:** FR-9.1
**Component / ADR:** §4.10 engine::runtime (ADR-02)
**Wave:** 1

## Scope

`no_network_during_check` (`tests/model_real.rs:249-279`) only sets HTTP(S)_PROXY/ALL_PROXY, which llama.cpp never reads, so a direct socket would still pass (epic-07 audit Gap 2). Add `no_network_under_sandbox` to `tests/model_real.rs`: run the shipped binary (`CARGO_BIN_EXE_sloperator`) with `check --fix` on a plain file inside a network-denying sandbox — macOS `sandbox-exec -p '(version 1)(allow default)(deny network*)'`, Linux `unshare -rn` — and assert exit 0 or 1 with no `sloperator:` error on stderr. Keep the existing proxy test.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `tests/model_real.rs` | existing | tests below (methods added) |

## Acceptance Criteria

- [ ] `check --fix` with the real model succeeds (exit 0/1, no `sloperator:` error) when the OS denies every network operation (FR-9.1).
- [ ] A control run proves the sandbox bites: the same sandbox command makes a TCP connect to 127.0.0.1 fail (e.g. `sandbox-exec … /usr/bin/nc -z 127.0.0.1 <listening port>` exits non-zero while the listener is up), so the test cannot pass with a no-op sandbox.
- [ ] When neither `sandbox-exec` nor `unshare -rn` works on the host, the test fails loudly naming the missing tool; it never passes silently.
- [ ] Without the default model, the test fails naming `sloperator model pull` (`require_model`).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::model_real` | `no_network_under_sandbox` | no network under an OS sandbox, with a control that the sandbox denies connects | FR-9.1 |

## Notes

Only `tests/model_real.rs` changes; no `src/` change is expected. If the sandbox shows the binary does open a socket, that is a real FR-9.1 defect: stop and report it rather than weakening the test.

Gate: `make test-integration TESTS='sloperator::model_real#no_network_under_sandbox'`
