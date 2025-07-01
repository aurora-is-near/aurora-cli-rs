mod cli;
mod command;
mod common;
mod context;

use clap::Parser;
use cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    command::run(cli).await
}
