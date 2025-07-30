use std::path::PathBuf;

use aurora_sdk_rs::{
    aurora::{
        H256, abi, ethabi,
        parameters::{
            connector::{Erc20Metadata, FungibleTokenMetadata},
            engine::{SubmitResult, TransactionStatus},
            silo::WhitelistKind,
        },
        types::{Address, Balance, EthGas, NEP141Wei, Wei, Yocto},
    },
    near::{
        crypto::{KeyType, PublicKey, SecretKey},
        primitives::{borsh::BorshDeserialize, types::AccountId, views::FinalExecutionStatus},
        token::NearToken,
    },
};
use serde_json::Value;

use crate::{
    cli::OutputFormat, common::output::CommandResult, context::Context, output, result_object,
};

use super::near;

/// Common function to read WASM files with proper error handling
fn read_wasm_file(context: &Context, path: &PathBuf) -> Option<Vec<u8>> {
    match std::fs::read(path) {
        Ok(data) => Some(data),
        Err(e) => {
            output!(
                &context.cli.output_format,
                result_object!("error" => format!("Failed to read WASM file {}: {}", path.display(), e))
            );
            None
        }
    }
}

macro_rules! handle_near_call {
    ($context:expr, $call:expr, $closure:expr) => {
        match $call {
            Ok(result) => $closure(result),
            Err(e) => {
                output!(&$context.cli.output_format, result_object!("error" => e.to_string()));
            }
        }
    };
    ($context:expr, $call:expr, $var_name:ident, $success_block:block) => {
        match $call {
            Ok($var_name) => $success_block,
            Err(e) => {
                output!(&$context.cli.output_format, result_object!("error" => e.to_string()));
            }
        }
    };
    ($context:expr, $call:expr, success: $msg:expr) => {
        match $call {
            Ok(_) => {
                output!(&$context.cli.output_format, CommandResult::success($msg));
            }
            Err(e) => {
                output!(&$context.cli.output_format, result_object!("error" => e.to_string()));
            }
        }
    };
}

pub async fn create_account(context: &Context, account: AccountId, balance: NearToken) {
    let account_str = account.to_string();
    handle_near_call!(
        context,
        near::create_account(
            context,
            account,
            NearToken::from_near(balance.as_yoctonear())
        )
        .await,
        |key_pair: SecretKey| {
            output!(
                &context.cli.output_format,
                result_object!("account_id" => account_str, "public_key" => key_pair.public_key().to_string(), "secret_key" => key_pair.to_string())
            );
        }
    );
}

pub async fn view_account(context: &Context, account: &AccountId) {
    handle_near_call!(
        context,
        near::view_account(context, account).await,
        |view| {
            output!(
                &context.cli.output_format,
                result_object!("account" => account.to_string(), "view" => format!("{:?}", view))
            );
        }
    );
}

pub async fn deploy_aurora(context: &Context, path: &PathBuf) {
    let Some(wasm) = read_wasm_file(context, path) else {
        return;
    };

    handle_near_call!(
        context,
        near::deploy_aurora(context, wasm).await,
        |outcome| {
            output!(
                &context.cli.output_format,
                result_object!("status" => "deployed", "path" => path.display().to_string(), "outcome" => outcome)
            );
        }
    );
}

pub async fn init(
    context: &Context,
    chain_id: u64,
    owner_id: AccountId,
    bridge_prover_id: Option<AccountId>,
    upgrade_delay_blocks: Option<u64>,
    custodian_address: Option<Address>,
    ft_metadata_path: FungibleTokenMetadata,
) {
    let bridge_prover_id = bridge_prover_id.unwrap_or_else(|| context.cli.engine.clone());
    handle_near_call!(
        context,
        near::init(
            context,
            chain_id,
            owner_id,
            bridge_prover_id,
            upgrade_delay_blocks,
            custodian_address,
            ft_metadata_path,
        )
        .await,
        |outcome| {
            output!(
                &context.cli.output_format,
                result_object!("outcome" => outcome)
            );
        }
    );
}

pub async fn get_chain_id(context: &Context) {
    handle_near_call!(context, near::get_chain_id(context).await, |id| {
        output!(&context.cli.output_format, result_object!("chain_id" => id));
    });
}

pub async fn get_nonce(context: &Context, address: Address) {
    handle_near_call!(context, near::get_nonce(context, address).await, |nonce| {
        output!(
            &context.cli.output_format,
            result_object!("address" => format!("{:?}", address), "nonce" => nonce)
        );
    });
}

pub async fn get_block_hash(context: &Context, height: u64) {
    handle_near_call!(
        context,
        near::get_block_hash(context, height).await,
        hash,
        {
            output!(
                &context.cli.output_format,
                result_object!("height" => height, "hash" => hash.to_string())
            );
        }
    );
}

pub async fn get_code(context: &Context, address: Address) {
    handle_near_call!(context, near::get_code(context, address).await, |code| {
        output!(
            &context.cli.output_format,
            result_object!("address" => format!("{:?}", address), "code" => format!("0x{}", hex::encode(code)))
        );
    });
}

pub async fn get_balance(context: &Context, address: Address) {
    handle_near_call!(
        context,
        near::get_balance(context, address).await,
        balance,
        {
            output!(
                &context.cli.output_format,
                result_object!("address" => format!("{:?}", address), "balance" => balance.to_string())
            );
        }
    );
}

pub async fn get_upgrade_index(context: &Context) {
    handle_near_call!(context, near::get_upgrade_index(context).await, |index| {
        output!(
            &context.cli.output_format,
            result_object!("upgrade_index" => index)
        );
    });
}

pub async fn get_version(context: &Context) {
    handle_near_call!(
        context,
        near::get_version(context).await,
        |version: String| {
            if context.cli.output_format == OutputFormat::Plain {
                println!("{}", version.trim());
                return;
            }

            output!(
                &context.cli.output_format,
                result_object!("version" => version.trim())
            );
        }
    );
}

pub async fn get_owner(context: &Context) {
    handle_near_call!(context, near::get_owner(context).await, owner, {
        output!(
            &context.cli.output_format,
            result_object!("owner" => owner.to_string())
        );
    });
}

pub async fn set_owner(context: &Context, account_id: AccountId) {
    handle_near_call!(context, near::set_owner(context, account_id).await, success: "Owner set successfully");
}

pub async fn get_bridge_prover(context: &Context) {
    handle_near_call!(context, near::get_bridge_prover(context).await, prover, {
        output!(
            &context.cli.output_format,
            result_object!("bridge_prover" => prover.to_string())
        );
    });
}

pub async fn get_storage_at(context: &Context, address: Address, key: H256) {
    handle_near_call!(
        context,
        near::get_storage_at(context, address, key).await,
        |value| {
            output!(
                &context.cli.output_format,
                result_object!("address" => format!("{:?}", address), "key" => format!("{:?}", key), "value" => format!("{:?}", value))
            );
        }
    );
}

pub async fn register_relayer(context: &Context, address: Address) {
    handle_near_call!(context, near::register_relayer(context, address).await, success: "Relayer registered successfully");
}

pub async fn start_hashchain(context: &Context, block_height: u64, block_hashchain: String) {
    handle_near_call!(context, near::start_hashchain(context, block_height, block_hashchain).await, success: "Hashchain started successfully");
}

pub async fn pause_contract(context: &Context) {
    handle_near_call!(
        context,
        near::pause_contract(context).await,
        success: "Contract paused successfully"
    );
}

pub async fn resume_contract(context: &Context) {
    handle_near_call!(
        context,
        near::resume_contract(context).await,
        success: "Contract resumed successfully"
    );
}

pub async fn pause_precompiles(context: &Context, mask: u32) {
    handle_near_call!(
        context,
        near::pause_precompiles(context, mask).await,
        success: "Precompiles paused successfully"
    );
}

pub async fn resume_precompiles(context: &Context, mask: u32) {
    handle_near_call!(
        context,
        near::resume_precompiles(context, mask).await,
        success: "Precompiles resumed successfully"
    );
}

pub async fn paused_precompiles(context: &Context) {
    handle_near_call!(context, near::paused_precompiles(context).await, |mask| {
        output!(
            &context.cli.output_format,
            result_object!("paused_precompiles_mask" => mask)
        );
    });
}

pub async fn factory_update(context: &Context, path: PathBuf) {
    let Some(wasm) = read_wasm_file(context, &path) else {
        return;
    };
    handle_near_call!(context, near::factory_update(context, wasm).await, success: "Factory updated successfully");
}

pub async fn factory_get_wnear_address(context: &Context) {
    handle_near_call!(
        context,
        near::factory_get_wnear_address(context).await,
        address,
        {
            output!(
                &context.cli.output_format,
                result_object!("wnear_address" => format!("{:?}", address))
            );
        }
    );
}

pub async fn factory_update_address_version(context: &Context, address: Address, version: u32) {
    handle_near_call!(context, near::factory_update_address_version(context, address, version).await, success: "Factory address version updated successfully");
}

pub async fn factory_set_wnear_address(context: &Context, address: Address) {
    handle_near_call!(context, near::factory_set_wnear_address(context, address).await, success: "WNEAR address set successfully");
}

pub async fn fund_xcc_sub_account(
    context: &Context,
    target: Address,
    wnear_account_id: Option<AccountId>,
    deposit: NearToken,
) {
    handle_near_call!(context, near::fund_xcc_sub_account(context, target, wnear_account_id, deposit).await, success: "XCC sub-account funded successfully");
}

pub async fn upgrade(context: &Context, path: PathBuf) {
    let Some(code) = read_wasm_file(context, &path) else {
        return;
    };
    handle_near_call!(context, near::upgrade(context, code).await, success: "Contract upgraded successfully");
}

pub async fn stage_upgrade(context: &Context, path: PathBuf) {
    let Some(code) = read_wasm_file(context, &path) else {
        return;
    };
    handle_near_call!(context, near::stage_upgrade(context, code).await, success: "Contract staged successfully");
}

pub async fn deploy_upgrade(context: &Context) {
    handle_near_call!(context, near::deploy_upgrade(context).await, success: "Upgrade deployed successfully");
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
    handle_near_call!(context, near::gen_key_pair(random, seed), |(addr, sk)| {
        output!(
            &context.cli.output_format,
            result_object!("address" => format!("{:?}", addr), "secret_key" => format!("{:?}", sk))
        );
    });
    Ok(())
}

pub async fn generate_near_key(context: &Context, key_type: KeyType) {
    handle_near_call!(
        context,
        near::gen_near_key_pair(key_type),
        |key: SecretKey| {
            output!(
                &context.cli.output_format,
                result_object!("generated_near_key" => format!("{:?}", key))
            );
        }
    );
}

pub async fn get_fixed_gas(context: &Context) {
    handle_near_call!(context, near::get_fixed_gas(context).await, |gas| {
        output!(
            &context.cli.output_format,
            result_object!("fixed_gas" => format!("{:?}", gas))
        );
    });
}

pub async fn set_fixed_gas(context: &Context, cost: EthGas) {
    handle_near_call!(context, near::set_fixed_gas(context, Some(cost)).await, success: "Fixed gas set successfully");
}

pub async fn get_silo_params(context: &Context) {
    handle_near_call!(context, near::get_silo_params(context).await, |params| {
        output!(
            &context.cli.output_format,
            result_object!("silo_params" => format!("{:?}", params))
        );
    });
}

pub async fn set_silo_params(context: &Context, gas: EthGas, fallback_address: Address) {
    let params = aurora_sdk_rs::aurora::parameters::silo::SiloParamsArgs {
        fixed_gas: gas,
        erc20_fallback_address: fallback_address,
    };
    handle_near_call!(context, near::set_silo_params(context, Some(params)).await, success: "Silo params set");
}

pub async fn disable_silo_mode(context: &Context) {
    handle_near_call!(context, near::set_silo_params(context, None).await, success: "Silo mode disabled");
}

pub async fn get_whitelist_status(context: &Context, kind: WhitelistKind) {
    handle_near_call!(
        context,
        near::get_whitelist_status(context, kind).await,
        |status| {
            output!(
                &context.cli.output_format,
                result_object!("whitelist_kind" => format!("{:?}", kind), "status" => format!("{:?}", status))
            );
        }
    );
}

pub async fn set_whitelist_status(context: &Context, kind: WhitelistKind, status: u8) {
    handle_near_call!(context, near::set_whitelist_status(context, kind, status != 0).await, success: "Whitelist status set");
}

pub async fn add_entry_to_whitelist(context: &Context, kind: WhitelistKind, entry: String) {
    handle_near_call!(context, near::add_entry_to_whitelist(context, kind, entry).await, success: "Entry added to whitelist");
}

pub async fn remove_entry_from_whitelist(context: &Context, kind: WhitelistKind, entry: String) {
    handle_near_call!(context, near::remove_entry_from_whitelist(context, kind, entry).await, success: "Entry removed from whitelist");
}

pub async fn set_key_manager(context: &Context, account_id: Option<AccountId>) {
    handle_near_call!(context, near::set_key_manager(context, account_id).await, success: "Key manager set");
}

pub async fn add_relayer_key(context: &Context, public_key: PublicKey, allowance: NearToken) {
    handle_near_call!(
        context,
        near::add_relayer_key(context, public_key, allowance).await,
        |outcome| {
            output!(
                &context.cli.output_format,
                result_object!("relayer_key_added" => format!("{:?}", outcome))
            );
        }
    );
}

pub async fn remove_relayer_key(context: &Context, public_key: PublicKey) {
    handle_near_call!(context, near::remove_relayer_key(context, public_key).await, success: "Relayer key removed");
}

pub async fn get_upgrade_delay_blocks(context: &Context) {
    handle_near_call!(
        context,
        near::get_upgrade_delay_blocks(context).await,
        |blocks| {
            output!(
                &context.cli.output_format,
                result_object!("upgrade_delay_blocks" => blocks)
            );
        }
    );
}

pub async fn set_upgrade_delay_blocks(context: &Context, blocks: u64) {
    handle_near_call!(context, near::set_upgrade_delay_blocks(context, blocks).await, success: "Upgrade delay blocks set");
}

pub async fn get_erc20_from_nep141(context: &Context, account_id: AccountId) {
    let account_str = account_id.to_string();
    handle_near_call!(
        context,
        near::get_erc20_from_nep141(context, account_id).await,
        |erc20| {
            output!(
                &context.cli.output_format,
                result_object!("nep141_account" => account_str, "erc20_address" => format!("{:?}", erc20))
            );
        }
    );
}

pub async fn get_nep141_from_erc20(context: &Context, address: Address) {
    handle_near_call!(
        context,
        near::get_nep141_from_erc20(context, address).await,
        acc_id,
        {
            output!(
                &context.cli.output_format,
                result_object!("erc20_address" => format!("{:?}", address), "nep141_account" => acc_id.to_string())
            );
        }
    );
}

pub async fn get_erc20_metadata(context: &Context, erc20_id: String) {
    let erc20_id_str = erc20_id.clone();
    handle_near_call!(
        context,
        near::get_erc20_metadata(context, erc20_id).await,
        |meta| {
            output!(
                &context.cli.output_format,
                result_object!("erc20_id" => erc20_id_str, "metadata" => format!("{:?}", meta))
            );
        }
    );
}

pub async fn set_erc20_metadata(
    context: &Context,
    erc20_id: String,
    name: String,
    symbol: String,
    decimals: u8,
) {
    let metadata = Erc20Metadata {
        name,
        symbol,
        decimals,
    };
    handle_near_call!(context,  near::set_erc20_metadata(context, erc20_id, metadata).await, success: "ERC20 metadata set");
}

pub async fn mirror_erc20_token(context: &Context, contract_id: AccountId, nep141: AccountId) {
    handle_near_call!(
        context,
        near::mirror_erc20_token(context, contract_id, nep141).await,
        |addr| {
            output!(
                &context.cli.output_format,
                result_object!("mirrored_erc20_token_address" => format!("{:?}", addr))
            );
        }
    );
}

pub async fn set_eth_connector_contract_account(context: &Context, account_id: AccountId) {
    handle_near_call!(context, near::set_eth_connector_contract_account(context, account_id).await, success: "Eth connector contract account set");
}

pub async fn get_eth_connector_contract_account(context: &Context) {
    handle_near_call!(
        context,
        near::get_eth_connector_contract_account(context).await,
        |account: String| {
            output!(
                &context.cli.output_format,
                result_object!("eth_connector_contract_account" => account.to_string())
            );
        }
    );
}

pub async fn set_eth_connector_contract_data(
    context: &Context,
    prover_id: AccountId,
    custodian_address: Address,
    ft_metadata_path: String,
) {
    handle_near_call!(context, near::set_eth_connector_contract_data(context, prover_id, custodian_address, ft_metadata_path).await, success: "Eth connector contract data set");
}

pub async fn set_paused_flags(context: &Context, mask: u8) {
    handle_near_call!(context, near::set_paused_flags(context, mask).await, success: "Paused flags set");
}

pub async fn get_paused_flags(context: &Context) {
    handle_near_call!(context, near::get_paused_flags(context).await, |flags| {
        output!(
            &context.cli.output_format,
            result_object!("paused_flags" => flags)
        );
    });
}

pub async fn add_relayer(
    context: &Context,
    deposit: NearToken,
    full_access_pub_key: PublicKey,
    function_call_pub_key: PublicKey,
) {
    handle_near_call!(
        context,
        near::add_relayer(context, deposit, full_access_pub_key, function_call_pub_key).await,
        |outcome| {
            output!(
                &context.cli.output_format,
                result_object!("relayer_added" => outcome)
            );
        }
    );
}

pub async fn withdraw_wnear_to_router(context: &Context, address: Address, amount: NearToken) {
    handle_near_call!(
        context,
        near::withdraw_wnear_to_router(context, address, amount).await,
        success: "wNEAR withdrawn to router successfully"
    );
}

pub async fn mirror_erc20_token_callback(
    context: &Context,
    contract_id: AccountId,
    nep141: AccountId,
) {
    handle_near_call!(
        context,
        near::mirror_erc20_token_callback(context, contract_id, nep141).await,
        success: "FT transfer executed successfully"
    );
}

pub async fn get_latest_hashchain(context: &Context) {
    handle_near_call!(
        context,
        near::get_latest_hashchain(context).await,
        |latest_hashchain: Value| {
            output!(
                &context.cli.output_format,
                result_object!("latest_hashchain" => latest_hashchain)
            );
        }
    );
}

pub async fn ft_total_supply(context: &Context) {
    handle_near_call!(
        context,
        near::ft_total_supply(context).await,
        |total_supply| {
            output!(
                &context.cli.output_format,
                result_object!("total_supply" => format!("{:?}", total_supply))
            );
        }
    );
}

pub async fn ft_balance_of(context: &Context, account_id: AccountId) {
    let account_id_str = account_id.to_string();
    handle_near_call!(
        context,
        near::ft_balance_of(context, account_id).await,
        |balance| {
            output!(
                &context.cli.output_format,
                result_object!("account_id" => account_id_str, "balance" => format!("{:?}", balance))
            );
        }
    );
}

pub async fn ft_balance_of_eth(context: &Context, address: Address) {
    handle_near_call!(
        context,
        near::ft_balance_of_eth(context, address).await,
        |balance| {
            output!(
                &context.cli.output_format,
                result_object!("address" => address.encode(), "balance" => format!("{:?}", balance))
            );
        }
    );
}

pub async fn ft_transfer(
    context: &Context,
    receiver_id: AccountId,
    amount: NearToken,
    memo: Option<String>,
) {
    handle_near_call!(
        context,
        near::ft_transfer(
            context,
            receiver_id,
            NEP141Wei::new(amount.as_yoctonear()),
            memo,
        ).await,
        success: "FT transfer executed successfully"
    );
}

pub async fn ft_transfer_call(
    context: &Context,
    receiver_id: AccountId,
    amount: NearToken,
    memo: Option<String>,
    msg: String,
) {
    handle_near_call!(
        context,
        near::ft_transfer_call(
            context,
            receiver_id,
            NEP141Wei::new(amount.as_yoctonear()),
            memo,
            msg,
        ).await,
        success: "FT transfer call executed successfully"
    );
}

pub async fn ft_on_transfer(
    context: &Context,
    sender_id: AccountId,
    amount: NearToken,
    msg: String,
) {
    handle_near_call!(
        context,
        near::ft_on_transfer(context, sender_id, Balance::new(amount.as_yoctonear()), msg).await,
        success: "FT on transfer executed successfully"
    );
}

pub async fn deploy_erc20_token(context: &Context, nep141: AccountId) {
    handle_near_call!(
        context,
        near::deploy_erc20_token(context, nep141).await,
        |address| {
            output!(
                &context.cli.output_format,
                result_object!("deployed_erc20_token_address" => format!("{:?}", address))
            );
        }
    );
}

pub async fn storage_deposit(
    context: &Context,
    account_id: Option<AccountId>,
    registration_only: Option<bool>,
) {
    handle_near_call!(
        context,
        near::storage_deposit(context, account_id, registration_only).await,
        success: "Storage deposit executed successfully"
    );
}

pub async fn storage_unregister(context: &Context, force: bool) {
    handle_near_call!(
        context,
        near::storage_unregister(context, force).await,
        success: "Storage unregistered successfully"
    );
}

pub async fn storage_withdraw(context: &Context, amount: Option<NearToken>) {
    handle_near_call!(
        context,
        near::storage_withdraw(context, amount.map(|n| Yocto::new(n.as_yoctonear()))).await,
        success: "Storage withdrawn successfully"
    );
}

pub async fn storage_balance_of(context: &Context, account_id: AccountId) {
    let account_id_str = account_id.to_string();
    handle_near_call!(
        context,
        near::storage_balance_of(context, account_id).await,
        |balance| {
            output!(
                &context.cli.output_format,
                result_object!("account_id" => account_id_str, "storage_balance" => format!("{:?}", balance))
            );
        }
    );
}
