# sloperator: cli_exit tests fail when $TMPDIR carries an ignore-all .gitignore

**Source:** epic-08-eval acceptance audit (reviewer), deferred D1. **Severity:** non-blocking (pre-existing, environmental).

- **Symptom:** `make test-integration` fails `standalone#builds_when_extracted`. Inside the extracted copy, 6 `cli_exit` tests (`exit_zero_clean`, `exit_one_on_warning`, `exit_zero_on_suggestion_only`, `exit_two_on_directive_error`, `exit_zero_flag_masks_findings_not_errors`, `min_severity_filters_report_and_exit`) see `sloperator: skipped (excluded): $TMPDIR/.../fixtures/cli/*.md`.
- **Cause:** On the dev host, `$TMPDIR/.gitignore` contains `*`. These tests put their fixtures in a temp dir that has no `!*` re-include, so the CLI's gitignore handling (DQ-13, `src/input/mod.rs:199`) skips them. `common::cli_tempdir` (`tests/common/mod.rs:136-142`) already guards against this for the other CLI tests.
- **Proof it predates epic-08:** a base checkout (62c04bb) shows the same 6 failures. With a clean `TMPDIR` the integration tier passes 7/7.
- **Fix:** have the `cli_exit` fixture dirs use `common::cli_tempdir()` (or write `!*`), so the tests do not depend on the host's temp-dir ignore files.
