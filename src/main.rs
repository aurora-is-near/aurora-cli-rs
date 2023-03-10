#![deny(clippy::pedantic, clippy::nursery)]
#![allow(clippy::too_many_lines, clippy::module_name_repetitions)]

use clap::Parser;

use cli::{Cli, Command};

mod cli;
mod client;
mod config;
#[cfg(feature = "advanced")]
mod eth_method;
#[cfg(feature = "advanced")]
mod transaction_reader;
mod utils;

#[cfg(feature = "advanced")]
const AURORA_MAINNET_ENDPOINT: &str = "https://mainnet.aurora.dev/";
const NEAR_MAINNET_ENDPOINT: &str = "https://archival-rpc.mainnet.near.org/";
#[cfg(feature = "advanced")]
const AURORA_TESTNET_ENDPOINT: &str = "https://testnet.aurora.dev/";
const NEAR_TESTNET_ENDPOINT: &str = "https://archival-rpc.testnet.near.org/";
#[cfg(feature = "simple")]
const NEAR_LOCAL_ENDPOINT: &str = "http://127.0.0.1:3030/";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    run(args).await
}

#[cfg(feature = "simple")]
async fn run(args: Cli) -> anyhow::Result<()> {
    use cli::command;

    let near_rpc = match args.network {
        cli::simple::Network::Mainnet => NEAR_MAINNET_ENDPOINT,
        cli::simple::Network::Testnet => NEAR_TESTNET_ENDPOINT,
        cli::simple::Network::Localnet => NEAR_LOCAL_ENDPOINT,
    };
    let client = client::Client::new(near_rpc, &args.engine, args.near_secret_file);

    match args.command {
        Command::GetChainId => command::get_chain_id(client).await?,
        Command::GetUpgradeIndex => command::get_upgrade_index(client).await?,
        Command::GetVersion => command::get_version(client).await?,
        Command::GetOwner => command::get_owner(client).await?,
        Command::GetBridgeProver => command::get_bridge_prover(client).await?,
        Command::GetNonce { address } => command::get_nonce(client, address).await?,
        Command::GetCode { address } => command::get_code(client, address).await?,
        Command::GetBalance { address } => command::get_balance(client, address).await?,
        Command::Call {
            address, function, ..
        } => command::call(client, address, function, args.aurora_secret_key.as_deref()).await?,
        Command::StageUpgrade => command::stage_upgrade(client).await?,
        Command::DeployUpgrade => command::deploy_upgrade(client).await?,
        Command::GetStorageAt { address, key } => {
            command::get_storage_at(client, address, key).await?;
        }
        Command::DeployEvmCode { code } => {
            command::deploy_evm_code(client, code, args.aurora_secret_key.as_deref()).await?;
        }
        Command::DeployAurora { path } => command::deploy_aurora(client, path).await?,
        Command::CreateAccount { account, balance } => {
            command::create_account(client, &account, balance).await?;
        }
        Command::ViewAccount { account } => command::view_account(client, &account).await?,
        Command::Init {
            chain_id,
            owner_id,
            bridge_prover_id,
            upgrade_delay_blocks,
        } => {
            command::init(
                client,
                chain_id,
                owner_id,
                bridge_prover_id,
                upgrade_delay_blocks,
            )
            .await?;
        }
        Command::EncodeAddress { account } => command::encode_address(&account),
    }

    Ok(())
}

#[cfg(feature = "advanced")]
async fn run(args: Cli) -> anyhow::Result<()> {
    let config_path = args.config_path.as_deref().unwrap_or("default-config.json");
    let config = config::Config::from_file(config_path)?;
    let network = &config.network;

    let api_key = config.aurora_api_key.as_deref().unwrap_or_default();
    let (aurora_endpoint, near_endpoint) = match network {
        config::Network::Mainnet => (AURORA_MAINNET_ENDPOINT, NEAR_MAINNET_ENDPOINT),
        config::Network::Testnet => (AURORA_TESTNET_ENDPOINT, NEAR_TESTNET_ENDPOINT),
        config::Network::Custom {
            near_rpc,
            aurora_rpc,
        } => (aurora_rpc.as_str(), near_rpc.as_str()),
    };

    match args.command {
        Command::Aurora { subcommand } => {
            let client = client::AuroraClient::new(
                &format!("{aurora_endpoint}{api_key}"),
                near_endpoint,
                &config.engine_account_id,
                config.near_key_path.clone(),
            );
            cli::aurora::execute_command(subcommand, &client, &config).await?;
        }
        Command::Near { subcommand } => {
            let client = client::NearClient::new(
                near_endpoint,
                &config.engine_account_id,
                config.near_key_path.clone(),
            );
            cli::near::execute_command(subcommand, &client, &config, config_path).await?;
        }
        Command::ProcessTxData {
            action,
            input_files_list_path,
        } => cli::process_tx_data::execute_command(action, input_files_list_path).await,
    }

    Ok(())
}
