use crate::{client::AuroraClient, config::Config, utils};
use aurora_engine_types::{
    parameters::{CrossContractCallArgs, PromiseArgs, PromiseCreateArgs},
    types::{Address, NearGas, Wei, Yocto},
    U256,
};
use borsh::BorshSerialize;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    Read {
        #[clap(subcommand)]
        subcommand: ReadCommand,
    },
    Write {
        #[clap(subcommand)]
        subcommand: WriteCommand,
    },
}

#[derive(Subcommand)]
pub enum ReadCommand {
    GetReceiptResult {
        receipt_id_b58: String,
    },
    EngineCall {
        #[clap(short, long)]
        sender_addr_hex: Option<String>,
        #[clap(short, long)]
        target_addr_hex: String,
        #[clap(short, long)]
        amount: Option<String>,
        #[clap(short, long)]
        input_data_hex: String,
    },
    EngineXccDryRun {
        #[clap(short, long)]
        sender_address_hex: String,
        #[clap(short, long)]
        target_near_account: String,
        #[clap(short, long)]
        method_name: String,
        #[clap(short, long)]
        json_args: Option<String>,
        #[clap(long)]
        json_args_stdin: Option<bool>,
        #[clap(short, long)]
        deposit_yocto: Option<String>,
        #[clap(short, long)]
        attached_gas: Option<String>,
    },
    EngineErc20 {
        #[clap(short, long)]
        sender_addr_hex: Option<String>,
        #[clap(short, long)]
        target_addr_hex: String,
        #[clap(short, long)]
        amount: Option<String>,
        #[clap(subcommand)]
        erc20: crate::cli::erc20::Erc20,
    },
    Solidity {
        #[clap(short, long)]
        sender_addr_hex: Option<String>,
        #[clap(short, long)]
        target_addr_hex: String,
        #[clap(short, long)]
        amount: Option<String>,
        #[clap(subcommand)]
        contract_call: crate::cli::solidity::Solidity,
    },
    GetBridgedNep141 {
        erc_20_address_hex: String,
    },
    GetAuroraErc20 {
        nep_141_account: String,
    },
    GetEngineBridgeProver,
}

#[derive(Subcommand)]
pub enum WriteCommand {
    EngineXcc {
        #[clap(short, long)]
        target_near_account: String,
        #[clap(short, long)]
        method_name: String,
        #[clap(short, long)]
        json_args: Option<String>,
        #[clap(long)]
        json_args_stdin: Option<bool>,
        #[clap(short, long)]
        deposit_yocto: Option<String>,
        #[clap(short, long)]
        attached_gas: Option<String>,
    },
    EngineCall {
        #[clap(short, long)]
        target_addr_hex: String,
        #[clap(short, long)]
        amount: Option<String>,
        #[clap(short, long)]
        input_data_hex: String,
    },
    Solidity {
        #[clap(short, long)]
        target_addr_hex: String,
        #[clap(short, long)]
        amount: Option<String>,
        #[clap(subcommand)]
        contract_call: crate::cli::solidity::Solidity,
    },
    EngineErc20 {
        #[clap(short, long)]
        target_addr_hex: String,
        #[clap(short, long)]
        amount: Option<String>,
        #[clap(subcommand)]
        erc20: crate::cli::erc20::Erc20,
    },
    FactoryUpdate {
        wasm_bytes_path: String,
    },
}

pub async fn execute_command<T: AsRef<str>>(
    command: Command,
    client: &AuroraClient<T>,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Command::Read { subcommand } => match subcommand {
            // getter methods
            ReadCommand::GetReceiptResult { receipt_id_b58 } => {
                let tx_hash = bs58::decode(receipt_id_b58.as_str()).into_vec().unwrap();
                let outcome = client
                    .get_near_receipt_outcome(tx_hash.as_slice().try_into().unwrap())
                    .await?;
                println!("{:?}", outcome);
            }
            ReadCommand::EngineCall {
                sender_addr_hex,
                target_addr_hex,
                amount,
                input_data_hex,
            } => {
                let (sender, target, amount) =
                    parse_read_call_args(sender_addr_hex, target_addr_hex, amount);
                let input = hex::decode(input_data_hex)?;
                let result = client
                    .view_contract_call(sender, target, amount, input)
                    .await
                    .unwrap();
                println!("{:?}", result);
            }
            ReadCommand::EngineErc20 {
                erc20,
                target_addr_hex,
                amount,
                sender_addr_hex,
            } => {
                let (sender, target, amount) =
                    parse_read_call_args(sender_addr_hex, target_addr_hex, amount);
                let input = erc20.abi_encode()?;
                let result = client
                    .view_contract_call(sender, target, amount, input)
                    .await
                    .unwrap();
                println!("{:?}", result);
            }
            ReadCommand::Solidity {
                contract_call,
                target_addr_hex,
                amount,
                sender_addr_hex,
            } => {
                let (sender, target, amount) =
                    parse_read_call_args(sender_addr_hex, target_addr_hex, amount);
                let input = contract_call.abi_encode()?;
                let result = client
                    .view_contract_call(sender, target, amount, input)
                    .await
                    .unwrap();
                println!("{:?}", result);
            }
            ReadCommand::EngineXccDryRun {
                target_near_account,
                sender_address_hex,
                method_name,
                json_args,
                json_args_stdin,
                deposit_yocto,
                attached_gas,
            } => {
                let promise = PromiseArgs::Create(parse_xcc_args(
                    target_near_account,
                    method_name,
                    json_args,
                    json_args_stdin,
                    deposit_yocto,
                    attached_gas,
                ));
                let precompile_args = CrossContractCallArgs::Eager(promise);
                let sender = Address::decode(&sender_address_hex).unwrap();
                let result = client
                    .view_contract_call(
                        sender,
                        aurora_engine_precompiles::xcc::cross_contract_call::ADDRESS,
                        Wei::zero(),
                        precompile_args.try_to_vec().unwrap(),
                    )
                    .await?;
                println!("{:?}", result);
            }
            ReadCommand::GetBridgedNep141 { erc_20_address_hex } => {
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
            ReadCommand::GetAuroraErc20 { nep_141_account } => {
                println!("{:?}", client.get_erc20_from_nep141(&nep_141_account).await);
            }
            ReadCommand::GetEngineBridgeProver => {
                println!("{:?}", client.get_bridge_prover().await);
            }
            // get_chain_id
            // get_upgrade_index
            // get_block_hash
            // get_code
            // get_balance
            // get_nonce
            // get_storage_at
            // get_paused_flags
            // get_accounts_counter
            // get nep141_from_erc20
            // ft_total_supply
            // ft_total_eth_supply_on_near
            // ft_total_eth_supply_on_aurora
            // ft_balance_of
            // ft_balance_of_eth
            // storage_balance_of
            // ft_metadata
            // verify_log_entry
        },
        Command::Write { subcommand } => match subcommand {
            // All "submit" engine method
            WriteCommand::EngineXcc {
                target_near_account,
                method_name,
                json_args,
                json_args_stdin,
                deposit_yocto,
                attached_gas,
            } => {
                let source_private_key_hex = config.get_evm_secret_key();
                let sk_bytes = utils::hex_to_arr32(source_private_key_hex)?;
                let sk = libsecp256k1::SecretKey::parse(&sk_bytes).unwrap();
                let promise = PromiseArgs::Create(parse_xcc_args(
                    target_near_account,
                    method_name,
                    json_args,
                    json_args_stdin,
                    deposit_yocto,
                    attached_gas,
                ));
                let precompile_args = CrossContractCallArgs::Eager(promise);
                let result = send_as_near_transaction(
                    client,
                    &sk,
                    Some(aurora_engine_precompiles::xcc::cross_contract_call::ADDRESS),
                    Wei::zero(),
                    precompile_args.try_to_vec().unwrap(),
                )
                .await?;
                println!("{:?}", result);
            }
            WriteCommand::EngineCall {
                target_addr_hex,
                amount,
                input_data_hex,
            } => {
                let (sk, target, amount) = parse_write_call_args(config, target_addr_hex, amount);
                let input = hex::decode(input_data_hex)?;
                let result =
                    send_as_near_transaction(client, &sk, Some(target), amount, input).await?;
                println!("{:?}", result);
            }
            WriteCommand::EngineErc20 {
                erc20,
                target_addr_hex,
                amount,
            } => {
                let (sk, target, amount) = parse_write_call_args(config, target_addr_hex, amount);
                let input = erc20.abi_encode()?;
                let result =
                    send_as_near_transaction(client, &sk, Some(target), amount, input).await?;
                println!("{:?}", result);
            }
            WriteCommand::Solidity {
                contract_call,
                target_addr_hex,
                amount,
            } => {
                let (sk, target, amount) = parse_write_call_args(config, target_addr_hex, amount);
                let input = contract_call.abi_encode()?;
                let result =
                    send_as_near_transaction(client, &sk, Some(target), amount, input).await?;
                println!("{:?}", result);
            }
            WriteCommand::FactoryUpdate { wasm_bytes_path } => {
                let args = std::fs::read(wasm_bytes_path).unwrap();
                let tx_outcome = client
                // I cannot find this engine method called as "factory_update"
                    .near_contract_call("factory_update".into(), args)
                    .await
                    .unwrap();
                println!("{:?}", tx_outcome);
            }
            // deploy_code 
            // call
            // meta-call
            // register_relayer
            // ft_on_transfer
            // deploy_erc20_token
            // begin_chain
            // begin_block
            // withdraw
            // deposit
            // is_used_proof
            // ft_transfer
            // ft_transfer_call
            // storage_deposit
            // storage_unregister
            // storage_withdraw
            // set_paused_flags
            // mint_account
        },
    };
    Ok(())
}

fn parse_read_call_args(
    sender_addr_hex: Option<String>,
    target_addr_hex: String,
    amount: Option<String>,
) -> (Address, Address, Wei) {
    let target = Address::decode(&target_addr_hex).unwrap();
    let sender = sender_addr_hex
        .map(|x| Address::decode(&x).unwrap())
        .unwrap_or_default();
    let amount = amount
        .as_ref()
        .map(|a| Wei::new(U256::from_dec_str(a).unwrap()))
        .unwrap_or_else(Wei::zero);

    (sender, target, amount)
}

fn parse_write_call_args(
    config: &Config,
    target_addr_hex: String,
    amount: Option<String>,
) -> (libsecp256k1::SecretKey, Address, Wei) {
    let source_private_key_hex = config.get_evm_secret_key();
    let sk_bytes = utils::hex_to_arr32(source_private_key_hex).unwrap();
    let sk = libsecp256k1::SecretKey::parse(&sk_bytes).unwrap();
    let target = Address::decode(&target_addr_hex).unwrap();
    let amount = amount
        .as_ref()
        .map(|a| Wei::new(U256::from_dec_str(a).unwrap()))
        .unwrap_or_else(Wei::zero);
    (sk, target, amount)
}

fn parse_xcc_args(
    target_near_account: String,
    method_name: String,
    json_args: Option<String>,
    json_args_stdin: Option<bool>,
    deposit_yocto: Option<String>,
    attached_gas: Option<String>,
) -> PromiseCreateArgs {
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
        Some(x) => Yocto::new(x.parse().unwrap()),
        None => Yocto::new(0),
    };
    let attached_gas = match attached_gas {
        Some(gas) => NearGas::new(gas.parse().unwrap()),
        None => NearGas::new(30_000_000_000_000),
    };
    PromiseCreateArgs {
        target_account_id: target_near_account.parse().unwrap(),
        method: method_name,
        args: near_args,
        attached_balance,
        attached_gas,
    }
}

async fn send_as_near_transaction<T: AsRef<str>>(
    client: &AuroraClient<T>,
    sk: &libsecp256k1::SecretKey,
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
