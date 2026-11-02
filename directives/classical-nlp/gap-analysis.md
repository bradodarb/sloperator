# Gap Analysis: Classical NLP

**Source:** [PRD](../../docs/classical-nlp/PRD-classical-nlp.md) | [Architecture](../../docs/classical-nlp/ARCHITECTURE-classical-nlp.md) | [SPEC](../../docs/classical-nlp/SPEC-classical-nlp.md)
**Reviewer:** product-reviewer
**Last round:** 1 (2026-11-02)

## Executive Summary

Recommendation: **fix_architecture**. Gap 1 leaves sentence-window planning outside the per-document segmentation cache; the architecture must establish that shared path before the dependent SPEC changes.

12 findings: BLOCKING 0, HIGH 4, MEDIUM 6, LOW 2. Dispositions: FIX-ARCHITECTURE 3, FIX-SPEC 2, DECIDE 3, DEFER 2, ACCEPT 2. Five FIX items remain open, including two MEDIUM architecture items. The most consequential delivery defect is Gap 2: default-on wordiness replacements can silently turn grammatical input into broken prose, and neither the verifier nor the planned tests would reject it. Gap 3 similarly loses an assertion through a Safe template. Three decisions need Brad; none is treated as an author fix.

The whole PRD, SPEC, architecture and nine initiative ADRs were read. No UX artifact is present; the architecture explicitly assigns CLI/JSON/MCP surfaces to FR-3. The requested context map and conventions file are absent (Gap 10); the root README, DEVELOPMENT, architecture, actual code and layering tests supply the available conventions.

## Triage

| Gap | Severity | Origin | Disposition | What must change / rationale |
|-----|----------|--------|-------------|------------------------------|
| 1 | HIGH | first-draft | FIX-ARCHITECTURE | Sentence windows must consume the existing per-pass segmentation rather than rescan each selecting finding. |
| 2 | HIGH | first-draft | FIX-SPEC | Guard or omit context-dependent wordiness replacements; preserve grammatical noun compounds and agreement. |
| 3 | HIGH | first-draft | FIX-SPEC | SLP003 must decline additive “not only/just” constructions rather than delete their first assertion. |
| 4 | MEDIUM | decision | DECIDE | Confirm ADR-08's placement/default trade-off (SPEC OD-1). |
| 5 | MEDIUM | decision | DECIDE | Configurable report minimums versus fixed report floors with rule-only params. |
| 6 | MEDIUM | drift | FIX-ARCHITECTURE | Absorb the SPEC's ten explicit architecture deviations into their owning sections/ADRs. |
| 7 | MEDIUM | first-draft | FIX-ARCHITECTURE | Remove the automatic permission to drop SLP006 widening on spike failure. |
| 8 | MEDIUM | first-draft | DEFER | Phase 6 stemmer story must make the named inflection test pass for the fallback backend. |
| 9 | MEDIUM | first-draft | DEFER | Phase 4/5 pipeline story must retain a behavioral multi-window cache regression test. |
| 10 | LOW | first-draft | ACCEPT | Missing context/conventions documents limit the conventions audit; available repo rules were reviewed. |
| 11 | HIGH | decision | DECIDE | Global common-capitalised-word exemption weakens FR-6.2 for names such as the title “It”. |
| 12 | LOW | first-draft | ACCEPT | Explicit sufficient-statistics, contraction and static purity deviations are adequate with the stated limits. |

## Round 1

### Gap 1: Sentence-window planning rescans an already segmented document

**Severity:** HIGH  
**Origin:** first-draft  
**Pass:** alignment, literal-vs-actual, feasibility  
**PRD Ref:** FR-1.4, NFR-2  
**Architecture Ref:** ADR-02; §4.1, §4.8, §4.11  
**SPEC Ref:** §4, §5, §9, §11, §27  
**Evidence:** `src/engine/window.rs:81-101`, `:237-263`; `src/pipeline/mod.rs:230`; SPEC §5's complete replacement body.

**Trigger:** A sentence-mode model pass has several model-eligible findings in one paragraph. The detection pass has already constructed `Prose` and segmented it. Each selecting finding then calls `sentence_range` before window deduplication.

**Expected:** NFR-2 says that the model path does not recompute segmentation per window. ADR-02's per-pass context is the shared owner.

**Actual:** SPEC §5 calls `segment::blanked` and `segment::scan_starts` for each invocation. It does not read the existing `Segments`. Architecture §4.11 retains the window API without the per-pass context, and the overview explicitly allows windows to call the boundary function directly. Moving the implementation into a common module gives a common algorithm, but does not share its computed result.

**Impact:** The same paragraph is allocated and scanned once per selecting finding, even when those findings ultimately deduplicate into one window. This violates the model-path performance contract. The planned tests count tagging, not boundary scans; the rules-only perf test never plans model windows. Existing window range tests would pass with the repeated work.

**Disposition:** FIX-ARCHITECTURE — establish how window planning consumes the document pass's cached sentence bounds and the blanked/trimming information it needs. Update ADR-02/03 and the window/pipeline interface, then revise SPEC §5 and call sites. Add a model-path regression that counts actual segmentation work for multiple findings/windows in one document pass. No stronger “one pass across all edited candidates” requirement is imposed: each newly parsed candidate may have its own analysis.

**Confidence:** 99%.

### Gap 2: The nominalisation rows are not context-independent Safe replacements

**Severity:** HIGH  
**Origin:** first-draft  
**Pass:** literal-vs-actual, edge cases  
**PRD Ref:** D-10, FR-5.1–FR-5.3  
**Architecture Ref:** §4.9 SLP001, ADR-08  
**SPEC Ref:** §20, §36.5  
**Evidence:** `src/rules/helpers.rs:35-65`, `:71-75`; `src/rules/builtin/slp001_ai_vocabulary_replaceable.rs:62-79`; `src/pipeline/mod.rs:113-125`; `src/verify/mod.rs:91-143`.

**Trigger and result:** “We make a decision tree.” becomes “We decide tree.” The same applies to “They made a decision table.” The row `a sufficient number of → enough` also turns “There is a sufficient number of seats.” into “There is enough seats.”

**Expected:** A Safe replacement remains grammatical and claim-preserving. An uncertain replacement has no fix or is omitted.

**Actual:** §20 appends unconditional `Some` rows to the unchanged `WordList`. Its whole-word boundary only excludes adjoining alphanumeric/underscore characters; a space before “tree” passes. It has no phrase-context or agreement guard. §20 requires a grammatical positive sentence per row but no counter-context for these rows.

**Impact:** Default-on rules-only fixes damage valid input. No number, modal, proper noun, protected byte or additional negation is involved. The verifier therefore has no grammar check to catch the damage, and idempotency still passes. The Phase 2 gate only checks the chosen grammatical fixture outputs; it does not force these counterexamples into view.

**Disposition:** FIX-SPEC — specify the conservative context conditions for every context-dependent row, or omit rows that cannot be proven with the mechanisms available in Phase 2. Do not solve this by unconditionally adding fix-less rows that violate the model-reliance contract. Add unchanged/no-Safe-fix regressions for noun compounds and agreement, including the concrete inputs above. This fix is needed under either placement option in Gap 4.

**Confidence:** 99%.

### Gap 3: SLP003 deletes an asserted property from “not only” constructions

**Severity:** HIGH  
**Origin:** first-draft  
**Pass:** edge cases, literal-vs-actual  
**PRD Ref:** D-10, FR-8.2, FR-8.3, FR-8.5  
**Architecture Ref:** §4.9 SLP003  
**SPEC Ref:** §30, §36.9  
**Evidence:** `src/rules/builtin/slp003_not_x_but_y.rs:8-11`; `src/rules/helpers.rs:159-175`; `src/pipeline/mod.rs:113-125`; SPEC §30's TEMPLATE and guards.

**Trigger:** “It's not only cheap, it's quick.” The current detector matches this construction. The proposed template captures X = “only cheap” (two lowercase words), the subjects match, Y = “quick” (one word), and both subjects lie in one sentence.

**Actual:** §30 deletes the first clause and emits “It's quick.” The input asserts cheapness as well as quickness; the output loses cheapness. The same flaw applies to “not just cheap”.

**Impact:** The Safe path loses an assertion. The D-8 allowance accepts the dropped “not”; there are no lost names, modals, numbers or protected spans. Neither verification, idempotency, nor the listed decline tests catches it. SLP004 does not rescue this example: its detector requires “but”.

**Disposition:** FIX-SPEC — explicitly distinguish a denied X from the additive “not only/just X” frame, and decline the latter in SLP003. Add a no-Safe-fix/unchanged test for both constructions, plus the existing ordinary-denial positive. Audit any other words admitted at the start of X under the same claim-preservation requirement.

**Confidence:** 99%.

### Gap 4: ADR-08 remains a product decision

**Severity:** MEDIUM  
**Origin:** decision  
**Pass:** alignment, scope  
**PRD Ref:** FR-5, FR-11.2; PRD §9 Q3  
**Architecture Ref:** ADR-08 (Proposed), §13  
**SPEC Ref:** §20, §21, Open Decisions OD-1  
**Evidence:** `src/rules/builtin/slp001_ai_vocabulary_replaceable.rs:9-27`, `:37-48`; `src/engine/window.rs:110-119`.

**Question:** Should the Safe wordiness rows extend default-on SLP001, or live in a new default-off wordiness rule?

- **(a)** Extend SLP001; put detection-only intensifiers in separate default-off SLP022. Default users gain fixes and new findings/scores; SLP001's AI-vocabulary name/family broaden.
- **(b)** Put Safe wordiness in a new default-off rule and intensifiers in a separate detection-only rule. Existing SLP001/default scores remain; users must enable the new fixes, and the subsequent codes/catalogue change.

**Recommendation:** (a), as SPEC OD-1 recommends: the PRD explicitly makes added table rows inherit their host rule's default and identifies SLP001 as the existing phrase table. Keep intensifiers outside a Safe rule so they cannot select model windows. OD-1's option (c) is excluded from the viable choices because it contradicts the detection-only requirement.

**Disposition:** DECIDE — Brad, not the author. This is the existing open OD-1; do not treat the draft's recommended option as approval. Gap 2 is independent of the placement decision.

### Gap 5: Fixed report floors drop the configurable-minimum contract

**Severity:** MEDIUM  
**Origin:** decision  
**Pass:** alignment, literal-vs-actual  
**PRD Ref:** D-5, FR-2.7, FR-4.2  
**Architecture Ref:** ADR-09, §4.4, PRD Deviation Note 3  
**SPEC Ref:** §8, §14, §16, PRD Deviation Notes

**Trigger:** A user wants raw reports to become unavailable below 10 sentences or 200 words. Rule minimum params can be raised to those values, but the raw report still uses constants 5/100, even for the same document.

**Expected:** D-5 says the provisional sentence/word minimums are configurable as params; FR-2.7 says availability uses configured minimums.

**Actual:** `Counts::derive` accepts no settings and uses fixed constants. Rule params control findings only. The deviation is explicit, but it changes user-visible configurability rather than providing an equivalent mechanism.

**Question:** Must report availability honor configurable minimums, or may this release fix raw-report floors at 5/100 and configure only rule firing?

- **(a)** Preserve configurable report minimums. Architecture/SPEC must define one unambiguous setting source for always-on raw metrics, including when all metric rules are off, and apply it consistently to reports and finding availability.
- **(b)** Approve fixed report floors and rule-only minimum params. The implementation remains simpler, but raw-report availability cannot be tuned and the PRD must record the reduced contract.

**Recommendation:** (a), because configurability is an explicit requirement and metrics are the priority. The author can design the setting mechanism after Brad confirms the contract.

**Disposition:** DECIDE — no dependent blocking FIX is asserted before that answer.

### Gap 6: Ten explicit SPEC deviations still need architecture absorption

**Severity:** MEDIUM  
**Origin:** drift  
**Pass:** alignment  
**PRD Ref:** NFR-7, NFR-9; design alignment contract  
**Architecture Ref:** §4.1, §4.2, §4.5, §4.8, §4.10–4.12, §11; ADR-03, ADR-06  
**SPEC Ref:** Architecture Deviation Notes (ten bullets)

**Trigger:** A story follows the architecture's verifier boundary mechanism, tag fields, tests, output layout or window-code description while another follows the SPEC.

**Actual:** The SPEC itself identifies changes that must be absorbed: blanked/scan helpers; `BlockText::to_text` plus `starts_sentence`; lexicon/common-word helpers; `Tagged.unknown`; revised cache-test form; `Detected` alias; duplicate-preserving `Window.codes`; overall metrics on a separate line; version bumps per unreleased release; standalone-safe test command wording. Architecture §4.10 still names `sentence_starts` for the first-token check, and §11 still describes a behavioral multi-window test that the SPEC replaces.

**Impact:** Two design sources give different instructions. These differences are explicit and mostly build-safe, so they do not justify holding the loop individually. Gap 1 and Gap 11 separately review the substantive issues rather than accepting them merely because a deviation note exists.

**Disposition:** FIX-ARCHITECTURE — absorb each justified change into its owning architecture section and ADR where applicable. Reconcile the test description with the Phase 4 deferred requirement (Gap 9). Do not resolve the open product decisions while editing drift.

### Gap 7: The Phase 4 contingency silently drops a required feature

**Severity:** MEDIUM  
**Origin:** first-draft  
**Pass:** alignment, lifecycle/failure  
**PRD Ref:** FR-7.5–FR-7.8, D-1  
**Architecture Ref:** §12 Phase 4 spike gate  
**SPEC Ref:** §28 final paragraph

**Trigger:** The Phase 4 spike cannot prove one new SLP006 subject class safe.

**Actual:** Both documents say to drop widening and ship the tagger for Phase 5 only. The PRD requires widened SLP006 fixes; no recorded decision authorizes abandoning them.

**Impact:** The documented fallback turns a failed feature gate into permission to ship less scope. This is a contingency, and the spike exposes the failure before delivery, so it is a non-blocking correction to the gate policy.

**Disposition:** FIX-ARCHITECTURE — make spike failure fail loudly and require correction or an explicit Brad scope decision; do not authorize automatic feature removal. Carry the same policy into SPEC §28. No current ruling is required to preserve the existing PRD.

### Gap 8: The fallback stemmer fails its own required inflection test

**Severity:** MEDIUM  
**Origin:** first-draft  
**Pass:** feasibility  
**PRD Ref:** FR-9.1, D-9  
**Architecture Ref:** §4.6, Phase 6 gate  
**SPEC Ref:** §32 fallback body, §36.10 `inflections_meet`

**Trigger:** The dependency gate chooses the in-tree fallback.

**Actual:** With the displayed suffix/length rules, “make” stays “make” (dropping e would leave three letters), “makes” becomes “make”, but “making” becomes “mak”. Likewise “caches” becomes “cach” while “cached”/“caching” become “cach”. The make family already disproves the specified equality test.

**Impact:** The fallback cannot meet the SPEC's own stem equivalence example. This is not blocking now: §36.10 explicitly requires a test for make/makes/making, so the Phase 6 story will catch it.

**Disposition:** DEFER — recorded in `deferred.md`; Phase 6 stemmer story must repair the fallback or select a passing backend and run the same inflection tests on the chosen implementation.

### Gap 9: The replacement cache proof does not exercise a multi-window fix

**Severity:** MEDIUM  
**Origin:** first-draft  
**Pass:** test coverage  
**PRD Ref:** NFR-2, FR-7.3  
**Architecture Ref:** ADR-02, §11 Pipeline row  
**SPEC Ref:** §6, §27, §36.8; NFR-2 Deviation Note  
**Evidence:** `src/pipeline/mod.rs:230-256`, `:280-283`; `tests/common/mod.rs:65-108`; SPEC §6's allowed-files scan.

**Trigger:** A future pipeline change creates more than one `Prose` for the same document pass. `Prose::new(` remains in the allowed pipeline file, so the structural test passes; the isolated `detect` unit test also passes.

**Actual:** The tests prove OnceLock reuse for one supplied `Prose` and constrain call-site files. They do not count context construction or tagging through the multi-window pipeline promised by NFR-2. A per-context counter being test-only does not prevent an in-crate pipeline test from exercising that flow.

**Disposition:** DEFER — `deferred.md` assigns the Phase 4/5 pipeline story a behavioral test using the existing fake model seam, with document passes distinguished from successive candidate documents. It must fail on duplicate computation within a pass. This is separate from Gap 1's actual repeated window segmentation.

### Gap 10: The requested conventions sources are absent

**Severity:** LOW  
**Origin:** first-draft  
**Pass:** conventions  
**PRD Ref:** NFR-7, NFR-9; review conventions input  
**SPEC Ref:** §6, §15, §23, §31

Reads of `docs/context/README.md` and `docs/context/conventions.md` returned missing-file errors. The docs tree has no context directory; the root README points to DEVELOPMENT, ARCHITECTURE, SPEC and ADRs instead.

**Disposition:** ACCEPT — this does not prevent review of the design. The available sources establish one crate, pure lower layers, snake_case rule params, camelCase JSON, exact dependency pins, one verify choke point and clean call-site updates. No claim is made that the absent convention pack was checked, and no implementation story is asked to create it.

### Gap 11: The global common-word exemption permits deletion of capitalised names

**Severity:** HIGH  
**Origin:** decision  
**Pass:** literal-vs-actual, edge cases  
**PRD Ref:** FR-6.2, FR-6.6, NFR-1  
**Architecture Ref:** §4.10, ADR-06  
**SPEC Ref:** §22, §23, DQ-10, FR-6.2 Deviation Note  
**Evidence:** `tests/rules_fixtures.rs:271-277` requires the existing title-case hedge fix; `src/verify/mod.rs:91-143` shows the remaining token/structure checks.

**Trigger:** A model changes “We watched It and a robust sequel.” (the title/name) to “We watched something and a sequel.” SLP002 selects the window for “robust” and removing it supplies the required improvement (`src/rules/builtin/slp002_ai_vocabulary.rs:6-13`, `:27-29`; `src/pipeline/mod.rs:280-291`). The capitalised token is non-initial. §22 case-folds it to the stop word “it”, and §23 excludes it from the proper-noun multiset. No modal, negation, number, URL, configured term or block changes, so the proposed new checks permit the deletion.

**Expected:** FR-6.2 rejects removal/change of any capitalised non-sentence-initial token. It specifies a sentence-initial exemption, not a global common-word exemption.

**Actual:** To preserve “It May Potentially fail.” → “It May fail.”, the SPEC exempts every stop word, modal and hedge in every origin/context. Compatibility requires handling that existing Safe fixture, but does not by itself require allowing model edits to remove all capitalised common-word names.

**Question:** Should the verifier preserve the literal capitalised-token proxy with narrow handling for existing proven Safe fixes, or approve a global common-word exemption?

- **(a)** Preserve the proxy for model edits and constrain compatibility exceptions to the proven Safe transformations that need them. Existing fixtures continue to pass, while non-initial capitalised common-word names remain guarded.
- **(b)** Approve the proposed global common-word exemption for every origin. Implementation stays as written, but names/titles made from common words may disappear and FR-6.2 must explicitly narrow its guarantee.

**Recommendation:** (a), because the verifier's purpose here is to catch lost names, and preserving one hedge regression does not justify a global weakening. Add a test for the concrete title example and a passing test for the required hedge fixture.

**Disposition:** DECIDE — this is a changed public verification guarantee, not authority for the author to silently redefine FR-6.2. The requirement at stake is explicit; no settled Brad decision about this exemption is recorded.

### Gap 12: Literal count/storage and test-form differences are acceptable where equivalent

**Severity:** LOW  
**Origin:** first-draft  
**Pass:** coverage, literal-vs-actual  
**PRD Ref:** FR-2.1, D-6, FR-6.3, NFR-5  
**Architecture Ref:** ADR-04, ADR-07  
**SPEC Ref:** §8, §22–23, PRD Deviation Notes

FR-2.1 asks for sentence-length lists; the SPEC stores n, Σx and Σx² and retains each sentence's word range in Segments. Overall distinct types use set union, which is necessary to avoid counting the same type twice. Modal contractions are normalised to their modal identity rather than compared as arbitrary surface strings. The no-network verification uses existing purity/network dependency scans instead of the PRD's nonexistent generic runtime test.

**Disposition:** ACCEPT — these are explicit, reasoned deviations. Sufficient statistics produce the requested population metrics; union gives the correct overall TTR. Contraction handling preserves modal identity, with ambiguous 'd rejected loudly. Static scans plus unchanged I/O boundaries are adequate for this pure feature; they are not represented as packet-level runtime network tests. The footprint size recording remains a Phase 1c/6 deliverable. This acceptance does not cover the distinct report-floor or name-proxy changes in Gaps 5/11.


### Coverage

Status assesses the written mechanism, not implemented delivery. COMPLETE means a concrete planned mechanism and test are specified; the feature code and new tests do not yet exist. INDIRECT rows have explicit deviation notes and are adjudicated below. PARTIAL rows identify a substantive gap or a deferred acceptance requirement.

| Requirement | Status | Architecture owner | SPEC mechanism / finding |
|-------------|--------|--------------------|--------------------------|
| FR-1.1 | COMPLETE | §4.1 / §4.11 | §4 |
| FR-1.2 | COMPLETE | §4.1 / §4.11 | §4 |
| FR-1.3 | COMPLETE | §4.1 / §4.11 | §4; §36.1 |
| FR-1.4 | PARTIAL | §4.1 / §4.11 | §3–5; Gap 1 (same definition, repeated computation) |
| FR-1.5 | COMPLETE | §4.1 / §4.11 | §4; §9 |
| FR-2.1 | INDIRECT | §4.3 / §4.4 | §8; Gap 12 (equivalent statistics) |
| FR-2.2 | COMPLETE | §4.3 / §4.4 | §8 |
| FR-2.3 | COMPLETE | §4.3 / §4.4 | §7–8 |
| FR-2.4 | COMPLETE | §4.3 / §4.4 | §8 |
| FR-2.5 | COMPLETE | §4.3 / §4.4 | §8 |
| FR-2.6 | COMPLETE | §4.3 / §4.4 | §7; §14 |
| FR-2.7 | PARTIAL | §4.3 / §4.4 | §8; §16; Gap 5 (fixed instead of configurable report floors) |
| FR-2.8 | COMPLETE | §4.3 / §4.4 | §8; §12 |
| FR-2.9 | COMPLETE | §4.3 / §4.4 | §8; §36.2 |
| FR-2.10 | COMPLETE | §4.3 / §4.4 | §4; §8 |
| FR-3.1 | COMPLETE | §4.11 / §4.12 | §12 |
| FR-3.2 | COMPLETE | §4.11 / §4.12 | §12 |
| FR-3.3 | COMPLETE | §4.11 / §4.12 | §13 |
| FR-3.4 | COMPLETE | §4.11 / §4.12 | §11–13 |
| FR-3.5 | COMPLETE | §4.11 / §4.12 | §12 |
| FR-3.6 | COMPLETE | §4.11 / §4.12 | §14 |
| FR-4.1 | COMPLETE | §4.9 / §4.13 | §16–17 |
| FR-4.2 | COMPLETE | §4.9 / §4.13 | §15–17 |
| FR-4.3 | COMPLETE | §4.9 / §4.13 | §16 |
| FR-4.4 | COMPLETE | §4.9 / §4.13 | §17 |
| FR-4.5 | COMPLETE | §4.9 / §4.13 | §17 |
| FR-4.6 | COMPLETE | §4.9 / §4.13 | §17; §36.4 |
| FR-4.7 | COMPLETE | §4.9 / §4.13 | §17; §36.4 |
| FR-5.1 | PARTIAL | §4.9 SLP001/022 | §20; Gap 2 |
| FR-5.2 | PARTIAL | §4.9 SLP001/022 | §20; §36.5; Gap 2 (positive grammar alone does not prove row safety) |
| FR-5.3 | PARTIAL | §4.9 SLP001/022 | §20; Gap 2 |
| FR-5.4 | COMPLETE | §4.9 SLP001/022 | §21 |
| FR-5.5 | COMPLETE | §4.9 SLP001/022 | §20; §36.5 |
| FR-6.1 | INDIRECT | §4.2 / §4.10 / §4.11 | §22–24; Gap 12; D-8/ADR-06 specify count semantics |
| FR-6.2 | PARTIAL | §4.2 / §4.10 / §4.11 | §3; §22–23; Gap 11 |
| FR-6.3 | INDIRECT | §4.2 / §4.10 / §4.11 | §22–23; Gap 12 (modal contraction identity) |
| FR-6.4 | COMPLETE | §4.2 / §4.10 / §4.11 | §23–24 |
| FR-6.5 | COMPLETE | §4.2 / §4.10 / §4.11 | §24; §36.6 |
| FR-6.6 | COMPLETE | §4.2 / §4.10 / §4.11 | §36.6 |
| FR-6.7 | COMPLETE | §4.2 / §4.10 / §4.11 | §25 |
| FR-6.8 | COMPLETE | §4.2 / §4.10 / §4.11 | §36.6 |
| FR-7.1 | COMPLETE | §4.5 / §4.8 / §4.9 | §26 |
| FR-7.2 | COMPLETE | §4.5 / §4.8 / §4.9 | §26 |
| FR-7.3 | PARTIAL | §4.5 / §4.8 / §4.9 | §10; §27; Gap 9 (behavioral pipeline proof deferred) |
| FR-7.4 | COMPLETE | §4.5 / §4.8 / §4.9 | §27–28 |
| FR-7.5 | PARTIAL | §4.5 / §4.8 / §4.9 | §28; Gap 7 (automatic drop contingency) |
| FR-7.6 | COMPLETE | §4.5 / §4.8 / §4.9 | §28 |
| FR-7.7 | COMPLETE | §4.5 / §4.8 / §4.9 | §28; §36.8 |
| FR-7.8 | PARTIAL | §4.5 / §4.8 / §4.9 | §28; §36.8; Gap 7 (automatic drop contingency) |
| FR-8.1 | COMPLETE | §4.9 / §4.10 | §29 |
| FR-8.2 | PARTIAL | §4.9 / §4.10 | §30; Gap 3 |
| FR-8.3 | PARTIAL | §4.9 / §4.10 | §29–30; Gap 3 (missing additive-frame guard) |
| FR-8.4 | COMPLETE | §4.9 / §4.10 | §29–31 |
| FR-8.5 | COMPLETE | §4.9 / §4.10 | §23; §29–30 |
| FR-8.6 | COMPLETE | §4.9 / §4.10 | §31 |
| FR-8.7 | COMPLETE | §4.9 / §4.10 | §29–30; §36.9 |
| FR-9.1 | PARTIAL | §4.6 / §4.7 / §4.9 | §32–34; Gap 8 (fallback; named later test catches it) |
| FR-9.2 | COMPLETE | §4.6 / §4.7 / §4.9 | §34 |
| FR-9.3 | COMPLETE | §4.6 / §4.7 / §4.9 | §34 |
| FR-9.4 | COMPLETE | §4.6 / §4.7 / §4.9 | §34 |
| FR-9.5 | COMPLETE | §4.6 / §4.7 / §4.9 | §33–35 |
| FR-9.6 | COMPLETE | §4.6 / §4.7 / §4.9 | §34 |
| FR-10.1 | COMPLETE | §4.14 | §2 |
| FR-10.2 | COMPLETE | §4.14 | §2 |
| FR-10.3 | COMPLETE | §4.14 | §2; §36.11 |
| FR-11.1 | COMPLETE | §4.9 / §12 | §18 |
| FR-11.2 | PARTIAL | §4.9 / §12 | §17–21; §34; Gap 4 (placement awaiting Brad) |
| FR-11.3 | COMPLETE | §4.9 / §12 | §15; §18 |
| NFR-1 | PARTIAL | §1 / §12 | §1; §12; §28; §31; §36; Gap 11 (compatibility exception policy) |
| NFR-2 | PARTIAL | §4.8 / §4.11 / §8 | §5–6; §9–11; §27; §35; Gaps 1, 9 |
| NFR-3 | COMPLETE | §4.4 / §6 | §8; §12; §36.2 |
| NFR-4 | COMPLETE | §4.13 / ADR-07 | §7; §15; §22; §26; §36.4 |
| NFR-5 | INDIRECT | §8 / §9 | §6; §18; §32; Gap 12 (static I/O/network checks; size recorded at delivery) |
| NFR-6 | COMPLETE | §4.10 / §7.2 | §23–24; §36.6–36.7; all new proposals use the existing verify path |
| NFR-7 | COMPLETE | §5 / ADR-01 | §4; §6; layering scans |
| NFR-8 | COMPLETE | §4.8 / §8 | §9; §27; OnceLock and compile-time Send + Sync assertion |
| NFR-9 | COMPLETE | §8 persistence | §3; §5; §10–13; §15; §23–25; §31; call-site search checked |
| NFR-10 | PARTIAL | §11 | §36; Gaps 2, 3, 8, 9 (missing counter-contexts / deferred tests) |

All 68 FR subrequirements and all 10 NFRs are listed. PRD Phase 1 calibration follow-up (D-4 / §7 step 6) is additionally mapped to SPEC §19: 1–3 MB, at least 12 public-domain-source files, source/licence manifest and recorded per-file/overall metrics. Actual provenance is a delivery gate, not something this design review can attest.

### Architecture and ADR alignment

| Component | SPEC delivery | Result |
|-----------|---------------|--------|
| §4.1 Segmenter | §3–5 | Covered; cache consumption missing (Gap 1); helper drift (Gap 6) |
| §4.2 Closed-class lexicon | §22 | Covered; name exemption is Gap 11; drift is Gap 6 |
| §4.3 Syllables | §7 | Covered, approximate algorithm and fixed expected list |
| §4.4 Metrics | §8; §14 | Covered; statistics deviation accepted (Gap 12), floors decision (Gap 5) |
| §4.5 Tagger | §26 | Covered; owned tables, ordered rules, unknown/certain flags |
| §4.6 Stemmer | §32 | Covered; fallback regression deferred to named test (Gap 8) |
| §4.7 Similarity | §33 | Covered; weighted Jaccard over stemmed term counts |
| §4.8 Per-document Prose | §6; §9; §27 | Covered for rules/metrics; windows bypass reuse (Gap 1), pipeline proof Gap 9 |
| §4.9 Rule integration | §10; §16–21; §28–31; §34 | All new/changed rules represented; Gaps 2–4, 7 |
| §4.10 Verifier | §3; §23; §25 | One choke point; Gap 11 decision; common helpers drift Gap 6 |
| §4.11 Pipeline/windows | §5; §11; §24 | Counts follow surviving final text; Gap 1 |
| §4.12 Reports/MCP | §12–14 | All PRD surfaces mapped; score object/formula unchanged |
| §4.13 Config bounds | §15 | Load-time bounds, unknown names, NaN rejection, named rule/param errors |
| §4.14 Reliance harness | §2 | Baseline before rule changes; per-phase records; residual <= baseline |
| §12 Calibration/delivery order | §1–35 in phase order; §19 corpus | Covered; conditional scope removal Gap 7 |
| ADR-01 | §6 | Honored; pure text layer, forbidden edges and I/O scan |
| ADR-02 | §9–11; §27 | Honored for rules; shared window-analysis boundary needs revision (Gap 1) |
| ADR-03 | §3–5; §23 | Boundary algorithm moved, old definition deleted; declared helper deviation (Gap 6) |
| ADR-04 | §8 | Honored; sufficient statistics plus type-set union |
| ADR-05 | §12–13 | Honored; additive metrics, unchanged Score and schema version |
| ADR-06 | §23–24 | Counted allowance, one entry per overlapping finding; declared duplicate policy (Gap 6) |
| ADR-07 | §7; §22; §26 | Honored; compile-time Rust tables, invariant tests |
| ADR-08 | §20–21; OD-1 | Proposed; needs Gap 4 decision |
| ADR-09 | §15–16 | Honored for rule bounds; report-floor policy needs Gap 5 decision |

The architecture's module directions are maintained by the SPEC: text uses doc/std plus existing Unicode utilities and optional Phase 6 stemmer; rules do not import verify; verify reads leaf RuleCode constants, not registry metadata. No new edit applier or frontend-only safety gate is introduced.

### Scope creep

| SPEC concept | PRD relationship | Verdict |
|--------------|------------------|---------|
| Prose, Segments, Counts/Metrics, owned tables | FR-1/2/7, NFR-2/7/8 | Justified implementation mechanisms |
| ParamSpec bounds and CLI/MCP explain bounds | FR-4.2, NFR-4, FR-11.3 | Justified single metadata source |
| SLP022 separate intensifier rule | FR-5.4, default-off FR-11.2 | Placement/default approval remains Gap 4 |
| Window.codes and resolves parameter | D-8 explicitly delegates model-origin context to SPEC | Justified; no new model or paid service |
| Calibration corpus guard/manifest | PRD Phase 1 step 6 | In scope; actual licence provenance remains a gate |
| Standalone document wording cleanup | Existing standalone test plus NFR-1 | Necessary gate hygiene; does not authorize arbitrary PRD changes |
| Global common-capitalised-word exemption | Extends FR-6.2's stated exemptions | Not approved merely by a deviation note: Gap 11 |
| Fixed raw-report floors | Narrows D-5 / FR-2.7 | Gap 5 |
| Automatic removal of SLP006 widening | Drops FR-7.5–7.8 | Gap 7 |
| New score formula, AI provenance claim, passive rewrite, summarizer | Not specified by this SPEC | Correctly absent |

### Edge cases walked

| Group / interface | Boundaries checked | Result / unresolved |
|-------------------|--------------------|---------------------|
| Segmenter/BlockText | Empty blocks, protected-only sentences, code punctuation, multiline container prefixes, UTF-8/source ranges, block-less prose, unknown block ids | Explicit handling and equality/property tests; Gap 1 for repeated window work |
| Counts/Metrics | Empty/short files, floor equality, zero denominators, repeated types across files, excluded headings/lists/tables, fixed float rounding, unavailable values | Mechanisms specified; Gap 5 for configured availability |
| Reports/FileResult | Lint versus fix/diff, unchanged fix, whole-document rollback, empty aggregate, null metrics, concise/github, MCP | Before counts come from original and final counts from surviving document; fixtures planned |
| Rule params | Unknown names, negative/zero minimums, upper bounds, NaN/infinities, existing literal updates | Load-time checks reject invalid values and identify rule/key; no runtime silent fallback |
| Phrase tables | Explicit inflections, casing, nominal compounds, syntactic agreement, protected runs, second application | Compound/agreement contexts fail safety: Gap 2 |
| Verifier/Meaning | Rule/model origins, retained/dropped negations, contractions, names mid-sentence, initials, title-case hedge compatibility, repeated exempt codes, SLP901 | Name whitelist weakens proxy: Gap 11. Counted D-8 allowance can still drop an unrelated negation; architecture §14 explicitly identifies that limitation, so this review does not claim positional polarity guarantees |
| Tagger/Prose | No candidate, known/unknown/ambiguous words, ordered rules, repeated consumers, concurrent access, no mutable globals, pass lifetime | OnceLock and flags specified; Gap 9 for actual pipeline test; Gap 7 for failure policy |
| SLP006 | Det N / Det Adj N, ambiguous tool/fast, unknown subject, noninitial subject, idioms, following verbs, old fixtures | Conservative branches specified; required widening cannot silently be dropped (Gap 7) |
| SLP003/004 | Same/different subjects, one/two sentences, word bounds, Y continuation, protected content, additive not-only/just frames, capitalization, repeated application | Gap 3; ordinary template positives/declines are otherwise specified |
| Similarity/stemmer | Empty bags, stop words, repeated terms, later-sentence anchoring, min sentences/content words, paragraph boundaries, backend selection failure | Detection only/default-off; Gap 8 for fallback correctness |
| Reliance/calibration | Before-change baseline, temp-path config isolation, phase recording, final regression, corpus manifest/size/length | Specified; no new runtime external data reads |
| Lifecycle/concurrency | New analysis after edits, rollback retains original counts, parallel files, serial model, no persistent state | Document-local caches and drop lifetimes explicit; no startup/shutdown resource mechanism needed for pure functions |

### Tests and real-code feasibility

The existing APIs and cited integration seams were opened: Document/BlockText/prose_words, window planning and prefix blanking, RuleCtx/detect/ParamSpec and every existing ParamSpec owner, WordList/PatternRule and lead-in handling, pipeline analyse/lint/fix/accept and check mapping, verify/token checks, score, CLI explain and rayon path, report text/JSON/Summary, MCP result/explain paths, config parsing/error helper, registry/defaults, test fixture macro, edit/verify call sites, fake rewriter, perf/layering/standalone tests and root Makefile. The call-site search found the expected signature/literal users. No missing existing API prevents the general design; prospective additions remain unimplemented.

| Requirement group | Planned test file / named test | Review status |
|-------------------|-------------------------------|---------------|
| FR-1 | text_segment; segment unit tests; sentence_starts_match_window_sentences | Concrete outputs and fixture/property checks; add cache-work regression via Gap 1 |
| FR-2 | metrics::hand_computed_values, below_floor_is_unavailable, merge_unions_types, paragraphs_only; syllable::fixed_list | Concrete expected values; configurable-floor decision Gap 5 |
| FR-3 | report text/json tests; text_metrics::fix_reports_before_and_after; MCP lint/fix metrics | Concrete key/line assertions, before/after mechanism; existing output tests identified |
| FR-4 | rules_fixtures SLP018–021; config bound tests; metric_rule_silent_when_unavailable; metric_finding_never_selects_window; enabling_metric_rule_adds_suggestion_weight | Concrete findings/class/score assertions |
| FR-5 | slp001_fix_matches; slp001_every_row_in_positive; SLP022 fixtures; idempotency | Planned positives are insufficient to catch Gap 2's negative contexts |
| FR-6 | verify_edits rejection/acceptance per Check; no_safe_fixture_fix_newly_rejected; pipeline rejection-name test | Concrete examples; Gap 11 needs policy-specific negative plus compatibility acceptance |
| FR-7 | tagger known/unknown/ambiguous tests; tagging_runs_once_per_pass; no_candidate_no_tagging; grammatical contexts | Concrete unit fixtures; multi-window pipeline proof deferred in Gap 9 |
| FR-8 | SLP003/004 fixed.md; continuation/sentence/subject decline tests; mid-sentence case; SLP007/015 unchanged | Missing additive-frame decline: Gap 3 |
| FR-9 | stem::inflections_meet; Jaccard hand values; stop-word/minimum/block-boundary tests; perf | Fallback fails the explicit make-family expectation: Gap 8, owned by Phase 6 |
| FR-10 | model_reliance::residual_does_not_exceed_baseline | No model; records default-setting residuals per phase; baseline population is a delivery step |
| FR-11 | every_rule_has_fixtures; new_rules_start_off; catalogue/explain updates | Existing default-set/version mechanisms verified in source; placement decision Gap 4 |
| NFR-1/9 | Existing touched-module tests, Score JSON, reference_states_limits, class changes | Authorized report/doc/class assertion updates identified; no shims specified |
| NFR-2 | rules_only_100kb_under_1s; cache tests | Rules-only perf never covers window planning (Gap 1); actual multi-window tagging test Gap 9 |
| NFR-3 | json_identical_across_runs; integer merge/derive tests | Same input bytes, explicit output comparison |
| NFR-4 | bounds errors; table sorted/unique/nonempty/valid-tag tests | Compile-time data types plus invariant gate; invalid config fails at load |
| NFR-5/7 | forbidden_uses, pure_modules_have_no_io, network_only_in_pull; recorded release size | Static coverage accepted with limits (Gap 12); size must be measured at delivery |
| NFR-6 | rejected_new_check_is_slp901_with_name; failing_template_fix_is_slp901 | Existing forged-proposal seam is real; every new fix follows verify |
| NFR-8 | Prose Send + Sync compile assertion; existing parallel-analysis path | No mutable global cache introduced |
| NFR-10 | SPEC §36's named test inventory and calibration/reliance groups | Concrete except the gaps above; future tests are not reported as passing |

**Executed baseline verification:** seven targeted existing tests passed: all six rules_core tests and rules_fixtures::safe_fix_contexts_grammatical. Selected through the project contract, with the repository selector equivalent `make test TESTS='sloperator::rules_fixtures#safe_fix_contexts_grammatical rules_core'`. The first attempted exact selectors omitted the binary prefix and were rejected with “no test ran”; the corrected run passed seven tests. No whole-tree build/test was run. These tests validate current helper behavior, not the new feature snippets or the author's claimed scratch compile.

**Deferred routing:** Gaps 8 and 9 are separately recorded in [deferred.md](deferred.md). The missing conventions inputs, explicit accepted deviations and known counted-allowance limitation are recorded here; none is silently called verified delivery.
