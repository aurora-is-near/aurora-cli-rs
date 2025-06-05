use std::io;

use aurora_engine_types::{
    parameters::{
        connector::{MirrorErc20TokenArgs, SetEthConnectorContractAccountArgs},
        engine::DeployErc20TokenArgs,
        silo::{FixedGasArgs, SiloParamsArgs, WhitelistArgs, WhitelistStatusArgs},
    },
    types::Address,
};

use crate::aurora::{ContractMethod, error::Error};

pub struct SetEthConnectorContractAccount {
    pub args: SetEthConnectorContractAccountArgs,
}

impl ContractMethod for SetEthConnectorContractAccount {
    type Response = ();

    fn method_name(&self) -> &'static str {
        "set_eth_connector_contract_account"
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(&self.args)
    }
}

pub struct MirrorErc20Token {
    pub args: MirrorErc20TokenArgs,
}

impl ContractMethod for MirrorErc20Token {
    type Response = Address;

    fn method_name(&self) -> &'static str {
        "mirror_erc20_token"
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(&self.args)
    }
}

pub struct FactoryUpdate {
    pub wasm: Vec<u8>,
}

impl ContractMethod for FactoryUpdate {
    type Response = ();

    fn method_name(&self) -> &'static str {
        "factory_update"
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        Ok(self.wasm.clone())
    }
}

pub struct SetSiloParams {
    pub args: Option<SiloParamsArgs>,
}

impl ContractMethod for SetSiloParams {
    type Response = ();

    fn method_name(&self) -> &'static str {
        "set_silo_params"
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(&self.args)
    }
}

pub struct SetFixedGas {
    pub args: FixedGasArgs,
}

impl ContractMethod for SetFixedGas {
    type Response = ();

    fn method_name(&self) -> &'static str {
        "set_fixed_gas"
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(&self.args)
    }
}

// Temporarily until engine 4.0.0 release
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

pub struct AddEntryToWhitelist {
    pub args: WhitelistArgs,
}

impl ContractMethod for AddEntryToWhitelist {
    type Response = ();

    fn method_name(&self) -> &'static str {
        "add_entry_to_whitelist"
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(&self.args)
    }
}

pub struct RemoveEntryFromWhitelist {
    pub args: WhitelistArgs,
}

impl ContractMethod for RemoveEntryFromWhitelist {
    type Response = ();

    fn method_name(&self) -> &'static str {
        "remove_entry_from_whitelist"
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(&self.args)
    }
}

pub struct SetWhitelistStatus {
    pub args: WhitelistStatusArgs,
}

impl ContractMethod for SetWhitelistStatus {
    type Response = ();

    fn method_name(&self) -> &'static str {
        "set_whitelist_status"
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(&self.args)
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

    fn parse_response(response: Vec<u8>) -> Result<Self::Response, Error> {
        borsh::from_slice::<Vec<u8>>(&response)
            .and_then(|addr_bytes| {
                Self::Response::try_from_slice(&addr_bytes)
                    .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))
            })
            .map_err(Into::into)
    }
}
