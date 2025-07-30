use std::io;

use aurora_engine_transactions::EthTransactionKind;
use aurora_engine_types::{
    parameters::{
        ExitToNearPrecompileCallbackCallArgs,
        connector::{
            InitCallArgs, MirrorErc20TokenArgs, NEP141FtOnTransferArgs, PauseEthConnectorCallArgs,
            SetErc20MetadataArgs, SetEthConnectorContractAccountArgs, StorageDepositCallArgs,
            StorageWithdrawCallArgs, TransferCallArgs, TransferCallCallArgs,
        },
        engine::{
            CallArgs, DeployErc20TokenArgs, LegacyNewCallArgs, PausePrecompilesCallArgs,
            RelayerKeyArgs, RelayerKeyManagerArgs, SetOwnerArgs, SetUpgradeDelayBlocksArgs,
            StartHashchainArgs, StorageUnregisterArgs, SubmitResult,
        },
        silo::{FixedGasArgs, SiloParamsArgs, WhitelistArgs, WhitelistStatusArgs},
        xcc::{AddressVersionUpdateArgs, FundXccArgs, WithdrawWnearToRouterArgs},
    },
    types::Address,
};

use crate::ContractMethod as ContractMethodDerive;
use crate::aurora::{ContractMethod, error::Error};

#[derive(ContractMethodDerive)]
#[contract_method(method = "set_eth_connector_contract_account", response = ())]
pub struct SetEthConnectorContractAccount {
    #[contract_param(serialize_as = "borsh")]
    pub args: SetEthConnectorContractAccountArgs,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "mirror_erc20_token", response = Address, deserialize_as = "borsh")]
pub struct MirrorErc20Token {
    #[contract_param(serialize_as = "borsh")]
    pub args: MirrorErc20TokenArgs,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "factory_update", response = ())]
pub struct FactoryUpdate {
    #[contract_param]
    pub wasm: Vec<u8>,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "set_silo_params", response = ())]
pub struct SetSiloParams {
    #[contract_param(serialize_as = "borsh")]
    pub args: Option<SiloParamsArgs>,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "set_fixed_gas", response = ())]
pub struct SetFixedGas {
    #[contract_param(serialize_as = "borsh")]
    pub args: FixedGasArgs,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "set_key_manager", response = ())]
pub struct SetKeyManager {
    #[contract_param(serialize_as = "json")]
    pub args: RelayerKeyManagerArgs,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "add_relayer_key", response = ())]
pub struct AddRelayerKey {
    #[contract_param(serialize_as = "json")]
    pub args: RelayerKeyArgs,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "remove_relayer_key", response = ())]
pub struct RemoveRelayerKey {
    #[contract_param(serialize_as = "json")]
    pub args: RelayerKeyArgs,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "set_upgrade_delay_blocks", response = ())]
pub struct SetUpgradeDelayBlocks {
    #[contract_param(serialize_as = "borsh")]
    pub args: SetUpgradeDelayBlocksArgs,
}

// Temporarily until engine 4.0.0 release
// This structure serializes itself rather than a separate field, so we keep manual implementation
#[derive(Debug, borsh::BorshDeserialize, borsh::BorshSerialize)]
pub struct Erc20FallbackAddressArgs {
    pub address: Option<Address>,
}

impl ContractMethod for Erc20FallbackAddressArgs {
    type Response = ();

    fn method_name(&self) -> &'static str {
        "set_erc20_fallback_address"
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(&self)
    }
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "add_entry_to_whitelist", response = ())]
pub struct AddEntryToWhitelist {
    #[contract_param(serialize_as = "borsh")]
    pub args: WhitelistArgs,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "remove_entry_from_whitelist", response = ())]
pub struct RemoveEntryFromWhitelist {
    #[contract_param(serialize_as = "borsh")]
    pub args: WhitelistArgs,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "set_whitelist_status", response = ())]
pub struct SetWhitelistStatus {
    #[contract_param(serialize_as = "borsh")]
    pub args: WhitelistStatusArgs,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "set_owner", response = ())]
pub struct SetOwner {
    #[contract_param(serialize_as = "borsh")]
    pub args: SetOwnerArgs,
}

pub struct Submit {
    pub transaction: EthTransactionKind,
}

impl ContractMethod for Submit {
    type Response = SubmitResult;

    fn method_name(&self) -> &'static str {
        "submit"
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        Ok((&self.transaction).into())
    }
}

pub struct DeployERC20 {
    pub args: DeployErc20TokenArgs,
}

impl ContractMethod for DeployERC20 {
    type Response = Address;

    fn method_name(&self) -> &'static str {
        "deploy_erc20_token"
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(&self.args)
    }

    fn parse_response(response: Vec<u8>) -> Result<Address, Error> {
        borsh::from_slice::<Vec<u8>>(&response)
            .and_then(|addr_bytes| {
                Self::Response::try_from_slice(&addr_bytes)
                    .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))
            })
            .map_err(Into::into)
    }
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "set_erc20_metadata", response = ())]
pub struct SetERC20Metadata {
    #[contract_param(serialize_as = "json")]
    pub args: SetErc20MetadataArgs,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "new", response = ())]
pub struct New {
    #[contract_param(serialize_as = "borsh")]
    pub args: LegacyNewCallArgs,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "set_eth_connector_contract_data", response = ())]
pub struct SetEthConnectorContractData {
    #[contract_param(serialize_as = "borsh")]
    pub args: InitCallArgs,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "set_paused_flags", response = ())]
pub struct SetPausedFlags {
    #[contract_param(serialize_as = "borsh")]
    pub args: PauseEthConnectorCallArgs,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "register_relayer", response = ())]
pub struct RegisterRelayer {
    #[contract_param(serialize_as = "borsh")]
    pub address: Address,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "start_hashchain", response = ())]
pub struct StartHashchain {
    #[contract_param(serialize_as = "borsh")]
    pub args: StartHashchainArgs,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "pause_contract", response = ())]
pub struct PauseContract;

#[derive(ContractMethodDerive)]
#[contract_method(method = "resume_contract", response = ())]
pub struct ResumeContract;

#[derive(ContractMethodDerive)]
#[contract_method(method = "pause_precompiles", response = ())]
pub struct PausePrecompiles {
    #[contract_param(serialize_as = "borsh")]
    pub args: PausePrecompilesCallArgs,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "resume_precompiles", response = ())]
pub struct ResumePrecompiles {
    #[contract_param(serialize_as = "borsh")]
    pub args: PausePrecompilesCallArgs,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "factory_set_wnear_address", response = ())]
pub struct FactorySetWnearAddress {
    #[contract_param(serialize_as = "borsh")]
    pub address: Address,
}

pub struct FundXccSubAccount {
    pub args: FundXccArgs,
}

impl ContractMethod for FundXccSubAccount {
    type Response = ();

    fn method_name(&self) -> &'static str {
        "fund_xcc_sub_account"
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(&self.args)
    }
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "upgrade", response = ())]
pub struct Upgrade {
    #[contract_param(serialize_as = "raw")]
    pub code: Vec<u8>,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "stage_upgrade", response = ())]
pub struct StageUpgrade {
    #[contract_param(serialize_as = "raw")]
    pub code: Vec<u8>,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "deploy_upgrade", response = ())]
pub struct DeployUpgrade;

#[derive(ContractMethodDerive)]
#[contract_method(method = "factory_update_address_version", response = ())]
pub struct FactoryUpdateAddressVersion {
    #[contract_param(serialize_as = "borsh")]
    pub args: AddressVersionUpdateArgs,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "withdraw_wnear_to_router", response = SubmitResult)]
pub struct WithdrawWnearToRouter {
    #[contract_param(serialize_as = "borsh")]
    pub args: WithdrawWnearToRouterArgs,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "mirror_erc20_token_callback", response = ())]
pub struct MirrorErc20TokenCallback {
    #[contract_param(serialize_as = "borsh")]
    pub args: MirrorErc20TokenArgs,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "ft_transfer", response = ())]
pub struct FtTransfer {
    #[contract_param(serialize_as = "json")]
    pub args: TransferCallArgs,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "ft_transfer_call", response = ())]
pub struct FtTransferCall {
    #[contract_param(serialize_as = "json")]
    pub args: TransferCallCallArgs,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "ft_on_transfer", response = ())]
pub struct FtOnTransfer {
    #[contract_param(serialize_as = "json")]
    pub args: NEP141FtOnTransferArgs,
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "exit_to_near_precompile_callback", response = ())]
pub struct ExitToNearPrecompileCallback {
    #[contract_param(serialize_as = "borsh")]
    pub args: ExitToNearPrecompileCallbackCallArgs,
}

pub struct StorageDeposit {
    pub args: StorageDepositCallArgs,
}

impl ContractMethod for StorageDeposit {
    type Response = ();

    fn deposit(&self) -> u128 {
        1 // 1 yocto
    }

    fn method_name(&self) -> &'static str {
        "storage_deposit"
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        serde_json::to_vec(&self.args).map_err(Into::into)
    }
}

pub struct StorageUnregister {
    pub args: StorageUnregisterArgs,
}

impl ContractMethod for StorageUnregister {
    type Response = ();

    fn deposit(&self) -> u128 {
        1 // 1 yocto
    }

    fn method_name(&self) -> &'static str {
        "storage_unregister"
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        serde_json::to_vec(&self.args).map_err(Into::into)
    }
}

pub struct StorageWithdraw {
    pub args: StorageWithdrawCallArgs,
}

impl ContractMethod for StorageWithdraw {
    type Response = ();

    fn method_name(&self) -> &'static str {
        "storage_withdraw"
    }

    fn deposit(&self) -> u128 {
        1 // 1 yocto
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        serde_json::to_vec(&self.args).map_err(Into::into)
    }
}

#[derive(ContractMethodDerive)]
#[contract_method(method = "call", response = ())]
pub struct Call {
    #[contract_param(serialize_as = "borsh")]
    pub args: CallArgs,
}

#[cfg(test)]
mod tests {
    use crate::aurora::{ContractMethod, common, contract::write::DeployERC20};

    #[test]
    fn test_erc20_deploy_success() -> anyhow::Result<()> {
        let addr = common::hex_to_address("0xdAC17F958D2ee523a2206206994597C13D831ec7")
            .map_err(|_| anyhow::anyhow!("Failed to decode address"))?;
        let borsh_addr_bytes = borsh::to_vec(&addr.as_bytes())?;

        assert_eq!(addr, DeployERC20::parse_response(borsh_addr_bytes)?);
        Ok(())
    }
}
