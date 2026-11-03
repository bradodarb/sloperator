//! `sloperator mcp` (ADR-10): the tool handler called directly with a fake model, a stdio protocol smoke test over the
//! built binary, and one integration-tier test with the real model.
#![cfg(feature = "mcp")]
#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may unwrap (architecture §8)

mod common;

use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use common::FakeRewriter;
use serde_json::{Value, json};
use sloperator::Error;
use sloperator::engine::{RewriteRequest, Rewriter, RewriterFactory};
use sloperator::mcp::{self, FactoryFn, Options, Server};
use sloperator::model::ResolvedModel;

/// Prose with model-class findings, so a model-backed fix opens the model.
const SLOP: &str = "In this fast-paced world, it is crucial to leverage robust tools that empower teams to delve into \
                    their data and unlock a rich tapestry of insights.\n";

/// How often the model was opened, how often a loaded model was dropped, and how many windows were rewritten.
#[derive(Clone, Default)]
struct Counts {
    opens: Arc<AtomicU32>,
    drops: Arc<AtomicU32>,
    rewrites: Arc<AtomicU32>,
}

/// A [`FakeRewriter`] (returns the window unchanged) that counts its rewrites and its drop.
struct Counted {
    inner: FakeRewriter,
    counts: Counts,
}

impl Rewriter for Counted {
    fn rewrite(&mut self, req: &RewriteRequest<'_>) -> Result<String, Error> {
        self.counts.rewrites.fetch_add(1, Ordering::SeqCst);
        self.inner.rewrite(req)
    }
}

impl Drop for Counted {
    fn drop(&mut self) {
        self.counts.drops.fetch_add(1, Ordering::SeqCst);
    }
}

struct CountingFactory(Counts);

impl RewriterFactory for CountingFactory {
    fn open(&self, _: &ResolvedModel) -> Result<Box<dyn Rewriter>, Error> {
        self.0.opens.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(Counted {
            inner: FakeRewriter::default(),
            counts: self.0.clone(),
        }))
    }
}

fn fake(counts: &Counts) -> FactoryFn {
    let counts = counts.clone();
    Box::new(move || Box::new(CountingFactory(counts)) as Box<dyn RewriterFactory>)
}

fn opts(cwd: &Path, rules_only: bool, idle: Option<Duration>) -> Options {
    Options {
        rules_only,
        idle_unload: idle,
        verbose: false,
        cwd: cwd.to_path_buf(),
    }
}

/// A directory whose `sloperator.toml` points the model at an empty stand-in file, so `model::resolve` succeeds.
fn model_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let gguf = dir.path().join("fake.gguf");
    fs::write(&gguf, b"").unwrap();
    fs::write(
        dir.path().join("sloperator.toml"),
        format!(
            "[model]\npath = {:?}\nprofile = \"qwen3\"\n",
            gguf.display().to_string()
        ),
    )
    .unwrap();
    dir
}

/// Runs one tool call on a fresh current-thread runtime; returns (isError, the structured result or the error text).
fn call(server: &Server, name: &str, args: Value) -> (bool, Value) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let Value::Object(args) = args else {
        panic!("args must be an object")
    };
    let r = rt.block_on(server.call(name, args)).unwrap();
    let is_error = r.is_error == Some(true);
    let v = match r.structured_content {
        Some(v) => v,
        None => Value::String(
            r.content
                .first()
                .and_then(|c| c.as_text())
                .map(|t| t.text.clone())
                .unwrap_or_default(),
        ),
    };
    (is_error, v)
}

fn rules_only_server(cwd: &Path) -> Server {
    Server::new(opts(cwd, true, None), fake(&Counts::default())).unwrap()
}

#[test]
fn tools_list_has_four_tools_with_schemas() {
    let tools = mcp::tools();
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
    assert_eq!(names, ["lint", "fix", "rules", "explain_rule"]);
    for t in &tools {
        assert_eq!(
            t.input_schema.get("type"),
            Some(&json!("object")),
            "{}",
            t.name
        );
        assert!(t.input_schema.contains_key("properties"), "{}", t.name);
        assert!(t.description.is_some(), "{}", t.name);
    }
    let fix = &tools[1];
    let desc = fix.description.as_deref().unwrap();
    assert!(desc.contains("Nothing is written to disk"), "{desc}");
    assert!(desc.contains("show the user the diff"), "{desc}");
    assert_eq!(fix.input_schema["required"], json!(["text"]));
    assert!(fix.input_schema["properties"]["rules_only"].is_object());
}

#[test]
fn lint_returns_findings_and_a_score() {
    let dir = tempfile::tempdir().unwrap();
    let (err, v) = call(
        &rules_only_server(dir.path()),
        "lint",
        json!({ "text": SLOP }),
    );
    assert!(!err, "{v}");
    let findings = v["findings"].as_array().unwrap();
    assert!(!findings.is_empty(), "{v}");
    let f = &findings[0];
    for key in ["code", "severity", "line", "col", "message"] {
        assert!(!f[key].is_null(), "finding lacks {key}: {f}");
    }
    assert!(f["code"].as_str().unwrap().starts_with("SLP"));
    assert!(v["score"]["value"].as_f64().unwrap() > 0.0, "{v}");
}

#[test]
fn fix_rules_only_returns_text_and_diff_and_never_touches_disk() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("doc.md");
    let on_disk = b"The file on disk stays exactly as it is.\n";
    fs::write(&file, on_disk).unwrap();
    let listing = || {
        let mut v: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        v.sort();
        v
    };
    let before = listing();
    let text = "It is important to note that this is great.\n";
    let (err, v) = call(
        &rules_only_server(dir.path()),
        "fix",
        json!({ "text": text, "path": "doc.md", "rules_only": true }),
    );
    assert!(!err, "{v}");
    assert_eq!(v["changed"], json!(true), "{v}");
    assert_eq!(v["fixed"], json!("This is great.\n"), "{v}");
    let diff = v["diff"].as_str().unwrap();
    assert!(
        diff.contains("--- a/doc.md") && diff.contains("+This is great."),
        "{diff}"
    );
    assert!(
        v["scoreBefore"]["weighted"].as_u64().unwrap()
            > v["scoreAfter"]["weighted"].as_u64().unwrap()
    );
    assert!(v["rejections"].as_array().unwrap().is_empty(), "{v}");
    assert_eq!(v["engines"], json!({ "rules": true, "model": false }));
    // The path was only a hint: the file is byte-identical and nothing was added beside it.
    assert_eq!(fs::read(&file).unwrap(), on_disk);
    assert_eq!(listing(), before);
}

#[test]
fn model_fix_loads_once_for_many_calls() {
    let dir = model_dir();
    let counts = Counts::default();
    let server = Server::new(opts(dir.path(), false, None), fake(&counts)).unwrap();
    for _ in 0..3 {
        let (err, v) = call(&server, "fix", json!({ "text": SLOP, "path": "doc.md" }));
        assert!(!err, "{v}");
        assert_eq!(v["engines"]["model"], json!(true));
        // The fake hands each window back unchanged, which verification rejects as no improvement.
        assert!(!v["rejections"].as_array().unwrap().is_empty(), "{v}");
    }
    assert_eq!(counts.opens.load(Ordering::SeqCst), 1);
    assert!(counts.rewrites.load(Ordering::SeqCst) >= 3);
    server.shutdown();
    assert_eq!(
        counts.drops.load(Ordering::SeqCst),
        1,
        "shutdown frees the model"
    );
}

#[test]
fn rules_only_server_never_opens_the_model() {
    let dir = model_dir();
    let counts = Counts::default();
    let server = Server::new(opts(dir.path(), true, None), fake(&counts)).unwrap();
    let (err, v) = call(&server, "fix", json!({ "text": SLOP, "path": "doc.md" }));
    assert!(!err, "{v}");
    assert_eq!(v["engines"]["model"], json!(false));
    assert_eq!(counts.opens.load(Ordering::SeqCst), 0);
}

#[test]
fn idle_unload_fires_and_the_next_fix_reloads() {
    let dir = model_dir();
    let counts = Counts::default();
    let idle = Duration::from_millis(200);
    let server = Server::new(opts(dir.path(), false, Some(idle)), fake(&counts)).unwrap();
    let fix = || call(&server, "fix", json!({ "text": SLOP, "path": "doc.md" }));
    assert!(!fix().0);
    assert_eq!(counts.opens.load(Ordering::SeqCst), 1);
    assert_eq!(counts.drops.load(Ordering::SeqCst), 0);
    std::thread::sleep(Duration::from_millis(800));
    assert_eq!(counts.drops.load(Ordering::SeqCst), 1, "idle unload");
    assert!(!fix().0);
    assert_eq!(counts.opens.load(Ordering::SeqCst), 2, "reloaded on demand");
}

#[test]
fn missing_model_is_a_tool_error_naming_model_pull() {
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("empty-cache");
    fs::create_dir(&cache).unwrap();
    fs::write(
        dir.path().join("sloperator.toml"),
        format!("[model]\ncache_dir = {:?}\n", cache.display().to_string()),
    )
    .unwrap();
    let counts = Counts::default();
    let server = Server::new(opts(dir.path(), false, None), fake(&counts)).unwrap();
    let (err, v) = call(&server, "fix", json!({ "text": SLOP, "path": "doc.md" }));
    assert!(err, "{v}");
    let msg = v.as_str().unwrap();
    assert!(msg.contains("sloperator model pull"), "{msg}");
    assert_eq!(counts.opens.load(Ordering::SeqCst), 0);
    // Never auto-download: the cache stays empty.
    assert_eq!(fs::read_dir(&cache).unwrap().count(), 0);
}

#[test]
fn oversize_input_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let server = rules_only_server(dir.path());
    let big = "a ".repeat(mcp::MAX_INPUT_BYTES / 2 + 1);
    for tool in ["lint", "fix"] {
        let (err, v) = call(&server, tool, json!({ "text": big }));
        assert!(err, "{tool}");
        assert!(v.as_str().unwrap().contains("1 MiB"), "{tool}: {v}");
    }
}

#[test]
fn rules_and_explain_rule() {
    let dir = tempfile::tempdir().unwrap();
    let server = rules_only_server(dir.path());
    let (err, v) = call(&server, "rules", json!({}));
    assert!(!err);
    let rules = v["rules"].as_array().unwrap();
    assert!(rules.len() > 5, "{v}");
    let code = rules[0]["code"].as_str().unwrap().to_string();
    let (err, v) = call(&server, "explain_rule", json!({ "code": code }));
    assert!(!err, "{v}");
    assert_eq!(v["code"], json!(code));
    assert!(!v["explain"].as_str().unwrap().is_empty());
    assert!(v["example"]["before"].is_string());
}

#[test]
fn explain_rule_unknown_code_is_a_tool_error() {
    let dir = tempfile::tempdir().unwrap();
    let (err, v) = call(
        &rules_only_server(dir.path()),
        "explain_rule",
        json!({ "code": "SLP999" }),
    );
    assert!(err, "{v}");
    assert!(
        v.as_str().unwrap().contains("unknown rule code `SLP999`"),
        "{v}"
    );
}

/// The built binary over stdio: initialize, tools/list and tools/call(lint) as JSON-RPC lines.
#[test]
fn stdio_protocol_smoke() {
    let lines = [
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "protocolVersion": "2025-06-18", "capabilities": {},
            "clientInfo": {"name": "smoke", "version": "0"}}}),
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
        json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call",
            "params": {"name": "lint", "arguments": {"text": SLOP}}}),
    ];
    let input: String = lines.iter().map(|l| format!("{l}\n")).collect();
    let out = assert_cmd::Command::cargo_bin("sloperator")
        .unwrap()
        .args(["mcp", "--rules-only"])
        .write_stdin(input)
        .timeout(Duration::from_secs(30))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    // stdout carries only protocol messages: every line is a JSON-RPC object.
    let msgs: Vec<Value> = stdout
        .lines()
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("not JSON-RPC: {l}: {e}")))
        .collect();
    let by_id = |id: u64| {
        msgs.iter()
            .find(|m| m["id"] == json!(id))
            .unwrap_or_else(|| panic!("no response {id} in {stdout}"))
    };
    assert_eq!(
        by_id(1)["result"]["serverInfo"]["name"],
        json!("sloperator")
    );
    let tools = by_id(2)["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 4);
    let lint = &by_id(3)["result"];
    assert_eq!(lint["isError"], json!(false));
    assert!(
        !lint["structuredContent"]["findings"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

/// The real model through the server: a fix succeeds and a second fix reuses the loaded model.
#[cfg(feature = "model")]
#[test]
#[ignore = "integration tier"]
fn real_model_fix_through_the_server_loads_once() {
    struct Real {
        inner: Box<dyn RewriterFactory>,
        opens: Arc<AtomicU32>,
    }
    impl RewriterFactory for Real {
        fn open(&self, m: &ResolvedModel) -> Result<Box<dyn Rewriter>, Error> {
            self.opens.fetch_add(1, Ordering::SeqCst);
            self.inner.open(m)
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let opens = Arc::new(AtomicU32::new(0));
    let o = Arc::clone(&opens);
    let make: FactoryFn = Box::new(move || {
        Box::new(Real {
            inner: sloperator::engine::default_factory(false),
            opens: o,
        }) as Box<dyn RewriterFactory>
    });
    let server = Server::new(opts(dir.path(), false, None), make).unwrap();
    for _ in 0..2 {
        let (err, v) = call(&server, "fix", json!({ "text": SLOP, "path": "doc.md" }));
        assert!(
            !err,
            "{v}\nthe integration tier needs the real model: run `sloperator model pull`"
        );
        assert_eq!(v["engines"]["model"], json!(true));
        assert!(v["fixed"].is_string() && v["scoreAfter"].is_object());
    }
    assert_eq!(
        opens.load(Ordering::SeqCst),
        1,
        "the second fix must not reload"
    );
    server.shutdown();
}

#[test]
fn lint_has_metrics() {
    let dir = tempfile::tempdir().unwrap();
    let server = rules_only_server(dir.path());
    let text = "The cat watched the dog and the bird beside the old tree near the quiet house all morning long. ".repeat(6);
    let (err, v) = call(&server, "lint", json!({ "text": text, "path": "doc.md" }));
    assert!(!err, "{v}");
    let m = &v["metrics"];
    assert_eq!(m["sentences"], 6);
    assert_eq!(m["words"], 114);
    for key in [
        "avgSentenceLength",
        "gradeLevel",
        "sentenceLengthSd",
        "sentenceLengthCv",
        "typeTokenRatio",
    ] {
        assert!(m[key].is_number(), "{key}: {m}");
    }
    let (err, v) = call(
        &server,
        "lint",
        json!({ "text": "Cat dog.", "path": "doc.md" }),
    );
    assert!(!err, "{v}");
    assert_eq!(v["metrics"]["words"], 2);
    assert!(v["metrics"]["gradeLevel"].is_null());
    fs::create_dir(dir.path().join("sub")).unwrap();
    fs::write(
        dir.path().join("sub/sloperator.toml"),
        "[metrics]\nmin_sentences = 10\n",
    )
    .unwrap();
    let (err, v) = call(
        &server,
        "lint",
        json!({ "text": text, "path": "sub/doc.md" }),
    );
    assert!(!err, "{v}");
    assert_eq!(v["metrics"]["sentences"], 6);
    assert!(v["metrics"]["avgSentenceLength"].is_null(), "{v}");
}

#[test]
fn fix_has_metrics_before_and_after() {
    let dir = tempfile::tempdir().unwrap();
    let server = rules_only_server(dir.path());
    let text = "We utilize the cache in order to look things up.\n".repeat(15);
    let (err, v) = call(
        &server,
        "fix",
        json!({ "text": text, "path": "doc.md", "rules_only": true }),
    );
    assert!(!err, "{v}");
    assert_eq!(v["changed"], true);
    assert_eq!(v["metricsBefore"]["words"], 150);
    assert_eq!(v["metricsAfter"]["words"], 120);
    for field in ["metricsBefore", "metricsAfter"] {
        assert_eq!(v[field]["sentences"], 15);
        for key in [
            "avgSentenceLength",
            "gradeLevel",
            "sentenceLengthSd",
            "sentenceLengthCv",
            "typeTokenRatio",
        ] {
            assert!(v[field][key].is_number(), "{field}.{key}: {v}");
        }
    }
    let (err, v) = call(
        &server,
        "fix",
        json!({ "text": "Cat dog.", "rules_only": true }),
    );
    assert!(!err, "{v}");
    assert_eq!(v["changed"], false);
    assert_eq!(v["metricsBefore"], v["metricsAfter"]);
    assert!(v["metricsAfter"]["gradeLevel"].is_null());
}
