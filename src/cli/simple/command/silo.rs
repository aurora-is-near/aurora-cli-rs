use aurora_engine_types::borsh::{BorshDeserialize, BorshSerialize};
use aurora_engine_types::parameters::silo::{
    FixedGasCostArgs, SiloParamsArgs, WhitelistAccountArgs, WhitelistAddressArgs, WhitelistArgs,
    WhitelistKind, WhitelistKindArgs, WhitelistStatusArgs,
};
use aurora_engine_types::types::Wei;
use near_primitives::views::CallResult;
use std::fmt::{Display, Formatter};

use super::{get_value, ContractCall};
use crate::cli::command::FromCallResult;
use crate::client::Client;
use crate::contract_call;
use crate::utils::hex_to_address;

/// Return fixed gas cost.
pub async fn get_fixed_gas_cost(client: Client) -> anyhow::Result<()> {
    get_value::<FixedGasCost>(client, "get_fixed_gas_cost", None).await
}

/// Set fixed gas cost.
pub async fn set_fixed_gas_cost(client: Client, cost: u128) -> anyhow::Result<()> {
    let args = FixedGasCostArgs {
        cost: Some(Wei::new_u128(cost)),
    }
    .try_to_vec()?;

    contract_call!(
        "set_fixed_gas_cost",
        "The fixed gas cost: {cost} has been set successfully",
        "Error while setting gas cost"
    )
    .proceed(client, args)
    .await
}

pub async fn set_silo_params(
    client: Client,
    cost: u128,
    rollback_address: String,
) -> anyhow::Result<()> {
    let args = Some(SiloParamsArgs {
        fixed_gas_cost: Wei::new_u128(cost),
        erc20_fallback_address: hex_to_address(&rollback_address)?,
    })
    .try_to_vec()?;

    contract_call!(
        "set_silo_params",
        "The silo parameters have been set successfully",
        "Error while setting silo parameters"
    )
    .proceed(client, args)
    .await
}

/// Get a status of the whitelist.
pub async fn get_whitelist_status(client: Client, kind: String) -> anyhow::Result<()> {
    let args = WhitelistKindArgs {
        kind: get_kind(&kind)?,
    }
    .try_to_vec()?;

    get_value::<WhitelistStatus>(client, "get_whitelist_status", Some(args)).await
}

/// Set a status of the whitelist.
pub async fn set_whitelist_status(client: Client, kind: String, status: u8) -> anyhow::Result<()> {
    let args = WhitelistStatusArgs {
        kind: get_kind(&kind)?,
        active: status > 0,
    }
    .try_to_vec()?;
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
    client: Client,
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
pub async fn add_entry_to_whitelist_batch(client: Client, path: String) -> anyhow::Result<()> {
    let args = std::fs::read_to_string(path)
        .and_then(|string| serde_json::from_str::<Vec<WhitelistArgs>>(&string).map_err(Into::into))
        .and_then(|entries| entries.try_to_vec())?;

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
    client: Client,
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
    .and_then(|list| list.try_to_vec().map_err(Into::into))
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

struct FixedGasCost(FixedGasCostArgs);

impl FromCallResult for FixedGasCost {
    fn from_result(result: CallResult) -> anyhow::Result<Self> {
        let args = FixedGasCostArgs::try_from_slice(&result.result)?;
        Ok(Self(args))
    }
}

impl Display for FixedGasCost {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = self
            .0
            .cost
            .map_or("none".to_string(), |cost| cost.to_string());
        f.write_str(&value)
    }
}
