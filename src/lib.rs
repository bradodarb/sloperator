//! sloperator: lint and rewrite LLM-generated prose so it reads as human-written.
//!
//! Modules other than `error`, `cli` and the port in `engine` are stubs that later stories fill (SPEC §4).

pub mod cli;
pub mod config;
pub mod directives;
pub mod doc;
pub mod edit;
pub mod engine;
pub mod error;
pub mod input;
#[cfg(feature = "mcp")]
pub mod mcp;
pub mod model;
pub mod pipeline;
pub mod report;
pub mod rules;
pub mod score;
pub mod text;
pub mod types;
pub mod verify;
pub mod write;

pub use error::Error;
