use crate::{
    config::{Config, Network},
    utils,
};
use aurora_engine_types::borsh::BorshDeserialize;
use aurora_engine_types::parameters::connector::{InitCallArgs, PauseEthConnectorCallArgs};
use aurora_engine_types::parameters::engine::{
    DeployErc20TokenArgs, GetStorageAtArgs, NewCallArgs, NewCallArgsV2, SubmitResult,
    TransactionStatus,
};
use aurora_engine_types::{
    H256, U256,
    account_id::AccountId,
    parameters::{CrossContractCallArgs, PromiseArgs, PromiseCreateArgs},
    types::{Address, NearGas, Wei, Yocto},
};
use clap::Subcommand;
use near_primitives::version::PROTOCOL_VERSION;
use near_primitives::{
    account::{AccessKey, Account},
    hash::CryptoHash,
    state_record::StateRecord,
    views::FinalExecutionOutcomeView,
};
use std::{path::Path, str::FromStr};

/// Chain ID for Aurora localnet, per the documentation on
/// <https://doc.aurora.dev/getting-started/network-endpoints>
#[allow(clippy::unreadable_literal)]
const AURORA_LOCAL_NET_CHAIN_ID: u64 = 1313161556;

use crate::{client::NearClient, utils::secret_key_from_hex};

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
        erc20: super::erc20::Erc20,
    },
    Solidity {
        #[clap(short, long)]
        sender_addr_hex: Option<String>,
        #[clap(short, long)]
        target_addr_hex: String,
        #[clap(short, long)]
        amount: Option<String>,
        #[clap(subcommand)]
        contract_call: super::solidity::Solidity,
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
        /// Unique identifier for the chain. The default value is 1313161556 (Aurora localnet).
        /// See [chainlist](https://chainlist.org/) for a list of taken chain IDs.
        #[clap(short, long)]
        chain_id: Option<u64>,
        /// Near account ID for the owner of the Engine contract.
        /// The owner has special admin privileges such as upgrading the contract code.
        /// The default value is the Engine Account ID itself.
        #[clap(short, long)]
        owner_id: Option<String>,
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
        /// (see [FT metadata](https://nomicon.io/Standards/Tokens/FungibleToken/Metadata)
        /// for fields). The default value is 18 decimals with name and symbol equal to "localETH".
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
        contract_call: super::solidity::Solidity,
    },
    EngineErc20 {
        #[clap(short, long)]
        target_addr_hex: String,
        #[clap(short, long)]
        amount: Option<String>,
        #[clap(subcommand)]
        erc20: super::erc20::Erc20,
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
    client: &NearClient,
    config: &Config,
    config_path: &str,
) -> anyhow::Result<()> {
    match command {
        Command::Read { subcommand } => match subcommand {
            ReadCommand::GetReceiptResult { receipt_id_b58 } => {
                let tx_hash = bs58::decode(receipt_id_b58.as_str()).into_vec()?;
                let outcome = client
                    .get_receipt_outcome(
                        tx_hash
                            .as_slice()
                            .try_into()
                            .map_err(|e| anyhow::anyhow!("{e}"))?,
                    )
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
                    parse_read_call_args(sender_addr_hex, &target_addr_hex, amount.as_deref())?;
                let input = utils::hex_to_vec(&input_data_hex)?;
                let result = client
                    .view_contract_call(sender, target, amount, input)
                    .await?;
                println!("{result:?}");
            }
            ReadCommand::EngineErc20 {
                erc20,
                target_addr_hex,
                amount,
                sender_addr_hex,
            } => {
                let (sender, target, amount) =
                    parse_read_call_args(sender_addr_hex, &target_addr_hex, amount.as_deref())?;
                let input = erc20.abi_encode()?;
                let result = client
                    .view_contract_call(sender, target, amount, input)
                    .await?;
                println!("{result:?}");
            }
            ReadCommand::Solidity {
                contract_call,
                target_addr_hex,
                amount,
                sender_addr_hex,
            } => {
                let (sender, target, amount) =
                    parse_read_call_args(sender_addr_hex, &target_addr_hex, amount.as_deref())?;
                let input = contract_call.abi_encode()?;
                let result = client
                    .view_contract_call(sender, target, amount, input)
                    .await?;
                if let TransactionStatus::Succeed(bytes) = result {
                    let parsed_output = contract_call.abi_decode(&bytes)?;
                    println!("{parsed_output:?}");
                } else {
                    println!("{result:?}");
                }
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
                        borsh::to_vec(&precompile_args).unwrap(),
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
                }
            }
            ReadCommand::GetAuroraErc20 { nep_141_account } => {
                let address = client
                    .get_erc20_from_nep141(&nep_141_account)
                    .await?
                    .encode();
                println!("{address}");
            }
            ReadCommand::GetEngineBridgeProver => {
                let bridge_prover = client.get_bridge_prover().await?;
                println!("{bridge_prover}");
            }
            ReadCommand::GetChainId => {
                let chain_id = {
                    let result = client.view_call("get_chain_id", vec![]).await?;
                    U256::from_big_endian(&result.result).low_u64()
                };
                println!("{chain_id}");
            }
            ReadCommand::GetUpgradeIndex => {
                let upgrade_index = {
                    let result = client.view_call("get_upgrade_index", vec![]).await?;
                    U256::from_big_endian(&result.result).low_u64()
                };
                println!("{upgrade_index}");
            }
            ReadCommand::GetBlockHash { block_number } => {
                let height_serialized: u128 = block_number.parse::<u128>().unwrap();
                let block_hash = client
                    .view_call("get_block_hash", height_serialized.to_le_bytes().to_vec())
                    .await?
                    .result;
                let block_hex = hex::encode(block_hash);
                println!("{block_hex}");
            }
            ReadCommand::GetCode { address_hex } => {
                let address = utils::hex_to_address(&address_hex)?.as_bytes().to_vec();
                let code = client.view_call("get_code", address).await?.result;
                let code_hex = hex::encode(code);
                println!("{code_hex}");
            }
            ReadCommand::GetBalance { address_hex } => {
                let address = utils::hex_to_address(&address_hex)?.as_bytes().to_vec();
                let balance = {
                    let result = client.view_call("get_balance", address).await?;
                    U256::from_big_endian(&result.result).low_u64()
                };
                println!("{balance}");
            }
            ReadCommand::GetNonce { address_hex } => {
                let address = utils::hex_to_address(&address_hex)?.as_bytes().to_vec();
                let nonce = {
                    let result = client.view_call("get_nonce", address).await?;
                    U256::from_big_endian(&result.result).low_u64()
                };
                println!("{nonce}");
            }
            ReadCommand::GetStorageAt {
                address_hex,
                key_hex,
            } => {
                let input = GetStorageAtArgs {
                    address: utils::hex_to_address(&address_hex)?,
                    key: utils::hex_to_arr(&key_hex)?,
                };
                let storage = {
                    let result = client
                        .view_call("get_storage_at", borsh::to_vec(&input)?)
                        .await?;
                    H256::from_slice(&result.result)
                };
                println!("{storage}");
            }
            ReadCommand::GetPausedFlags => {
                let paused_flags = client.view_call("get_paused_flags", vec![]).await?.result;
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
                let chain_id = chain_id.unwrap_or(AURORA_LOCAL_NET_CHAIN_ID);
                let owner_id = owner_id.as_deref().unwrap_or(&config.engine_account_id);
                let prover_account: AccountId = {
                    let prover_account = prover_account
                        .as_deref()
                        .unwrap_or(&config.engine_account_id);
                    prover_account
                        .parse()
                        .map_err(|_| anyhow::anyhow!("Prover account is an invalid Near account"))?
                };
                let eth_custodian_address = eth_custodian_address
                    .as_deref()
                    .map(utils::hex_to_address)
                    .transpose()?
                    .unwrap_or_default();
                let metadata = utils::ft_metadata::parse_ft_metadata(ft_metadata)?;

                let new_args = NewCallArgs::V2(NewCallArgsV2 {
                    chain_id: aurora_engine_types::types::u256_to_arr(&U256::from(chain_id)),
                    owner_id: owner_id
                        .parse()
                        .map_err(|_| anyhow::anyhow!("Owner account is an invalid Near account"))?,
                    upgrade_delay_blocks: upgrade_delay_blocks.unwrap_or_default(),
                });

                let init_args = InitCallArgs {
                    prover_account,
                    eth_custodian_address: eth_custodian_address.encode(),
                    metadata,
                };

                let deploy_response = client.deploy_contract(wasm_bytes).await?;
                assert_tx_success(&deploy_response);
                let next_nonce = deploy_response.transaction.nonce + 1;

                let new_response = client
                    .contract_call_with_nonce("new", borsh::to_vec(&new_args)?, next_nonce)
                    .await?;
                assert_tx_success(&new_response);
                let next_nonce = new_response.transaction.nonce + 1;

                let init_response = client
                    .contract_call_with_nonce(
                        "new_eth_connector",
                        borsh::to_vec(&init_args).unwrap(),
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
                let source_private_key_hex = config.get_evm_secret_key()?;
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
                let result = client
                    .send_aurora_transaction(
                        &sk,
                        Some(aurora_engine_precompiles::xcc::cross_contract_call::ADDRESS),
                        Wei::zero(),
                        borsh::to_vec(&precompile_args).unwrap(),
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
                    parse_write_call_args(config, &target_addr_hex, amount.as_deref())?;
                let input = utils::hex_to_vec(&input_data_hex)?;
                let result = client
                    .send_aurora_transaction(&sk, Some(target), amount, input)
                    .await?;
                println!("{result:?}");
            }
            WriteCommand::EngineErc20 {
                erc20,
                target_addr_hex,
                amount,
            } => {
                let (sk, target, amount) =
                    parse_write_call_args(config, &target_addr_hex, amount.as_deref())?;
                let input = erc20.abi_encode()?;
                let result = client
                    .send_aurora_transaction(&sk, Some(target), amount, input)
                    .await?;
                println!("{result:?}");
            }
            WriteCommand::Solidity {
                contract_call,
                target_addr_hex,
                amount,
            } => {
                let (sk, target, amount) =
                    parse_write_call_args(config, &target_addr_hex, amount.as_deref())?;
                let input = contract_call.abi_encode()?;
                let result = client
                    .send_aurora_transaction(&sk, Some(target), amount, input)
                    .await?;
                println!("{result:?}");
            }
            WriteCommand::FactoryUpdate { wasm_bytes_path } => {
                let args = std::fs::read(wasm_bytes_path).unwrap();
                let tx_outcome = client.contract_call("factory_update", args).await.unwrap();
                println!("{tx_outcome:?}");
            }
            WriteCommand::DeployCode { code_byte_hex } => {
                let input = utils::hex_to_vec(&code_byte_hex)?;
                let tx_outcome = client.contract_call("deploy_code", input).await?;
                if let near_primitives::views::FinalExecutionStatus::SuccessValue(bytes) =
                    tx_outcome.status
                {
                    let result = SubmitResult::try_from_slice(&bytes)
                        .expect("Failed to parse Engine outcome");
                    if let TransactionStatus::Succeed(bytes) = result.status {
                        println!("Contact deployed to address: 0x{}", hex::encode(bytes));
                    } else {
                        println!("Transaction reverted:\n{result:?}");
                    }
                } else {
                    println!("Transaction failed:\n{tx_outcome:?}");
                }
            }
            WriteCommand::RegisterRelayer {
                relayer_eth_address_hex,
            } => {
                let relayer = utils::hex_to_vec(&relayer_eth_address_hex)?;
                let tx_outcome = client.contract_call("register_relayer", relayer).await?;
                println!("{tx_outcome:?}");
            }
            WriteCommand::DeployERC20Token { nep141 } => {
                let nep141: AccountId = nep141.parse().unwrap();
                let input = borsh::to_vec(&DeployErc20TokenArgs { nep141 })?;
                let tx_outcome = client.contract_call("deploy_erc20_token", input).await?;
                println!("{tx_outcome:?}");
            }
            WriteCommand::Deposit { raw_proof } => {
                let tx_outcome = client
                    .contract_call("deposit", raw_proof.as_bytes().to_vec())
                    .await?;
                println!("{tx_outcome:?}");
            }
            WriteCommand::SetPausedFlags { paused_mask } => {
                let input = borsh::to_vec(&PauseEthConnectorCallArgs {
                    paused_mask: u8::from_str(&paused_mask).unwrap(),
                })?;
                let tx_outcome = client.contract_call("set_paused_flags", input).await?;
                println!("{tx_outcome:?}");
            }
        },
        Command::Init { subcommand } => match subcommand {
            InitCommand::Genesis { path } => {
                let mut genesis = near_chain_configs::Genesis::from_file(
                    &path,
                    near_chain_configs::GenesisValidationMode::UnsafeFast,
                )?;
                let records = genesis.force_read_records();
                let aurora_id: near_primitives::account::id::AccountId =
                    config.engine_account_id.parse()?;
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
                    account: Account::new(
                        aurora_amount,
                        0,
                        0,
                        CryptoHash::default(),
                        0,
                        PROTOCOL_VERSION,
                    ),
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
    }
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
) -> anyhow::Result<(Address, Address, Wei)> {
    let target = utils::hex_to_address(target_addr_hex)?;
    let sender = sender_addr_hex
        .and_then(|x| utils::hex_to_address(&x).ok())
        .unwrap_or_default();
    let amount = amount
        .and_then(|a| U256::from_dec_str(a).ok())
        .map_or_else(Wei::zero, Wei::new);

    Ok((sender, target, amount))
}

fn parse_write_call_args(
    config: &Config,
    target_addr_hex: &str,
    amount: Option<&str>,
) -> anyhow::Result<(libsecp256k1::SecretKey, Address, Wei)> {
    let source_private_key_hex = config.get_evm_secret_key()?;
    let secret_key = secret_key_from_hex(source_private_key_hex)?;
    let target = utils::hex_to_address(target_addr_hex)?;
    let amount = amount
        .and_then(|a| U256::from_dec_str(a).ok())
        .map_or_else(Wei::zero, Wei::new);
    Ok((secret_key, target, amount))
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

fn assert_tx_success(outcome: &FinalExecutionOutcomeView) {
    assert!(
        matches!(
            &outcome.status,
            near_primitives::views::FinalExecutionStatus::SuccessValue(_)
        ),
        "Transaction failed: {outcome:?}"
    );
}
