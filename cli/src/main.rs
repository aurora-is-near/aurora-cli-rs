#![deny(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::too_many_lines,
    clippy::module_name_repetitions,
    clippy::needless_raw_string_hashes
)]

use clap::Parser;

mod cli;
mod client;
mod config;
#[cfg(feature = "advanced")]
mod eth_method;
#[cfg(feature = "advanced")]
mod transaction_reader;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();
    cli::run(args).await
}
