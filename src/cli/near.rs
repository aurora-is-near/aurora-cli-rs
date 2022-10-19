use std::str::FromStr;

use crate::{
    cli::borsh_io::{
        AccountBalance, AccountBalanceSerde,
        FungibleTokenMetadata, GetStorageAtInput, BeginChainArgs, BeginBlockArgs, WithdrawCallArgs, PauseEthConnectorCallArgs
    },
    client::AuroraClient,
    config::Config,
    utils,
};
use aurora_engine::parameters::DeployErc20TokenArgs;
use aurora_engine_types::{
    parameters::{CrossContractCallArgs, PromiseArgs, PromiseCreateArgs},
    types::{Address, NearGas, RawU256, Wei, Yocto, NEP141Wei},
    U256, account_id::AccountId,
};
use borsh::{BorshDeserialize, BorshSerialize};
use clap::Subcommand;
use serde_json::json;

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
    GetBlockHash,
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
    // get_accounts_counter
    GetAccountsCounter,
    // ft_total_supply
    FtTotalSupply,
    // ft_total_eth_supply_on_near
    FtTotalSupplyOnNear,
    // ft_total_eth_supply_on_aurora
    FtTotalEthSupplyOnAurora,
    // ft_balance_of
    FtBalanceOf {
        account_id: String,
    },
    // ft_balance_of_eth
    FtBalanceOfEth {
        account_id: String,
    },
    // storage_balance_of
    StorageBalanceOf {
        account_id: String,
    },
    // ft_metadata
    FtMetadata,
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
    // call
    Call {
        call_byte_hex: String,
    },
    // register_relayer
    RegisterRelayer {
        relayer_eth_address_hex: String,
    },
    // ft_on_transfer
    FTOnTransfer {
        sender_near_id: String,
        amount: String,
        msg: String,
    },
    // deploy_erc20_token
    DeployERC20Token {
        nep141: String,
    },
    // begin_chain
    BeginChain {
        chain_id: String,
        genesis_alloc: String,
    },
    // begin_block
    BeginBlock {
        /// The current block's hash (for replayer use).
        hash: String,
        /// The current block's beneficiary address.
        coinbase: String,
        /// The current block's timestamp (in seconds since the Unix epoch).
        timestamp: String,
        /// The current block's number (the genesis block is number zero).
        number: String,
        /// The current block's difficulty.
        difficulty: String,
        /// The current block's gas limit.
        gaslimit: String,
    },
    // withdraw
    Withdraw {
        recipient_address: String,
        amount: String,
    },
    /* 
    // deposit
    Deposit {
        raw_proof: String,
    },
    */
    // ft_transfer
    FTTransfer {
        receiver_id: String,
        amount: String,
        memo: String,
    },
    // ft_transfer_call
    FTTransferCall {
        receiver_id: String,
        amount: String,
        memo: String,
        msg: String,
    },
    // storage_deposit
    StorageDeposit {
        account_id: String,
        registration_only: Option<bool>,
    },
    // storage_unregister
    StorageUnregister {
        force: Option<bool>,
    },
    // storage_withdraw
    StorageWithdraw {
        amount: String,
    },
    // set_paused_flags
    SetPausedFlags {
        paused_mask: String,
    },
}

pub async fn execute_command<T: AsRef<str>>(
    command: Command,
    client: &AuroraClient<T>,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Command::Read { subcommand } => match subcommand {
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
            ReadCommand::GetChainId => {
                let chain_id = {
                    let result = client.near_view_call("get_chain_id".into(), vec![]).await?;
                    U256::from_big_endian(&result.result).low_u64()
                };
                println!("{:?}", chain_id);
            }
            ReadCommand::GetUpgradeIndex => {
                let upgrade_index = {
                    let result = client
                        .near_view_call("get_upgrade_index".into(), vec![])
                        .await?;
                    U256::from_big_endian(&result.result).low_u64()
                };
                println!("{:?}", upgrade_index);
            }
            ReadCommand::GetBlockHash => {
                let block_hash = {
                    let result = client
                        .near_view_call("get_block_hash".into(), vec![])
                        .await?;
                    U256::from_big_endian(&result.result).low_u64()
                };
                println!("{:?}", block_hash);
            }
            ReadCommand::GetCode { address_hex } => {
                let code = client
                    .near_view_call("get_code".into(), address_hex.as_bytes().to_vec())
                    .await?
                    .result;
                println!("{:?}", code);
            }
            ReadCommand::GetBalance { address_hex } => {
                let balance = {
                    let result = client
                        .near_view_call("get_balance".into(), address_hex.as_bytes().to_vec())
                        .await?;
                    U256::from_big_endian(&result.result).low_u64()
                };
                println!("{:?}", balance);
            }
            ReadCommand::GetNonce { address_hex } => {
                let nonce = {
                    let result = client
                        .near_view_call("get_nonce".into(), address_hex.as_bytes().to_vec())
                        .await?;
                    U256::from_big_endian(&result.result).low_u64()
                };
                println!("{:?}", nonce);
            }
            ReadCommand::GetStorageAt {
                address_hex,
                key_hex,
            } => {
                let mut buffer: Vec<u8> = Vec::new();
                let input = GetStorageAtInput {
                    address: Address::decode(&address_hex).unwrap(),
                    key: hex::decode(key_hex).unwrap(),
                };
                input.serialize(&mut buffer)?;
                let storage = {
                    let result = client
                        .near_view_call("get_storage_at".into(), buffer)
                        .await?;
                    U256::from_big_endian(&result.result).low_u64()
                };
                println!("{:?}", storage);
            }
            ReadCommand::GetPausedFlags => {
                let paused_flags = client
                    .near_view_call("get_paused_flags".into(), vec![])
                    .await?
                    .result;
                println!("{:?}", paused_flags);
            }
            ReadCommand::GetAccountsCounter => {
                let paused_flags = client
                    .near_view_call("get_accounts_counter".into(), vec![])
                    .await?
                    .result;
                println!("{:?}", paused_flags);
            }
            ReadCommand::FtTotalSupply => {
                let ft_total_supply = {
                    let result = client
                        .near_view_call("ft_total_supply".into(), vec![])
                        .await?;
                    U256::from_big_endian(&result.result).low_u64()
                };
                println!("{:?}", ft_total_supply);
            }
            ReadCommand::FtTotalSupplyOnNear => {
                let ft_total_supply_on_near = {
                    let result = client
                        .near_view_call("ft_total_supply_on_near".into(), vec![])
                        .await?;
                    U256::from_big_endian(&result.result).low_u64()
                };
                println!("{:?}", ft_total_supply_on_near);
            }
            ReadCommand::FtTotalEthSupplyOnAurora => {
                let ft_total_eth_supply_on_aurora = {
                    let result = client
                        .near_view_call("ft_total_eth_supply_on_aurora".into(), vec![])
                        .await?;
                    U256::from_big_endian(&result.result).low_u64()
                };
                println!("{:?}", ft_total_eth_supply_on_aurora);
            }
            ReadCommand::FtBalanceOf { account_id } => {
                let obj = json!({ "account_id": account_id });
                let ft_balance_of = {
                    let result = client
                        .near_view_call("ft_balance_of".into(), obj.to_string().as_bytes().to_vec())
                        .await?;
                    U256::from_big_endian(&result.result).low_u64()
                };
                println!("{:?}", ft_balance_of);
            }
            ReadCommand::FtBalanceOfEth { account_id } => {
                let obj = json!({ "account_id": account_id });
                let ft_balance_of_eth = {
                    let result = client
                        .near_view_call(
                            "ft_balance_of_eth".into(),
                            obj.to_string().as_bytes().to_vec(),
                        )
                        .await?;
                    U256::from_big_endian(&result.result).low_u64()
                };
                println!("{:?}", ft_balance_of_eth);
            }
            ReadCommand::StorageBalanceOf { account_id } => {
                let obj = json!({ "account_id": account_id });
                let storage_balance_of = {
                    let result = client
                        .near_view_call(
                            "storage_balance_of".into(),
                            obj.to_string().as_bytes().to_vec(),
                        )
                        .await?;
                    U256::from_big_endian(&result.result).low_u64()
                };
                println!("{:?}", storage_balance_of);
            }
            ReadCommand::FtMetadata => {
                let ft_metadata = {
                    let result = client.near_view_call("ft_metadata".into(), vec![]).await?;
                    result.result
                };
                let ft_metadata_json: FungibleTokenMetadata =
                    FungibleTokenMetadata::try_from_slice(&ft_metadata).unwrap();
                println!("{:?}", ft_metadata_json);
            } 
            // is_used_proof
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
            WriteCommand::DeployCode { code_byte_hex } => {
                let input = hex::decode(code_byte_hex)?;
                let tx_outcome = client
                    .near_contract_call("deploy_code".into(), input)
                    .await?;
                println!("{:?}", tx_outcome);
            }
            WriteCommand::Call { call_byte_hex } => {
                let input = hex::decode(call_byte_hex)?;
                let tx_outcome = client.near_contract_call("call".into(), input).await?;
                println!("{:?}", tx_outcome);
            }
            WriteCommand::RegisterRelayer {
                relayer_eth_address_hex,
            } => {
                let relayer = hex::decode(relayer_eth_address_hex)?;
                let tx_outcome = client
                    .near_contract_call("register_relayer".into(), relayer)
                    .await?;
                println!("{:?}", tx_outcome);
            }
            WriteCommand::FTOnTransfer {
                sender_near_id,
                amount,
                msg,
            } => {
                let obj = json!({ "sender_id": sender_near_id, "amount": amount, "msg": msg });
                let tx_outcome = client
                    .near_contract_call(
                        "ft_on_transfer".into(),
                        obj.to_string().as_bytes().to_vec(),
                    )
                    .await?;
                println!("{:?}", tx_outcome);
            }
            WriteCommand::DeployERC20Token { nep141 } => {
                let mut buffer: Vec<u8> = Vec::new();
                let nep141: AccountId = nep141.parse().unwrap();
                let input = DeployErc20TokenArgs {
                    nep141
                };
                input.serialize(&mut buffer)?;
                let tx_outcome = client
                    .near_contract_call("deploy_erc20_token".into(), buffer)
                    .await?;
                println!("{:?}", tx_outcome);
            }
            WriteCommand::BeginChain {
                chain_id,
                genesis_alloc,
            } => {
                let genesis_accts: Vec<AccountBalanceSerde> = serde_json::from_str(&genesis_alloc)?;
                let mut genesis_accts_borsh: Vec<AccountBalance> = Vec::new();
                for i in genesis_accts {
                    let acct = AccountBalance {
                        address: i.address,
                        balance: i.balance,
                    };
                    genesis_accts_borsh.push(acct);
                }
                let mut buffer: Vec<u8> = Vec::new();
                let chain_id: RawU256 = U256::from(chain_id.parse::<u64>().unwrap()).into();
                let input = BeginChainArgs {
                    chain_id,
                    genesis_alloc: genesis_accts_borsh,
                };
                input.serialize(&mut buffer)?;
                let tx_outcome = client
                    .near_contract_call("begin_chain".into(), buffer)
                    .await?;
                println!("{:?}", tx_outcome);
            } 
            WriteCommand::BeginBlock { hash, coinbase, timestamp, number, difficulty, gaslimit } => {
                let mut buffer: Vec<u8> = Vec::new();
                let hash: RawU256 = U256::from(hash.parse::<u64>().unwrap()).into();
                let coinbase = Address::decode(&coinbase).unwrap();
                let timestamp: RawU256 = U256::from(timestamp.parse::<u64>().unwrap()).into();
                let number: RawU256 = U256::from(number.parse::<u64>().unwrap()).into();
                let difficulty: RawU256 = U256::from(difficulty.parse::<u64>().unwrap()).into();
                let gaslimit: RawU256 = U256::from(gaslimit.parse::<u64>().unwrap()).into();
                let input = BeginBlockArgs {
                    hash,
                    coinbase,
                    timestamp,
                    number,
                    difficulty,
                    gaslimit,
                };
                input.serialize(&mut buffer)?;
                let tx_outcome = client
                    .near_contract_call("begin_block".into(), buffer)
                    .await?;
                println!("{:?}", tx_outcome);
            }
            WriteCommand::Withdraw {
                recipient_address,
                amount,
            } => {
                let mut buffer: Vec<u8> = Vec::new();
                let addr = Address::decode(&recipient_address).unwrap();
                let input = WithdrawCallArgs {
                    recipient_address: addr,
                    amount: NEP141Wei::new(u128::from_str(&amount).unwrap()),
                };
                input.serialize(&mut buffer)?;
                let tx_outcome = client
                    .near_contract_call("withdraw".into(), buffer)
                    .await?;
                println!("{:?}", tx_outcome);
            }
            // This only should be done by a bridge
            /* 
            WriteCommand::Deposit { raw_proof } => {
                let tx_outcome = client
                    .near_contract_call("deposit".into(), raw_proof.as_bytes().to_vec())
                    .await?;
                println!("{:?}", tx_outcome);
            },
            */
            WriteCommand::FTTransfer {
                receiver_id,
                amount,
                memo,
            } => {
                let obj = json!({ "receiver_id": receiver_id, "amount": amount, "memo": memo });
                let tx_outcome = client
                    .near_contract_call("ft_transfer".into(), obj.to_string().as_bytes().to_vec())
                    .await?;
                println!("{:?}", tx_outcome);
            }
            WriteCommand::FTTransferCall {
                receiver_id,
                amount,
                memo,
                msg,
            } => {
                let obj = json!({ "receiver_id": receiver_id, "amount": amount, "memo": memo, "msg": msg });
                let tx_outcome = client
                    .near_contract_call(
                        "ft_transfer_call".into(),
                        obj.to_string().as_bytes().to_vec(),
                    )
                    .await?;
                println!("{:?}", tx_outcome);
            },
            WriteCommand::StorageDeposit {
                account_id,
                registration_only,
            } => {
                let obj = json!({ "account_id": account_id, "registration_only": registration_only });
                let tx_outcome = client
                    .near_contract_call(
                        "storage_deposit".into(),
                        obj.to_string().as_bytes().to_vec(),
                    )
                    .await?;
                println!("{:?}", tx_outcome);
            },
            WriteCommand::StorageUnregister { force } => {
                let obj = json!({ "force": force });
                let tx_outcome = client
                    .near_contract_call(
                        "storage_unregister".into(),
                        obj.to_string().as_bytes().to_vec(),
                    )
                    .await?;
                println!("{:?}", tx_outcome);
            },
            WriteCommand::StorageWithdraw { amount } => {
                let obj = json!({ "amount": amount });
                let tx_outcome = client
                    .near_contract_call(
                        "storage_withdraw".into(),
                        obj.to_string().as_bytes().to_vec(),
                    )
                    .await?;
                println!("{:?}", tx_outcome);
            },
            WriteCommand::SetPausedFlags { paused_mask } => {
                let mut buffer: Vec<u8> = Vec::new();
                let input = PauseEthConnectorCallArgs {
                    paused_mask: u8::from_str(&paused_mask).unwrap(),
                };
                input.serialize(&mut buffer)?;
                let tx_outcome = client
                    .near_contract_call("set_paused_flags".into(), buffer)
                    .await?;
                println!("{:?}", tx_outcome);
            },
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
