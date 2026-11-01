//! The command line (SPEC §20): `check`, `rules`, `config` and `model`, and the one place the exit code is decided.
//!
//! `check` resolves the inputs and their settings, analyses the files in parallel, lints them or fixes them serially
//! in sorted order (ADR-09), prints a diff or commits the fixes, renders the report and picks the exit code: 2 on any
//! error, else 0 with `--exit-zero`, else 1 when anything was reported, else 0 (FR-2.7, FR-2.8, D-8).

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use rayon::prelude::*;

mod args;
pub use args::{CheckArgs, Cli, Command, ModelCmd};

use crate::Error;
use crate::config::{CONFIG_FILE_NAME, CliOverrides, ModelSettings, Resolver};
use crate::engine::LazyRewriter;
use crate::input::{self, Origin, SourceRef};
use crate::model;
use crate::pipeline::{self, Analysis, Engines, FileResult, Mode};
use crate::report::{self, Summary};
use crate::rules::registry::Registry;
use crate::rules::{FixClass, RuleCode, Severity};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitCode {
    Clean = 0,
    Findings = 1,
    Failure = 2,
}

/// The injectable seams (ADR-02, architecture §11).
pub struct Deps {
    pub rewriter_factory: Box<dyn crate::engine::RewriterFactory>,
    pub fs: Box<dyn crate::write::Fs>,
    pub cwd: PathBuf,
    pub color: bool,
}

/// The exit code for a clap parse failure: help and version are 0 (FR-2.16), everything else is 2 (FR-2.15).
pub fn parse_error_exit(e: &clap::Error) -> ExitCode {
    match e.kind() {
        clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => {
            ExitCode::Clean
        }
        _ => ExitCode::Failure,
    }
}

/// The only place where the exit code is decided.
pub fn run(
    cli: Cli,
    deps: &mut Deps,
    stdin: &mut dyn Read,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> ExitCode {
    let ui = Ui {
        verbose: cli.verbose,
        quiet: cli.quiet,
    };
    let result = match cli.cmd {
        Command::Check(args) => check(&args, ui, deps, stdin, out, err),
        Command::Rules { code } => show_rules(code.as_deref(), out),
        Command::Config { path, config } => {
            show_config(path.as_deref(), config.as_deref(), deps, out)
        }
        Command::Model(cmd) => model_cmd(cmd, ui, deps, out, err),
    };
    match result {
        Ok(code) => code,
        Err(e) => {
            // A failed write to stderr cannot be reported anywhere else; the exit code still carries the failure.
            let _ = writeln!(err, "sloperator: {e}");
            ExitCode::Failure
        }
    }
}

/// `-v` / `-q` (SPEC §20 design notes): `-v` adds stage timings on stderr, `-q` prints errors only.
#[derive(Clone, Copy)]
struct Ui {
    verbose: u8,
    quiet: bool,
}

impl Ui {
    fn timing(self, err: &mut dyn Write, stage: &str, since: Instant) {
        if self.verbose > 0 {
            let _ = writeln!(
                err,
                "sloperator: {stage} {:.1} ms",
                since.elapsed().as_secs_f64() * 1000.0
            );
        }
    }
}

/// Steps 1-10 of SPEC §20. A whole-run error (bad config, bad flag code, missing path, missing model) is `Err` and
/// exits 2 before any file is touched; per-file errors are printed and also exit 2, and then nothing is written
/// (DQ-10).
fn check(
    args: &CheckArgs,
    ui: Ui,
    deps: &Deps,
    stdin: &mut dyn Read,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Result<ExitCode, Error> {
    // 1. Settings: CLI overrides and the explicit --config chain.
    let t = Instant::now();
    let overrides = CliOverrides {
        select: args.select.clone(),
        extend_select: args.extend_select.clone(),
        ignore: args.ignore.clone(),
        min_severity: args.min_severity,
        seed: args.seed,
    };
    let explicit = args.config.as_ref().map(|p| deps.cwd.join(p));
    let mut resolver = Resolver::new(overrides, explicit.as_deref())?;

    // 2. Inputs, and stdin when `-` is among them.
    let mut sink = std::io::sink();
    let notes: &mut dyn Write = if ui.quiet { &mut sink } else { &mut *err };
    let refs = input::resolve(
        &args.paths,
        args.stdin_filename.as_deref(),
        &deps.cwd,
        &mut resolver,
        notes,
    )?;
    let stdin_text = match refs.iter().find_map(|r| match &r.origin {
        Origin::Stdin { as_path } => Some(as_path.clone()),
        Origin::File(_) => None,
    }) {
        Some(as_path) => Some(read_stdin(stdin, as_path)?),
        None => None,
    };
    ui.timing(err, "resolve", t);

    // 3. Mode and engines (FR-2.10; clap rejects --fix with --diff and --rules-only with --model-only).
    let mode = if args.fix {
        Mode::Fix
    } else if args.diff {
        Mode::Diff
    } else {
        Mode::Lint
    };
    let engines = Engines {
        rules: !args.model_only,
        model: !args.rules_only,
    };

    // 4. The model is resolved and preflighted before any file is read (FR-9.6, D-5); it loads on first use.
    let mut rw = match (mode, refs.first()) {
        (Mode::Fix | Mode::Diff, Some(first)) if engines.model => Some(LazyRewriter::new(
            deps.rewriter_factory.as_ref(),
            &first.settings.model,
        )?),
        _ => None,
    };

    // 5. Read and analyse in parallel; the collected Vec keeps the sorted `Origin` order (ADR-09).
    let t = Instant::now();
    let analysed: Vec<Result<Analysis, Error>> = refs
        .par_iter()
        .map(|r| analyse(r, stdin_text.as_deref()))
        .collect();
    ui.timing(err, "analyse", t);

    // 6-7. Lint, or fix serially in sorted order. Each per-file error is printed and fails the run.
    let t = Instant::now();
    let mut results: Vec<FileResult> = Vec::new();
    let mut errors = 0;
    for a in analysed {
        let r = a.and_then(|a| match mode {
            Mode::Lint => Ok(pipeline::lint(a)),
            Mode::Fix | Mode::Diff => {
                let seed = a.source.settings.model.seed;
                pipeline::fix(a, engines, rw.as_mut(), seed)
            }
        });
        match r {
            Ok(r) => results.push(r),
            Err(e) => {
                let _ = writeln!(err, "sloperator: {e}");
                errors += 1;
            }
        }
    }
    ui.timing(err, if mode == Mode::Lint { "lint" } else { "fix" }, t);

    // 8. Diff prints the changes; Fix commits them only when nothing failed (DQ-10). Stdin's text goes to `out`.
    let mut content_on_out = false;
    match mode {
        Mode::Lint => {}
        Mode::Diff => {
            content_on_out = true;
            for r in &results {
                if let Some(fixed) = &r.fixed {
                    out.write_all(
                        crate::edit::unified_diff(origin_path(&r.origin), &r.original, fixed)
                            .as_bytes(),
                    )
                    .map_err(stdout_error)?;
                }
            }
        }
        Mode::Fix => {
            content_on_out = results
                .iter()
                .any(|r| matches!(r.origin, Origin::Stdin { .. }));
            if errors == 0
                && let Err(e) = crate::write::commit(&results, deps.fs.as_ref(), out)
            {
                let _ = writeln!(err, "sloperator: {e}");
                errors += 1;
            }
        }
    }

    // 9. The report goes to stderr when stdout carries content (DQ-11); `-q` prints errors only.
    if !ui.quiet {
        let summary = Summary::of(&results, errors);
        let rendered = if content_on_out {
            report::render(args.output_format, &results, &summary, false, err)
        } else {
            report::render(args.output_format, &results, &summary, deps.color, out)
        };
        rendered.map_err(stdout_error)?;
    }

    // 10. The exit code.
    let reported = results.iter().any(|r| !r.findings.is_empty());
    Ok(if errors > 0 {
        ExitCode::Failure
    } else if args.exit_zero || !reported {
        ExitCode::Clean
    } else {
        ExitCode::Findings
    })
}

fn analyse(r: &SourceRef, stdin_text: Option<&str>) -> Result<Analysis, Error> {
    pipeline::analyse(input::read(r, stdin_text)?)
}

fn read_stdin(stdin: &mut dyn Read, as_path: PathBuf) -> Result<String, Error> {
    let mut bytes = Vec::new();
    stdin.read_to_end(&mut bytes).map_err(|source| Error::Io {
        path: as_path.clone(),
        source,
    })?;
    String::from_utf8(bytes).map_err(|e| Error::NotUtf8 {
        path: as_path,
        offset: e.utf8_error().valid_up_to(),
    })
}

fn origin_path(origin: &Origin) -> &Path {
    match origin {
        Origin::File(p) => p,
        Origin::Stdin { as_path } => as_path,
    }
}

fn stdout_error(source: std::io::Error) -> Error {
    Error::Io {
        path: PathBuf::from("<stdout>"),
        source,
    }
}

/// `rules`: one line per rule (code, name, default severity, default state, fix class) and the rule-set version.
/// `rules CODE`: the rule's explanation, parameters and before/after example. An unknown code is exit 2 (FR-2.12,
/// FR-2.15).
fn show_rules(code: Option<&str>, out: &mut dyn Write) -> Result<ExitCode, Error> {
    let reg = Registry::builtin();
    let text = match code {
        None => {
            let mut s = format!("rule set {}\n", reg.version);
            for r in reg.all() {
                let m = r.meta();
                s.push_str(&format!(
                    "{}  {:<32} {:<10} {:<3}  {}\n",
                    m.code,
                    m.name,
                    severity_name(m.severity),
                    if m.default_on { "on" } else { "off" },
                    fix_name(m.fix)
                ));
            }
            s
        }
        Some(code) => {
            let unknown = || Error::UnknownRule {
                code: code.to_string(),
                where_: None,
            };
            let parsed: RuleCode = code.parse().map_err(|_| unknown())?;
            let rule = reg.get(parsed).ok_or_else(unknown)?;
            let m = rule.meta();
            let mut s = format!(
                "{} {}\nseverity: {}  default: {}  fix: {}\n\n{}\n",
                m.code,
                m.name,
                severity_name(m.severity),
                if m.default_on { "on" } else { "off" },
                fix_name(m.fix),
                m.explain
            );
            if !m.params.is_empty() {
                s.push_str("\nparameters:\n");
                for p in m.params {
                    s.push_str(&format!("  {} = {}  {}\n", p.name, p.default, p.help));
                }
            }
            s.push_str(&format!(
                "\nexample:\n  before: {}\n  after:  {}\n",
                m.example.0, m.example.1
            ));
            s
        }
    };
    out.write_all(text.as_bytes()).map_err(stdout_error)?;
    Ok(ExitCode::Clean)
}

/// `config [PATH]`: the settings that apply to PATH (a file, or a directory's files; default the working directory)
/// as pretty JSON with snake_case keys, like the file (FR-2.13, DQ-9).
fn show_config(
    path: Option<&Path>,
    explicit: Option<&Path>,
    deps: &Deps,
    out: &mut dyn Write,
) -> Result<ExitCode, Error> {
    let explicit = explicit.map(|p| deps.cwd.join(p));
    let mut resolver = Resolver::new(CliOverrides::default(), explicit.as_deref())?;
    let target = path.map_or_else(|| deps.cwd.clone(), |p| deps.cwd.join(p));
    let file = if target.is_dir() {
        target.join(CONFIG_FILE_NAME)
    } else {
        target
    };
    let settings = resolver.for_file(&file)?;
    let json = serde_json::to_string_pretty(&*settings).map_err(|e| Error::Io {
        path: PathBuf::from("<stdout>"),
        source: std::io::Error::other(e),
    })?;
    writeln!(out, "{json}").map_err(stdout_error)?;
    Ok(ExitCode::Clean)
}

/// `model pull|list|path` (SPEC §15, FR-2.14). The model settings are the ones that apply to the working directory.
fn model_cmd(
    cmd: ModelCmd,
    ui: Ui,
    deps: &Deps,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Result<ExitCode, Error> {
    let mut resolver = Resolver::new(CliOverrides::default(), None)?;
    let mut settings = resolver
        .for_file(&deps.cwd.join(CONFIG_FILE_NAME))?
        .model
        .clone();
    match cmd {
        ModelCmd::Pull { name } => {
            let name = name.unwrap_or_else(|| settings.name.clone());
            let spec = known_spec(&name)?;
            let dir = model::cache_dir(&settings)?;
            if model::up_to_date(&dir, spec) {
                writeln!(
                    out,
                    "{name}: up to date ({})",
                    model::cached_path(&dir, spec).display()
                )
                .map_err(stdout_error)?;
                return Ok(ExitCode::Clean);
            }
            let mut sink = std::io::sink();
            let progress: &mut dyn Write = if ui.quiet { &mut sink } else { err };
            let path = model::pull::pull(spec, &dir, model::pull::HF_BASE, progress)?;
            writeln!(out, "{}", path.display()).map_err(stdout_error)?;
            Ok(ExitCode::Clean)
        }
        ModelCmd::List => {
            let dir = model::cache_dir(&settings)?;
            for spec in model::REGISTRY {
                let path = model::cached_path(&dir, spec);
                let state = if path.is_file() { "present" } else { "absent" };
                writeln!(
                    out,
                    "{}\t{}\t{} B\t{}\t{state}\t{}",
                    spec.name,
                    spec.file,
                    spec.bytes,
                    spec.licence,
                    path.display()
                )
                .map_err(stdout_error)?;
            }
            Ok(ExitCode::Clean)
        }
        ModelCmd::Path { name } => {
            if let Some(name) = name {
                known_spec(&name)?;
                settings = ModelSettings {
                    name,
                    path: None,
                    profile: None,
                    ..settings
                };
            }
            let resolved = model::resolve(&settings)?; // ModelMissing → exit 2
            writeln!(out, "{}", resolved.path.display()).map_err(stdout_error)?;
            Ok(ExitCode::Clean)
        }
    }
}

/// The registry entry named `name`; an unknown name is a usage error listing the known ones.
fn known_spec(name: &str) -> Result<&'static model::ModelSpec, Error> {
    model::spec(name).ok_or_else(|| {
        Error::Usage(format!(
            "unknown model `{name}` (known: {})",
            model::REGISTRY
                .iter()
                .map(|m| m.name)
                .collect::<Vec<_>>()
                .join(", ")
        ))
    })
}

fn severity_name(s: Severity) -> &'static str {
    match s {
        Severity::Suggestion => "suggestion",
        Severity::Warning => "warning",
        Severity::Error => "error",
    }
}

fn fix_name(f: FixClass) -> &'static str {
    match f {
        FixClass::Safe => "safe",
        FixClass::Model => "model",
        FixClass::None => "none",
    }
}
