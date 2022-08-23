mod cli;
mod client;
mod config;
mod eth_method;
mod transaction_reader;
mod utils;

const AURORA_MAINNET_ENDPOINT: &str = "https://mainnet.aurora.dev/";
const NEAR_MAINNET_ENDPOINT: &str = "https://archival-rpc.mainnet.near.org/";
const AURORA_TESTNET_ENDPOINT: &str = "https://testnet.aurora.dev/";
const NEAR_TESTNET_ENDPOINT: &str = "https://archival-rpc.testnet.near.org/";

use clap::Parser;
use cli::{Cli, Command};
use client::AuroraClient;
use config::Network;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let config_path = args.config_path.as_deref().unwrap_or("default-config.json");
    let config = config::Config::from_file(config_path)?;
    let network = config.network;

    let api_key = config.aurora_api_key.as_deref().unwrap_or_default();
    let (aurora_endpoint, near_endpoint) = match network {
        Network::Mainnet => (AURORA_MAINNET_ENDPOINT, NEAR_MAINNET_ENDPOINT),
        Network::Testnet => (AURORA_TESTNET_ENDPOINT, NEAR_TESTNET_ENDPOINT),
    };
    let client = AuroraClient::new(
        format!("{}{}", aurora_endpoint, api_key),
        near_endpoint,
        config.engine_account_id.clone(),
        config.near_key_path.clone(),
    );

    match args.command {
        Command::Aurora { subcommand } => {
            cli::aurora::execute_command(subcommand, &client, &config).await?
        }
        Command::Near { subcommand } => {
            cli::near::execute_command(subcommand, &client, &config).await?
        }
        Command::ProcessTxData {
            action,
            input_files_list_path,
        } => cli::process_tx_data::execute_command(action, input_files_list_path).await,
    }

    Ok(())
}
