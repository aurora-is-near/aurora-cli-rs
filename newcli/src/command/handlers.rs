use std::path::PathBuf;

use aurora_sdk_rs::{
    aurora::{
        H256, abi, ethabi,
        parameters::{
            connector::FungibleTokenMetadata,
            engine::{SubmitResult, TransactionStatus},
            silo::WhitelistKind,
        },
        types::{Address, Balance, EthGas, NEP141Wei, Wei, Yocto},
    },
    near::{
        crypto::{KeyType, PublicKey},
        primitives::{borsh::BorshDeserialize, types::AccountId, views::FinalExecutionStatus},
        token::NearToken,
    },
};

use crate::{common::output::CommandResult, context::Context, output, result_object};

use super::near;

pub async fn create_account(
    context: &Context,
    account: AccountId,
    balance: NearToken,
) -> anyhow::Result<()> {
    let account_str = account.to_string();
    let key_pair = near::create_account(context, account, balance).await?;
    output!(
        &context.cli.output_format,
        result_object!("account_id" => account_str, "public_key" => key_pair.public_key().to_string(), "private_key" => key_pair.to_string())
    );
    Ok(())
}

pub async fn view_account(context: &Context, account: &AccountId) -> anyhow::Result<()> {
    let view = near::view_account(context, account).await?;
    output!(
        &context.cli.output_format,
        result_object!("account" => account.to_string(), "view" => format!("{:?}", view))
    );
    Ok(())
}

pub async fn deploy_aurora(context: &Context, path: &PathBuf) -> anyhow::Result<()> {
    let wasm = std::fs::read(path)?;
    let outcome = near::deploy_aurora(context, wasm).await?;
    output!(
        &context.cli.output_format,
        result_object!("status" => "deployed", "path" => path.display().to_string(), "outcome" => outcome)
    );
    Ok(())
}

pub async fn init(
    context: &Context,
    chain_id: u64,
    owner_id: AccountId,
    bridge_prover_id: AccountId,
    upgrade_delay_blocks: Option<u64>,
    custodian_address: Option<Address>,
    ft_metadata_path: FungibleTokenMetadata,
) -> anyhow::Result<()> {
    near::init(
        context,
        chain_id,
        owner_id,
        bridge_prover_id,
        upgrade_delay_blocks,
        custodian_address,
        ft_metadata_path,
    )
    .await?;

    output!(
        &context.cli.output_format,
        CommandResult::success("Aurora EVM and ETH connector initialized successfully")
    );
    Ok(())
}

pub async fn get_chain_id(context: &Context) -> anyhow::Result<()> {
    let id = near::get_chain_id(context).await?;
    output!(&context.cli.output_format, result_object!("chain_id" => id));
    Ok(())
}

pub async fn get_nonce(context: &Context, address: Address) -> anyhow::Result<()> {
    let nonce = near::get_nonce(context, address).await?;
    output!(
        &context.cli.output_format,
        result_object!("address" => format!("{:?}", address), "nonce" => nonce)
    );
    Ok(())
}

pub async fn get_block_hash(context: &Context, height: u64) -> anyhow::Result<()> {
    let hash = near::get_block_hash(context, height).await?;
    output!(
        &context.cli.output_format,
        result_object!("height" => height, "hash" => hash.to_string())
    );
    Ok(())
}

pub async fn get_code(context: &Context, address: Address) -> anyhow::Result<()> {
    let code = near::get_code(context, address).await?;
    output!(
        &context.cli.output_format,
        result_object!("address" => format!("{:?}", address), "code" => format!("0x{:?}", hex::encode(code)))
    );
    Ok(())
}

pub async fn get_balance(context: &Context, address: Address) -> anyhow::Result<()> {
    let balance = near::get_balance(context, address).await?;
    output!(
        &context.cli.output_format,
        result_object!("address" => format!("{:?}", address), "balance" => balance.to_string())
    );
    Ok(())
}

pub async fn get_upgrade_index(context: &Context) -> anyhow::Result<()> {
    let index = near::get_upgrade_index(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("upgrade_index" => index)
    );
    Ok(())
}

pub async fn get_version(context: &Context) -> anyhow::Result<()> {
    let version = near::get_version(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("version" => version)
    );
    Ok(())
}

pub async fn get_owner(context: &Context) -> anyhow::Result<()> {
    let owner = near::get_owner(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("owner" => owner.to_string())
    );
    Ok(())
}

pub async fn set_owner(context: &Context, account_id: AccountId) -> anyhow::Result<()> {
    near::set_owner(context, account_id).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Owner set successfully")
    );
    Ok(())
}

pub async fn get_bridge_prover(context: &Context) -> anyhow::Result<()> {
    let prover = near::get_bridge_prover(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("bridge_prover" => prover.to_string())
    );
    Ok(())
}

pub async fn get_storage_at(context: &Context, address: Address, key: H256) -> anyhow::Result<()> {
    near::get_storage_at(context, address, key).await?;
    Ok(())
}

pub async fn register_relayer(context: &Context, address: Address) -> anyhow::Result<()> {
    near::register_relayer(context, address).await?;
    Ok(())
}

pub async fn start_hashchain(
    context: &Context,
    block_height: u64,
    block_hashchain: String,
) -> anyhow::Result<()> {
    near::start_hashchain(context, block_height, block_hashchain).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Hashchain started successfully")
    );
    Ok(())
}

pub async fn pause_contract(context: &Context) -> anyhow::Result<()> {
    near::pause_contract(context).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Contract paused successfully")
    );
    Ok(())
}

pub async fn resume_contract(context: &Context) -> anyhow::Result<()> {
    near::resume_contract(context).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Contract resumed successfully")
    );
    Ok(())
}

pub async fn pause_precompiles(context: &Context, mask: u32) -> anyhow::Result<()> {
    near::pause_precompiles(context, mask).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Precompiles paused successfully")
    );
    Ok(())
}

pub async fn resume_precompiles(context: &Context, mask: u32) -> anyhow::Result<()> {
    near::resume_precompiles(context, mask).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Precompiles resumed successfully")
    );
    Ok(())
}

pub async fn paused_precompiles(context: &Context) -> anyhow::Result<()> {
    let mask = near::paused_precompiles(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("paused_precompiles_mask" => mask)
    );
    Ok(())
}

pub async fn factory_update(context: &Context, path: String) -> anyhow::Result<()> {
    let code = std::fs::read(path)?;
    near::factory_update(context, code).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Factory updated successfully")
    );
    Ok(())
}

pub async fn factory_get_wnear_address(context: &Context) -> anyhow::Result<()> {
    let address = near::factory_get_wnear_address(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("wnear_address" => format!("{:?}", address))
    );
    Ok(())
}

pub async fn factory_update_address_version(
    context: &Context,
    address: Address,
    version: u32,
) -> anyhow::Result<()> {
    near::factory_update_address_version(context, address, version).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Factory address version updated successfully")
    );
    Ok(())
}

pub async fn factory_set_wnear_address(context: &Context, address: Address) -> anyhow::Result<()> {
    near::factory_set_wnear_address(context, address).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("WNEAR address set successfully")
    );
    Ok(())
}

pub async fn fund_xcc_sub_account(
    context: &Context,
    target: Address,
    wnear_account_id: Option<AccountId>,
    deposit: NearToken,
) -> anyhow::Result<()> {
    near::fund_xcc_sub_account(context, target, wnear_account_id, deposit).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("XCC sub-account funded successfully")
    );
    Ok(())
}

pub async fn upgrade(context: &Context, path: String) -> anyhow::Result<()> {
    let code = std::fs::read(path)?;
    near::upgrade(context, code).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Contract upgraded successfully")
    );
    Ok(())
}

pub async fn stage_upgrade(context: &Context, path: String) -> anyhow::Result<()> {
    let code = std::fs::read(path)?;
    near::stage_upgrade(context, code).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Contract staged successfully")
    );
    Ok(())
}

pub async fn deploy_upgrade(context: &Context) -> anyhow::Result<()> {
    near::deploy_upgrade(context).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Upgrade deployed successfully")
    );
    Ok(())
}

pub async fn deploy(
    context: &Context,
    code: String,
    args: Option<String>,
    abi_path: Option<PathBuf>,
    aurora_secret_key: String,
) -> anyhow::Result<()> {
    let input =
        if let Some((abi_path, args)) = abi_path.and_then(|path| args.map(|args| (path, args))) {
            let contract = abi::read_contract(abi_path)?;
            let constructor = contract
                .constructor()
                .ok_or_else(|| anyhow::anyhow!("Constructor not found in ABI"))?;
            let args: serde_json::Value = serde_json::from_str(&args)?;
            let tokens = abi::parse_args(&constructor.inputs, &args)?;
            let code = hex::decode(code)?;
            constructor.encode_input(code, &tokens)?
        } else {
            hex::decode(code)?
        };

    let result = near::deploy(context, input, aurora_secret_key).await?;
    output!(
        &context.cli.output_format,
        result_object!("deploy_result" => format!("{:?}", result.status))
    );
    Ok(())
}

pub async fn call(
    context: &Context,
    address: Address,
    input: Option<String>,
    value: Option<u128>,
    from: Option<AccountId>,
) -> anyhow::Result<()> {
    let outcome = near::call(context, address, input, value, from).await?;
    match outcome.status {
        FinalExecutionStatus::NotStarted | FinalExecutionStatus::Started => {}
        FinalExecutionStatus::Failure(failure) => {
            output!(
                &context.cli.output_format,
                result_object!("status" => "failure", "error" => failure.to_string())
            );
        }
        FinalExecutionStatus::SuccessValue(result) => {
            let submit_result = SubmitResult::try_from_slice(&result)?;
            match submit_result.status {
                TransactionStatus::Succeed(_) => {
                    output!(
                        &context.cli.output_format,
                        result_object!("status" => "success")
                    );
                }
                TransactionStatus::Revert(_) => {
                    output!(
                        &context.cli.output_format,
                        result_object!("status" => "reverted")
                    );
                }
                _ => {
                    output!(
                        &context.cli.output_format,
                        result_object!("status" => "failed")
                    );
                }
            }
        }
    }
    Ok(())
}

pub async fn view_call(
    context: &Context,
    address: Address,
    function: String,
    args: Option<String>,
    from: Address,
    abi_path: String,
) -> anyhow::Result<()> {
    let contract = abi::read_contract(abi_path)?;
    let tokens = near::view_call(context, address, function, args, from, contract).await?;

    let result = tokens
        .iter()
        .map(ethabi::Token::to_string)
        .collect::<Vec<_>>();

    output!(
        &context.cli.output_format,
        result_object!("address" => format!("{:?}", address), "result" => result)
    );
    Ok(())
}

pub async fn submit(
    context: &Context,
    address: Address,
    function: String,
    args: Option<String>,
    abi_path: String,
    value: Wei,
    aurora_secret_key: String,
) -> anyhow::Result<()> {
    let result = near::submit(
        context,
        address,
        function,
        args,
        abi_path,
        value,
        aurora_secret_key,
    )
    .await?;
    output!(
        &context.cli.output_format,
        result_object!("submit_result" => format!("{:?}", result))
    );
    Ok(())
}

pub async fn encode_address(context: &Context, account: &AccountId) -> anyhow::Result<()> {
    let addr = near::encode_to_address(account);
    output!(
        &context.cli.output_format,
        result_object!("account" => account.to_string(), "encoded_address" => format!("{:?}", addr))
    );
    Ok(())
}

pub async fn key_pair(context: &Context, random: bool, seed: Option<u64>) -> anyhow::Result<()> {
    let (addr, sk) = near::gen_key_pair(random, seed)?;
    output!(
        &context.cli.output_format,
        result_object!("address" => format!("{:?}", addr), "secret_key" => format!("{:?}", sk))
    );
    Ok(())
}

pub async fn generate_near_key(context: &Context, key_type: KeyType) -> anyhow::Result<()> {
    let key = near::gen_near_key_pair(key_type)?;
    output!(
        &context.cli.output_format,
        result_object!("generated_near_key" => format!("{:?}", key))
    );
    Ok(())
}

pub async fn get_fixed_gas(context: &Context) -> anyhow::Result<()> {
    let gas = near::get_fixed_gas(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("fixed_gas" => format!("{:?}", gas))
    );
    Ok(())
}

pub async fn set_fixed_gas(context: &Context, cost: EthGas) -> anyhow::Result<()> {
    near::set_fixed_gas(context, Some(cost)).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Fixed gas set successfully")
    );
    Ok(())
}

pub async fn get_silo_params(context: &Context) -> anyhow::Result<()> {
    let params = near::get_silo_params(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("silo_params" => format!("{:?}", params))
    );
    Ok(())
}

pub async fn set_silo_params(
    context: &Context,
    gas: EthGas,
    fallback_address: Address,
) -> anyhow::Result<()> {
    let params = aurora_sdk_rs::aurora::parameters::silo::SiloParamsArgs {
        fixed_gas: gas,
        erc20_fallback_address: fallback_address,
    };
    near::set_silo_params(context, Some(params)).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Silo params set")
    );
    Ok(())
}

pub async fn disable_silo_mode(context: &Context) -> anyhow::Result<()> {
    near::set_silo_params(context, None).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Silo mode disabled")
    );
    Ok(())
}

pub async fn get_whitelist_status(context: &Context, kind: WhitelistKind) -> anyhow::Result<()> {
    let status = near::get_whitelist_status(context, kind).await?;
    output!(
        &context.cli.output_format,
        result_object!("whitelist_kind" => format!("{:?}", kind), "status" => format!("{:?}", status))
    );
    Ok(())
}

pub async fn set_whitelist_status(
    context: &Context,
    kind: WhitelistKind,
    status: u8,
) -> anyhow::Result<()> {
    near::set_whitelist_status(context, kind, status != 0).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Whitelist status set")
    );
    Ok(())
}

pub async fn add_entry_to_whitelist(
    context: &Context,
    kind: WhitelistKind,
    entry: String,
) -> anyhow::Result<()> {
    near::add_entry_to_whitelist(context, kind, entry).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Entry added to whitelist")
    );
    Ok(())
}

pub async fn remove_entry_from_whitelist(
    context: &Context,
    kind: WhitelistKind,
    entry: String,
) -> anyhow::Result<()> {
    near::remove_entry_from_whitelist(context, kind, entry).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Entry removed from whitelist")
    );
    Ok(())
}

pub async fn set_key_manager(
    context: &Context,
    account_id: Option<AccountId>,
) -> anyhow::Result<()> {
    near::set_key_manager(context, account_id).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Key manager set")
    );
    Ok(())
}

pub async fn add_relayer_key(
    context: &Context,
    public_key: PublicKey,
    allowance: NearToken,
) -> anyhow::Result<()> {
    let outcome = near::add_relayer_key(context, public_key, allowance).await?;
    output!(
        &context.cli.output_format,
        result_object!("relayer_key_added" => format!("{:?}", outcome))
    );
    Ok(())
}

pub async fn remove_relayer_key(context: &Context, public_key: PublicKey) -> anyhow::Result<()> {
    near::remove_relayer_key(context, public_key).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Relayer key removed")
    );
    Ok(())
}

pub async fn get_upgrade_delay_blocks(context: &Context) -> anyhow::Result<()> {
    let blocks = near::get_upgrade_delay_blocks(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("upgrade_delay_blocks" => blocks)
    );
    Ok(())
}

pub async fn set_upgrade_delay_blocks(context: &Context, blocks: u64) -> anyhow::Result<()> {
    near::set_upgrade_delay_blocks(context, blocks).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Upgrade delay blocks set")
    );
    Ok(())
}

pub async fn get_erc20_from_nep141(context: &Context, account_id: AccountId) -> anyhow::Result<()> {
    let account_str = account_id.to_string();
    let erc20 = near::get_erc20_from_nep141(context, account_id).await?;
    output!(
        &context.cli.output_format,
        result_object!("nep141_account" => account_str, "erc20_address" => format!("{:?}", erc20))
    );
    Ok(())
}

pub async fn get_nep141_from_erc20(context: &Context, address: Address) -> anyhow::Result<()> {
    let acc_id = near::get_nep141_from_erc20(context, address).await?;
    output!(
        &context.cli.output_format,
        result_object!("erc20_address" => format!("{:?}", address), "nep141_account" => acc_id.to_string())
    );
    Ok(())
}

pub async fn get_erc20_metadata(context: &Context, erc20_id: String) -> anyhow::Result<()> {
    let erc20_id_str = erc20_id.clone();
    let meta = near::get_erc20_metadata(context, erc20_id).await?;
    output!(
        &context.cli.output_format,
        result_object!("erc20_id" => erc20_id_str, "metadata" => format!("{:?}", meta))
    );
    Ok(())
}

pub async fn set_erc20_metadata(
    context: &Context,
    erc20_id: String,
    name: String,
    symbol: String,
    decimals: u8,
) -> anyhow::Result<()> {
    let metadata = aurora_sdk_rs::aurora::parameters::connector::Erc20Metadata {
        name,
        symbol,
        decimals,
    };
    near::set_erc20_metadata(context, erc20_id, metadata).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("ERC20 metadata set")
    );
    Ok(())
}

pub async fn mirror_erc20_token(
    context: &Context,
    contract_id: AccountId,
    nep141: AccountId,
) -> anyhow::Result<()> {
    let addr = near::mirror_erc20_token(context, contract_id, nep141).await?;
    output!(
        &context.cli.output_format,
        result_object!("mirrored_erc20_token_address" => format!("{:?}", addr))
    );
    Ok(())
}

pub async fn set_eth_connector_contract_account(
    context: &Context,
    account_id: AccountId,
) -> anyhow::Result<()> {
    near::set_eth_connector_contract_account(context, account_id).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Eth connector contract account set")
    );
    Ok(())
}

pub async fn get_eth_connector_contract_account(context: &Context) -> anyhow::Result<()> {
    let account = near::get_eth_connector_contract_account(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("eth_connector_contract_account" => account.to_string())
    );
    Ok(())
}

pub async fn set_eth_connector_contract_data(
    context: &Context,
    prover_id: AccountId,
    custodian_address: Address,
    ft_metadata_path: String,
) -> anyhow::Result<()> {
    near::set_eth_connector_contract_data(context, prover_id, custodian_address, ft_metadata_path)
        .await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Eth connector contract data set")
    );
    Ok(())
}

pub async fn set_paused_flags(context: &Context, mask: u8) -> anyhow::Result<()> {
    near::set_paused_flags(context, mask).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Paused flags set")
    );
    Ok(())
}

pub async fn get_paused_flags(context: &Context) -> anyhow::Result<()> {
    let flags = near::get_paused_flags(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("paused_flags" => format!("{:?}", flags))
    );
    Ok(())
}

pub async fn add_relayer(
    context: &Context,
    deposit: NearToken,
    full_access_pub_key: PublicKey,
    function_call_pub_key: PublicKey,
) -> anyhow::Result<()> {
    let outcome =
        near::add_relayer(context, deposit, full_access_pub_key, function_call_pub_key).await?;
    output!(
        &context.cli.output_format,
        result_object!("relayer_added" => format!("{:?}", outcome))
    );
    Ok(())
}

pub async fn withdraw_wnear_to_router(
    context: &Context,
    address: Address,
    amount: NearToken,
) -> anyhow::Result<()> {
    near::withdraw_wnear_to_router(context, address, amount).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("wNEAR withdrawn to router successfully")
    );
    Ok(())
}

pub async fn mirror_erc20_token_callback(
    context: &Context,
    contract_id: AccountId,
    nep141: AccountId,
) -> anyhow::Result<()> {
    near::mirror_erc20_token_callback(context, contract_id, nep141).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("ERC-20 token mirrored successfully")
    );
    Ok(())
}

pub async fn get_latest_hashchain(context: &Context) -> anyhow::Result<()> {
    let latest_hashchain = near::get_latest_hashchain(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("latest_hashchain" => latest_hashchain.to_string())
    );
    Ok(())
}

pub async fn ft_total_supply(context: &Context) -> anyhow::Result<()> {
    let total_supply = near::ft_total_supply(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("total_supply" => format!("{:?}", total_supply))
    );
    Ok(())
}

pub async fn ft_balance_of(context: &Context, account_id: AccountId) -> anyhow::Result<()> {
    let balance = near::ft_balance_of(context, account_id.clone()).await?;
    output!(
        &context.cli.output_format,
        result_object!("account_id" => account_id.to_string(), "balance" => format!("{:?}", balance))
    );
    Ok(())
}

pub async fn ft_balance_of_eth(context: &Context, address: Address) -> anyhow::Result<()> {
    let balance = near::ft_balance_of_eth(context, address).await?;
    output!(
        &context.cli.output_format,
        result_object!("address" => address.encode(), "balance" => format!("{:?}", balance))
    );
    Ok(())
}

pub async fn ft_transfer(
    context: &Context,
    receiver_id: AccountId,
    amount: NearToken,
    memo: Option<String>,
) -> anyhow::Result<()> {
    near::ft_transfer(
        context,
        receiver_id,
        NEP141Wei::new(amount.as_yoctonear()),
        memo,
    )
    .await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("FT transfer executed successfully")
    );
    Ok(())
}

pub async fn ft_transfer_call(
    context: &Context,
    receiver_id: AccountId,
    amount: NearToken,
    memo: Option<String>,
    msg: String,
) -> anyhow::Result<()> {
    near::ft_transfer_call(
        context,
        receiver_id,
        NEP141Wei::new(amount.as_yoctonear()),
        memo,
        msg,
    )
    .await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("FT transfer call executed successfully")
    );
    Ok(())
}

pub async fn ft_on_transfer(
    context: &Context,
    sender_id: AccountId,
    amount: NearToken,
    msg: String,
) -> anyhow::Result<()> {
    near::ft_on_transfer(context, sender_id, Balance::new(amount.as_yoctonear()), msg).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("FT on transfer executed successfully")
    );
    Ok(())
}

pub async fn deploy_erc20_token(context: &Context, nep141: AccountId) -> anyhow::Result<()> {
    let address = near::deploy_erc20_token(context, nep141).await?;
    output!(
        &context.cli.output_format,
        result_object!("deployed_erc20_token_address" => format!("{:?}", address))
    );
    Ok(())
}

pub async fn storage_deposit(
    context: &Context,
    account_id: Option<AccountId>,
    registration_only: Option<bool>,
) -> anyhow::Result<()> {
    near::storage_deposit(context, account_id, registration_only).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Storage deposit executed successfully")
    );
    Ok(())
}

pub async fn storage_unregister(context: &Context, force: bool) -> anyhow::Result<()> {
    near::storage_unregister(context, force).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Storage unregistered successfully")
    );
    Ok(())
}

pub async fn storage_withdraw(context: &Context, amount: Option<NearToken>) -> anyhow::Result<()> {
    near::storage_withdraw(context, amount.map(|n| Yocto::new(n.as_yoctonear()))).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Storage withdrawn successfully")
    );
    Ok(())
}

pub async fn storage_balance_of(context: &Context, account_id: AccountId) -> anyhow::Result<()> {
    let balance = near::storage_balance_of(context, account_id.clone()).await?;
    output!(
        &context.cli.output_format,
        result_object!("account_id" => account_id.to_string(), "storage_balance" => format!("{:?}", balance))
    );
    Ok(())
}
