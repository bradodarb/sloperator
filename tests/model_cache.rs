//! Model registry, cache resolve, `pull`, `model list|path` and the lazy rewriter port (SPEC §23.11, §16).
//! No test reaches the network: `pull` is pointed at a local `TcpListener`.
#![allow(clippy::unwrap_used, clippy::expect_used)] // test helpers may unwrap (architecture §8)

mod common;

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;

use clap::Parser;
use sha2::{Digest, Sha256};

use common::{FakeFactory, TempModelCache};
use sloperator::Error;
use sloperator::cli::{self, Cli, Deps, ExitCode};
use sloperator::engine::DisabledFactory;
use sloperator::engine::LazyRewriter;
use sloperator::model::{self, DEFAULT_MODEL, ModelSpec, PROFILES, REGISTRY};

#[test]
fn registry_entries_have_sha256_and_profile() {
    assert!(!REGISTRY.is_empty());
    assert!(
        model::spec(DEFAULT_MODEL).is_some(),
        "DEFAULT_MODEL is not in the registry"
    );
    for m in REGISTRY {
        assert_eq!(m.sha256.len(), 64, "{}: sha256 is not 64 chars", m.name);
        assert!(
            m.sha256
                .chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
            "{}: sha256 is not lowercase hex",
            m.name
        );
        assert!(m.bytes > 0, "{}: no size", m.name);
        assert!(!m.licence.is_empty(), "{}: no licence", m.name);
        assert!(m.file.ends_with(".gguf"), "{}: file is not a GGUF", m.name);
        assert!(m.repo.contains('/'), "{}: repo is not owner/name", m.name);
        assert!(
            PROFILES.iter().any(|p| std::ptr::eq(*p, m.profile)),
            "{}: profile `{}` is not in PROFILES",
            m.name,
            m.profile.name
        );
    }
    let mut names: Vec<_> = REGISTRY.iter().map(|m| m.name).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), REGISTRY.len(), "duplicate registry names");
}

#[test]
fn missing_model_error_names_pull_and_rules_only() {
    let cache = TempModelCache::new();
    let err = model::resolve(&cache.settings()).unwrap_err();
    let spec = model::spec(DEFAULT_MODEL).unwrap();
    match &err {
        Error::ModelMissing { name, path } => {
            assert_eq!(name, DEFAULT_MODEL);
            assert_eq!(path, &model::cached_path(cache.dir.path(), spec));
        }
        other => panic!("expected ModelMissing, got {other:?}"),
    }
    let msg = err.to_string();
    assert!(
        msg.contains(&format!("sloperator model pull {DEFAULT_MODEL}")),
        "{msg}"
    );
    assert!(msg.contains("--rules-only"), "{msg}");

    // Once the file is in the cache, the same settings resolve to it with the entry's profile.
    let planted = cache.plant(spec);
    let resolved = model::resolve(&cache.settings()).unwrap();
    assert_eq!(resolved.path, planted);
    assert_eq!(resolved.profile.name, spec.profile.name);
}

/// Every file under `dir`, skipping build output.
fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let kind = entry.file_type().unwrap();
        if kind.is_dir() {
            if entry.file_name() != "target" && entry.file_name() != ".git" {
                files(&path, out);
            }
        } else if kind.is_file() {
            out.push(path);
        }
    }
}

#[test]
fn no_weights_in_crate() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut all = Vec::new();
    files(root, &mut all);
    assert!(!all.is_empty());
    let mut violations = Vec::new();
    for path in all {
        let rel = path.strip_prefix(root).unwrap().display().to_string();
        if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("gguf"))
        {
            violations.push(format!("{rel}: a GGUF file"));
        }
        let len = fs::metadata(&path).unwrap().len();
        if len > 10 * 1024 * 1024 {
            violations.push(format!("{rel}: {len} bytes (over 10 MB)"));
        }
    }
    assert!(
        violations.is_empty(),
        "weights in the crate (FR-9.10):\n{}",
        violations.join("\n")
    );
}

#[test]
fn lazy_rewriter_preflights_without_opening() {
    let cache = TempModelCache::new();
    cache.plant(model::spec(DEFAULT_MODEL).unwrap());
    let factory = FakeFactory::default();

    let mut lazy = LazyRewriter::new(&factory, &cache.settings()).unwrap();
    assert_eq!(factory.preflights.get(), 1, "new runs preflight once");
    assert_eq!(factory.opens.get(), 0, "new opens nothing");
    assert_eq!(lazy.loads, 0);

    lazy.get().unwrap();
    lazy.get().unwrap();
    assert_eq!(factory.opens.get(), 1, "get opens once");
    assert_eq!(lazy.loads, 1);
    assert_eq!(factory.preflights.get(), 1);

    // A missing model fails in `new`, before preflight or open (FR-9.6).
    let empty = TempModelCache::new();
    let factory = FakeFactory::default();
    let err = LazyRewriter::new(&factory, &empty.settings())
        .err()
        .unwrap();
    assert!(matches!(err, Error::ModelMissing { .. }), "{err:?}");
    assert_eq!(factory.preflights.get(), 0);
    assert_eq!(factory.opens.get(), 0);
}

/// Serves `body` once over HTTP/1.1 on a local port; returns the base URL and the server thread.
fn serve_once(body: Vec<u8>) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let handle = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut req = Vec::new();
        let mut byte = [0u8; 1];
        while !req.ends_with(b"\r\n\r\n") {
            if stream.read(&mut byte).unwrap() == 0 {
                break;
            }
            req.push(byte[0]);
        }
        let head = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream.write_all(head.as_bytes()).unwrap();
        stream.write_all(&body).unwrap();
    });
    (base, handle)
}

fn hex_sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// A registry-shaped entry for `content`, with `sha256` as its expected digest.
fn test_spec(content: &[u8], sha256: String) -> &'static ModelSpec {
    Box::leak(Box::new(ModelSpec {
        name: "test-model",
        repo: "test/Test-GGUF",
        file: "test.gguf",
        sha256: Box::leak(sha256.into_boxed_str()),
        bytes: content.len() as u64,
        licence: "mit",
        profile: &model::registry::QWEN3,
    }))
}

/// Every path under `dir` whose name ends in `.part`.
fn parts(dir: &Path) -> Vec<PathBuf> {
    let mut all = Vec::new();
    files(dir, &mut all);
    all.into_iter()
        .filter(|p| p.to_string_lossy().ends_with(".part"))
        .collect()
}

#[test]
fn pull_from_local_server_verifies_sha256() {
    let content = b"GGUF pretend weights for the pull test".repeat(1000);
    let spec = test_spec(&content, hex_sha256(&content));
    let (base, server) = serve_once(content.clone());
    let cache = tempfile::tempdir().unwrap();
    let mut progress = Vec::new();

    let path = model::pull::pull(spec, cache.path(), &base, &mut progress).unwrap();
    server.join().unwrap();

    assert_eq!(path, model::cached_path(cache.path(), spec));
    assert_eq!(fs::read(&path).unwrap(), content);
    let sidecar = fs::read_to_string(model::sidecar_path(&path)).unwrap();
    assert_eq!(sidecar.split_whitespace().next(), Some(spec.sha256));
    assert!(parts(cache.path()).is_empty(), "a .part remains");
    assert!(model::up_to_date(cache.path(), spec));
    let progress = String::from_utf8(progress).unwrap();
    assert!(progress.contains("100%"), "{progress}");
}

#[test]
fn pull_rejects_corrupt_file_and_removes_part() {
    let good = b"the real weights".to_vec();
    let mut corrupt = good.clone();
    corrupt[0] ^= 0xff; // same length, different bytes
    let spec = test_spec(&good, hex_sha256(&good));
    let (base, server) = serve_once(corrupt);
    let cache = tempfile::tempdir().unwrap();

    let err = model::pull::pull(spec, cache.path(), &base, &mut Vec::new()).unwrap_err();
    server.join().unwrap();

    match &err {
        Error::Download { url, reason } => {
            assert_eq!(
                url,
                &format!("{base}/test/Test-GGUF/resolve/main/test.gguf")
            );
            assert!(reason.contains("sha256 mismatch"), "{reason}");
        }
        other => panic!("expected Error::Download, got {other:?}"),
    }
    assert!(err.to_string().contains(&base), "{err}");
    assert!(parts(cache.path()).is_empty(), "a .part remains");
    let path = model::cached_path(cache.path(), spec);
    assert!(!path.exists(), "a corrupt file was installed");
    assert!(!model::sidecar_path(&path).exists());
    assert!(!model::up_to_date(cache.path(), spec));
}

/// Runs the CLI in-process with `cwd` as the working directory.
fn run_in(cwd: &Path, args: &[&str]) -> (ExitCode, String, String) {
    let cli =
        Cli::try_parse_from(std::iter::once("sloperator").chain(args.iter().copied())).unwrap();
    let mut deps = Deps {
        rewriter_factory: Box::new(DisabledFactory),
        fs: Box::new(sloperator::write::RealFs),
        cwd: cwd.to_path_buf(),
        color: false,
    };
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = cli::run(cli, &mut deps, &mut "".as_bytes(), &mut out, &mut err);
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

#[test]
fn list_and_path_report_presence() {
    let cache = TempModelCache::new();
    let project = tempfile::tempdir().unwrap();
    fs::write(
        project.path().join("sloperator.toml"),
        format!("[model]\ncache_dir = '{}'\n", cache.dir.path().display()),
    )
    .unwrap();
    let spec = model::spec(DEFAULT_MODEL).unwrap();
    let path = model::cached_path(cache.dir.path(), spec);

    // Absent: list says so, path exits 2 with ModelMissing.
    let (code, out, _) = run_in(project.path(), &["model", "list"]);
    assert_eq!(code, ExitCode::Clean);
    for m in REGISTRY {
        let line = out
            .lines()
            .find(|l| l.starts_with(&format!("{}\t", m.name)))
            .unwrap_or_else(|| panic!("{} not listed:\n{out}", m.name));
        assert!(line.contains(m.file), "{line}");
        assert!(line.contains(&m.bytes.to_string()), "{line}");
        assert!(line.contains(m.licence), "{line}");
        assert!(line.contains("absent"), "{line}");
    }
    let (code, out, err) = run_in(project.path(), &["model", "path"]);
    assert_eq!(code, ExitCode::Failure);
    assert!(out.is_empty(), "{out}");
    assert!(
        err.contains(&format!("sloperator model pull {DEFAULT_MODEL}")),
        "{err}"
    );

    // Present: list and path report it.
    cache.plant(spec);
    let (code, out, _) = run_in(project.path(), &["model", "list"]);
    assert_eq!(code, ExitCode::Clean);
    let line = out
        .lines()
        .find(|l| l.starts_with(&format!("{DEFAULT_MODEL}\t")))
        .unwrap();
    assert!(line.contains("present"), "{line}");
    assert!(line.contains(&path.display().to_string()), "{line}");
    let (code, out, _) = run_in(project.path(), &["model", "path"]);
    assert_eq!(code, ExitCode::Clean);
    assert_eq!(out.trim(), path.display().to_string());
    let (code, out, _) = run_in(project.path(), &["model", "path", DEFAULT_MODEL]);
    assert_eq!(code, ExitCode::Clean);
    assert_eq!(out.trim(), path.display().to_string());

    // With a matching sidecar, pull has nothing to do and never reaches the network.
    fs::write(
        model::sidecar_path(&path),
        format!("{}  {}\n", spec.sha256, spec.file),
    )
    .unwrap();
    assert!(model::up_to_date(cache.dir.path(), spec));
    let (code, out, _) = run_in(project.path(), &["model", "pull"]);
    assert_eq!(code, ExitCode::Clean);
    assert!(out.contains("up to date"), "{out}");

    // An unknown name is a usage error.
    let (code, _, err) = run_in(project.path(), &["model", "path", "no-such-model"]);
    assert_eq!(code, ExitCode::Failure);
    assert!(err.contains("unknown model"), "{err}");
}
