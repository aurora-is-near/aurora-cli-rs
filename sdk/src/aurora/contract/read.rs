use std::io;

use crate::aurora::{ContractMethod, MethodType};
use aurora_engine_types::{
    H256, U256,
    parameters::{
        connector::{Erc20Identifier, Erc20Metadata, PausedMask},
        engine::{GetStorageAtArgs, StorageBalance, TransactionStatus, ViewCallArgs},
        silo::{SiloParamsArgs, WhitelistKindArgs, WhitelistStatusArgs},
    },
    types::{Address, EthGas, Wei},
};
use near_primitives::types::AccountId;

macro_rules! view_method {
    ($name:ident, $method:literal, $response:ty) => {
        pub struct $name;

        impl ContractMethod for $name {
            type Response = $response;

            fn method_type() -> crate::aurora::MethodType {
                crate::aurora::MethodType::View
            }

            fn method_name(&self) -> &'static str {
                $method
            }
        }
    };
}

view_method!(GetChainId, "get_chain_id", u64);
view_method!(GetOwner, "get_owner", AccountId);
view_method!(GetFixedGas, "get_fixed_gas", Option<EthGas>);
view_method!(GetUpgradeIndex, "get_upgrade_index", u64);
view_method!(GetVersion, "get_version", String);
view_method!(
    GetFallbackAddress,
    "get_erc20_fallback_address",
    Option<Address>
);
view_method!(GetBridgeProver, "get_bridge_prover", String);
view_method!(GetSiloParams, "get_silo_params", Option<SiloParamsArgs>);
view_method!(GetUpgradeDelayBlocks, "get_upgrade_delay_blocks", u64);
view_method!(
    GetEthConnectorContractAccount,
    "get_eth_connector_contract_account",
    String
);
view_method!(GetPausedFlags, "get_paused_flags", PausedMask);
view_method!(PausedPrecompiles, "paused_precompiles", u64);
view_method!(
    GetLatestHashchain,
    "get_latest_hashchain",
    serde_json::Value
);
view_method!(FtTotalSupply, "ft_total_supply", String);

pub struct GetBalance {
    pub address: Address,
}

impl ContractMethod for GetBalance {
    type Response = Wei;

    fn method_name(&self) -> &'static str {
        "get_balance"
    }

    fn method_type() -> crate::aurora::MethodType {
        MethodType::View
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        Ok(self.address.as_bytes().to_vec())
    }
}

pub struct GetNonce {
    pub address: Address,
}

impl ContractMethod for GetNonce {
    type Response = U256;

    fn method_name(&self) -> &'static str {
        "get_nonce"
    }

    fn method_type() -> crate::aurora::MethodType {
        MethodType::View
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        Ok(self.address.as_bytes().to_vec())
    }
}

pub struct GetBlockHash {
    pub height: u64,
}

impl ContractMethod for GetBlockHash {
    type Response = String;

    fn method_name(&self) -> &'static str {
        "get_block_hash"
    }

    fn method_type() -> crate::aurora::MethodType {
        MethodType::View
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        Ok(self.height.to_le_bytes().to_vec())
    }

    fn parse_response(response: Vec<u8>) -> Result<Self::Response, crate::aurora::error::Error> {
        Ok(hex::encode(response))
    }
}

pub struct GetCode {
    pub address: Address,
}

impl ContractMethod for GetCode {
    type Response = Vec<u8>;

    fn method_name(&self) -> &'static str {
        "get_code"
    }

    fn method_type() -> crate::aurora::MethodType {
        MethodType::View
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        Ok(self.address.as_bytes().to_vec())
    }
}

pub struct GetWhitelistStatus {
    pub args: WhitelistKindArgs,
}

impl ContractMethod for GetWhitelistStatus {
    type Response = WhitelistStatusArgs;

    fn method_name(&self) -> &'static str {
        "get_whitelist_status"
    }

    fn method_type() -> crate::aurora::MethodType {
        MethodType::View
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(&self.args).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

pub struct GetErc20FromNep141 {
    pub nep141_account_id: AccountId,
}

impl ContractMethod for GetErc20FromNep141 {
    type Response = String;

    fn method_name(&self) -> &'static str {
        "get_erc20_from_nep141"
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        Ok(self.nep141_account_id.as_bytes().to_vec())
    }

    fn method_type() -> crate::aurora::MethodType {
        MethodType::View
    }

    fn parse_response(response: Vec<u8>) -> Result<Self::Response, crate::aurora::error::Error> {
        Ok(hex::encode(response))
    }
}

pub struct GetNep141FromErc20 {
    pub address: Address,
}

impl ContractMethod for GetNep141FromErc20 {
    type Response = AccountId;

    fn method_name(&self) -> &'static str {
        "get_nep141_from_erc20"
    }

    fn method_type() -> crate::aurora::MethodType {
        MethodType::View
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        Ok(self.address.as_bytes().to_vec())
    }
}

pub struct GetErc20Metadata {
    pub id: Erc20Identifier,
}

impl ContractMethod for GetErc20Metadata {
    type Response = Erc20Metadata;

    fn method_name(&self) -> &'static str {
        "get_erc20_metadata"
    }

    fn method_type() -> crate::aurora::MethodType {
        MethodType::View
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        serde_json::to_vec(&self.id).map_err(|e| io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

pub struct GetStorageAt {
    pub args: Option<GetStorageAtArgs>,
}

impl ContractMethod for GetStorageAt {
    type Response = H256;

    fn method_name(&self) -> &'static str {
        "get_storage_at"
    }

    fn method_type() -> crate::aurora::MethodType {
        MethodType::View
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(&self.args)
    }
}

pub struct FactoryGetWnearAddress;

impl ContractMethod for FactoryGetWnearAddress {
    type Response = String;

    fn method_name(&self) -> &'static str {
        "factory_get_wnear_address"
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        Ok(vec![])
    }

    fn method_type() -> crate::aurora::MethodType {
        MethodType::View
    }

    fn parse_response(response: Vec<u8>) -> Result<Self::Response, crate::aurora::error::Error> {
        Ok(hex::encode(response))
    }
}

pub struct FtBalanceOf {
    pub account_id: AccountId,
}

impl ContractMethod for FtBalanceOf {
    type Response = String;

    fn method_name(&self) -> &'static str {
        "ft_balance_of"
    }

    fn method_type() -> crate::aurora::MethodType {
        MethodType::View
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        Ok(self.account_id.as_bytes().to_vec())
    }

    fn parse_response(response: Vec<u8>) -> Result<Self::Response, crate::aurora::error::Error> {
        String::from_utf8(response).map_err(Into::into)
    }
}

pub struct FtBalanceOfEth {
    pub address: Address,
}

impl ContractMethod for FtBalanceOfEth {
    type Response = Wei;

    fn method_name(&self) -> &'static str {
        "ft_balance_of_eth"
    }

    fn method_type() -> crate::aurora::MethodType {
        MethodType::View
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(&self.address)
    }

    fn parse_response(response: Vec<u8>) -> Result<Self::Response, crate::aurora::error::Error> {
        serde_json::from_slice(&response).map_err(Into::into)
    }
}

pub struct StorageBalanceOf {
    pub account_id: AccountId,
}

impl ContractMethod for StorageBalanceOf {
    type Response = StorageBalance;

    fn method_name(&self) -> &'static str {
        "storage_balance_of"
    }

    fn method_type() -> crate::aurora::MethodType {
        MethodType::View
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        Ok(self.account_id.as_bytes().to_vec())
    }
}

pub struct ViewCall {
    pub args: ViewCallArgs,
}

impl ContractMethod for ViewCall {
    type Response = TransactionStatus;

    fn method_name(&self) -> &'static str {
        "view"
    }

    fn method_type() -> crate::aurora::MethodType {
        MethodType::View
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(&self.args)
    }
}
