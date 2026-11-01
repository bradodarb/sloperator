//! The real model through `engine::runtime` (SPEC §21, §23.19). Integration tier only: every test needs the default
//! model in the cache (`sloperator model pull`) and fails, naming that command, when it is absent (architecture §11).
//! nextest runs each test in its own process, which the one-backend-per-process runtime requires.
#![cfg(feature = "model")]
#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may unwrap (architecture §8)

mod common;

use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

use assert_cmd::Command;
use common::{cli_tempdir, run_cli_with};
use sloperator::Error;
use sloperator::cli::ExitCode;
use sloperator::config::{ModelSettings, WindowMode};
use sloperator::doc::{self, Format, ProtectSettings};
use sloperator::engine::runtime::RuntimeFactory;
use sloperator::engine::{self, RewriteRequest, Rewriter, RewriterFactory};
use sloperator::model::{self, ResolvedModel};

/// Plain prose with model-class findings and no protected bytes, so a good rewrite can be accepted.
const PLAIN: &[&str] = &[
    "In this fast-paced world, it is crucial to leverage robust tools that empower teams to delve into their data and unlock a rich tapestry of insights.\n",
    "Our new dashboard serves as a testament to the team's dedication, and it stands as a pivotal milestone in the journey toward seamless reporting.\n",
];

/// The eval paragraphs sent to the model: links, inline code and numbers among them.
const EVAL: &[&str] = &["p01.md", "p02.md", "p03.md", "p04.md", "p05.md", "p06.md"];

/// The default model, resolved from the user cache. Absent → the test fails naming `sloperator model pull`.
fn require_model() -> ResolvedModel {
    let s = ModelSettings {
        name: model::DEFAULT_MODEL.to_string(),
        path: None,
        profile: None,
        cache_dir: None,
        window: WindowMode::Paragraph,
        max_window_chars: 4000,
        max_tokens: 0,
        seed: engine::DEFAULT_SEED,
    };
    match model::resolve(&s) {
        Ok(m) => m,
        Err(e) => {
            panic!("{e}\nthe integration tier needs the real model: run `sloperator model pull`")
        }
    }
}

/// A rewriter that counts its calls and delegates to the real one.
struct CountingRewriter {
    inner: Box<dyn Rewriter>,
    windows: Rc<Cell<u32>>,
}

impl Rewriter for CountingRewriter {
    fn rewrite(&mut self, req: &RewriteRequest<'_>) -> Result<String, Error> {
        self.windows.set(self.windows.get() + 1);
        self.inner.rewrite(req)
    }
}

/// The real factory, counting how often the model is opened and how many windows it rewrites.
#[derive(Clone, Default)]
struct Counts {
    opens: Rc<Cell<u32>>,
    windows: Rc<Cell<u32>>,
}

struct CountingFactory {
    inner: RuntimeFactory,
    counts: Counts,
}

impl RewriterFactory for CountingFactory {
    fn preflight(&self, m: &ResolvedModel) -> Result<(), Error> {
        self.inner.preflight(m)
    }
    fn open(&self, m: &ResolvedModel) -> Result<Box<dyn Rewriter>, Error> {
        self.counts.opens.set(self.counts.opens.get() + 1);
        Ok(Box::new(CountingRewriter {
            inner: self.inner.open(m)?,
            windows: Rc::clone(&self.counts.windows),
        }))
    }
}

fn counting() -> (Box<dyn RewriterFactory>, Counts) {
    let counts = Counts::default();
    let factory = CountingFactory {
        inner: RuntimeFactory::new(false),
        counts: counts.clone(),
    };
    (Box::new(factory), counts)
}

fn eval_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("eval")
        .join("paragraphs")
}

/// A CLI temp dir holding the eval paragraphs and the plain paragraphs, one paragraph (one window) per file.
fn corpus() -> (tempfile::TempDir, Vec<PathBuf>) {
    let tmp = cli_tempdir();
    let mut files = Vec::new();
    for name in EVAL {
        let path = tmp.path().join(name);
        fs::copy(eval_dir().join(name), &path).unwrap();
        files.push(path);
    }
    for (i, text) in PLAIN.iter().enumerate() {
        let path = tmp.path().join(format!("plain{i}.md"));
        fs::write(&path, text).unwrap();
        files.push(path);
    }
    (tmp, files)
}

fn protected(text: &str) -> Vec<String> {
    let d = doc::parse(
        Arc::from(text),
        Format::Markdown,
        &ProtectSettings::default(),
    );
    let mut s: Vec<String> = d
        .protected_strings()
        .into_iter()
        .map(str::to_string)
        .collect();
    s.sort();
    s
}

fn path_arg(p: &Path) -> &str {
    p.to_str().unwrap()
}

#[test]
#[ignore = "integration tier"]
fn loads_once_for_many_windows() {
    require_model();
    let (tmp, _) = corpus();
    let (factory, counts) = counting();
    let (code, out, err) = run_cli_with(
        &["check", "--diff", "--model-only", path_arg(tmp.path())],
        "",
        factory,
    );
    assert_ne!(code, ExitCode::Failure, "stdout:\n{out}\nstderr:\n{err}");
    assert!(
        counts.windows.get() >= 2,
        "expected several windows, got {}",
        counts.windows.get()
    );
    assert_eq!(
        counts.opens.get(),
        1,
        "the model must load once for every window (FR-9.4)"
    );
}

#[test]
#[ignore = "integration tier"]
fn same_seed_same_output() {
    let m = require_model();
    let mut rw = RuntimeFactory::new(false).open(&m).unwrap();
    let window = PLAIN[0].trim_end();
    let req = RewriteRequest {
        system: engine::prompt::SYSTEM,
        few_shot: engine::prompt::FEW_SHOT,
        window,
        profile: m.profile,
        max_tokens: 96,
        seed: 1234,
    };
    let first = rw.rewrite(&req).unwrap();
    let second = rw.rewrite(&req).unwrap();
    assert!(!first.trim().is_empty(), "the model produced no text");
    assert_eq!(
        first, second,
        "the same seed must give the same output (FR-9.8)"
    );
}

#[test]
#[ignore = "integration tier"]
fn real_rewrites_accepted_or_rejected_never_unverified() {
    require_model();
    let (tmp, files) = corpus();
    let before: Vec<String> = files
        .iter()
        .map(|f| fs::read_to_string(f).unwrap())
        .collect();
    let (factory, counts) = counting();
    let (code, out, err) = run_cli_with(
        &[
            "check",
            "--fix",
            "--model-only",
            "--output-format",
            "json",
            path_arg(tmp.path()),
        ],
        "",
        factory,
    );
    assert_ne!(code, ExitCode::Failure, "stdout:\n{out}\nstderr:\n{err}");
    let report: serde_json::Value = serde_json::from_str(&out).unwrap();

    let mut written = 0;
    for (path, old) in files.iter().zip(&before) {
        let new = fs::read_to_string(path).unwrap();
        if new != *old {
            written += 1;
            // A written rewrite passed verify: the protected bytes are exactly the original ones (NFR-8, D-4).
            assert_eq!(
                protected(&new),
                protected(old),
                "{} lost protected bytes",
                path.display()
            );
        }
    }
    // --model-only: every SLP901 is a rejected model rewrite.
    let rejected = report["summary"]["rejected"].as_u64().unwrap();
    let slp901 = report["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|f| f["findings"].as_array().unwrap())
        .filter(|f| f["code"] == "SLP901")
        .count() as u64;
    assert_eq!(rejected, slp901, "{out}");
    let windows = u64::from(counts.windows.get());
    assert!(windows > 0, "no window reached the model");
    // One window per file: each rewrite is either written (verified) or reported as SLP901, never both or neither.
    assert_eq!(
        windows,
        written + rejected,
        "{windows} rewrites, {written} written, {rejected} rejected\n{out}"
    );
}

#[test]
#[ignore = "integration tier"]
fn no_network_during_check() {
    require_model();
    // A port that was just free and is now closed: any proxied request would fail.
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let proxy = format!("http://127.0.0.1:{port}");
    let tmp = cli_tempdir();
    let file = tmp.path().join("plain.md");
    fs::write(&file, PLAIN[0]).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_sloperator"))
        .current_dir(tmp.path())
        .env("NO_COLOR", "1")
        .env("HTTPS_PROXY", &proxy)
        .env("HTTP_PROXY", &proxy)
        .env("ALL_PROXY", &proxy)
        .args(["check", "--fix", "plain.md"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let code = output.status.code();
    assert!(
        matches!(code, Some(0 | 1)),
        "check --fix must succeed offline (FR-9.1); exit {code:?}\nstderr:\n{stderr}"
    );
    assert!(!stderr.contains("sloperator:"), "stderr:\n{stderr}");
}

/// The macOS sandbox profile: everything allowed except every network operation (loopback included).
const DENY_NETWORK: &str = "(version 1)(allow default)(deny network*)";

/// The command prefix that runs a program with the network denied by the OS: `sandbox-exec` on macOS,
/// `unshare -rn` (a fresh, empty network namespace) on Linux. Each candidate is probed by running `true` under it;
/// when none works the test fails naming the tools, so a host without a sandbox never passes silently.
fn network_sandbox() -> Vec<String> {
    let candidates: [&[&str]; 2] = [&["sandbox-exec", "-p", DENY_NETWORK], &["unshare", "-rn"]];
    let mut tried = Vec::new();
    for prefix in candidates {
        let probe = std::process::Command::new(prefix[0])
            .args(&prefix[1..])
            .arg("true")
            .output();
        match probe {
            Ok(o) if o.status.success() => {
                return prefix.iter().map(|s| (*s).to_string()).collect();
            }
            Ok(o) => tried.push(format!(
                "`{}` exited {:?}: {}",
                prefix.join(" "),
                o.status.code(),
                String::from_utf8_lossy(&o.stderr).trim()
            )),
            Err(e) => tried.push(format!("`{}` could not run: {e}", prefix[0])),
        }
    }
    panic!(
        "no OS network sandbox on this host: neither `sandbox-exec` (macOS) nor `unshare -rn` (Linux) works\n{}",
        tried.join("\n")
    );
}

/// `prefix program` as a std command; the caller adds the program's arguments.
fn sandboxed(prefix: &[String], program: &str) -> std::process::Command {
    let mut c = std::process::Command::new(&prefix[0]);
    c.args(&prefix[1..]).arg(program);
    c
}

#[test]
#[ignore = "integration tier"]
fn no_network_under_sandbox() {
    require_model();
    let sandbox = network_sandbox();

    // Control: the sandbox must deny a TCP connect to a live loopback listener, which the same probe makes unsandboxed.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port().to_string();
    let nc_args = ["-z", "-w", "2", "127.0.0.1", port.as_str()];
    let open = std::process::Command::new("nc")
        .args(nc_args)
        .output()
        .unwrap_or_else(|e| panic!("the sandbox control needs `nc` on PATH: {e}"));
    assert!(
        open.status.success(),
        "control: unsandboxed `nc -z 127.0.0.1 {port}` must connect to the live listener; exit {:?}\n{}",
        open.status.code(),
        String::from_utf8_lossy(&open.stderr)
    );
    let denied = sandboxed(&sandbox, "nc").args(nc_args).output().unwrap();
    assert!(
        !denied.status.success(),
        "control: `{} nc -z 127.0.0.1 {port}` connected — the sandbox does not deny network",
        sandbox.join(" ")
    );
    drop(listener);

    let tmp = cli_tempdir();
    fs::write(tmp.path().join("plain.md"), PLAIN[0]).unwrap();
    let output = sandboxed(&sandbox, env!("CARGO_BIN_EXE_sloperator"))
        .current_dir(tmp.path())
        .env("NO_COLOR", "1")
        .args(["check", "--fix", "plain.md"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let code = output.status.code();
    assert!(
        matches!(code, Some(0 | 1)),
        "check --fix must succeed with the network denied by `{}` (FR-9.1); exit {code:?}\nstderr:\n{stderr}",
        sandbox.join(" ")
    );
    assert!(!stderr.contains("sloperator:"), "stderr:\n{stderr}");
}
