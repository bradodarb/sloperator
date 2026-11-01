//! `sloperator mcp` (ADR-10): an MCP server over stdio that exposes `lint`, `fix`, `rules` and `explain_rule`.
//!
//! A thin adapter like `cli`: every tool runs the shared pipeline (`pipeline::analyse` → `lint` / `fix`) on the text
//! it is given and returns JSON. Text in, text out: the server never reads or writes the file a `path` names; `path`
//! only picks the format and the `sloperator.toml` that applies. stdout carries the protocol; logs go to stderr.
//! This module and nothing else names the MCP SDK (tests/layering.rs).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, ErrorData,
    Implementation, JsonObject, ListToolsResult, PaginatedRequestParams, ServerCapabilities,
    ServerConfig, Tool, ToolAnnotations,
};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, ServerHandler, ServiceExt};
use serde_json::{Value, json};

use crate::Error;
use crate::config::{CliOverrides, Resolver};
use crate::input::{self, Origin, Source};
use crate::pipeline::{self, Engines, FileResult, Located};
use crate::rules::RuleCode;
use crate::rules::registry::Registry;

mod worker;
pub use worker::{FactoryFn, ModelWorker};

/// The largest `text` a tool accepts.
pub const MAX_INPUT_BYTES: usize = 1 << 20;

/// The default `--idle-unload`, in seconds.
pub const DEFAULT_IDLE_UNLOAD_SECS: u64 = 600;

/// How the server runs.
#[derive(Clone, Debug)]
pub struct Options {
    /// Never load the model; `fix` uses the rules engine only.
    pub rules_only: bool,
    /// Unload the model after this long with no model-backed fix; `None` keeps it loaded.
    pub idle_unload: Option<Duration>,
    /// Model load/unload notes on stderr.
    pub verbose: bool,
    /// Relative `path` hints are joined onto this directory.
    pub cwd: PathBuf,
}

/// The tool handler. Cheap to clone; clones share the model worker.
#[derive(Clone)]
pub struct Server {
    opts: Arc<Options>,
    worker: Option<Arc<ModelWorker>>,
}

impl Server {
    /// A server; with the model engine on, the model worker thread starts (and loads nothing yet).
    ///
    /// # Errors
    /// `Error::Runtime` when the worker thread cannot be started.
    pub fn new(opts: Options, make: FactoryFn) -> Result<Server, Error> {
        let worker = if opts.rules_only {
            None
        } else {
            Some(Arc::new(ModelWorker::spawn(
                make,
                opts.idle_unload,
                opts.verbose,
            )?))
        };
        Ok(Server {
            opts: Arc::new(opts),
            worker,
        })
    }

    /// Unloads the model and stops the worker; later model-backed fixes fail.
    pub fn shutdown(&self) {
        if let Some(w) = &self.worker {
            w.shutdown();
        }
    }

    /// Runs one tool. A bad argument or a pipeline failure is a tool error (`isError: true`); an unknown tool name is
    /// a protocol error.
    ///
    /// # Errors
    /// `ErrorData::invalid_params` for an unknown tool name.
    pub async fn call(&self, name: &str, args: JsonObject) -> Result<CallToolResult, ErrorData> {
        let result = match name {
            "lint" => self.lint(&args).await,
            "fix" => self.fix(&args).await,
            "rules" => Ok(rules_json()),
            "explain_rule" => explain_json(&args),
            other => {
                return Err(ErrorData::invalid_params(
                    format!("unknown tool `{other}`"),
                    None,
                ));
            }
        };
        Ok(match result {
            Ok(v) => CallToolResult::structured(v),
            Err(msg) => CallToolResult::error(vec![ContentBlock::text(msg)]),
        })
    }

    async fn lint(&self, args: &JsonObject) -> Result<Value, String> {
        let source = self.source(args)?;
        let r = blocking(move || pipeline::analyse(source).map(pipeline::lint)).await?;
        Ok(json!({
            "score": r.score,
            "findings": r.findings.iter().map(finding_json).collect::<Vec<_>>(),
        }))
    }

    async fn fix(&self, args: &JsonObject) -> Result<Value, String> {
        let source = self.source(args)?;
        let as_path = match &source.origin {
            Origin::Stdin { as_path } | Origin::File(as_path) => as_path.clone(),
        };
        let rules_only = match args.get("rules_only") {
            None | Some(Value::Null) => false,
            Some(Value::Bool(b)) => *b,
            Some(_) => return Err("`rules_only` must be a boolean".into()),
        };
        let r = match (&self.worker, rules_only) {
            (Some(w), false) => w.fix(source).await.map_err(|e| e.to_string())?,
            _ => {
                blocking(move || {
                    let seed = source.settings.model.seed;
                    let engines = Engines {
                        rules: true,
                        model: false,
                    };
                    pipeline::analyse(source).and_then(|a| pipeline::fix(a, engines, None, seed))
                })
                .await?
            }
        };
        Ok(fix_json(&r, &as_path, self.worker.is_some() && !rules_only))
    }

    /// The `text` and `path` arguments as a stdin-like source: `path` picks the format and the settings, nothing more.
    fn source(&self, args: &JsonObject) -> Result<Source, String> {
        let text = match args.get("text") {
            Some(Value::String(t)) => t,
            Some(_) => return Err("`text` must be a string".into()),
            None => return Err("missing required argument `text`".into()),
        };
        if text.len() > MAX_INPUT_BYTES {
            return Err(format!(
                "`text` is {} bytes; the limit is {MAX_INPUT_BYTES} bytes (1 MiB)",
                text.len()
            ));
        }
        let as_path = match args.get("path") {
            None | Some(Value::Null) => PathBuf::from("-"),
            Some(Value::String(p)) if !p.is_empty() => PathBuf::from(p),
            Some(_) => return Err("`path` must be a non-empty string".into()),
        };
        let mut resolver =
            Resolver::new(CliOverrides::default(), None).map_err(|e| e.to_string())?;
        let settings = resolver
            .for_file(&self.opts.cwd.join(&as_path))
            .map_err(|e| e.to_string())?;
        Ok(Source {
            format: input::format_for(&as_path),
            origin: Origin::Stdin { as_path },
            text: text.as_str().into(),
            settings,
        })
    }
}

/// Runs CPU-bound pipeline work off the async runtime.
async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, Error> + Send + 'static,
) -> Result<T, String> {
    match tokio::task::spawn_blocking(f).await {
        Ok(r) => r.map_err(|e| e.to_string()),
        Err(e) => Err(format!("internal error: {e}")),
    }
}

fn finding_json(l: &Located) -> Value {
    let name = Registry::builtin()
        .get(l.finding.code)
        .map_or("", |r| r.meta().name);
    json!({
        "code": l.finding.code,
        "name": name,
        "severity": l.finding.severity,
        "line": l.start.0,
        "col": l.start.1,
        "endLine": l.end.0,
        "endCol": l.end.1,
        "message": l.finding.message,
        "matched": l.matched,
    })
}

fn fix_json(r: &FileResult, as_path: &Path, model: bool) -> Value {
    let fixed = r.fixed.as_deref().unwrap_or(&r.original);
    let diff = match &r.fixed {
        Some(f) => crate::edit::unified_diff(as_path, &r.original, f),
        None => String::new(),
    };
    let (rejections, findings): (Vec<&Located>, Vec<&Located>) =
        r.findings.iter().partition(|l| l.finding.code.is_system());
    json!({
        "changed": r.fixed.is_some(),
        "fixed": fixed,
        "diff": diff,
        "findings": findings.into_iter().map(finding_json).collect::<Vec<_>>(),
        "rejections": rejections.into_iter().map(finding_json).collect::<Vec<_>>(),
        "scoreBefore": r.score_before,
        "scoreAfter": r.score,
        "engines": { "rules": true, "model": model },
    })
}

fn rules_json() -> Value {
    let reg = Registry::builtin();
    let rules: Vec<Value> = reg
        .all()
        .map(|r| {
            let m = r.meta();
            json!({
                "code": m.code,
                "name": m.name,
                "severity": m.severity,
                "defaultOn": m.default_on,
                "fix": m.fix,
            })
        })
        .collect();
    json!({ "ruleSetVersion": reg.version, "rules": rules })
}

fn explain_json(args: &JsonObject) -> Result<Value, String> {
    let code = match args.get("code") {
        Some(Value::String(c)) => c.as_str(),
        _ => return Err("missing required string argument `code`".into()),
    };
    let unknown = || format!("unknown rule code `{code}`; the `rules` tool lists them");
    let parsed: RuleCode = code.parse().map_err(|_| unknown())?;
    let m = Registry::builtin().get(parsed).ok_or_else(unknown)?.meta();
    Ok(json!({
        "code": m.code,
        "name": m.name,
        "severity": m.severity,
        "defaultOn": m.default_on,
        "fix": m.fix,
        "explain": m.explain,
        "params": m.params.iter().map(|p| json!({
            "name": p.name, "default": p.default, "help": p.help,
        })).collect::<Vec<_>>(),
        "example": { "before": m.example.0, "after": m.example.1 },
    }))
}

const TEXT_PROP: &str = "The prose to check, as a string (at most 1 MiB).";
const PATH_PROP: &str = "Optional file name the text stands for. Only a hint: it picks the format (.txt is plain \
                         text, anything else Markdown) and which sloperator.toml applies. The file is never read or \
                         written.";

/// The four tools with their input schemas.
pub fn tools() -> Vec<Tool> {
    let read_only = || ToolAnnotations::new().read_only(true);
    let schema = |v: Value| match v {
        Value::Object(m) => Arc::new(m),
        _ => Arc::new(JsonObject::new()),
    };
    vec![
        Tool::new(
            "lint",
            "Lint prose for patterns typical of LLM-generated text. Returns the findings (code, severity, line, col, \
             message) and the slop score (weighted findings per 1,000 prose words; lower is better). Text in, text \
             out: nothing is written to disk.",
            schema(json!({
                "type": "object",
                "properties": {
                    "text": { "type": "string", "description": TEXT_PROP },
                    "path": { "type": "string", "description": PATH_PROP },
                },
                "required": ["text"],
            })),
        )
        .with_annotations(read_only()),
        Tool::new(
            "fix",
            "Rewrite prose so it reads as human-written: the rules engine's safe fixes, then (unless rules_only) a \
             local model rewrite of flagged passages, every edit verified to keep numbers, URLs, code and terms. \
             Returns the fixed text, a unified diff, the remaining findings, the rejected rewrites (SLP901/SLP902) and \
             the score before and after. Nothing is written to disk: you decide whether to apply the text. A model \
             rewrite can pass verification yet flatten a deliberate voice, so show the user the diff before applying \
             it.",
            schema(json!({
                "type": "object",
                "properties": {
                    "text": { "type": "string", "description": TEXT_PROP },
                    "path": { "type": "string", "description": PATH_PROP },
                    "rules_only": {
                        "type": "boolean",
                        "default": false,
                        "description": "Use only the rules engine's safe fixes; never load the model.",
                    },
                },
                "required": ["text"],
            })),
        )
        .with_annotations(read_only()),
        Tool::new(
            "rules",
            "List every rule: code, name, default severity, whether it is on by default, and its fix class.",
            schema(json!({ "type": "object", "properties": {} })),
        )
        .with_annotations(read_only()),
        Tool::new(
            "explain_rule",
            "Explain one rule: what it flags, its parameters and a before/after example.",
            schema(json!({
                "type": "object",
                "properties": {
                    "code": { "type": "string", "description": "A rule code such as SLP001." },
                },
                "required": ["code"],
            })),
        )
        .with_annotations(read_only()),
    ]
}

impl ServerHandler for Server {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("sloperator", env!("CARGO_PKG_VERSION")))
            .with_instructions(
                "Lint and rewrite LLM-generated prose. Tools take text and return text; the server never writes \
                 files.",
            )
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(tools()))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let args = request.arguments.unwrap_or_default();
        self.call(&request.name, args).await.map(Into::into)
    }
}

/// Serves MCP on stdin/stdout until the client disconnects, then unloads the model.
///
/// # Errors
/// `Error::Runtime` when the async runtime cannot start or the protocol fails.
pub fn run(opts: Options, make: FactoryFn) -> Result<(), Error> {
    let verbose = opts.verbose;
    let server = Server::new(opts, make)?;
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all() // rmcp uses tokio timers for request timeouts
        .build()
        .map_err(|e| Error::Runtime(format!("starting the async runtime: {e}")))?;
    let result = rt.block_on(async {
        let running = server
            .clone()
            .serve(rmcp::transport::stdio())
            .await
            .map_err(|e| Error::Runtime(format!("mcp: {e}")))?;
        if verbose {
            eprintln!("sloperator mcp: serving on stdio");
        }
        running
            .waiting()
            .await
            .map_err(|e| Error::Runtime(format!("mcp: {e}")))?;
        Ok(())
    });
    server.shutdown();
    result
}
