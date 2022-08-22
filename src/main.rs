mod cli;
mod client;
mod eth_method;
mod transaction_reader;
mod utils;

const AURORA_MAINNET_ENDPOINT: &str = "https://mainnet.aurora.dev/";
const NEAR_MAINNET_ENDPOINT: &str = "https://archival-rpc.mainnet.near.org/";
const AURORA_TESTNET_ENDPOINT: &str = "https://testnet.aurora.dev/";
const NEAR_TESTNET_ENDPOINT: &str = "https://archival-rpc.testnet.near.org/";

use aurora_engine_types::{
    types::{Address, Wei},
    U256,
};
use clap::Parser;
use cli::{Cli, Command, Network, ProcessTxAction};
use client::{AuroraClient, ClientError};
use std::sync::Arc;
use transaction_reader::{aggregator, filter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let network = args.network.unwrap_or_default();

    let api_key = match args.api_key_path {
        Some(path) => std::fs::read_to_string(path)?,
        None => {
            let default_path = ".api_key";
            std::fs::read_to_string(default_path).unwrap_or_default()
        }
    };
    let engine_account_id = args.engine_account_id.unwrap_or_else(|| "aurora".into());
    let (aurora_endpoint, near_endpoint) = match network {
        Network::Mainnet => (AURORA_MAINNET_ENDPOINT, NEAR_MAINNET_ENDPOINT),
        Network::Testnet => (AURORA_TESTNET_ENDPOINT, NEAR_TESTNET_ENDPOINT),
    };
    let client = AuroraClient::new(
        format!("{}{}", aurora_endpoint, api_key),
        near_endpoint,
        engine_account_id,
    );

    match args.command {
        Command::GetResult { tx_hash_hex } => {
            let tx_hash = aurora_engine_types::H256::from_slice(&hex::decode(tx_hash_hex).unwrap());
            let outcome = client
                .get_transaction_outcome(tx_hash, "relay.aurora")
                .await?;
            println!("{:?}", outcome);
        }
        Command::GetNearResult {
            tx_hash_b58,
            relayer,
        } => {
            let tx_hash = bs58::decode(tx_hash_b58.as_str()).into_vec().unwrap();
            let tx_hash = aurora_engine_types::H256::from_slice(&tx_hash);
            let relayer = relayer.as_deref().unwrap_or("relay.aurora");
            let outcome = client
                .get_near_transaction_outcome(tx_hash, relayer)
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
            let target = Address::decode(&target_addr_hex).unwrap();
            let amount = Wei::new(U256::from_dec_str(&amount).unwrap());
            send_transaction(&client, &sk, Some(target), amount, Vec::new()).await?;
        }
        Command::ContractCall {
            source_private_key_hex,
            target_addr_hex,
            amount,
            input_data_hex,
        } => {
            let sk_bytes = utils::hex_to_arr32(&source_private_key_hex)?;
            let sk = secp256k1::SecretKey::parse(&sk_bytes).unwrap();
            let target = Address::decode(&target_addr_hex).unwrap();
            let amount = amount
                .as_ref()
                .map(|a| Wei::new(U256::from_dec_str(a).unwrap()))
                .unwrap_or_else(Wei::zero);
            let input = hex::decode(input_data_hex)?;
            send_transaction(&client, &sk, Some(target), amount, input).await?;
        }
        Command::ContractView {
            target_addr_hex,
            amount,
            input_data_hex,
            sender_addr_hex,
        } => {
            let target = Address::decode(&target_addr_hex).unwrap();
            let sender = sender_addr_hex
                .map(|x| Address::decode(&x).unwrap())
                .unwrap_or_default();
            let amount = amount
                .as_ref()
                .map(|a| Wei::new(U256::from_dec_str(a).unwrap()))
                .unwrap_or_else(Wei::zero);
            let input = hex::decode(input_data_hex)?;

            let result = client
                .view_contract_call(sender, target, amount, input)
                .await
                .unwrap();
            println!("{:?}", result);
        }
        Command::Deploy {
            source_private_key_hex,
            input_data_hex,
        } => {
            let sk_bytes = utils::hex_to_arr32(&source_private_key_hex)?;
            let sk = secp256k1::SecretKey::parse(&sk_bytes).unwrap();
            let input = hex::decode(input_data_hex)?;
            send_transaction(&client, &sk, None, Wei::zero(), input).await?;
        }
        Command::GetNep141 { erc_20_address_hex } => {
            let erc20 = Address::decode(&erc_20_address_hex).unwrap();
            match client.get_nep141_from_erc20(erc20).await {
                Ok(nep_141_account) => println!("{}", nep_141_account),
                Err(e) => {
                    let error_msg = format!("{:?}", e);
                    if error_msg.contains("ERC20_NOT_FOUND") {
                        println!("No NEP-141 account associated with {}", erc_20_address_hex);
                    } else {
                        panic!("{}", error_msg);
                    }
                }
            };
        }
        Command::GetBridgeProver => {
            println!("{:?}", client.get_bridge_prover().await);
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
                ProcessTxAction::AverageGasProfile { min_near_gas } => {
                    let f1 = filter::MatchFlatStatus(transaction_reader::FlatTxStatus::Succeeded);
                    match min_near_gas {
                        None => {
                            let f = Arc::new(f1);
                            transaction_reader::process_data::<aggregator::AverageGasProfile, _>(
                                paths, &f,
                            )
                            .await
                        }
                        Some(min_gas) => {
                            let f2 = filter::MinNearGasUsed(min_gas);
                            let f = Arc::new(filter::And::new(f1, f2));
                            transaction_reader::process_data::<aggregator::AverageGasProfile, _>(
                                paths, &f,
                            )
                            .await
                        }
                    }
                }
                ProcessTxAction::FilterTo { target_addr_hex } => {
                    let to = Address::decode(&target_addr_hex).unwrap();
                    let f = Arc::new(filter::EthTxTo(to));
                    transaction_reader::process_data::<aggregator::Echo, _>(paths, &f).await
                }
                ProcessTxAction::GasDistribution => {
                    let f1 = filter::MatchFlatStatus(transaction_reader::FlatTxStatus::Succeeded);
                    let f2 = filter::MatchFlatStatus(transaction_reader::FlatTxStatus::GasLimit);
                    let f = Arc::new(filter::Or::new(f1, f2));
                    transaction_reader::process_data::<aggregator::GroupByGas, _>(paths, &f).await
                }
                ProcessTxAction::NearGasVsEvmGas => {
                    let f = Arc::new(filter::StatusExecuted);
                    transaction_reader::process_data::<aggregator::GasComparison, _>(paths, &f)
                        .await
                }
                ProcessTxAction::OutcomeDistribution => {
                    let f = Arc::new(filter::NoFilter);
                    transaction_reader::process_data::<aggregator::GroupByFlatStatus, _>(paths, &f)
                        .await
                }
                ProcessTxAction::FilterGasRange {
                    min_near,
                    min_evm,
                    max_near,
                    max_evm,
                } => {
                    let f = Arc::new(filter::GeneralGasFilter {
                        min_near,
                        min_evm,
                        max_near,
                        max_evm,
                    });
                    transaction_reader::process_data::<aggregator::Echo, _>(paths, &f).await
                }
                ProcessTxAction::FromToGasUsed => {
                    let f = Arc::new(filter::NoFilter);
                    transaction_reader::process_data::<aggregator::FromToGasUsage, _>(paths, &f)
                        .await
                }
            }
        }
    }

    Ok(())
}

async fn send_transaction<T: AsRef<str>>(
    client: &AuroraClient<T>,
    sk: &secp256k1::SecretKey,
    to: Option<Address>,
    amount: Wei,
    input: Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = utils::address_from_secret_key(sk);
    println!("FROM {:?}", source);

    let nonce = client.get_nonce(source).await?;
    let chain_id = client.get_chain_id().await?;

    let tx_hash = client
        .eth_transaction(to, amount, sk, chain_id, nonce, input)
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

    Ok(())
}
