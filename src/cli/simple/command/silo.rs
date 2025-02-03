use aurora_engine_types::borsh::BorshDeserialize;
use aurora_engine_types::parameters::silo::{
    FixedGasArgs, SiloParamsArgs, WhitelistAccountArgs, WhitelistAddressArgs, WhitelistArgs,
    WhitelistKind, WhitelistKindArgs, WhitelistStatusArgs,
};
use aurora_engine_types::types::EthGas;
use near_primitives::views::CallResult;
use std::fmt::{Display, Formatter};

use super::{get_value, ContractCall};
use crate::cli::command::FromCallResult;
use crate::client::Context;
use crate::contract_call;
use crate::utils::hex_to_address;

/// Return fixed gas cost.
pub async fn get_fixed_gas_cost(client: Context) -> anyhow::Result<()> {
    get_value::<FixedGas>(client, "get_fixed_gas", None).await
}

/// Set fixed gas cost.
pub async fn set_fixed_gas(client: Context, cost: u64) -> anyhow::Result<()> {
    let args = borsh::to_vec(&FixedGasArgs {
        fixed_gas: Some(EthGas::new(cost)),
    })?;

    contract_call!(
        "set_fixed_gas",
        "The fixed gas: {cost} has been set successfully",
        "Error while setting fixed gas"
    )
    .proceed(client, args)
    .await
}

/// Return Silo parameters.
pub async fn get_silo_params(client: Context) -> anyhow::Result<()> {
    get_value::<SiloParams>(client, "get_silo_params", None).await
}

pub async fn set_silo_params(
    client: Context,
    gas: u64,
    fallback_address: String,
) -> anyhow::Result<()> {
    let args = borsh::to_vec(&Some(SiloParamsArgs {
        fixed_gas: EthGas::new(gas),
        erc20_fallback_address: hex_to_address(&fallback_address)?,
    }))?;

    contract_call!(
        "set_silo_params",
        "The silo parameters have been set successfully",
        "Error while setting silo parameters"
    )
    .proceed(client, args)
    .await
}

/// Turn off silo mode.
pub async fn disable_silo_mode(client: Context) -> anyhow::Result<()> {
    let args = borsh::to_vec(&None::<SiloParamsArgs>)?;

    contract_call!(
        "set_silo_params",
        "The silo mode has been disabled successfully",
        "Error while disabling silo mode"
    )
    .proceed(client, args)
    .await
}

/// Get a status of the whitelist.
pub async fn get_whitelist_status(client: Context, kind: String) -> anyhow::Result<()> {
    let args = borsh::to_vec(&WhitelistKindArgs {
        kind: get_kind(&kind)?,
    })?;

    get_value::<WhitelistStatus>(client, "get_whitelist_status", Some(args)).await
}

/// Set a status of the whitelist.
pub async fn set_whitelist_status(client: Context, kind: String, status: u8) -> anyhow::Result<()> {
    let args = borsh::to_vec(&WhitelistStatusArgs {
        kind: get_kind(&kind)?,
        active: status > 0,
    })?;
    let str_status = if status == 0 { "disabled" } else { "enabled" };

    contract_call!(
        "set_whitelist_status",
        "The whitelist has been {str_status} successfully",
        "Error while setting whitelist status"
    )
    .proceed(client, args)
    .await
}

/// Add an entry to the whitelist.
pub async fn add_entry_to_whitelist(
    client: Context,
    kind: String,
    entry: String,
) -> anyhow::Result<()> {
    let args = get_whitelist_args(&kind, &entry)?;

    contract_call!(
        "add_entry_to_whitelist",
        "The entry: {entry} has been added to the whitelist successfully",
        "Error while adding entry to whitelist"
    )
    .proceed(client, args)
    .await
}

/// Add a batch of entries to the whitelist.
pub async fn add_entry_to_whitelist_batch(client: Context, path: String) -> anyhow::Result<()> {
    let args = std::fs::read_to_string(path)
        .and_then(|string| serde_json::from_str::<Vec<WhitelistArgs>>(&string).map_err(Into::into))
        .and_then(|entries| borsh::to_vec(&entries))?;

    contract_call!(
        "add_entry_to_whitelist_batch",
        "The batch of entries has been added to the whitelist successfully",
        "Error while setting batch entry to whitelist"
    )
    .proceed(client, args)
    .await
}

/// Remove an entry from the whitelist.
pub async fn remove_entry_from_whitelist(
    client: Context,
    kind: String,
    entry: String,
) -> anyhow::Result<()> {
    let args = get_whitelist_args(&kind, &entry)?;

    contract_call!(
        "remove_entry_from_whitelist",
        "The entry: {entry} has been removed from the whitelist successfully",
        "Error while removing entry to whitelist"
    )
    .proceed(client, args)
    .await
}

fn get_kind(kind: &str) -> anyhow::Result<WhitelistKind> {
    Ok(match kind {
        "admin" => WhitelistKind::Admin,
        "evm-admin" => WhitelistKind::EvmAdmin,
        "account" => WhitelistKind::Account,
        "address" => WhitelistKind::Address,
        _ => anyhow::bail!("Wrong whitelist kind: {kind}"),
    })
}

fn get_whitelist_args(kind: &str, entry: &str) -> anyhow::Result<Vec<u8>> {
    let kind = get_kind(kind)?;

    Ok(match kind {
        WhitelistKind::Admin | WhitelistKind::Account => {
            WhitelistArgs::WhitelistAccountArgs(WhitelistAccountArgs {
                kind,
                account_id: entry.parse().map_err(|e| anyhow::anyhow!("{e}"))?,
            })
        }
        WhitelistKind::EvmAdmin | WhitelistKind::Address => {
            WhitelistArgs::WhitelistAddressArgs(WhitelistAddressArgs {
                kind,
                address: hex_to_address(entry)?,
            })
        }
    })
    .and_then(|list| borsh::to_vec(&list).map_err(Into::into))
}

struct WhitelistStatus(WhitelistStatusArgs);

impl FromCallResult for WhitelistStatus {
    fn from_result(result: CallResult) -> anyhow::Result<Self> {
        let args = WhitelistStatusArgs::try_from_slice(&result.result)?;
        Ok(Self(args))
    }
}

impl Display for WhitelistStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = if self.0.active { "1" } else { "0" };
        f.write_str(value)
    }
}

struct FixedGas(FixedGasArgs);

impl FromCallResult for FixedGas {
    fn from_result(result: CallResult) -> anyhow::Result<Self> {
        let args = FixedGasArgs::try_from_slice(&result.result)?;
        Ok(Self(args))
    }
}

impl Display for FixedGas {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = self
            .0
            .fixed_gas
            .map_or("none".to_string(), |cost| cost.to_string());
        f.write_str(&value)
    }
}

struct SiloParams(Option<SiloParamsArgs>);

impl FromCallResult for SiloParams {
    fn from_result(result: CallResult) -> anyhow::Result<Self> {
        let args = Option::<SiloParamsArgs>::try_from_slice(&result.result)?;
        Ok(Self(args))
    }
}

impl Display for SiloParams {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(params) = &self.0 {
            let gas = params.fixed_gas;
            let fallback_address = params.erc20_fallback_address.encode();

            f.write_fmt(format_args!(
                "FixedGas: {gas}, fallback address: 0x{fallback_address}"
            ))
        } else {
            f.write_str("Silo mode is disabled")
        }
    }
}
