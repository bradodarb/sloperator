# Story 01: `doc`: parse, span map, protected strings; `types.rs`

**Epic:** [epic-03-protection-core](../epic-03-protection-core.md)
**SPEC Refs:** §6, §9 (types.rs), §23.3
**PRD Refs:** FR-8.1, FR-8.2, FR-8.3, FR-8.4 (newline detection), FR-4.4, FR-6.7, D-7, D-10
**Component / ADR:** §4.4 doc; ADR-03, ADR-04
**Wave:** 1

## Scope

Implement `src/doc/{mod,markdown,plain,protect}.rs` as §6 specifies: `Format`, `Newline`, `Block`, `Span`, `ProtectSettings`, `Document` and its methods, `BlockText` with its offset map, `parse` and `skeleton`. Add `src/types.rs` with `RuleCode` (including `is_system`), `Proposal` and `ProposalOrigin`. Add the ten `fixtures/protect/*` files. Wire `doc::parse` into the skeleton `check` in `cli/mod.rs`.

## Files to Touch

| File | Line(s) | Change |
|------|---------|--------|
| `src/doc/mod.rs` | new | types, `parse`, `skeleton`, `Document` methods |
| `src/doc/markdown.rs` | new | pulldown-cmark walk → spans |
| `src/doc/plain.rs` | new | paragraphs, directive lines tagged Protected(Directive) |
| `src/doc/protect.rs` | new | user patterns and terms split prose |
| `src/types.rs` | new | RuleCode, Proposal, ProposalOrigin |
| `src/cli/mod.rs` | from epic-01 story-01 | skeleton check calls doc::parse |
| `fixtures/protect/` | new | code, links, front_matter, table, nested_lists, crlf, math, html, refdefs, mixed .md |
| `tests/doc_spans.rs` | new | tests below |

## Acceptance Criteria

- [ ] Code blocks, inline code, link destinations, autolinks, bare URLs, images, ref definitions, front matter, HTML and math are Protected; link text is Prose (FR-8.1).
- [ ] Heading, list, table-cell and quote markup is Protected and their text is Prose (FR-8.2).
- [ ] Configured patterns and terms split prose into Protected runs (FR-8.3).
- [ ] Spans cover every byte of every fixture exactly once (§6 invariant).
- [ ] CRLF input is detected as `Newline::Crlf` (FR-8.4).
- [ ] Every test in Test Cases passes with the Gate command below.

## Test Cases

| # | Test class | Test method | Verifies | Validates |
|---|------------|-------------|----------|-----------|
| 1 | `sloperator::doc_spans` | `code_blocks_and_inline_code_protected` | code is protected | FR-8.1 |
| 2 | `sloperator::doc_spans` | `link_destination_protected_link_text_prose` | link dest protected, text prose | FR-8.1, ADR-04 |
| 3 | `sloperator::doc_spans` | `autolink_bare_url_image_refdef_protected` | URLs and refdefs protected | FR-8.1 |
| 4 | `sloperator::doc_spans` | `front_matter_html_math_protected` | front matter, HTML, math protected | FR-8.1, FR-7.4 |
| 5 | `sloperator::doc_spans` | `heading_list_cell_quote_text_is_prose_markup_protected` | block markup protected | FR-8.2 |
| 6 | `sloperator::doc_spans` | `user_patterns_and_terms_split_prose` | user patterns/terms protected | FR-8.3, FR-7.4 |
| 7 | `sloperator::doc_spans` | `spans_cover_every_byte_without_overlap` | span map invariant | §6 |
| 8 | `sloperator::doc_spans` | `plain_text_paragraphs_and_directive_lines` | plain-text parse | D-10, FR-6.7 |
| 9 | `sloperator::doc_spans` | `crlf_detected` | newline detection | FR-8.4 |

## Notes

`pulldown-cmark` without default features (§2). `protected_strings` is a sorted multiset (§16) and includes continuation prefixes.

Gate: `make test TESTS='sloperator::doc_spans'`
