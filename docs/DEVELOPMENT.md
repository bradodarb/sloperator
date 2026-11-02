# Development

How to build and test sloperator. The CLI and library reference is [REFERENCE.md](REFERENCE.md).

Prerequisites:

- GNU make 3.82 or newer (the `Makefile` uses `.ONESHELL`); macOS ships 3.81, so `brew install make` and run `gmake`
- Rust from `rustup`; `rust-toolchain.toml` pins the toolchain (1.99.1, with `rustfmt` and `clippy`) and rustup
  installs it on first use
- `cargo-nextest` 0.9.150: `cargo install --locked cargo-nextest@0.9.150`
- `cargo-deny` 0.20.2: `cargo install --locked cargo-deny@0.20.2`
- `clang` and `cmake`, for the `llama-cpp-2` build behind the default `model` feature (Metal on macOS)
- a model for `--diff`/`--fix` and the integration tier: `sloperator model pull` (default `qwen3-4b`, about 2.5 GB)

The `Makefile` wraps cargo (`make help` lists the targets):

| Target | What it runs |
|--------|--------------|
| `make build` | `cargo build --release --locked` with default features (the model-enabled `target/release/sloperator`) |
| `make test` | the unit tier (`--no-default-features`, no model), then the NFR-3 perf test in the release profile |
| `make test-integration` | the `#[ignore]`d integration tier with `--features model` (needs a pulled model) |
| `make check` | `cargo fmt --check`, clippy with and without default features (`-D warnings`), `cargo deny check` |
| `make clean` | removes `target/` |

`make test TESTS='<selector> ...'` runs only the matching tests: `binary_id#test` selects one test exactly (for
example `sloperator::cli_exit#exit_two_on_bad_config`), and a bare word matches a binary id or test name. `make
test-integration FEATURES=metal` changes the integration tier's cargo features. Test reports are nextest JUnit XML
under `target/nextest/<profile>/junit.xml`. The perf pass builds into `target/perf`, so `make test` never replaces the
model-enabled release binary that `make build` produced.

```sh
cargo build --release --locked                        # default features (model)
cargo build --release --locked --no-default-features  # rules-only binary
```

`sloperator check --diff` previews a fix as a unified diff and writes nothing; `sloperator check --fix` writes the
verified edits to the files. `--rules-only` skips the model and applies only deterministic rule fixes.

`tests/layering.rs` and `tests/standalone.rs` guard the module rules and keep the crate self-contained: no forbidden
`crate::` edges, no I/O in the pure modules, and no reference to the repository it was developed in.
`builds_when_extracted` copies the crate (without `.git`, `target/` and `docs/`) to a temp directory and runs
`cargo test` there offline; it is `#[ignore]`d and runs only in the integration tier.

Design documents live in `docs/` (PRD, SPEC, ARCHITECTURE, ADRs, research), the delivery plan in `docs/plan/` and
deferred work in `docs/backlog/`.
