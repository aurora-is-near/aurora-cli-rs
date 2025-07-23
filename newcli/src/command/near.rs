use std::path::Path;

use anyhow::Ok;
use aurora_sdk_rs::{
    aurora::{
        H256, U256,
        abi::{self},
        common::{self, IntoAurora, hex_to_arr, str_to_identifier},
        contract::{
            read::{
                FactoryGetWnearAddress, FtBalanceOf, FtTotalSupply, GetBalance, GetBlockHash,
                GetChainId, GetCode, GetErc20FromNep141, GetErc20Metadata,
                GetEthConnectorContractAccount, GetFixedGas, GetLatestHashchain,
                GetNep141FromErc20, GetNonce, GetOwner, GetPausedFlags, GetSiloParams,
                GetStorageAt, GetUpgradeDelayBlocks, GetUpgradeIndex, GetVersion,
                GetWhitelistStatus, PausedPrecompiles, ViewCall,
            },
            write::{
                AddEntryToWhitelist, DeployERC20, DeployUpgrade, FactoryUpdate,
                FactoryUpdateAddressVersion, FtOnTransfer, FtTransfer, FtTransferCall,
                MirrorErc20Token, MirrorErc20TokenCallback, PauseContract, PausePrecompiles,
                RegisterRelayer, RemoveEntryFromWhitelist, RemoveRelayerKey, ResumeContract,
                ResumePrecompiles, SetERC20Metadata, SetEthConnectorContractAccount,
                SetEthConnectorContractData, SetFixedGas, SetKeyManager, SetOwner, SetPausedFlags,
                SetSiloParams, SetUpgradeDelayBlocks, SetWhitelistStatus, StageUpgrade,
                StartHashchain, StorageDeposit, StorageUnregister, StorageWithdraw, Submit,
                Upgrade, WithdrawWnearToRouter,
            },
        },
        ethabi::{self, Contract},
        near_account_to_evm_address,
        parameters::{
            connector::{
                Erc20Metadata, FungibleTokenMetadata, InitCallArgs, MirrorErc20TokenArgs,
                NEP141FtOnTransferArgs, PauseEthConnectorCallArgs, PausedMask,
                SetErc20MetadataArgs, SetEthConnectorContractAccountArgs, StorageDepositCallArgs,
                StorageWithdrawCallArgs, TransferCallArgs, TransferCallCallArgs,
                WithdrawSerializeType,
            },
            engine::{
                CallArgs, DeployErc20TokenArgs, FunctionCallArgsV2, GetStorageAtArgs,
                LegacyNewCallArgs, PausePrecompilesCallArgs, RelayerKeyArgs, RelayerKeyManagerArgs,
                SetOwnerArgs, SetUpgradeDelayBlocksArgs, StartHashchainArgs, StorageBalance,
                StorageUnregisterArgs, SubmitResult, TransactionStatus, ViewCallArgs,
            },
            silo::{
                FixedGasArgs, SiloParamsArgs, WhitelistAccountArgs, WhitelistAddressArgs,
                WhitelistArgs, WhitelistKind, WhitelistKindArgs, WhitelistStatusArgs,
            },
            xcc::{AddressVersionUpdateArgs, CodeVersion, FundXccArgs, WithdrawWnearToRouterArgs},
        },
        transactions::{EthTransactionKind, legacy::TransactionLegacy},
        types::{Address, Balance, EthGas, NEP141Wei, Wei, Yocto},
    },
    near::{
        crypto::{KeyType, PublicKey, SecretKey},
        operations::Function,
        primitives::{
            account::{AccessKey, AccessKeyPermission, FunctionCallPermission},
            types::AccountId,
            views::{AccountView, FinalExecutionOutcomeView},
        },
        token::NearToken,
    },
};
use libsecp256k1::SecretKey as SecretKeyEth;
use serde_json::json;

use crate::{common::parse_ft_metadata, context::Context};

pub async fn create_account(
    context: &Context,
    account: AccountId,
    balance: NearToken,
) -> anyhow::Result<SecretKey> {
    let signer = context.cli.signer()?;
    let is_sub_account = account.is_sub_account_of(&signer.get_account_id());
    let new_key_pair = SecretKey::from_random(KeyType::ED25519);

    let request = if is_sub_account {
        context
            .client
            .near()
            .batch(&account)
            .create_account()
            .add_key(new_key_pair.public_key(), AccessKey::full_access())
            .transfer(balance)
    } else {
        let root_contract_id = context.cli.root_contract_id()?;
        context.client.near().batch(&root_contract_id).call(
            Function::new("create_account")
                .args_json(json!({
                    "new_account_id": account,
                    "new_public_key": new_key_pair.public_key(),
                }))?
                .deposit(balance),
        )
    };

    request.transact().await?;
    Ok(new_key_pair)
}

pub async fn view_account(context: &Context, account: &AccountId) -> anyhow::Result<AccountView> {
    context
        .client
        .near()
        .view_account(account)
        .await
        .map_err(Into::into)
}

pub async fn deploy_aurora(
    context: &Context,
    wasm: Vec<u8>,
) -> anyhow::Result<FinalExecutionOutcomeView> {
    context
        .client
        .near()
        .batch(&context.cli.engine)
        .deploy(&wasm)
        .transact()
        .await
        .map_err(Into::into)
}

pub async fn init(
    context: &Context,
    chain_id: u64,
    owner_id: AccountId,
    bridge_prover: AccountId,
    upgrade_delay_blocks: Option<u64>,
    custodian_address: Option<Address>,
    ft_metadata: FungibleTokenMetadata,
) -> anyhow::Result<()> {
    let aurora_init_args = LegacyNewCallArgs {
        chain_id: H256::from_low_u64_be(chain_id).into(),
        owner_id: owner_id.into_aurora(),
        bridge_prover_id: bridge_prover.clone().into_aurora(),
        upgrade_delay_blocks: upgrade_delay_blocks.unwrap_or_default(),
    };

    let eth_conn_init_args = InitCallArgs {
        prover_account: bridge_prover.into_aurora(),
        eth_custodian_address: custodian_address.map_or_else(
            || Address::default().encode(),
            |address| address.encode().trim_start_matches("0x").to_string(),
        ),
        metadata: ft_metadata,
    };

    context
        .client
        .near()
        .batch(&context.cli.engine)
        .call(Function::new("new").args_borsh(aurora_init_args)?)
        .call(Function::new("new_eth_connector").args_borsh(eth_conn_init_args)?)
        .transact()
        .await?;

    Ok(())
}

pub async fn get_storage_at(
    context: &Context,
    address: Address,
    key: H256,
) -> anyhow::Result<H256> {
    context
        .client
        .view(
            &context.cli.engine,
            GetStorageAt {
                args: Some(GetStorageAtArgs {
                    address,
                    key: key.into(),
                }),
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn register_relayer(context: &Context, address: Address) -> anyhow::Result<()> {
    context
        .client
        .call(&context.cli.engine, RegisterRelayer { address })
        .await
        .map_err(Into::into)
}

pub async fn start_hashchain(
    context: &Context,
    block_height: u64,
    block_hashchain: String,
) -> anyhow::Result<()> {
    context
        .client
        .call(
            &context.cli.engine,
            StartHashchain {
                args: StartHashchainArgs {
                    block_height,
                    block_hashchain: hex_to_arr(&block_hashchain)?,
                },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn pause_contract(context: &Context) -> anyhow::Result<()> {
    context
        .client
        .call(&context.cli.engine, PauseContract)
        .await
        .map_err(Into::into)
}

pub async fn resume_contract(context: &Context) -> anyhow::Result<()> {
    context
        .client
        .call(&context.cli.engine, ResumeContract)
        .await
        .map_err(Into::into)
}

pub async fn pause_precompiles(context: &Context, mask: u32) -> anyhow::Result<()> {
    context
        .client
        .call(
            &context.cli.engine,
            PausePrecompiles {
                args: PausePrecompilesCallArgs { paused_mask: mask },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn resume_precompiles(context: &Context, mask: u32) -> anyhow::Result<()> {
    context
        .client
        .call(
            &context.cli.engine,
            ResumePrecompiles {
                args: PausePrecompilesCallArgs { paused_mask: mask },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn paused_precompiles(context: &Context) -> anyhow::Result<u64> {
    context
        .client
        .view(&context.cli.engine, PausedPrecompiles)
        .await
        .map_err(Into::into)
}

pub async fn factory_update(context: &Context, wasm: Vec<u8>) -> anyhow::Result<()> {
    context
        .client
        .call(&context.cli.engine, FactoryUpdate { wasm })
        .await
        .map_err(Into::into)
}

pub async fn factory_update_address_version(
    context: &Context,
    address: Address,
    version: u32,
) -> anyhow::Result<()> {
    context
        .client
        .call(
            &context.cli.engine,
            FactoryUpdateAddressVersion {
                args: AddressVersionUpdateArgs {
                    address,
                    version: CodeVersion(version),
                },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn fund_xcc_sub_account(
    context: &Context,
    address: Address,
    wnear_account_id: Option<AccountId>,
    deposit: NearToken,
) -> anyhow::Result<()> {
    context
        .client
        .near()
        .call(&context.cli.engine, "fund_xcc_sub_account")
        .args_borsh(FundXccArgs {
            target: address,
            wnear_account_id: wnear_account_id.map(IntoAurora::into_aurora),
        })?
        .deposit(deposit)
        .transact()
        .await?;

    Ok(())
}

pub async fn upgrade(context: &Context, code: Vec<u8>) -> anyhow::Result<()> {
    context
        .client
        .call(&context.cli.engine, Upgrade { code })
        .await?;
    Ok(())
}

pub async fn stage_upgrade(context: &Context, code: Vec<u8>) -> anyhow::Result<()> {
    context
        .client
        .call(&context.cli.engine, StageUpgrade { code })
        .await
        .map_err(Into::into)
}

pub async fn factory_get_wnear_address(context: &Context) -> anyhow::Result<String> {
    context
        .client
        .view(&context.cli.engine, FactoryGetWnearAddress)
        .await
        .map_err(Into::into)
}

pub async fn deploy_upgrade(context: &Context) -> anyhow::Result<()> {
    context
        .client
        .call(&context.cli.engine, DeployUpgrade)
        .await
        .map_err(Into::into)
}

pub async fn factory_set_wnear_address(context: &Context, address: Address) -> anyhow::Result<()> {
    context
        .client
        .call(
            &context.cli.engine,
            aurora_sdk_rs::aurora::contract::write::FactorySetWnearAddress { address },
        )
        .await
        .map_err(Into::into)
}

pub async fn get_chain_id(context: &Context) -> anyhow::Result<u64> {
    context
        .client
        .view(&context.cli.engine, GetChainId)
        .await
        .map_err(Into::into)
}

pub async fn get_nonce(context: &Context, address: Address) -> anyhow::Result<u64> {
    context
        .view(GetNonce { address })
        .await
        .map(|n| n.as_u64())
        .map_err(Into::into)
}

pub async fn get_block_hash(context: &Context, height: u64) -> anyhow::Result<String> {
    context
        .client
        .view(&context.cli.engine, GetBlockHash { height })
        .await
        .map_err(Into::into)
}

pub async fn get_code(context: &Context, address: Address) -> anyhow::Result<Vec<u8>> {
    context
        .client
        .view(&context.cli.engine, GetCode { address })
        .await
        .map_err(Into::into)
}

pub async fn get_balance(context: &Context, address: Address) -> anyhow::Result<Wei> {
    context
        .client
        .view(&context.cli.engine, GetBalance { address })
        .await
        .map_err(Into::into)
}

pub async fn get_upgrade_index(context: &Context) -> anyhow::Result<u64> {
    context
        .client
        .view(&context.cli.engine, GetUpgradeIndex)
        .await
        .map_err(Into::into)
}

pub async fn get_version(context: &Context) -> anyhow::Result<String> {
    context
        .client
        .view(&context.cli.engine, GetVersion)
        .await
        .map_err(Into::into)
}

pub async fn get_owner(context: &Context) -> anyhow::Result<AccountId> {
    context
        .client
        .view(&context.cli.engine, GetOwner)
        .await
        .map_err(Into::into)
}

pub async fn set_owner(context: &Context, owner: AccountId) -> anyhow::Result<()> {
    context
        .client
        .call(
            &context.cli.engine,
            SetOwner {
                args: SetOwnerArgs {
                    new_owner: owner.into_aurora(),
                },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn get_bridge_prover(context: &Context) -> anyhow::Result<String> {
    context
        .client
        .view(
            &context.cli.engine,
            aurora_sdk_rs::aurora::contract::read::GetBridgeProver,
        )
        .await
        .map_err(Into::into)
}

pub async fn submit(
    context: &Context,
    to: Address,
    function: String,
    args: Option<String>,
    abi_path: String,
    amount: Wei,
    aurora_secret_key: String,
) -> anyhow::Result<SubmitResult> {
    let secret_key = hex_to_arr(aurora_secret_key.trim())
        .and_then(|bytes| SecretKeyEth::parse(&bytes).map_err(Into::into))
        .map_err(|e| anyhow::anyhow!("Couldn't create secret key from hex: {e}"))?;

    let contract = abi::read_contract(abi_path)?;
    let function = contract.function(&function)?;
    let args: serde_json::Value = args.map_or(Ok(serde_json::Value::Null), |args| {
        serde_json::from_str(&args).map_err(Into::into)
    })?;

    let tokens = abi::parse_args(&function.inputs, &args)?;
    let input = function.encode_input(&tokens)?;

    send_aurora_transaction(context, Some(to), amount, input, secret_key).await
}

pub fn encode_to_address(account_id: &AccountId) -> Address {
    near_account_to_evm_address(account_id.as_bytes())
}

pub fn gen_key_pair(random: bool, seed: Option<u64>) -> anyhow::Result<(Address, SecretKeyEth)> {
    common::gen_key_pair(random, seed)
}

pub fn gen_near_key_pair(key_type: KeyType) -> anyhow::Result<SecretKey> {
    Ok(SecretKey::from_random(key_type))
}

pub async fn get_fixed_gas(context: &Context) -> anyhow::Result<Option<EthGas>> {
    context
        .client
        .view(&context.cli.engine, GetFixedGas)
        .await
        .map_err(Into::into)
}

pub async fn set_fixed_gas(context: &Context, fixed_gas: Option<EthGas>) -> anyhow::Result<()> {
    context
        .client
        .call(
            &context.cli.engine,
            SetFixedGas {
                args: FixedGasArgs { fixed_gas },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn get_silo_params(context: &Context) -> anyhow::Result<Option<SiloParamsArgs>> {
    context
        .client
        .view(&context.cli.engine, GetSiloParams)
        .await
        .map_err(Into::into)
}

pub async fn set_silo_params(
    context: &Context,
    params: Option<SiloParamsArgs>,
) -> anyhow::Result<()> {
    context
        .client
        .call(&context.cli.engine, SetSiloParams { args: params })
        .await
        .map_err(Into::into)
}

pub async fn get_whitelist_status(
    context: &Context,
    kind: WhitelistKind,
) -> anyhow::Result<WhitelistStatusArgs> {
    context
        .client
        .view(
            &context.cli.engine,
            GetWhitelistStatus {
                args: WhitelistKindArgs { kind },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn set_whitelist_status(
    context: &Context,
    kind: WhitelistKind,
    status: bool,
) -> anyhow::Result<()> {
    context
        .client
        .call(
            &context.cli.engine,
            SetWhitelistStatus {
                args: WhitelistStatusArgs {
                    kind,
                    active: status,
                },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn add_entry_to_whitelist(
    context: &Context,
    kind: WhitelistKind,
    entry: String,
) -> anyhow::Result<()> {
    let args = args_from_kind(kind, entry)?;

    context
        .client
        .call(&context.cli.engine, AddEntryToWhitelist { args })
        .await
        .map_err(Into::into)
}

pub async fn remove_entry_from_whitelist(
    context: &Context,
    kind: WhitelistKind,
    entry: String,
) -> anyhow::Result<()> {
    let args = args_from_kind(kind, entry)?;

    context
        .client
        .call(&context.cli.engine, RemoveEntryFromWhitelist { args })
        .await
        .map_err(Into::into)
}

pub async fn set_key_manager(
    context: &Context,
    account_id: Option<AccountId>,
) -> anyhow::Result<()> {
    context
        .client
        .call(
            &context.cli.engine,
            SetKeyManager {
                args: RelayerKeyManagerArgs {
                    key_manager: account_id.map(IntoAurora::into_aurora),
                },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn add_relayer_key(
    context: &Context,
    public_key: PublicKey,
    allowance: NearToken,
) -> anyhow::Result<FinalExecutionOutcomeView> {
    context
        .client
        .near()
        .call(&context.cli.engine, "add_relayer_key")
        .args_json(RelayerKeyArgs {
            public_key: public_key.into_aurora(),
        })?
        .deposit(allowance)
        .transact()
        .await
        .map_err(Into::into)
}

pub async fn remove_relayer_key(context: &Context, public_key: PublicKey) -> anyhow::Result<()> {
    context
        .client
        .call(
            &context.cli.engine,
            RemoveRelayerKey {
                args: RelayerKeyArgs {
                    public_key: public_key.into_aurora(),
                },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn get_upgrade_delay_blocks(context: &Context) -> anyhow::Result<u64> {
    context
        .client
        .view(&context.cli.engine, GetUpgradeDelayBlocks)
        .await
        .map_err(Into::into)
}

pub async fn set_upgrade_delay_blocks(
    context: &Context,
    upgrade_delay_blocks: u64,
) -> anyhow::Result<()> {
    context
        .client
        .call(
            &context.cli.engine,
            SetUpgradeDelayBlocks {
                args: SetUpgradeDelayBlocksArgs {
                    upgrade_delay_blocks,
                },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn get_erc20_from_nep141(
    context: &Context,
    account_id: AccountId,
) -> anyhow::Result<String> {
    context
        .client
        .view(
            &context.cli.engine,
            GetErc20FromNep141 {
                nep141_account_id: account_id,
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn get_nep141_from_erc20(
    context: &Context,
    address: Address,
) -> anyhow::Result<AccountId> {
    context
        .client
        .view(&context.cli.engine, GetNep141FromErc20 { address })
        .await
        .map_err(Into::into)
}

pub async fn get_erc20_metadata(
    context: &Context,
    erc20_id: String,
) -> anyhow::Result<Erc20Metadata> {
    let id = str_to_identifier(&erc20_id)?;

    context
        .client
        .view(&context.cli.engine, GetErc20Metadata { id })
        .await
        .map_err(Into::into)
}

pub async fn set_erc20_metadata(
    context: &Context,
    erc20_id: String,
    metadata: Erc20Metadata,
) -> anyhow::Result<()> {
    let id = str_to_identifier(&erc20_id)?;

    context
        .client
        .call(
            &context.cli.engine,
            SetERC20Metadata {
                args: SetErc20MetadataArgs {
                    erc20_identifier: id,
                    metadata,
                },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn mirror_erc20_token(
    context: &Context,
    contract_id: AccountId,
    nep141: AccountId,
) -> anyhow::Result<Address> {
    context
        .client
        .call(
            &context.cli.engine,
            MirrorErc20Token {
                args: MirrorErc20TokenArgs {
                    contract_id: contract_id.into_aurora(),
                    nep141: nep141.into_aurora(),
                },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn set_eth_connector_contract_account(
    context: &Context,
    account_id: AccountId,
) -> anyhow::Result<()> {
    context
        .client
        .call(
            &context.cli.engine,
            SetEthConnectorContractAccount {
                args: SetEthConnectorContractAccountArgs {
                    account: account_id.into_aurora(),
                    withdraw_serialize_type: WithdrawSerializeType::Borsh,
                },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn get_eth_connector_contract_account(context: &Context) -> anyhow::Result<String> {
    context
        .client
        .view(&context.cli.engine, GetEthConnectorContractAccount)
        .await
        .map_err(Into::into)
}

pub async fn set_eth_connector_contract_data<P: AsRef<Path> + Send>(
    context: &Context,
    prover_id: AccountId,
    custodian_address: Address,
    ft_metadata_path: P,
) -> anyhow::Result<()> {
    let ft_metadata = parse_ft_metadata(std::fs::read_to_string(ft_metadata_path).ok())?;

    context
        .client
        .call(
            &context.cli.engine,
            SetEthConnectorContractData {
                args: InitCallArgs {
                    prover_account: prover_id.into_aurora(),
                    eth_custodian_address: custodian_address.encode(),
                    metadata: ft_metadata,
                },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn set_paused_flags(context: &Context, paused_mask: PausedMask) -> anyhow::Result<()> {
    context
        .client
        .call(
            &context.cli.engine,
            SetPausedFlags {
                args: PauseEthConnectorCallArgs { paused_mask },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn get_paused_flags(context: &Context) -> anyhow::Result<PausedMask> {
    context
        .client
        .view(&context.cli.engine, GetPausedFlags)
        .await
        .map_err(Into::into)
}

pub async fn add_relayer(
    context: &Context,
    deposit: NearToken,
    full_access_pub_key: PublicKey,
    func_call_pub_key: PublicKey,
) -> anyhow::Result<FinalExecutionOutcomeView> {
    let relay: AccountId = format!("relay.{}", context.cli.engine).parse()?;

    context
        .client
        .near()
        .batch(&relay)
        .create_account()
        .transfer(deposit)
        .add_key(full_access_pub_key, AccessKey::full_access())
        .add_key(
            func_call_pub_key,
            AccessKey {
                nonce: 0,
                permission: AccessKeyPermission::FunctionCall(FunctionCallPermission {
                    allowance: None,
                    receiver_id: context.cli.engine.clone().into(),
                    method_names: vec![
                        "submit".to_string(),
                        "submit_with_args".to_string(),
                        "call".to_string(),
                    ],
                }),
            },
        )
        .signer_id(&context.cli.engine)
        .transact()
        .await
        .map_err(Into::into)
}

pub async fn withdraw_wnear_to_router(
    context: &Context,
    address: Address,
    amount: NearToken,
) -> anyhow::Result<SubmitResult> {
    context
        .client
        .call(
            &context.cli.engine,
            WithdrawWnearToRouter {
                args: WithdrawWnearToRouterArgs {
                    target: address,
                    amount: Yocto::new(amount.as_yoctonear()),
                },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn mirror_erc20_token_callback(
    context: &Context,
    contract_id: AccountId,
    nep141: AccountId,
) -> anyhow::Result<()> {
    context
        .client
        .call(
            &context.cli.engine,
            MirrorErc20TokenCallback {
                args: MirrorErc20TokenArgs {
                    contract_id: contract_id.into_aurora(),
                    nep141: nep141.into_aurora(),
                },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn get_latest_hashchain(context: &Context) -> anyhow::Result<serde_json::Value> {
    context
        .client
        .view(&context.cli.engine, GetLatestHashchain)
        .await
        .map_err(Into::into)
}

pub async fn ft_total_supply(context: &Context) -> anyhow::Result<String> {
    context
        .client
        .view(&context.cli.engine, FtTotalSupply)
        .await
        .map_err(Into::into)
}

pub async fn ft_balance_of(context: &Context, account_id: AccountId) -> anyhow::Result<String> {
    context
        .client
        .view(&context.cli.engine, FtBalanceOf { account_id })
        .await
        .map_err(Into::into)
        .map(|wei| wei.to_string())
}

pub async fn ft_balance_of_eth(context: &Context, address: Address) -> anyhow::Result<Wei> {
    context
        .client
        .view(&context.cli.engine, GetBalance { address })
        .await
        .map_err(Into::into)
}

pub async fn ft_transfer(
    context: &Context,
    receiver_id: AccountId,
    amount: NEP141Wei,
    memo: Option<String>,
) -> anyhow::Result<()> {
    context
        .client
        .call(
            &context.cli.engine,
            FtTransfer {
                args: TransferCallArgs {
                    receiver_id: receiver_id.into_aurora(),
                    amount,
                    memo,
                },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn ft_transfer_call(
    context: &Context,
    receiver_id: AccountId,
    amount: NEP141Wei,
    memo: Option<String>,
    msg: String,
) -> anyhow::Result<()> {
    context
        .client
        .call(
            &context.cli.engine,
            FtTransferCall {
                args: TransferCallCallArgs {
                    receiver_id: receiver_id.into_aurora(),
                    amount,
                    memo,
                    msg,
                },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn ft_on_transfer(
    context: &Context,
    sender_id: AccountId,
    amount: Balance,
    msg: String,
) -> anyhow::Result<()> {
    context
        .client
        .call(
            &context.cli.engine,
            FtOnTransfer {
                args: NEP141FtOnTransferArgs {
                    sender_id: sender_id.into_aurora(),
                    amount,
                    msg,
                },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn deploy_erc20_token(context: &Context, nep141: AccountId) -> anyhow::Result<Address> {
    context
        .client
        .call(
            &context.cli.engine,
            DeployERC20 {
                args: DeployErc20TokenArgs {
                    nep141: nep141.into_aurora(),
                },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn storage_deposit(
    context: &Context,
    account_id: Option<AccountId>,
    registration_only: Option<bool>,
) -> anyhow::Result<()> {
    context
        .client
        .call(
            &context.cli.engine,
            StorageDeposit {
                args: StorageDepositCallArgs {
                    account_id: account_id.map(IntoAurora::into_aurora),
                    registration_only,
                },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn storage_unregister(context: &Context, force: bool) -> anyhow::Result<()> {
    context
        .client
        .call(
            &context.cli.engine,
            StorageUnregister {
                args: StorageUnregisterArgs { force },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn storage_withdraw(context: &Context, amount: Option<Yocto>) -> anyhow::Result<()> {
    context
        .client
        .call(
            &context.cli.engine,
            StorageWithdraw {
                args: StorageWithdrawCallArgs { amount },
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn storage_balance_of(
    context: &Context,
    account_id: AccountId,
) -> anyhow::Result<StorageBalance> {
    context
        .client
        .view(
            &context.cli.engine,
            aurora_sdk_rs::aurora::contract::read::StorageBalanceOf { account_id },
        )
        .await
        .map_err(Into::into)
}

pub async fn deploy(
    context: &Context,
    input: Vec<u8>,
    aurora_secret_key: String,
) -> anyhow::Result<SubmitResult> {
    let secret_key = common::hex_to_arr(aurora_secret_key.trim())
        .and_then(|bytes| SecretKeyEth::parse(&bytes).map_err(Into::into))
        .map_err(|e| anyhow::anyhow!("Couldn't create secret key from hex: {e}"))?;

    send_aurora_transaction(context, None, Wei::zero(), input, secret_key).await
}

pub async fn call(
    context: &Context,
    address: Address,
    input: Option<String>,
    value: Option<u128>,
    from: Option<AccountId>,
) -> anyhow::Result<FinalExecutionOutcomeView> {
    let input = input.map_or(Ok(vec![]), |input| {
        Ok(hex::decode(input.trim_start_matches("0x"))?)
    })?;

    let call_tx = context
        .client
        .near()
        .call(&context.cli.engine, "call")
        .args_borsh(CallArgs::V2(FunctionCallArgsV2 {
            contract: address,
            value: Wei::new_u128(value.unwrap_or_default()).to_bytes(),
            input,
        }))?;

    let call_tx = if let Some(from) = from {
        call_tx.signer_id(&from)
    } else {
        call_tx
    };

    call_tx.transact().await.map_err(Into::into)
}

pub async fn view_call(
    context: &Context,
    address: Address,
    function: String,
    args: Option<String>,
    from: Address,
    contract: Contract,
) -> anyhow::Result<Vec<ethabi::Token>> {
    let function = contract.function(&function)?;
    let args: serde_json::Value = args.map_or(Ok(serde_json::Value::Null), |args| {
        serde_json::from_str(&args).map_err(Into::into)
    })?;
    let tokens = abi::parse_args(&function.inputs, &args)?;
    let input = function.encode_input(&tokens)?;

    let status = context
        .client
        .view(
            &context.cli.engine,
            ViewCall {
                args: ViewCallArgs {
                    sender: from,
                    address,
                    amount: Wei::zero().to_bytes(),
                    input,
                },
            },
        )
        .await?;

    if let TransactionStatus::Succeed(bytes) = status {
        Ok(function.decode_output(&bytes)?)
    } else {
        anyhow::bail!("View call failed: {status:?}");
    }
}

fn args_from_kind(kind: WhitelistKind, entry: String) -> anyhow::Result<WhitelistArgs> {
    match kind {
        WhitelistKind::Admin | WhitelistKind::Account => {
            let account_id = entry
                .parse()
                .map_err(|_| anyhow::anyhow!("failed to parse account id"))?;
            Ok(WhitelistArgs::WhitelistAccountArgs(WhitelistAccountArgs {
                kind,
                account_id,
            }))
        }
        WhitelistKind::EvmAdmin | WhitelistKind::Address => {
            let address = Address::decode(entry.trim_start_matches("0x"))
                .map_err(|_| anyhow::anyhow!("Invalid EVM Address"))?;
            Ok(WhitelistArgs::WhitelistAddressArgs(WhitelistAddressArgs {
                kind,
                address,
            }))
        }
    }
}

async fn send_aurora_transaction(
    context: &Context,
    to: Option<Address>,
    amount: Wei,
    input: Vec<u8>,
    aurora_secret_key: SecretKeyEth,
) -> anyhow::Result<SubmitResult> {
    let sender_address = common::address_from_secret_key(&aurora_secret_key)?;
    let nonce = get_nonce(context, sender_address).await?;

    let tx = TransactionLegacy {
        nonce: nonce.into(),
        gas_price: U256::zero(),
        gas_limit: U256::from(u64::MAX),
        to,
        value: amount,
        data: input,
    };

    let chain_id = get_chain_id(context).await?;
    let signed_tx = common::sign_transaction(tx, chain_id, &aurora_secret_key)?;

    context
        .client
        .call(
            &context.cli.engine,
            Submit {
                transaction: EthTransactionKind::Legacy(signed_tx),
            },
        )
        .await
        .map_err(Into::into)
}
