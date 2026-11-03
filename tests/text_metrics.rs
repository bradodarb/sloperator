//! Pipeline metrics for original and final text (FR-3.4).
#![allow(clippy::unwrap_used, clippy::expect_used)]

use sloperator::config::{CliOverrides, Resolver};
use sloperator::engine::DEFAULT_SEED;
use sloperator::input::{Format, Origin, Source};
use sloperator::pipeline::{self, Engines};
use std::sync::Arc;

#[test]
fn fix_reports_before_and_after() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("doc.md");
    let settings = Resolver::new(CliOverrides::default(), None)
        .unwrap()
        .for_file(&path)
        .unwrap();
    let source = Source {
        origin: Origin::File(path),
        format: Format::Markdown,
        text: Arc::from("We utilize the cache in order to look things up.\n".repeat(5)),
        settings,
    };
    let a = pipeline::analyse(source.clone()).unwrap();
    let before = a.prose.counts(&a.doc).clone();
    let r = pipeline::fix(
        a,
        Engines {
            rules: true,
            model: false,
        },
        None,
        DEFAULT_SEED,
    )
    .unwrap();
    let fixed = r.fixed.as_deref().expect("safe fixes change text");
    assert_eq!(fixed, "We use the cache to look things up.\n".repeat(5));
    let after = pipeline::analyse(Source {
        text: Arc::from(fixed),
        ..source
    })
    .unwrap();
    assert_eq!(r.metrics_before, Some(before));
    assert_eq!(r.metrics, *after.prose.counts(&after.doc));
    assert_eq!(r.metrics_before.as_ref().unwrap().words, 50);
    assert_eq!(r.metrics.words, 40);
    assert_eq!(r.floors, after.source.settings.metrics);
}
