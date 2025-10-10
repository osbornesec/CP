#![allow(
    clippy::all,
    reason = "Legacy CLI entrypoint retains historical lint exemptions"
)]

use anyhow::Result;
use cpinfo_parser::cli::runner::run_cli;

#[tokio::main]
async fn main() -> Result<()> {
    run_cli().await
}
