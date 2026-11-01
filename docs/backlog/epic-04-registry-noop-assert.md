# sloperator epic-04: no-op metadata assertion in rules_registry

**Source:** acceptance audit of epic-04-rules-score-directives (reviewer).
**Severity:** minor (non-blocking).

In `tests/rules_registry.rs:66` (`every_rule_has_code_name_severity_state_fix`), the line `let _ = (m.severity, m.default_on, m.fix);` asserts nothing at runtime. The type system already requires these fields, so FR-4.1 still holds.

To give the test meaning, check each rule's values against the catalogue table in SPEC §10. For example: assert each rule's severity and fix class, and that `default_on` agrees with `Registry::default_set()`.
