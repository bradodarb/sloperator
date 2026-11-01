//! The model worker (ADR-10): one thread that owns the rewriter factory and the loaded model, and serves model-backed
//! fixes one at a time over a channel.
//!
//! The llama.cpp types are not `Send`, so the model never leaves this thread: the factory is built here, the model is
//! opened here on the first fix that needs it (FR-9.4) and dropped here after `idle` with no request. Serving the
//! channel in order is what serialises model use.

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use tokio::sync::oneshot;

use crate::Error;
use crate::engine::{LazyRewriter, RewriterFactory};
use crate::input::Source;
use crate::model;
use crate::pipeline::{self, Engines, FileResult};

/// Builds the rewriter factory on the worker thread, so a factory need not be `Send`.
pub type FactoryFn = Box<dyn FnOnce() -> Box<dyn RewriterFactory> + Send>;

struct Job {
    source: Source,
    reply: oneshot::Sender<Result<FileResult, Error>>,
}

/// The handle the server holds. Dropping it (or [`ModelWorker::shutdown`]) closes the channel, which unloads the model
/// and ends the thread.
pub struct ModelWorker {
    tx: Mutex<Option<Sender<Job>>>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl ModelWorker {
    /// Starts the thread. Nothing is loaded until the first fix. `idle` = `None` keeps the model loaded for good.
    ///
    /// # Errors
    /// `Error::Runtime` when the thread cannot be started.
    pub fn spawn(make: FactoryFn, idle: Option<Duration>, verbose: bool) -> Result<Self, Error> {
        let (tx, rx) = mpsc::channel();
        let thread = std::thread::Builder::new()
            .name("sloperator-model".into())
            .spawn(move || serve(&rx, make, idle, verbose))
            .map_err(|e| Error::Runtime(format!("starting the model worker: {e}")))?;
        Ok(ModelWorker {
            tx: Mutex::new(Some(tx)),
            thread: Mutex::new(Some(thread)),
        })
    }

    /// Runs `pipeline::fix` with both engines on the worker thread and waits for it without blocking the runtime.
    ///
    /// # Errors
    /// Whatever the pipeline returns (a missing model is `Error::ModelMissing`), or `Error::Runtime` when the worker
    /// has stopped.
    pub async fn fix(&self, source: Source) -> Result<FileResult, Error> {
        let stopped = || Error::Runtime("the model worker has stopped".into());
        let (reply, rx) = oneshot::channel();
        {
            let tx = self.tx.lock().map_err(|_| stopped())?;
            tx.as_ref()
                .ok_or_else(stopped)?
                .send(Job { source, reply })
                .map_err(|_| stopped())?;
        }
        rx.await.map_err(|_| stopped())?
    }

    /// Closes the channel and waits for the thread, so the model is freed before the process exits.
    pub fn shutdown(&self) {
        if let Ok(mut tx) = self.tx.lock() {
            tx.take();
        }
        let thread = self.thread.lock().ok().and_then(|mut t| t.take());
        if let Some(t) = thread {
            let _ = t.join();
        }
    }
}

impl Drop for ModelWorker {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// The thread body. `open` is the rewriter for one resolved model path; a request for another model replaces it.
fn serve(rx: &Receiver<Job>, make: FactoryFn, idle: Option<Duration>, verbose: bool) {
    let factory = make();
    let mut open: Option<(PathBuf, LazyRewriter<'_>)> = None;
    loop {
        let job = match (&open, idle) {
            (Some(_), Some(after)) => match rx.recv_timeout(after) {
                Ok(job) => job,
                Err(RecvTimeoutError::Timeout) => {
                    if let Some((path, rw)) = open.take()
                        && verbose
                        && rw.loads > 0
                    {
                        eprintln!(
                            "sloperator mcp: model unloaded after {}s idle ({})",
                            after.as_secs(),
                            path.display()
                        );
                    }
                    continue;
                }
                Err(RecvTimeoutError::Disconnected) => break,
            },
            _ => match rx.recv() {
                Ok(job) => job,
                Err(_) => break,
            },
        };
        let result = fix(factory.as_ref(), &mut open, job.source, verbose);
        // The caller may have gone away (a cancelled request); the result is then dropped.
        let _ = job.reply.send(result);
    }
}

fn fix<'f>(
    factory: &'f dyn RewriterFactory,
    open: &mut Option<(PathBuf, LazyRewriter<'f>)>,
    source: Source,
    verbose: bool,
) -> Result<FileResult, Error> {
    let settings = &source.settings.model;
    let path = model::resolve(settings)?.path; // ModelMissing names `sloperator model pull`
    if open.as_ref().is_none_or(|(p, _)| *p != path) {
        *open = None; // free the old model before the new one loads
        *open = Some((path.clone(), LazyRewriter::new(factory, settings)?));
    }
    let seed = settings.seed;
    let analysis = pipeline::analyse(source)?;
    let Some((_, rw)) = open.as_mut() else {
        return Err(Error::Runtime("model slot empty".into()));
    };
    let before = rw.loads;
    let t = Instant::now();
    let engines = Engines {
        rules: true,
        model: true,
    };
    let result = pipeline::fix(analysis, engines, Some(rw), seed);
    if verbose && rw.loads > before {
        eprintln!(
            "sloperator mcp: model loaded ({}) in {:.1} s",
            path.display(),
            t.elapsed().as_secs_f64()
        );
    }
    result
}
