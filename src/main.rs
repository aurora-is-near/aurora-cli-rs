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

use aurora_engine_types::{
    types::{Address, Wei},
    U256,
};
use borsh::BorshSerialize;
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
        Command::GetNearResult { receipt_id_b58 } => {
            let tx_hash = bs58::decode(receipt_id_b58.as_str()).into_vec().unwrap();
            let tx_hash = aurora_engine_types::H256::from_slice(&tx_hash);
            let outcome = client.get_near_receipt_outcome(tx_hash).await?;
            println!("{:?}", outcome);
        }
        Command::Xcc {
            target_near_account,
            method_name,
            json_args,
            json_args_stdin,
            deposit_yocto,
            attached_gas,
        } => {
            let source_private_key_hex = config.get_evm_secret_key();
            let sk_bytes = utils::hex_to_arr32(source_private_key_hex)?;
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
        Command::ProcessTxData {
            action,
            input_files_list_path,
        } => cli::process_tx_data::execute_command(action, input_files_list_path).await,
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
