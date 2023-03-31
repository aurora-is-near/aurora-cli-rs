use clap::{Parser, Subcommand};

use crate::config::{Config, Network};

pub mod aurora;
pub mod erc20;
pub mod near;
pub mod process_tx_data;
pub mod solidity;

/// Advanced command line interface for communication with Aurora Engine
#[derive(Parser)]
#[command(author, version, long_about = None)]
pub struct Cli {
    /// Path to config file
    #[clap(short, long)]
    pub config_path: Option<String>,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Commands for communication with Aurora Engine
    Aurora {
        #[clap(subcommand)]
        subcommand: aurora::Command,
    },
    /// Commands for communication with NEAR node
    Near {
        #[clap(subcommand)]
        subcommand: near::Command,
    },
    /// Transaction operations
    ProcessTxData {
        #[clap(subcommand)]
        action: process_tx_data::ProcessTxAction,
        input_files_list_path: String,
    },
}

pub async fn run(args: Cli) -> anyhow::Result<()> {
    let config_path = args.config_path.as_deref().unwrap_or("default-config.json");
    let config = Config::from_file(config_path)?;
    let network = &config.network;

    let api_key = config.aurora_api_key.as_deref().unwrap_or_default();
    let (aurora_endpoint, near_endpoint) = match network {
        Network::Mainnet => (super::AURORA_MAINNET_ENDPOINT, super::NEAR_MAINNET_ENDPOINT),
        Network::Testnet => (super::AURORA_TESTNET_ENDPOINT, super::NEAR_TESTNET_ENDPOINT),
        Network::Custom {
            near_rpc,
            aurora_rpc,
        } => (aurora_rpc.as_str(), near_rpc.as_str()),
    };

    match args.command {
        Command::Aurora { subcommand } => {
            let client = crate::client::AuroraClient::new(
                &format!("{aurora_endpoint}{api_key}"),
                near_endpoint,
                &config.engine_account_id,
            );
            aurora::execute_command(subcommand, &client, &config).await?;
        }
        Command::Near { subcommand } => {
            let client = crate::client::NearClient::new(
                near_endpoint,
                &config.engine_account_id,
                config.near_key_path.clone(),
            );
            near::execute_command(subcommand, &client, &config, config_path).await?;
        }
        Command::ProcessTxData {
            action,
            input_files_list_path,
        } => process_tx_data::execute_command(action, input_files_list_path).await,
    }

    Ok(())
}
