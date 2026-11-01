//! The single `std::process::exit` call (SPEC §20, tests/layering.rs::single_exit_point).

use std::io::IsTerminal;

use clap::Parser;
use sloperator::cli::{self, Cli, Deps};

fn main() {
    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            let code = cli::parse_error_exit(&e);
            let _ = e.print();
            std::process::exit(code as i32);
        }
    };
    // The runtime is compiled only with the `model` feature; other builds use the model-less factory (ADR-02).
    let factory = sloperator::engine::default_factory(cli.verbose > 0);
    let cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("sloperator: {e}");
            std::process::exit(2)
        }
    };
    let color = std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none(); // FR-3.5
    let mut deps = Deps {
        rewriter_factory: factory,
        fs: Box::new(sloperator::write::RealFs),
        cwd,
        color,
    };
    // Unlocked handles: `mcp` reads stdin and writes stdout from the async runtime's own threads, which would block
    // forever on a lock held here for the whole run.
    let code = cli::run(
        cli,
        &mut deps,
        &mut std::io::stdin(),
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    );
    std::process::exit(code as i32);
}
