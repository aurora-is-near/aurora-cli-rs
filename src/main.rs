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
    let (aurora_endpoint, near_endpoint) = match network {
        Network::Mainnet => (AURORA_MAINNET_ENDPOINT, NEAR_MAINNET_ENDPOINT),
        Network::Testnet => (AURORA_TESTNET_ENDPOINT, NEAR_TESTNET_ENDPOINT),
    };
    let client = AuroraClient::new(format!("{}{}", aurora_endpoint, api_key), near_endpoint);

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
