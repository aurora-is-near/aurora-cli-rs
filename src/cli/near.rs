use crate::{
    client::{AuroraClient, ClientError},
    config::{Config, Network},
    utils,
};
use aurora_engine::{
    fungible_token::{FungibleReferenceHash, FungibleTokenMetadata},
    parameters::{
        DeployErc20TokenArgs, GetStorageAtArgs, InitCallArgs, NewCallArgs,
        PauseEthConnectorCallArgs,
    },
};
use aurora_engine_types::{
    account_id::AccountId,
    parameters::{CrossContractCallArgs, PromiseArgs, PromiseCreateArgs},
    types::{Address, NearGas, Wei, Yocto},
    U256,
};
use borsh::{BorshDeserialize, BorshSerialize};
use clap::Subcommand;
use near_primitives::{
    account::{AccessKey, Account},
    hash::CryptoHash,
    state_record::StateRecord,
    views::FinalExecutionOutcomeView,
};
use std::{path::Path, str::FromStr};

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
    Init {
        #[clap(subcommand)]
        subcommand: InitCommand,
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
    /// Deploy and initialize a new instance of the Aurora Engine.
    /// Uses the `engine_account_id` from the config as the target account.
    /// `config.near_key_path` must point to a full access key for `engine_account_id`.
    EngineInit {
        /// Path to the Wasm artifact for the engine contract.
        #[clap(short, long)]
        wasm_path: String,
        #[clap(short, long)]
        chain_id: u64,
        #[clap(short, long)]
        owner_id: String,
        /// How many blocks after staging upgrade can deploy it.
        /// Default value is 0 (i.e. no delay in upgrading).
        #[clap(short, long)]
        upgrade_delay_blocks: Option<u64>,
        /// The account used to check deposit proofs in the ETH connector.
        /// The default value is equal to the `engine_account_id`.
        #[clap(short, long)]
        prover_account: Option<String>,
        /// The address of the locker on Ethereum for the ETH connector.
        /// The default value is 0x00.
        #[clap(short, long)]
        eth_custodian_address: Option<String>,
        /// The metadata for the ETH token the connector creates.
        /// The value is expected to be a value JSON string
        /// (see https://nomicon.io/Standards/Tokens/FungibleToken/Metadata for fields).
        /// The default value is 18 decimals with name and symbol equal to "localETH".
        #[clap(short, long)]
        ft_metadata: Option<String>,
    },
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

#[derive(Subcommand)]
pub enum InitCommand {
    /// Add aurora account to the nearcore genesis file
    Genesis {
        #[clap(short, long)]
        path: String,
    },
    /// Modify CLI config to use local nearcore as RPC.
    /// Optionally change the CLI access key to the one for the aurora account.
    LocalConfig {
        #[clap(short, long)]
        nearcore_config_path: String,
        #[clap(short, long)]
        aurora_access_key_path: Option<String>,
    },
}

pub async fn execute_command(
    command: Command,
    client: &AuroraClient,
    config: &Config,
    config_path: &str,
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
                let input = utils::hex_to_vec(&input_data_hex)?;
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
                let sender = utils::hex_to_address(&sender_address_hex)?;
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
                let erc20 = utils::hex_to_address(&erc_20_address_hex)?;
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
                let input = GetStorageAtArgs {
                    address: utils::hex_to_address(&address_hex)?,
                    key: utils::hex_to_arr(&key_hex)?,
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
            WriteCommand::EngineInit {
                wasm_path,
                chain_id,
                owner_id,
                upgrade_delay_blocks,
                prover_account,
                eth_custodian_address,
                ft_metadata,
            } => {
                let wasm_bytes = tokio::fs::read(wasm_path).await?;
                let prover_account: AccountId = {
                    let prover_account = prover_account
                        .as_deref()
                        .unwrap_or(&config.engine_account_id);
                    prover_account
                        .parse()
                        .expect("Prover account is an invalid Near account")
                };
                let eth_custodian_address = eth_custodian_address
                    .as_deref()
                    .map(utils::hex_to_address)
                    .transpose()
                    .expect("Invalid eth_custodian_address")
                    .unwrap_or_default();
                let metadata = parse_ft_metadata(ft_metadata);

                let new_args = NewCallArgs {
                    chain_id: aurora_engine_types::types::u256_to_arr(&U256::from(chain_id)),
                    owner_id: owner_id.parse().expect("Invalid owner_id"),
                    bridge_prover_id: prover_account.clone(),
                    upgrade_delay_blocks: upgrade_delay_blocks.unwrap_or_default(),
                };

                let init_args = InitCallArgs {
                    prover_account,
                    eth_custodian_address: eth_custodian_address.encode(),
                    metadata,
                };

                let deploy_response = client.near_deploy_contract(wasm_bytes).await?;
                assert_tx_success(&deploy_response);
                let next_nonce = deploy_response.transaction.nonce + 1;

                let new_response = client
                    .near_contract_call_with_nonce(
                        "new".into(),
                        new_args.try_to_vec().unwrap(),
                        next_nonce,
                    )
                    .await?;
                assert_tx_success(&new_response);
                let next_nonce = new_response.transaction.nonce + 1;

                let init_response = client
                    .near_contract_call_with_nonce(
                        "new_eth_connector".into(),
                        init_args.try_to_vec().unwrap(),
                        next_nonce,
                    )
                    .await?;
                assert_tx_success(&init_response);

                println!(
                    "Deploy of Engine to {} successful",
                    config.engine_account_id
                );
            }
            WriteCommand::EngineXcc {
                target_near_account,
                method_name,
                json_args,
                json_args_stdin,
                deposit_yocto,
                attached_gas,
            } => {
                let source_private_key_hex = config.get_evm_secret_key();
                let sk_bytes = utils::hex_to_arr(source_private_key_hex)?;
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
                let input = utils::hex_to_vec(&input_data_hex)?;
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
                let input = utils::hex_to_vec(&code_byte_hex)?;
                let tx_outcome = client
                    .near_contract_call("deploy_code".into(), input)
                    .await?;
                println!("{tx_outcome:?}");
            }
            WriteCommand::RegisterRelayer {
                relayer_eth_address_hex,
            } => {
                let relayer = utils::hex_to_vec(&relayer_eth_address_hex)?;
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
        Command::Init { subcommand } => match subcommand {
            InitCommand::Genesis { path } => {
                let mut genesis = near_chain_configs::Genesis::from_file(
                    &path,
                    near_chain_configs::GenesisValidationMode::UnsafeFast,
                );
                let records = genesis.force_read_records();
                let aurora_id: near_primitives::account::id::AccountId =
                    config.engine_account_id.parse().unwrap();
                let contains_aurora = records.0.iter().any(|record| {
                    if let StateRecord::AccessKey { account_id, .. } = record {
                        account_id == &aurora_id
                    } else {
                        false
                    }
                });
                if contains_aurora {
                    println!("Aurora account already present");
                    return Ok(());
                }
                let secret_key = near_crypto::SecretKey::from_random(near_crypto::KeyType::ED25519);
                let public_key = secret_key.public_key();
                let aurora_amount = 1_000_000_000_000_000_000_000_000_000_000_000; // 1e9 NEAR
                let aurora_key_record = StateRecord::AccessKey {
                    account_id: aurora_id.clone(),
                    public_key: public_key.clone(),
                    access_key: AccessKey::full_access(),
                };
                let aurora_account_record = StateRecord::Account {
                    account_id: aurora_id.clone(),
                    account: Account::new(aurora_amount, 0, CryptoHash::default(), 0),
                };
                records.0.push(aurora_key_record);
                records.0.push(aurora_account_record);
                genesis.config.total_supply += aurora_amount;
                genesis.to_file(&path);
                println!("Aurora account added to {path}");
                let key_path = Path::new(&path).parent().unwrap().join("aurora_key.json");
                let key_file = near_crypto::KeyFile {
                    account_id: aurora_id,
                    public_key,
                    secret_key,
                };
                key_file
                    .write_to_file(&key_path)
                    .expect("Failed to write Aurora access key file");
                println!("Aurora access key written to {key_path:?}");
            }
            InitCommand::LocalConfig {
                nearcore_config_path,
                aurora_access_key_path,
            } => {
                let nearcore_config: serde_json::Value = {
                    let data = std::fs::read_to_string(nearcore_config_path)
                        .expect("Failed to read nearcore config");
                    serde_json::from_str(&data).expect("Failed to parse nearcore config")
                };
                let rpc_addr = extract_rpc_addr(&nearcore_config)
                    .expect("Failed to parse rpc address from nearcore config");
                let rpc_addr = format!("http://{rpc_addr}");
                let mut config = config.clone();
                config.network = Network::Custom {
                    near_rpc: rpc_addr,
                    aurora_rpc: String::new(),
                };

                if let Some(path) = aurora_access_key_path {
                    config.near_key_path = Some(path);
                }

                config
                    .to_file(config_path)
                    .expect("Failed to write new CLI config file");
                println!("Updated CLI config at {config_path}");
            }
        },
    };
    Ok(())
}

fn extract_rpc_addr(nearcore_config: &serde_json::Value) -> Option<&str> {
    nearcore_config
        .as_object()?
        .get("rpc")?
        .as_object()?
        .get("addr")?
        .as_str()
}

fn parse_read_call_args(
    sender_addr_hex: Option<String>,
    target_addr_hex: &str,
    amount: Option<&str>,
) -> (Address, Address, Wei) {
    let target = utils::hex_to_address(target_addr_hex).unwrap();
    let sender = sender_addr_hex
        .map(|x| utils::hex_to_address(&x).unwrap())
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
    let sk_bytes = utils::hex_to_arr(source_private_key_hex).unwrap();
    let sk = libsecp256k1::SecretKey::parse(&sk_bytes).unwrap();
    let target = utils::hex_to_address(target_addr_hex).unwrap();
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

fn parse_ft_metadata(input: Option<String>) -> FungibleTokenMetadata {
    let input = match input {
        Some(x) => x,
        None => return default_ft_metadata(),
    };

    let json: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&input).unwrap();
    FungibleTokenMetadata {
        spec: json.get("spec").expect("Missing spec field").to_string(),
        name: json.get("name").expect("Missing name field").to_string(),
        symbol: json
            .get("symbol")
            .expect("Missing symbol field")
            .to_string(),
        icon: json
            .get("icon")
            .map(aurora_engine_types::ToString::to_string),
        reference: json
            .get("reference")
            .map(aurora_engine_types::ToString::to_string),
        reference_hash: json.get("reference_hash").map(|x| {
            let bytes = base64::decode(x.as_str().expect("reference_hash must be a string"))
                .expect("reference_hash must be a base64-encoded string");
            FungibleReferenceHash::try_from_slice(&bytes)
                .expect("reference_hash must be base64-encoded 32-byte array")
        }),
        decimals: serde_json::from_value(
            json.get("decimals")
                .expect("Missing decimals field")
                .clone(),
        )
        .expect("decimals field must be a u8 number"),
    }
}

fn default_ft_metadata() -> FungibleTokenMetadata {
    FungibleTokenMetadata {
        spec: "ft-1.0.0".to_string(),
        name: "localETH".to_string(),
        symbol: "localETH".to_string(),
        icon: None,
        reference: None,
        reference_hash: None,
        decimals: 18,
    }
}

fn assert_tx_success(outcome: &FinalExecutionOutcomeView) {
    if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = &outcome.status {
    } else {
        panic!("Transaction failed: {outcome:?}");
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
