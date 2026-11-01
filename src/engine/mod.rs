//! The rewriter port (SPEC §16, ADR-02): the port, the model-less factory and the lazy loader, plus window
//! selection (`window`), placeholder masking and the output contract (`mask`) and the prompt (`prompt`).
//! The llama-cpp-2 adapter (`runtime`, feature `model`) is the real factory; `DisabledFactory` serves other builds.

use crate::Error;
use crate::config::ModelSettings;
use crate::model::{PromptProfile, ResolvedModel};

pub mod mask;
pub mod prompt;
#[cfg(feature = "model")]
pub mod runtime;
pub mod window;

/// The sampling seed when neither `model.seed` nor `--seed` is set (FR-9.8). `config` owns the value.
pub use crate::config::DEFAULT_SEED;

/// What a rewrite call is given.
#[derive(Debug)]
pub struct RewriteRequest<'a> {
    pub system: &'static str,
    pub few_shot: &'static [(&'static str, &'static str)],
    pub window: &'a str,
    pub profile: &'static PromptProfile,
    pub max_tokens: u32,
    pub seed: u32,
}

/// The port (ADR-02). Synchronous, one per process, and it owns the loaded model.
pub trait Rewriter {
    fn rewrite(&mut self, req: &RewriteRequest<'_>) -> Result<String, Error>;
}

pub trait RewriterFactory {
    /// A cheap check that runs before any file is processed (Fix/Diff with the model engine on). It loads nothing.
    fn preflight(&self, m: &ResolvedModel) -> Result<(), Error> {
        let _ = m;
        Ok(())
    }
    fn open(&self, m: &ResolvedModel) -> Result<Box<dyn Rewriter>, Error>;
}

/// The factory in a build without a runtime. Preflight fails, so the run exits 2 before doing any work.
pub struct DisabledFactory;

impl RewriterFactory for DisabledFactory {
    fn preflight(&self, _: &ResolvedModel) -> Result<(), Error> {
        Err(Error::ModelUnsupported)
    }
    fn open(&self, _: &ResolvedModel) -> Result<Box<dyn Rewriter>, Error> {
        Err(Error::ModelUnsupported)
    }
}

/// The factory this build serves: the llama-cpp-2 runtime with the `model` feature, else [`DisabledFactory`].
/// `verbose` keeps the runtime's own logs on stderr.
pub fn default_factory(verbose: bool) -> Box<dyn RewriterFactory> {
    #[cfg(feature = "model")]
    {
        Box::new(runtime::RuntimeFactory::new(verbose))
    }
    #[cfg(not(feature = "model"))]
    {
        let _ = verbose;
        Box::new(DisabledFactory)
    }
}

/// Resolves the model and runs preflight eagerly (FR-9.6). The model itself is opened on the first `get` (FR-9.4).
pub struct LazyRewriter<'f> {
    factory: &'f dyn RewriterFactory,
    model: ResolvedModel,
    open: Option<Box<dyn Rewriter>>,
    /// How many times the model was opened: 0 until the first `get`, then 1.
    pub loads: u32,
}

impl<'f> LazyRewriter<'f> {
    pub fn new(factory: &'f dyn RewriterFactory, s: &ModelSettings) -> Result<Self, Error> {
        let model = crate::model::resolve(s)?; // ModelMissing → exit 2, naming `model pull` and --rules-only
        factory.preflight(&model)?;
        Ok(LazyRewriter {
            factory,
            model,
            open: None,
            loads: 0,
        })
    }

    pub fn profile(&self) -> &'static PromptProfile {
        self.model.profile
    }

    pub fn get(&mut self) -> Result<&mut dyn Rewriter, Error> {
        if self.open.is_none() {
            self.open = Some(self.factory.open(&self.model)?);
            self.loads += 1;
        }
        match self.open.as_deref_mut() {
            Some(r) => Ok(r),
            None => Err(Error::Runtime("model not open".into())),
        }
    }
}
