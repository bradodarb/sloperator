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
    #[cfg(feature = "model")]
    let factory: Box<dyn sloperator::engine::RewriterFactory> = Box::new(
        sloperator::engine::runtime::RuntimeFactory::new(cli.verbose > 0),
    );
    #[cfg(not(feature = "model"))]
    let factory: Box<dyn sloperator::engine::RewriterFactory> =
        Box::new(sloperator::engine::DisabledFactory);
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
    let code = cli::run(
        cli,
        &mut deps,
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock(),
    );
    std::process::exit(code as i32);
}
