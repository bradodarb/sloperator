# Model reliance

Model-class findings and fix-less Safe findings remaining after a rules-only fix of
`paragraphs/p01.md`–`p20.md`, per 1,000 prose words in the final text (rounded to two decimals).
Default settings and temp-directory paths keep checkout configuration out. No model or network is used.

Run `make test TESTS='model_reliance'` (workflow contract:
`make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='model_reliance'`).
Use `NEXTEST_SUCCESS_OUTPUT=immediate` before the command to show the printed current value.
It fails if the residual exceeds the baseline. Later phases append their measured values;
the test reads only the baseline row.

| phase | residual per 1,000 prose words | date |
|-------|-------------------------------|------|
| baseline | 19.56 | 2026-11-02 |
| phase 2 | 19.56 | 2026-11-02 |
| phase 4 | 19.56 | 2026-11-02 |
| phase 5 | 19.56 | 2026-11-02 |
| phase 6 | 19.56 | 2026-11-02 |
