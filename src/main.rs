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
use borsh::BorshSerialize;
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
        args.signer_key_path,
    );

    match args.command {
        Command::GetResult { tx_hash_hex } => {
            let tx_hash = aurora_engine_types::H256::from_slice(&hex::decode(tx_hash_hex).unwrap());
            let outcome = client.get_transaction_outcome(tx_hash).await?;
            println!("{:?}", outcome);
        }
        Command::GetNearResult { receipt_id_b58 } => {
            let tx_hash = bs58::decode(receipt_id_b58.as_str()).into_vec().unwrap();
            let tx_hash = aurora_engine_types::H256::from_slice(&tx_hash);
            let outcome = client.get_near_receipt_outcome(tx_hash).await?;
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
        Command::Xcc {
            source_private_key_hex,
            target_near_account,
            method_name,
            json_args,
            json_args_stdin,
            deposit_yocto,
            attached_gas,
        } => {
            let sk_bytes = utils::hex_to_arr32(&source_private_key_hex)?;
            let sk = secp256k1::SecretKey::parse(&sk_bytes).unwrap();
            let near_args = match json_args {
                Some(args) => args.into_bytes(),
                None => match json_args_stdin {
                    Some(true) => {
                        let mut buf = String::new();
                        std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf).unwrap();
                        buf.into_bytes()
                    }
                    None | Some(false) => Vec::new(),
                },
            };
            let attached_balance = match deposit_yocto {
                Some(x) => aurora_engine_types::types::Yocto::new(x.parse().unwrap()),
                None => aurora_engine_types::types::Yocto::new(0),
            };
            // TODO: there is an issue with the NEAR nonce tracking if I do two calls in a row
            /*if attached_balance.as_u128() > 0 {
                // If we want to spend NEAR then we need to approve the precompile to spend our wNEAR.
                const APPROVE_SELECTOR: &[u8] = &[0x09u8, 0x5e, 0xa7, 0xb3];
                let input = [APPROVE_SELECTOR, &ethabi::encode(&[
                    ethabi::Token::Address(aurora_engine_precompiles::xcc::cross_contract_call::ADDRESS.raw()),
                    ethabi::Token::Uint(U256::from(u128::MAX)),
                ])].concat();
                let result = send_as_near_transaction(
                    &client,
                    &sk,
                    Address::decode("34aadb3d3f359c7bfefa87f7a0ed4dbe5ba17d78").ok(),
                    Wei::zero(),
                    input,
                )
                .await?;
                println!("APPROVE: {:?}\n\n", result);
            }*/
            let attached_gas = match attached_gas {
                Some(gas) => aurora_engine_types::types::NearGas::new(gas.parse().unwrap()),
                None => aurora_engine_types::types::NearGas::new(30_000_000_000_000),
            };
            let promise = aurora_engine_types::parameters::PromiseArgs::Create(
                aurora_engine_types::parameters::PromiseCreateArgs {
                    target_account_id: target_near_account.parse().unwrap(),
                    method: method_name,
                    args: near_args,
                    attached_balance,
                    attached_gas,
                },
            );
            let precompile_args =
                aurora_engine_types::parameters::CrossContractCallArgs::Eager(promise);
            let result = send_as_near_transaction(
                &client,
                &sk,
                Some(aurora_engine_precompiles::xcc::cross_contract_call::ADDRESS),
                Wei::zero(),
                precompile_args.try_to_vec().unwrap(),
            )
            .await?;
            println!("{:?}", result);
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
        Command::FactoryUpdate { wasm_bytes_path } => {
            let args = std::fs::read(wasm_bytes_path).unwrap();
            let tx_outcome = client
                .near_contract_call("factory_update".into(), args)
                .await
                .unwrap();
            println!("{:?}", tx_outcome);
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

async fn send_as_near_transaction<T: AsRef<str>>(
    client: &AuroraClient<T>,
    sk: &secp256k1::SecretKey,
    to: Option<Address>,
    amount: Wei,
    input: Vec<u8>,
) -> Result<near_primitives::views::FinalExecutionOutcomeView, Box<dyn std::error::Error>> {
    let sender_address = utils::address_from_secret_key(sk);
    let nonce = {
        let result = client
            .near_view_call("get_nonce".into(), sender_address.as_bytes().to_vec())
            .await?;
        U256::from_big_endian(&result.result)
    };
    let tx = aurora_engine_transactions::legacy::TransactionLegacy {
        nonce,
        gas_price: U256::zero(),
        gas_limit: U256::from(u64::MAX),
        to,
        value: amount,
        data: input,
    };
    let chain_id = {
        let result = client
            .near_view_call("get_chain_id".into(), sender_address.as_bytes().to_vec())
            .await?;
        U256::from_big_endian(&result.result).low_u64()
    };
    let signed_tx = aurora_engine_transactions::EthTransactionKind::Legacy(
        utils::sign_transaction(tx, chain_id, sk),
    );
    let result = client
        .near_contract_call("submit".into(), (&signed_tx).into())
        .await?;
    Ok(result)
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
        match client.get_transaction_outcome(tx_hash).await {
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
