//! Shared test infrastructure (SPEC §23). Each test binary uses a subset, so unused items are expected.
#![allow(dead_code, clippy::expect_used)] // test helpers may unwrap (architecture §8)

use std::cell::{Cell, RefCell};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use clap::Parser;
use sloperator::Error;
use sloperator::cli::{self, Cli, Deps, ExitCode};
use sloperator::config::{ModelSettings, WindowMode};
use sloperator::engine::{DisabledFactory, RewriteRequest, Rewriter, RewriterFactory};
use sloperator::model::{self, ModelSpec, ResolvedModel};

/// The path of a file under `fixtures/`.
pub fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(path)
}

/// Runs the CLI in-process with `Vec<u8>` writers; returns the exit code, stdout and stderr.
/// A clap parse failure is rendered to stderr (stdout for help and version) and mapped like `main` does.
pub fn run_cli(args: &[&str], stdin: &str) -> (ExitCode, String, String) {
    run_cli_with(args, stdin, Box::new(DisabledFactory))
}

/// [`run_cli`] with `factory` as the rewriter factory.
pub fn run_cli_with(
    args: &[&str],
    stdin: &str,
    factory: Box<dyn RewriterFactory>,
) -> (ExitCode, String, String) {
    let argv = std::iter::once("sloperator").chain(args.iter().copied());
    let cli = match Cli::try_parse_from(argv) {
        Ok(c) => c,
        Err(e) => {
            let code = cli::parse_error_exit(&e);
            let text = e.render().to_string();
            return match code {
                ExitCode::Clean => (code, text, String::new()),
                _ => (code, String::new(), text),
            };
        }
    };
    let mut deps = Deps {
        rewriter_factory: factory,
        fs: Box::new(sloperator::write::RealFs),
        cwd: PathBuf::from(env!("CARGO_MANIFEST_DIR")),
        color: false,
    };
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = cli::run(cli, &mut deps, &mut stdin.as_bytes(), &mut out, &mut err);
    (
        code,
        String::from_utf8_lossy(&out).into_owned(),
        String::from_utf8_lossy(&err).into_owned(),
    )
}

/// Every request a [`FakeRewriter`] received: the masked window and the seed.
pub type Calls = Rc<RefCell<Vec<(String, u32)>>>;

/// A rewriter that returns a canned output, or the window unchanged when none is set (SPEC §23.12). With `fail`
/// set it returns `Error::Runtime` instead. Every request is recorded in `calls`.
#[derive(Default)]
pub struct FakeRewriter {
    pub output: Option<String>,
    pub fail: Option<String>,
    pub calls: Calls,
}

impl Rewriter for FakeRewriter {
    fn rewrite(&mut self, req: &RewriteRequest<'_>) -> Result<String, Error> {
        self.calls
            .borrow_mut()
            .push((req.window.to_string(), req.seed));
        if let Some(reason) = &self.fail {
            return Err(Error::Runtime(reason.clone()));
        }
        Ok(self
            .output
            .clone()
            .unwrap_or_else(|| req.window.to_string()))
    }
}

/// A factory that counts its calls; `open` hands out a [`FakeRewriter`] with `output` and `fail` that records into
/// `calls`.
#[derive(Default)]
pub struct FakeFactory {
    pub output: Option<String>,
    pub fail: Option<String>,
    pub calls: Calls,
    pub preflights: Cell<u32>,
    pub opens: Cell<u32>,
}

impl FakeFactory {
    /// A factory whose rewriter always returns `output`.
    pub fn returning(output: &str) -> FakeFactory {
        FakeFactory {
            output: Some(output.to_string()),
            ..FakeFactory::default()
        }
    }
}

impl RewriterFactory for FakeFactory {
    fn preflight(&self, _: &ResolvedModel) -> Result<(), Error> {
        self.preflights.set(self.preflights.get() + 1);
        Ok(())
    }
    fn open(&self, _: &ResolvedModel) -> Result<Box<dyn Rewriter>, Error> {
        self.opens.set(self.opens.get() + 1);
        Ok(Box::new(FakeRewriter {
            output: self.output.clone(),
            fail: self.fail.clone(),
            calls: Rc::clone(&self.calls),
        }))
    }
}

/// A [`FakeFactory`] the test keeps a handle on after `run_cli_with` takes the box.
pub struct SharedFactory(pub Rc<FakeFactory>);

impl RewriterFactory for SharedFactory {
    fn preflight(&self, m: &ResolvedModel) -> Result<(), Error> {
        self.0.preflight(m)
    }
    fn open(&self, m: &ResolvedModel) -> Result<Box<dyn Rewriter>, Error> {
        self.0.open(m)
    }
}

/// A temp dir for CLI runs. Its `.gitignore` re-includes everything, so a global ignore rule that matches the temp
/// path cannot make the CLI skip the inputs (the `tests/input.rs` sandbox does the same).
pub fn cli_tempdir() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("temp dir");
    fs::write(tmp.path().join(".gitignore"), "!*\n").expect("write .gitignore");
    tmp
}

/// Writes `sloperator.toml` into `dir` with a `[model]` table whose `path` is a planted placeholder file (a model
/// that resolves without a cache). Returns the model path.
pub fn plant_model_config(dir: &Path) -> PathBuf {
    let model = dir.join("model.gguf");
    fs::write(&model, b"not a real model").expect("plant model file");
    fs::write(
        dir.join("sloperator.toml"),
        format!(
            "[model]\npath = {:?}\nprofile = \"qwen3\"\n",
            model.to_string_lossy()
        ),
    )
    .expect("write sloperator.toml");
    model
}

/// A model cache in a temp directory. Its settings name the default model and point `cache_dir` at it.
pub struct TempModelCache {
    pub dir: tempfile::TempDir,
}

impl TempModelCache {
    pub fn new() -> TempModelCache {
        TempModelCache {
            dir: tempfile::tempdir().expect("temp model cache"),
        }
    }

    /// Model settings with the built-in defaults and this cache as `cache_dir`.
    pub fn settings(&self) -> ModelSettings {
        ModelSettings {
            name: model::DEFAULT_MODEL.to_string(),
            path: None,
            profile: None,
            cache_dir: Some(self.dir.path().to_path_buf()),
            window: WindowMode::Paragraph,
            max_window_chars: 4000,
            max_tokens: 0,
            seed: sloperator::engine::DEFAULT_SEED,
        }
    }

    /// Writes a tiny placeholder file where `spec`'s weights would live; returns its path.
    pub fn plant(&self, spec: &ModelSpec) -> PathBuf {
        let path = model::cached_path(self.dir.path(), spec);
        fs::create_dir_all(path.parent().expect("cached path has a parent"))
            .expect("create cache dir");
        fs::write(&path, b"not a real model").expect("plant model file");
        path
    }
}

/// An `Fs` that fails its Nth write (1-based) and passes every other write to `RealFs` (SPEC §23, NFR-5).
/// `writes` counts the attempts, failed or not.
pub struct FailingFs {
    pub fail_on: u32,
    pub writes: Cell<u32>,
}

impl FailingFs {
    pub fn new(fail_on: u32) -> FailingFs {
        FailingFs {
            fail_on,
            writes: Cell::new(0),
        }
    }
}

impl sloperator::write::Fs for FailingFs {
    fn write_atomic(&self, path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
        let n = self.writes.get() + 1;
        self.writes.set(n);
        if n == self.fail_on {
            return Err(std::io::Error::other(format!("FailingFs: write {n} fails")));
        }
        sloperator::write::RealFs.write_atomic(path, bytes)
    }
}
