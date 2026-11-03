# Story 01: `text::stem`: run the dependency gate and ship one backend

**Epic:** [epic-09-redundancy](../epic-09-redundancy.md)
**SPEC Refs:** §32 (`text::stem`)
**PRD Refs:** FR-9.1, D-9, NFR-5
**Component / ADR:** Architecture §4.6 Stemmer facade; D-9
**Wave:** 1

## Scope

Add `rust-stemmers = "=1.2.0"` to Cargo.toml and run `make -f "$AEGIS_DIR/Makefile" check UNIT=sloperator` (it includes `cargo deny check`; deny.toml:2-14). If it passes, keep body (a) and add the `only_in(&["rust_stemmers"], "src/text/stem.rs")` layering test. Otherwise remove the dependency and keep body (b), the in-tree suffix stripper with the final-`e` floor of 3. Create src/text/stem.rs with exactly one body behind `pub fn stem(word_lower: &str) -> Cow<'_, str>`, add `pub mod stem;` to src/text/mod.rs, and record the gate result and the choice in CHANGELOG.md.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/text/stem.rs` | new | one `stem` body and unit tests |
| `src/text/mod.rs` | module list | `pub mod stem;` |
| `Cargo.toml` | [dependencies] | `rust-stemmers = "=1.2.0"` only if (a) |
| `deny.toml` | 2-14 | only if (a) needs it |
| `tests/layering.rs` | structural tests | `rust_stemmers` only in src/text/stem.rs (only if (a)) |
| `CHANGELOG.md` | 0.2.0 section | the `cargo deny check` result and the backend chosen |

## Acceptance Criteria

- [ ] make/makes/making have one stem, and cache/caches/cached/caching have one stem, under the shipped backend; the assertion is unchanged from §36.10 (FR-9.1, Gap 8)
- [ ] Exactly one `stem` body exists; with (b), Cargo.toml and Cargo.lock are unchanged (D-9)
- [ ] CHANGELOG records the `cargo deny check` outcome and the choice (D-9)
- [ ] If (a), `rust_stemmers` is named only in src/text/stem.rs and pinned exactly (NFR-5)
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::text::stem::tests` | `inflections_meet` | inflection families share a stem under the shipped backend | FR-9.1, D-9 |
| 2 | `sloperator::layering` | `(all, incl. the `rust_stemmers` only_in test if (a))` | dependency confined; text stays pure | NFR-5, NFR-7 |

## Notes

The dependency gate runs through the project contract's `check` verb, not `cargo` directly. Input to `stem` is lower case. The scratch run in §32 checked (b) on both families.

Gate: `make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='text::stem::tests sloperator::layering'`
