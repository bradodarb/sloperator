//! Configuration (SPEC §13): `sloperator.toml` discovery, the `extend` chain, per-file overrides and resolution.
//!
//! Resolution order, lowest to highest (FR-7.5): built-in defaults, the `extend` chain root first, the discovered or
//! `--config` file, every matching `[[per_file]]` entry in file order, then CLI flags. At each layer the rule set is
//! `select` (which replaces the set when present), plus `extend_select`, minus `ignore`. The parent file is never
//! merged implicitly (FR-7.1, D-8): only `extend` pulls another file in.

pub mod file;

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub use crate::doc::{Format, ProtectSettings};
pub use file::WindowMode;

use crate::Error;
use crate::rules::registry::Registry;
use crate::rules::{Enabled, RuleCode, Severity};
use crate::text::metrics::Floors;
use file::{Loaded, config_err};

/// The name discovery looks for in each directory, upward from the input file's directory (FR-7.1).
pub const CONFIG_FILE_NAME: &str = "sloperator.toml";
/// The model used when no config names one (D-6). `model` owns the registry and the default (SPEC §15).
pub const DEFAULT_MODEL_NAME: &str = crate::model::DEFAULT_MODEL;
/// The seed when neither `--seed` nor `model.seed` is given (SPEC §16).
pub const DEFAULT_SEED: u32 = 42;
/// Windows longer than this are skipped with SLP902 (DQ-5).
pub const DEFAULT_MAX_WINDOW_CHARS: usize = 4000;
/// The walker filter when no config sets `include`.
pub const DEFAULT_INCLUDE: &[&str] = &["**/*.md", "**/*.markdown", "**/*.txt"];
/// The `file` that errors in CLI-given codes name (FR-2.11, FR-2.15).
pub const COMMAND_LINE: &str = "<command line>";

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct RuleConf {
    pub severity: Severity,
    pub params: BTreeMap<&'static str, f64>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct ModelSettings {
    pub name: String,
    pub path: Option<PathBuf>,
    pub profile: Option<String>,
    pub cache_dir: Option<PathBuf>,
    pub window: WindowMode,
    pub max_window_chars: usize,
    pub max_tokens: u32,
    pub seed: u32,
}

/// The resolved, immutable settings for one file (architecture §6).
#[derive(Clone, Debug, serde::Serialize)]
pub struct Settings {
    /// The `sloperator.toml` in effect; `None` = built-in defaults.
    pub source: Option<PathBuf>,
    /// Enabled rules only.
    pub rules: BTreeMap<RuleCode, RuleConf>,
    pub min_severity: Severity,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub respect_gitignore: bool,
    pub protect: ProtectSettings,
    pub model: ModelSettings,
    pub metrics: Floors,
}

impl Settings {
    /// The enabled rules of `reg` with their resolved severity and params, in registry order.
    pub fn enabled(&self, reg: &'static Registry) -> Vec<Enabled<'static>> {
        reg.all()
            .filter_map(|rule| {
                self.rules.get(&rule.meta().code).map(|c| Enabled {
                    rule,
                    severity: c.severity,
                    params: c.params.clone(),
                })
            })
            .collect()
    }
}

#[derive(Clone, Debug, Default)]
pub struct CliOverrides {
    pub select: Vec<String>,
    pub extend_select: Vec<String>,
    pub ignore: Vec<String>,
    pub min_severity: Option<Severity>,
    pub seed: Option<u32>,
}

/// One validated rule-selection layer: a file's top level, one `[[per_file]]` entry, or the CLI.
#[derive(Debug, Default)]
struct Selection {
    select: Option<Vec<RuleCode>>,
    extend_select: Vec<RuleCode>,
    ignore: Vec<RuleCode>,
    min_severity: Option<Severity>,
    rules: BTreeMap<RuleCode, RuleOverride>,
}

#[derive(Clone, Debug, Default)]
struct RuleOverride {
    severity: Option<Severity>,
    params: BTreeMap<&'static str, f64>,
}

#[derive(Debug)]
struct PerFileLayer {
    matcher: globset::GlobMatcher,
    dir: PathBuf,
    selection: Selection,
}

/// A loaded chain with every value validated, ready to resolve against any file.
#[derive(Debug)]
struct Chain {
    base: Arc<Settings>,
    per_file: Vec<PerFileLayer>,
    /// The selection state after the chain and before per-file and CLI layers, so per-file entries apply on top.
    state: State,
}

/// The rule set and per-rule overrides as layers apply.
#[derive(Clone, Debug)]
struct State {
    set: BTreeSet<RuleCode>,
    overrides: BTreeMap<RuleCode, RuleOverride>,
    min_severity: Severity,
}

impl State {
    fn apply(&mut self, s: &Selection) {
        if let Some(sel) = &s.select {
            self.set = sel.iter().copied().collect();
        }
        self.set.extend(s.extend_select.iter().copied());
        for c in &s.ignore {
            self.set.remove(c);
        }
        if let Some(m) = s.min_severity {
            self.min_severity = m;
        }
        for (code, o) in &s.rules {
            let cur = self.overrides.entry(*code).or_default();
            if o.severity.is_some() {
                cur.severity = o.severity;
            }
            cur.params.extend(o.params.iter().map(|(k, v)| (*k, *v)));
        }
    }

    fn rules(&self, reg: &Registry) -> BTreeMap<RuleCode, RuleConf> {
        self.set
            .iter()
            .filter_map(|code| reg.get(*code).map(|r| (*code, r.meta())))
            .map(|(code, meta)| {
                let o = self.overrides.get(&code);
                let mut params: BTreeMap<&'static str, f64> =
                    meta.params.iter().map(|p| (p.name, p.default)).collect();
                if let Some(o) = o {
                    params.extend(o.params.iter().map(|(k, v)| (*k, *v)));
                }
                let severity = o.and_then(|o| o.severity).unwrap_or(meta.severity);
                (code, RuleConf { severity, params })
            })
            .collect()
    }
}

/// Resolves the settings for each input file: CLI overrides, the explicit `--config` chain, and a cache of the
/// chains discovered so far. Filled on the main thread; parallel analysis only reads the `Arc<Settings>` (ADR-09).
pub struct Resolver {
    reg: &'static Registry,
    cli: Selection,
    cli_seed: Option<u32>,
    explicit: Option<Arc<Chain>>,
    defaults: Option<Arc<Chain>>,
    /// Config file -> its validated chain.
    chains: HashMap<PathBuf, Arc<Chain>>,
    /// Directory -> the config file discovery found for it (`None` = no file up to the root).
    dirs: HashMap<PathBuf, Option<PathBuf>>,
}

impl Resolver {
    /// Validates the CLI codes (with `file` = `<command line>`) and loads `--config` when given (FR-7.2).
    pub fn new(cli: CliOverrides, explicit: Option<&Path>) -> Result<Resolver, Error> {
        let reg = Registry::builtin();
        let cmd = Path::new(COMMAND_LINE);
        let selection = Selection {
            select: if cli.select.is_empty() {
                None
            } else {
                Some(codes(&cli.select, reg, cmd, "select")?)
            },
            extend_select: codes(&cli.extend_select, reg, cmd, "extend_select")?,
            ignore: codes(&cli.ignore, reg, cmd, "ignore")?,
            min_severity: cli.min_severity,
            rules: BTreeMap::new(),
        };
        let mut r = Resolver {
            reg,
            cli: selection,
            cli_seed: cli.seed,
            explicit: None,
            defaults: None,
            chains: HashMap::new(),
            dirs: HashMap::new(),
        };
        if let Some(p) = explicit {
            r.explicit = Some(r.chain_of(p)?);
        }
        Ok(r)
    }

    /// The nearest `sloperator.toml` found upward from the file's directory (FR-7.1), or the `--config` file, then
    /// every matching `[[per_file]]` entry, then CLI flags (FR-7.5).
    pub fn for_file(&mut self, path: &Path) -> Result<Arc<Settings>, Error> {
        let abs = absolute(path);
        let chain = match &self.explicit {
            Some(c) => c.clone(),
            None => {
                let dir = abs.parent().map(Path::to_path_buf).unwrap_or_default();
                match self.discover(&dir) {
                    Some(p) => self.chain_of(&p)?,
                    None => self.defaults()?,
                }
            }
        };
        let matching: Vec<&PerFileLayer> = chain
            .per_file
            .iter()
            .filter(|l| {
                abs.strip_prefix(&l.dir)
                    .is_ok_and(|rel| l.matcher.is_match(rel))
            })
            .collect();
        if matching.is_empty() && self.cli_is_empty() {
            return Ok(chain.base.clone());
        }
        let mut state = chain.state.clone();
        for l in matching {
            state.apply(&l.selection);
        }
        state.apply(&self.cli);
        let mut s = (*chain.base).clone();
        s.rules = state.rules(self.reg);
        s.min_severity = state.min_severity;
        Ok(Arc::new(s))
    }

    /// The run's metric floors: explicit config, nearest config above cwd, or built-in defaults.
    ///
    /// # Errors
    /// A config chain that cannot be read or contains invalid settings.
    pub fn run_floors(&mut self, cwd: &Path) -> Result<Floors, Error> {
        let chain = match &self.explicit {
            Some(c) => c.clone(),
            None => match self.discover(&absolute(cwd)) {
                Some(p) => self.chain_of(&p)?,
                None => self.defaults()?,
            },
        };
        Ok(chain.base.metrics)
    }

    fn cli_is_empty(&self) -> bool {
        self.cli.select.is_none()
            && self.cli.extend_select.is_empty()
            && self.cli.ignore.is_empty()
            && self.cli.min_severity.is_none()
    }

    fn discover(&mut self, dir: &Path) -> Option<PathBuf> {
        if let Some(hit) = self.dirs.get(dir) {
            return hit.clone();
        }
        let found = dir
            .ancestors()
            .map(|d| d.join(CONFIG_FILE_NAME))
            .find(|p| p.is_file());
        self.dirs.insert(dir.to_path_buf(), found.clone());
        found
    }

    fn defaults(&mut self) -> Result<Arc<Chain>, Error> {
        if let Some(c) = &self.defaults {
            return Ok(c.clone());
        }
        let c = Arc::new(build(&[], None, self.reg, &self.cli, self.cli_seed)?);
        self.defaults = Some(c.clone());
        Ok(c)
    }

    fn chain_of(&mut self, path: &Path) -> Result<Arc<Chain>, Error> {
        let canon = path.canonicalize().map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        if let Some(c) = self.chains.get(&canon) {
            return Ok(c.clone());
        }
        let loaded = file::load_chain(&canon)?;
        let c = Arc::new(build(
            &loaded,
            Some(canon.clone()),
            self.reg,
            &self.cli,
            self.cli_seed,
        )?);
        self.chains.insert(canon, c.clone());
        Ok(c)
    }
}

/// Validates every file of `chain` (root first) and folds it over the defaults. `base` has the CLI applied, for files
/// that no `[[per_file]]` entry matches.
fn build(
    chain: &[Loaded],
    source: Option<PathBuf>,
    reg: &'static Registry,
    cli: &Selection,
    cli_seed: Option<u32>,
) -> Result<Chain, Error> {
    let mut state = State {
        set: reg.default_set(),
        overrides: BTreeMap::new(),
        min_severity: Severity::Warning,
    };
    let mut include: Vec<String> = DEFAULT_INCLUDE.iter().map(|s| s.to_string()).collect();
    let mut exclude: Vec<String> = Vec::new();
    let mut respect_gitignore = true;
    let mut protect = ProtectSettings::default();
    let mut metrics = Floors::default();
    let mut model = ModelSettings {
        name: DEFAULT_MODEL_NAME.to_string(),
        path: None,
        profile: None,
        cache_dir: None,
        window: WindowMode::Paragraph,
        max_window_chars: DEFAULT_MAX_WINDOW_CHARS,
        max_tokens: 0,
        seed: DEFAULT_SEED,
    };
    let mut model_path_from: Option<&Path> = None;
    let mut per_file = Vec::new();

    for l in chain {
        let f = &l.file;
        let p = l.path.as_path();
        state.apply(&Selection {
            select: f
                .select
                .as_ref()
                .map(|v| codes(v, reg, p, "select"))
                .transpose()?,
            extend_select: codes(
                f.extend_select.as_deref().unwrap_or_default(),
                reg,
                p,
                "extend_select",
            )?,
            ignore: codes(f.ignore.as_deref().unwrap_or_default(), reg, p, "ignore")?,
            min_severity: f.min_severity,
            rules: rule_tables(f.rules.as_ref(), reg, p, "rules")?,
        });
        if let Some(v) = &f.include {
            globs(v, p, "include")?;
            include = v.clone();
        }
        if let Some(v) = &f.exclude {
            globs(v, p, "exclude")?;
            exclude = v.clone();
        }
        if let Some(v) = f.respect_gitignore {
            respect_gitignore = v;
        }
        for (i, pf) in f.per_file.iter().flatten().enumerate() {
            let key = format!("per_file[{i}]");
            let matcher = globset::Glob::new(&pf.glob)
                .map_err(|e| config_err(p, &format!("{key}.glob"), &e.to_string()))?
                .compile_matcher();
            per_file.push(PerFileLayer {
                matcher,
                dir: l.dir.clone(),
                selection: Selection {
                    select: pf
                        .select
                        .as_ref()
                        .map(|v| codes(v, reg, p, &format!("{key}.select")))
                        .transpose()?,
                    extend_select: codes(
                        pf.extend_select.as_deref().unwrap_or_default(),
                        reg,
                        p,
                        &format!("{key}.extend_select"),
                    )?,
                    ignore: codes(
                        pf.ignore.as_deref().unwrap_or_default(),
                        reg,
                        p,
                        &format!("{key}.ignore"),
                    )?,
                    min_severity: pf.min_severity,
                    rules: rule_tables(pf.rules.as_ref(), reg, p, &format!("{key}.rules"))?,
                },
            });
        }
        if let Some(pr) = &f.protect {
            for (i, pat) in pr.patterns.iter().flatten().enumerate() {
                regex::Regex::new(pat).map_err(|e| {
                    config_err(p, &format!("protect.patterns[{i}]"), &e.to_string())
                })?;
            }
            let patterns = pr
                .patterns
                .clone()
                .unwrap_or_else(|| protect.patterns.clone());
            protect = ProtectSettings::new(
                patterns,
                pr.terms.clone().unwrap_or_else(|| protect.terms.clone()),
                pr.front_matter.unwrap_or(protect.front_matter),
                pr.math.unwrap_or(protect.math),
                pr.html.unwrap_or(protect.html),
            )
            .map_err(|e| config_err(p, "protect.patterns", &e.to_string()))?;
        }
        if let Some(m) = &f.metrics {
            if let Some(v) = m.min_sentences {
                metrics.sentences = v.get();
            }
            if let Some(v) = m.min_words {
                metrics.words = v.get();
            }
        }
        if let Some(m) = &f.model {
            if let Some(v) = &m.name {
                model.name = v.clone();
            }
            if let Some(v) = &m.path {
                model.path = Some(l.dir.join(v));
                model_path_from = Some(p);
            }
            if let Some(v) = &m.profile {
                model.profile = Some(v.clone());
            }
            if let Some(v) = &m.cache_dir {
                model.cache_dir = Some(v.clone());
            }
            if let Some(v) = m.window {
                model.window = v;
            }
            if let Some(v) = m.max_window_chars {
                model.max_window_chars = v;
            }
            if let Some(v) = m.max_tokens {
                model.max_tokens = v;
            }
            if let Some(v) = m.seed {
                model.seed = v;
            }
        }
    }
    if let (Some(from), None) = (model_path_from, &model.profile) {
        return Err(config_err(
            from,
            "model.path",
            "`model.path` requires `model.profile`",
        ));
    }
    if let Some(seed) = cli_seed {
        model.seed = seed;
    }

    let mut with_cli = state.clone();
    with_cli.apply(cli);
    let base = Settings {
        source,
        rules: with_cli.rules(reg),
        min_severity: with_cli.min_severity,
        include,
        exclude,
        respect_gitignore,
        protect,
        model,
        metrics,
    };
    Ok(Chain {
        base: Arc::new(base),
        per_file,
        state,
    })
}

/// Parses rule codes; an unparsable, unregistered or system code is `Error::Config` on `key` (FR-7.6).
fn codes(raw: &[String], reg: &Registry, file: &Path, key: &str) -> Result<Vec<RuleCode>, Error> {
    raw.iter().map(|s| code(s, reg, file, key)).collect()
}

fn code(raw: &str, reg: &Registry, file: &Path, key: &str) -> Result<RuleCode, Error> {
    let c: RuleCode = raw.parse().map_err(|e: String| config_err(file, key, &e))?;
    if c.is_system() || reg.get(c).is_none() {
        return Err(config_err(file, key, &format!("unknown rule code `{raw}`")));
    }
    Ok(c)
}

/// Validates `[rules.SLPxxx]` tables: a registered code, `severity`, and the rule's declared numeric params.
fn rule_tables(
    tables: Option<&BTreeMap<String, toml::Table>>,
    reg: &Registry,
    file: &Path,
    key: &str,
) -> Result<BTreeMap<RuleCode, RuleOverride>, Error> {
    let mut out = BTreeMap::new();
    for (raw, table) in tables.into_iter().flatten() {
        let at = format!("{key}.{raw}");
        let c = code(raw, reg, file, &at)?;
        let meta = reg
            .get(c)
            .map(|r| r.meta())
            .ok_or_else(|| config_err(file, &at, "unknown rule code"))?;
        let mut o = RuleOverride::default();
        for (name, value) in table {
            let k = format!("{at}.{name}");
            if name == "severity" {
                let s = value
                    .clone()
                    .try_into::<Severity>()
                    .map_err(|e| config_err(file, &k, e.message()))?;
                o.severity = Some(s);
                continue;
            }
            let Some(spec) = meta.params.iter().find(|p| p.name == name) else {
                return Err(config_err(file, &k, &format!("unknown parameter for {c}")));
            };
            let n = match value {
                toml::Value::Float(f) => *f,
                toml::Value::Integer(i) => *i as f64,
                other => {
                    return Err(config_err(
                        file,
                        &k,
                        &format!("expected a number, found {}", other.type_str()),
                    ));
                }
            };
            o.params.insert(spec.name, n);
        }
        out.insert(c, o);
    }
    Ok(out)
}

fn globs(raw: &[String], file: &Path, key: &str) -> Result<(), Error> {
    for (i, g) in raw.iter().enumerate() {
        globset::Glob::new(g)
            .map_err(|e| config_err(file, &format!("{key}[{i}]"), &e.to_string()))?;
    }
    Ok(())
}

/// The canonical path of `path`, or of its parent joined with the file name when the file does not exist (stdin
/// names), so it compares with the canonical directories of config files.
fn absolute(path: &Path) -> PathBuf {
    if let Ok(p) = path.canonicalize() {
        return p;
    }
    let abs = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    match (
        abs.parent().and_then(|d| d.canonicalize().ok()),
        abs.file_name(),
    ) {
        (Some(d), Some(n)) => d.join(n),
        _ => abs,
    }
}
