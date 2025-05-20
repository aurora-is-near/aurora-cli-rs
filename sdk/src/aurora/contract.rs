use super::{ContractMethod, ContractMethodResponse};

use aurora_engine_types::{
    parameters::{
        connector::{MirrorErc20TokenArgs, SetEthConnectorContractAccountArgs},
        silo::{SiloParamsArgs, WhitelistArgs, WhitelistStatusArgs},
    },
    types::{Address, EthGas},
};
use borsh::BorshDeserialize;
use near_primitives::types::AccountId;

impl ContractMethodResponse for () {
    fn parse(_rsp: Vec<u8>) -> Result<Self, super::error::Error> {
        Ok(())
    }
}

pub struct GetOwner;

impl ContractMethod for GetOwner {
    type Response = AccountId;

    fn method_type() -> super::MethodType {
        super::MethodType::View
    }

    fn method_name(&self) -> &'static str {
        "get_owner"
    }
}

impl ContractMethodResponse for AccountId {
    fn parse(rsp: Vec<u8>) -> Result<Self, super::error::Error> {
        Self::try_from_slice(&rsp).map_err(Into::into)
    }
}

pub struct SetEthConnectorContractAccount {
    args: SetEthConnectorContractAccountArgs,
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

impl ContractMethodResponse for Address {
    fn parse(rsp: Vec<u8>) -> Result<Self, super::error::Error> {
        Self::try_from_slice(&rsp).map_err(Into::into)
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
    pub args: SiloParamsArgs,
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

pub struct GetFixedGas;

impl ContractMethod for GetFixedGas {
    type Response = Option<EthGas>;

    fn method_type() -> super::MethodType {
        super::MethodType::View
    }

    fn method_name(&self) -> &'static str {
        "get_fixed_gas"
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        Ok(Vec::new())
    }
}

impl ContractMethodResponse for Option<EthGas> {
    fn parse(rsp: Vec<u8>) -> Result<Self, super::error::Error> {
        borsh::from_slice(&rsp).map_err(Into::into)
    }
}

pub struct GetFallbackAddress;

impl ContractMethod for GetFallbackAddress {
    type Response = Option<Address>;

    fn method_type() -> super::MethodType {
        super::MethodType::View
    }

    fn method_name(&self) -> &'static str {
        "get_fallback_address"
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        Ok(Vec::new())
    }
}

impl ContractMethodResponse for Option<Address> {
    fn parse(rsp: Vec<u8>) -> Result<Self, super::error::Error> {
        borsh::from_slice(&rsp).map_err(Into::into)
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
