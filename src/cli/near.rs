use crate::{
    client::{AuroraClient, ClientError},
    config::Config,
    utils,
};
use aurora_engine::parameters::{
    DeployErc20TokenArgs, GetStorageAtArgs, PauseEthConnectorCallArgs,
};
use aurora_engine_types::{
    account_id::AccountId,
    parameters::{CrossContractCallArgs, PromiseArgs, PromiseCreateArgs},
    types::{Address, NearGas, Wei, Yocto},
    U256,
};
use borsh::BorshSerialize;
use clap::Subcommand;
use near_primitives::views::FinalExecutionOutcomeView;
use std::str::FromStr;

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
    // get nep141_from_erc20
    GetBridgedNep141 {
        erc_20_address_hex: String,
    },
    GetAuroraErc20 {
        nep_141_account: String,
    },
    GetEngineBridgeProver,
    // get_chain_id
    GetChainId,
    // get_upgrade_index
    GetUpgradeIndex,
    // get_block_hash
    GetBlockHash {
        block_number: String,
    },
    // get_code
    GetCode {
        address_hex: String,
    },
    // get_balance
    GetBalance {
        address_hex: String,
    },
    // get_nonce
    GetNonce {
        address_hex: String,
    },
    // get_storage_at
    GetStorageAt {
        address_hex: String,
        key_hex: String,
    },
    // get_paused_flags
    GetPausedFlags,
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
    // deploy_code
    DeployCode {
        code_byte_hex: String,
    },
    // register_relayer
    RegisterRelayer {
        relayer_eth_address_hex: String,
    },
    // deploy_erc20_token
    DeployERC20Token {
        nep141: String,
    },
    // deposit
    Deposit {
        raw_proof: String,
    }, // storage_deposit
    // set_paused_flags
    SetPausedFlags {
        paused_mask: String,
    },
}

pub async fn execute_command(
    command: Command,
    client: &AuroraClient,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Command::Read { subcommand } => match subcommand {
            ReadCommand::GetReceiptResult { receipt_id_b58 } => {
                let tx_hash = bs58::decode(receipt_id_b58.as_str()).into_vec().unwrap();
                let outcome = client
                    .get_near_receipt_outcome(tx_hash.as_slice().try_into().unwrap())
                    .await?;
                println!("{outcome:?}");
            }
            ReadCommand::EngineCall {
                sender_addr_hex,
                target_addr_hex,
                amount,
                input_data_hex,
            } => {
                let (sender, target, amount) =
                    parse_read_call_args(sender_addr_hex, &target_addr_hex, amount.as_deref());
                let input = hex::decode(input_data_hex)?;
                let result = client
                    .view_contract_call(sender, target, amount, input)
                    .await
                    .unwrap();
                println!("{result:?}");
            }
            ReadCommand::EngineErc20 {
                erc20,
                target_addr_hex,
                amount,
                sender_addr_hex,
            } => {
                let (sender, target, amount) =
                    parse_read_call_args(sender_addr_hex, &target_addr_hex, amount.as_deref());
                let input = erc20.abi_encode()?;
                let result = client
                    .view_contract_call(sender, target, amount, input)
                    .await
                    .unwrap();
                println!("{result:?}");
            }
            ReadCommand::Solidity {
                contract_call,
                target_addr_hex,
                amount,
                sender_addr_hex,
            } => {
                let (sender, target, amount) =
                    parse_read_call_args(sender_addr_hex, &target_addr_hex, amount.as_deref());
                let input = contract_call.abi_encode()?;
                let result = client
                    .view_contract_call(sender, target, amount, input)
                    .await
                    .unwrap();
                println!("{result:?}");
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
                    &target_near_account,
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
                println!("{result:?}");
            }
            ReadCommand::GetBridgedNep141 { erc_20_address_hex } => {
                let erc20 = Address::decode(&erc_20_address_hex).unwrap();
                match client.get_nep141_from_erc20(erc20).await {
                    Ok(nep_141_account) => println!("{nep_141_account}"),
                    Err(e) => {
                        let error_msg = format!("{e:?}");
                        if error_msg.contains("ERC20_NOT_FOUND") {
                            println!("No NEP-141 account associated with {erc_20_address_hex}");
                        } else {
                            panic!("{error_msg}");
                        }
                    }
                };
            }
            ReadCommand::GetAuroraErc20 { nep_141_account } => {
                println!(
                    "{:?}",
                    client.get_erc20_from_nep141(&nep_141_account).await?
                );
            }
            ReadCommand::GetEngineBridgeProver => {
                println!("{:?}", client.get_bridge_prover().await?);
            }
            ReadCommand::GetChainId => {
                let chain_id = {
                    let result = client.near_view_call("get_chain_id".into(), vec![]).await?;
                    U256::from_big_endian(&result.result).low_u64()
                };
                println!("{chain_id:?}");
            }
            ReadCommand::GetUpgradeIndex => {
                let upgrade_index = {
                    let result = client
                        .near_view_call("get_upgrade_index".into(), vec![])
                        .await?;
                    U256::from_big_endian(&result.result).low_u64()
                };
                println!("{upgrade_index:?}");
            }
            ReadCommand::GetBlockHash { block_number } => {
                let height_serialized: u128 = block_number.parse::<u128>().unwrap();
                let block_hash = {
                    let result = client
                        .near_view_call(
                            "get_block_hash".into(),
                            height_serialized.to_le_bytes().to_vec(),
                        )
                        .await?
                        .result;
                    result
                };
                println!("{:?}", hex::encode(block_hash));
            }
            ReadCommand::GetCode { address_hex } => {
                let code = client
                    .near_view_call("get_code".into(), address_hex.as_bytes().to_vec())
                    .await?
                    .result;
                println!("{code:?}");
            }
            ReadCommand::GetBalance { address_hex } => {
                let balance = {
                    let result = client
                        .near_view_call("get_balance".into(), address_hex.as_bytes().to_vec())
                        .await?;
                    U256::from_big_endian(&result.result).low_u64()
                };
                println!("{balance:?}");
            }
            ReadCommand::GetNonce { address_hex } => {
                let nonce = {
                    let result = client
                        .near_view_call("get_nonce".into(), address_hex.as_bytes().to_vec())
                        .await?;
                    U256::from_big_endian(&result.result).low_u64()
                };
                println!("{nonce:?}");
            }
            ReadCommand::GetStorageAt {
                address_hex,
                key_hex,
            } => {
                let mut buffer: Vec<u8> = Vec::new();
                let key_bytes32: [u8; 32] = hex::decode(key_hex)?.try_into().map_err(|_| "Invalid key length")?;
                let input = GetStorageAtArgs {
                    address: Address::decode(&address_hex).unwrap(),
                    key: key_bytes32,
                };
                input.serialize(&mut buffer)?;
                let storage = {
                    let result = client
                        .near_view_call("get_storage_at".into(), buffer)
                        .await?;
                    hex::encode(result.result)
                };
                println!("{storage:?}");
            }
            ReadCommand::GetPausedFlags => {
                let paused_flags = client
                    .near_view_call("get_paused_flags".into(), vec![])
                    .await?
                    .result;
                println!("{paused_flags:?}");
            }
        },
        Command::Write { subcommand } => match subcommand {
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
                    &target_near_account,
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
                println!("{result:?}");
            }
            WriteCommand::EngineCall {
                target_addr_hex,
                amount,
                input_data_hex,
            } => {
                let (sk, target, amount) =
                    parse_write_call_args(config, &target_addr_hex, amount.as_deref());
                let input = hex::decode(input_data_hex)?;
                let result =
                    send_as_near_transaction(client, &sk, Some(target), amount, input).await?;
                println!("{result:?}");
            }
            WriteCommand::EngineErc20 {
                erc20,
                target_addr_hex,
                amount,
            } => {
                let (sk, target, amount) =
                    parse_write_call_args(config, &target_addr_hex, amount.as_deref());
                let input = erc20.abi_encode()?;
                let result =
                    send_as_near_transaction(client, &sk, Some(target), amount, input).await?;
                println!("{result:?}");
            }
            WriteCommand::Solidity {
                contract_call,
                target_addr_hex,
                amount,
            } => {
                let (sk, target, amount) =
                    parse_write_call_args(config, &target_addr_hex, amount.as_deref());
                let input = contract_call.abi_encode()?;
                let result =
                    send_as_near_transaction(client, &sk, Some(target), amount, input).await?;
                println!("{result:?}");
            }
            WriteCommand::FactoryUpdate { wasm_bytes_path } => {
                let args = std::fs::read(wasm_bytes_path).unwrap();
                let tx_outcome = client
                    .near_contract_call("factory_update".into(), args)
                    .await
                    .unwrap();
                println!("{tx_outcome:?}");
            }
            WriteCommand::DeployCode { code_byte_hex } => {
                let input = hex::decode(code_byte_hex)?;
                let tx_outcome = client
                    .near_contract_call("deploy_code".into(), input)
                    .await?;
                println!("{tx_outcome:?}");
            }
            WriteCommand::RegisterRelayer {
                relayer_eth_address_hex,
            } => {
                let relayer = hex::decode(relayer_eth_address_hex)?;
                let tx_outcome = client
                    .near_contract_call("register_relayer".into(), relayer)
                    .await?;
                println!("{tx_outcome:?}");
            }
            WriteCommand::DeployERC20Token { nep141 } => {
                let mut buffer: Vec<u8> = Vec::new();
                let nep141: AccountId = nep141.parse().unwrap();
                let input = DeployErc20TokenArgs { nep141 };
                input.serialize(&mut buffer)?;
                let tx_outcome = client
                    .near_contract_call("deploy_erc20_token".into(), buffer)
                    .await?;
                println!("{tx_outcome:?}");
            }
            WriteCommand::Deposit { raw_proof } => {
                let tx_outcome = client
                    .near_contract_call("deposit".into(), raw_proof.as_bytes().to_vec())
                    .await?;
                println!("{tx_outcome:?}");
            }
            WriteCommand::SetPausedFlags { paused_mask } => {
                let mut buffer: Vec<u8> = Vec::new();
                let input = PauseEthConnectorCallArgs {
                    paused_mask: u8::from_str(&paused_mask).unwrap(),
                };
                input.serialize(&mut buffer)?;
                let tx_outcome = client
                    .near_contract_call("set_paused_flags".into(), buffer)
                    .await?;
                println!("{tx_outcome:?}");
            }
        },
    };
    Ok(())
}

fn parse_read_call_args(
    sender_addr_hex: Option<String>,
    target_addr_hex: &str,
    amount: Option<&str>,
) -> (Address, Address, Wei) {
    let target = Address::decode(target_addr_hex).unwrap();
    let sender = sender_addr_hex
        .map(|x| Address::decode(&x).unwrap())
        .unwrap_or_default();
    let amount = amount.map_or_else(Wei::zero, |a| Wei::new(U256::from_dec_str(a).unwrap()));

    (sender, target, amount)
}

fn parse_write_call_args(
    config: &Config,
    target_addr_hex: &str,
    amount: Option<&str>,
) -> (libsecp256k1::SecretKey, Address, Wei) {
    let source_private_key_hex = config.get_evm_secret_key();
    let sk_bytes = utils::hex_to_arr32(source_private_key_hex).unwrap();
    let sk = libsecp256k1::SecretKey::parse(&sk_bytes).unwrap();
    let target = Address::decode(target_addr_hex).unwrap();
    let amount = amount.map_or_else(Wei::zero, |a| Wei::new(U256::from_dec_str(a).unwrap()));
    (sk, target, amount)
}

fn parse_xcc_args(
    target_near_account: &str,
    method_name: String,
    json_args: Option<String>,
    json_args_stdin: Option<bool>,
    deposit_yocto: Option<String>,
    attached_gas: Option<String>,
) -> PromiseCreateArgs {
    let near_args = json_args.map_or_else(
        || match json_args_stdin {
            Some(true) => {
                let mut buf = String::new();
                std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf).unwrap();
                buf.into_bytes()
            }
            None | Some(false) => Vec::new(),
        },
        String::into_bytes,
    );
    let attached_balance =
        deposit_yocto.map_or_else(|| Yocto::new(0), |x| Yocto::new(x.parse().unwrap()));
    let attached_gas = attached_gas.map_or_else(
        || NearGas::new(30_000_000_000_000),
        |gas| NearGas::new(gas.parse().unwrap()),
    );

    PromiseCreateArgs {
        target_account_id: target_near_account.parse().unwrap(),
        method: method_name,
        args: near_args,
        attached_balance,
        attached_gas,
    }
}

async fn send_as_near_transaction(
    client: &AuroraClient,
    sk: &libsecp256k1::SecretKey,
    to: Option<Address>,
    amount: Wei,
    input: Vec<u8>,
) -> Result<FinalExecutionOutcomeView, ClientError> {
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
