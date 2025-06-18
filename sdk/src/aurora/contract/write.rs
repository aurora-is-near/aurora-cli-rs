use std::io;

use aurora_engine_types::{
    parameters::{
        connector::{
            MirrorErc20TokenArgs, SetErc20MetadataArgs, SetEthConnectorContractAccountArgs,
        },
        engine::DeployErc20TokenArgs,
        silo::{FixedGasArgs, SiloParamsArgs, WhitelistArgs, WhitelistStatusArgs},
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
