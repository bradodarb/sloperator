//! clap 4 derive definitions (SPEC §20).

use std::path::PathBuf;

use crate::report::OutputFormat;
use crate::rules::Severity;

#[derive(Debug, clap::Parser)]
#[command(
    name = "sloperator",
    version,
    about = "Lint and rewrite LLM-generated prose"
)]
pub struct Cli {
    /// Stage timings on stderr
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,
    /// Print errors only
    #[arg(short, long, global = true)]
    pub quiet: bool,
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Report findings and scores; --fix writes, --diff previews
    Check(CheckArgs),
    /// List rules, or explain one
    Rules { code: Option<String> },
    /// Print the resolved configuration for a path as JSON
    Config {
        path: Option<PathBuf>,
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Download, list or locate models
    #[command(subcommand)]
    Model(ModelCmd),
}

/// `model` subcommands (SPEC §15, FR-2.14). NAME defaults to the configured `model.name`.
#[derive(Debug, clap::Subcommand)]
pub enum ModelCmd {
    /// Download a model into the cache and verify its sha256
    Pull { name: Option<String> },
    /// List the registry: name, file, size, licence, present/absent
    List,
    /// Print the model's path; exit 2 when it is not in the cache
    Path { name: Option<String> },
}

#[derive(Debug, clap::Args)]
pub struct CheckArgs {
    /// Files, directories, globs, or `-` for stdin (default: `.`)
    pub paths: Vec<String>,
    /// Apply verified fixes in place (stdin: print the fixed text)
    #[arg(long)]
    pub fix: bool,
    /// Print the fixes as a unified diff; write nothing
    #[arg(long, conflicts_with = "fix")]
    pub diff: bool,
    /// Use only the rules engine
    #[arg(long, conflicts_with = "model_only")]
    pub rules_only: bool,
    /// Use only the model engine
    #[arg(long)]
    pub model_only: bool,
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub output_format: OutputFormat,
    /// Report and count only findings at or above this severity
    #[arg(long, value_enum)]
    pub min_severity: Option<Severity>,
    /// Exit 0 even when findings are reported (errors still exit 2)
    #[arg(long)]
    pub exit_zero: bool,
    #[arg(long, value_delimiter = ',')]
    pub select: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    pub extend_select: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    pub ignore: Vec<String>,
    /// Use this config file instead of discovery
    #[arg(long)]
    pub config: Option<PathBuf>,
    /// The path stdin stands for: picks the format and the config
    #[arg(long)]
    pub stdin_filename: Option<PathBuf>,
    /// The model's sampling seed
    #[arg(long)]
    pub seed: Option<u32>,
}
