//! Command-line interface module for `cpinfo` parser.
//!
//! This module provides the CLI argument parsing, command execution,
//! and section handling functionality for the `cpinfo` parser application.

pub mod args;
pub mod runner;
pub mod section_handler;

pub use args::Args;
pub use runner::run_cli;
