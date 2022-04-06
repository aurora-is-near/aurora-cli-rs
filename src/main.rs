mod cli;
mod client;
mod eth_method;
mod transaction_reader;
mod utils;

const AURORA_MAINNET_ENDPOINT: &str = "https://mainnet.aurora.dev/";
const NEAR_MAINNET_ENDPOINT: &str = "https://archival-rpc.mainnet.near.org";

use aurora_engine_types::{
    types::{Address, Wei},
    U256,
};
use clap::Parser;
use cli::{Cli, Command, Network, ProcessTxAction};
use client::{AuroraClient, ClientError};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let network = args.network.unwrap_or_default();

    let client = match network {
        Network::Mainnet => AuroraClient::new(AURORA_MAINNET_ENDPOINT, NEAR_MAINNET_ENDPOINT),
    };

    match args.command {
        Command::GetResult { tx_hash_hex } => {
            let tx_hash = aurora_engine_types::H256::from_slice(&hex::decode(tx_hash_hex).unwrap());
            let outcome = client
                .get_transaction_outcome(tx_hash, "relay.aurora")
                .await?;
            println!("{:?}", outcome);
        }
        Command::Transfer {
            source_private_key_hex,
            target_addr_hex,
            amount,
        } => {
            let sk_bytes = utils::hex_to_arr32(&source_private_key_hex)?;
            let sk = secp256k1::SecretKey::parse(&sk_bytes).unwrap();
            let source = utils::address_from_secret_key(&sk);
            println!("FROM {:?}", source);

            let nonce = client.get_nonce(source).await?;
            let chain_id = client.get_chain_id().await?;

            let target = Address::decode(&target_addr_hex).unwrap();
            let amount = Wei::new(U256::from_dec_str(&amount).unwrap());
            let tx_hash = client
                .transfer(target, amount, &sk, chain_id, nonce)
                .await
                .unwrap();

            // Wait for the RPC to pick up the transaction
            loop {
                match client
                    .get_transaction_outcome(tx_hash, "relay.aurora")
                    .await
                {
                    Ok(result) => {
                        println!("{:?}", result);
                        break;
                    }
                    Err(ClientError::AuroraTransactionNotFound(_)) => {
                        continue;
                    }
                    Err(other) => return Err(Box::new(other) as Box<dyn std::error::Error>),
                }
            }
        }
        Command::ProcessTxData {
            action,
            input_files_list_path,
        } => {
            let paths_contents = tokio::fs::read_to_string(input_files_list_path)
                .await
                .unwrap();
            let paths: Vec<String> = paths_contents
                .split('\n')
                .filter(|line| !line.is_empty())
                .map(|line| line.to_owned())
                .collect();

            match action {
                ProcessTxAction::AverageGasProfile => {
                    transaction_reader::get_average_gas_profile(paths).await
                }
                ProcessTxAction::FilterTo { target_addr_hex } => {
                    let to = Address::decode(&target_addr_hex).unwrap();
                    transaction_reader::get_txs_to(paths, to).await;
                }
                ProcessTxAction::GasDistribution => {
                    transaction_reader::count_transactions_by_gas(paths).await
                }
                ProcessTxAction::NearGasVsEvmGas => {
                    transaction_reader::get_near_gas_vs_evm_gas(paths).await
                }
                ProcessTxAction::OutcomeDistribution => {
                    transaction_reader::count_transactions_by_type(paths).await
                }
            }
        }
    }

    Ok(())
}
