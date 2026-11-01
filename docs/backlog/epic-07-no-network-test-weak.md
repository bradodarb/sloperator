# sloperator epic-07: no_network_during_check is a weak network test

**Source:** acceptance audit of epic-07-llama-runtime (reviewer)
**Where:** `tests/model_real.rs:251-279`

The test points HTTP(S)_PROXY/ALL_PROXY at a closed port. Only clients that read the proxy variables honour them. A direct socket opened by the runtime or by a future dependency would not fail the test. Matches SPEC §23.19 as written, so non-blocking.

**Suggested fix:** run the check in a network-denied sandbox (macOS `sandbox-exec` deny network / Linux `unshare -n`) or assert no outbound connect via a syscall-level guard.
