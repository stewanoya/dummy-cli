use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;
mod config;
mod provider;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    commands::run(cli).await
}
