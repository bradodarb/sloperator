//! `directives` (SPEC §12, §23.8): what `extract` suppresses and what it rejects.
#![allow(clippy::unwrap_used, clippy::expect_used)] // test helpers may unwrap (architecture §8)

mod common;

use std::path::Path;
use std::sync::Arc;

use sloperator::Error;
use sloperator::directives::{self, Suppressions};
use sloperator::doc::{self, Document, Format, ProtectReason, ProtectSettings, SpanKind};
use sloperator::rules::registry::Registry;
use sloperator::rules::{Family, Finding, FixClass, Rule, RuleCode, RuleCtx, RuleMeta, Severity};

const SLP001: RuleCode = RuleCode(1);
const SLP002: RuleCode = RuleCode(2);

/// A rule that finds nothing; it only puts a code in a test registry.
struct Stub(RuleMeta);

impl Rule for Stub {
    fn meta(&self) -> &RuleMeta {
        &self.0
    }
    fn check(&self, _ctx: &RuleCtx<'_>, _out: &mut Vec<Finding>) {}
}

fn stub(code: RuleCode) -> Box<dyn Rule> {
    Box::new(Stub(RuleMeta {
        code,
        name: "stub",
        family: Family::Hedging,
        severity: Severity::Warning,
        default_on: true,
        fix: FixClass::None,
        explain: "",
        example: ("", ""),
        params: &[],
        since: "0.1.0",
    }))
}

/// SLP001 and SLP002, plus a rule that claims a system code: a directive must still reject it.
fn registry() -> Registry {
    Registry::from_rules(vec![
        stub(SLP001),
        stub(SLP002),
        stub(RuleCode::REWRITE_REJECTED),
    ])
}

fn parse(src: &str, format: Format) -> Document {
    doc::parse(Arc::from(src), format, &ProtectSettings::default())
}

fn extract(src: &str, format: Format) -> Result<(Document, Suppressions), Error> {
    let mut d = parse(src, format);
    let sup = directives::extract(&mut d, Path::new("doc.md"), &registry())?;
    Ok((d, sup))
}

fn reason_of(d: &Document, needle: &str) -> Option<ProtectReason> {
    let at = d.src.find(needle).unwrap();
    let span = d
        .spans
        .iter()
        .find(|s| s.range.start <= at && at < s.range.end)
        .unwrap();
    match span.kind {
        SpanKind::Protected(r) => Some(r),
        SpanKind::Prose => None,
    }
}

fn has_directive_span(d: &Document) -> bool {
    d.spans
        .iter()
        .any(|s| s.kind == SpanKind::Protected(ProtectReason::Directive))
}

#[test]
fn ignore_next_line_named_and_all() {
    let src = "\
<!-- sloperator-ignore SLP001 -->

First prose line.
Second prose line.

<!-- sloperator-ignore -->
<!-- sloperator-ignore SLP002 -->

Third prose line.
";
    let (d, sup) = extract(src, Format::Markdown).unwrap();
    // Named: the next prose line, for that code only.
    assert!(sup.is_suppressed(3, SLP001));
    assert!(!sup.is_suppressed(3, SLP002));
    assert!(!sup.is_suppressed(4, SLP001));
    // Blank lines and other directives in between are skipped; both directives land on the same line.
    assert!(sup.is_suppressed(9, SLP001));
    assert!(sup.is_suppressed(9, SLP002));
    assert!(!sup.is_suppressed(2, SLP001));
    assert!(!sup.is_suppressed(1, SLP001));
    // A suppressed line is frozen, and a system code is never suppressed.
    assert!(sup.frozen_ranges().contains(&d.line_range(3)));
    assert!(!sup.is_suppressed(9, RuleCode::REWRITE_REJECTED));
    // The directive lines are re-tagged.
    assert_eq!(
        reason_of(&d, "<!-- sloperator-ignore SLP001 -->"),
        Some(ProtectReason::Directive)
    );
}

#[test]
fn disable_enable_region() {
    let src = "\
Before.

<!-- sloperator-disable SLP001 -->

Inside one.

Inside two.

<!-- sloperator-enable SLP001 -->

After.
";
    let (d, sup) = extract(src, Format::Markdown).unwrap();
    assert!(!sup.is_suppressed(1, SLP001));
    assert!(sup.is_suppressed(5, SLP001));
    assert!(sup.is_suppressed(7, SLP001));
    assert!(!sup.is_suppressed(5, SLP002));
    assert!(!sup.is_suppressed(11, SLP001));
    let frozen = d.line_range(4).start..d.line_range(9).start;
    assert_eq!(sup.frozen_ranges().len(), 1);
    assert_eq!(sup.frozen_ranges()[0], frozen);

    // No codes: the region covers every rule.
    let src = "<!-- sloperator-disable -->\nInside.\n<!-- sloperator-enable -->\nAfter.\n";
    let (_, sup) = extract(src, Format::Markdown).unwrap();
    assert!(sup.is_suppressed(2, SLP001));
    assert!(sup.is_suppressed(2, SLP002));
    assert!(!sup.is_suppressed(4, SLP001));
}

#[test]
fn unclosed_disable_exits_two_with_line() {
    let src = "Text.\n\n<!-- sloperator-disable SLP001 -->\n\nMore.\n";
    match extract(src, Format::Markdown) {
        Err(Error::Directive { file, line, reason }) => {
            assert_eq!(file, Path::new("doc.md"));
            assert_eq!(line, 3);
            assert!(
                reason.contains("without matching sloperator-enable"),
                "{reason}"
            );
        }
        other => panic!("expected Error::Directive, got {other:?}"),
    }
}

#[test]
fn ignore_file() {
    let src = "Prose one.\n\n<!-- sloperator-ignore-file -->\n\nProse two.\n";
    let (d, sup) = extract(src, Format::Markdown).unwrap();
    for line in 1..=5 {
        assert!(sup.is_suppressed(line, SLP001));
        assert!(sup.is_suppressed(line, SLP002));
    }
    assert!(!sup.is_suppressed(1, RuleCode::REWRITE_REJECTED));
    assert_eq!(sup.frozen_ranges().len(), 1);
    assert_eq!(sup.frozen_ranges()[0], 0..d.src.len());
}

#[test]
fn unknown_code_exits_two_with_line() {
    let src = "Text.\n\n<!-- sloperator-ignore SLP999 -->\n\nMore.\n";
    match extract(src, Format::Markdown) {
        Err(Error::UnknownRule { code, where_ }) => {
            assert_eq!(code, "SLP999");
            assert_eq!(where_.as_deref(), Some("doc.md:3"));
        }
        other => panic!("expected Error::UnknownRule, got {other:?}"),
    }
    // A word that is not a code at all is unknown too.
    let src = "<!-- sloperator-disable nonsense -->\n";
    assert!(matches!(
        extract(src, Format::Markdown),
        Err(Error::UnknownRule { .. })
    ));
}

#[test]
fn system_code_rejected() {
    let src = "Text.\n\n<!-- sloperator-ignore SLP901 -->\n\nMore.\n";
    match extract(src, Format::Markdown) {
        Err(Error::UnknownRule { code, where_ }) => {
            assert_eq!(code, "SLP901");
            assert_eq!(where_.as_deref(), Some("doc.md:3"));
        }
        other => panic!("expected Error::UnknownRule, got {other:?}"),
    }
}

#[test]
fn plain_text_bracket_directives() {
    let src = "\
[sloperator-ignore SLP001]
First line.

Second paragraph.

[sloperator-disable SLP002]
Inside.

[sloperator-enable SLP002]
After.
";
    let (d, sup) = extract(src, Format::PlainText).unwrap();
    assert!(sup.is_suppressed(2, SLP001));
    assert!(!sup.is_suppressed(2, SLP002));
    assert!(!sup.is_suppressed(4, SLP001));
    assert!(sup.is_suppressed(7, SLP002));
    assert!(!sup.is_suppressed(7, SLP001));
    assert!(!sup.is_suppressed(10, SLP002));
    assert_eq!(
        reason_of(&d, "[sloperator-ignore SLP001]"),
        Some(ProtectReason::Directive)
    );

    let (_, sup) = extract("[sloperator-ignore-file]\nProse.\n", Format::PlainText).unwrap();
    assert!(sup.is_suppressed(2, SLP001));

    assert!(matches!(
        extract("[sloperator-disable SLP001]\nProse.\n", Format::PlainText),
        Err(Error::Directive { line: 1, .. })
    ));
    assert!(matches!(
        extract("[sloperator-ignore SLP999]\nProse.\n", Format::PlainText),
        Err(Error::UnknownRule { .. })
    ));
}

/// `doc::plain` keeps its own copy of the plain-text pattern (SPEC §5); the two must not drift apart.
#[test]
fn plain_text_pattern_matches_doc_copy() {
    let plain = include_str!("../src/doc/plain.rs");
    assert!(
        plain.contains(directives::TXT_DIRECTIVE),
        "src/doc/plain.rs no longer holds directives::TXT_DIRECTIVE"
    );
}

#[test]
fn directive_in_code_fence_ignored() {
    let src = "\
Prose before.

```
<!-- sloperator-disable SLP001 -->
```

    <!-- sloperator-disable SLP001 -->

Prose after.
";
    let (d, sup) = extract(src, Format::Markdown).unwrap();
    for line in 1..=9 {
        assert!(!sup.is_suppressed(line, SLP001), "line {line}");
    }
    assert!(sup.frozen_ranges().is_empty());
    assert!(!has_directive_span(&d));
    assert_eq!(
        reason_of(&d, "<!-- sloperator-disable SLP001 -->"),
        Some(ProtectReason::CodeBlock)
    );
}

#[test]
fn directive_in_inline_code_ignored() {
    let src = "\
Use `<!-- sloperator-ignore-file -->` to skip a file.

`<!-- sloperator-disable SLP001 -->`

Prose after.
";
    let (d, sup) = extract(src, Format::Markdown).unwrap();
    for line in 1..=5 {
        assert!(!sup.is_suppressed(line, SLP001), "line {line}");
    }
    assert!(sup.frozen_ranges().is_empty());
    assert!(!has_directive_span(&d));
}

/// FR-6.6: a disable/enable region and an ignored line (`sloperator-ignore`) are frozen for the rules engine and the model engine alike.
#[test]
fn suppressed_region_not_edited_by_either_engine() {
    use common::FakeFactory;
    use sloperator::config::{CliOverrides, Resolver};
    use sloperator::engine::{DEFAULT_SEED, LazyRewriter};
    use sloperator::input::{Origin, Source};
    use sloperator::pipeline::{self, Engines};

    let text = "We utilize the cache.\n\n\
                <!-- sloperator-disable -->\n\
                We utilize the cache. It plays a crucial role in the build.\n\
                <!-- sloperator-enable -->\n\n\
                It plays a crucial role in the build.\n";
    let factory = FakeFactory::returning("It speeds up the build.");
    let cache = common::TempModelCache::new();
    cache.plant(sloperator::model::spec(sloperator::model::DEFAULT_MODEL).unwrap());
    let mut lazy = LazyRewriter::new(&factory, &cache.settings()).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("doc.md");
    let settings = Resolver::new(CliOverrides::default(), None)
        .unwrap()
        .for_file(&path)
        .unwrap();
    let a = pipeline::analyse(Source {
        origin: Origin::File(path),
        format: Format::Markdown,
        text: Arc::from(text),
        settings,
    })
    .unwrap();
    let both = Engines {
        rules: true,
        model: true,
    };
    let r = pipeline::fix(a, both, Some(&mut lazy), DEFAULT_SEED).unwrap();

    // Outside the region both engines edit; inside it neither touches a byte, and the model never sees it.
    assert_eq!(
        r.fixed.as_deref(),
        Some(
            "We use the cache.\n\n\
             <!-- sloperator-disable -->\n\
             We utilize the cache. It plays a crucial role in the build.\n\
             <!-- sloperator-enable -->\n\n\
             It speeds up the build.\n"
        )
    );
    let calls = factory.calls.borrow();
    assert_eq!(calls.len(), 1, "{calls:?}");
    assert_eq!(calls[0].0, "It plays a crucial role in the build.");

    // A window that overlaps a frozen line is skipped (SLP902), never sent.
    let text = "<!-- sloperator-ignore -->\n\
                We utilize the cache.\n\
                It plays a crucial role in the build.\n";
    let factory = FakeFactory::returning("unused");
    let mut lazy = LazyRewriter::new(&factory, &cache.settings()).unwrap();
    let path = tmp.path().join("doc2.md");
    let settings = Resolver::new(CliOverrides::default(), None)
        .unwrap()
        .for_file(&path)
        .unwrap();
    let a = pipeline::analyse(Source {
        origin: Origin::File(path),
        format: Format::Markdown,
        text: Arc::from(text),
        settings,
    })
    .unwrap();
    let r = pipeline::fix(a, both, Some(&mut lazy), DEFAULT_SEED).unwrap();
    assert!(factory.calls.borrow().is_empty());
    assert_eq!(factory.opens.get(), 0);
    assert_eq!(r.fixed, None);
    assert!(
        r.findings
            .iter()
            .any(|l| l.finding.code == RuleCode::WINDOW_SKIPPED),
        "{:?}",
        r.findings
    );
}
