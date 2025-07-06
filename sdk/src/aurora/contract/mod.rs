use std::io;

use aurora_engine_types::{
    H256, U256,
    parameters::{
        connector::{Erc20Metadata, PausedMask},
        engine::{StorageBalance, SubmitResult, TransactionStatus},
        silo::{SiloParamsArgs, WhitelistStatusArgs},
    },
    types::{Address, EthGas, Wei},
};
use borsh::BorshDeserialize;
use near_primitives::types::AccountId;

use super::{ContractMethodResponse, error::Error};

pub mod read;
pub mod write;

#[cfg(test)]
mod test_macro;

impl ContractMethodResponse for () {
    fn parse(_rsp: Vec<u8>) -> Result<Self, Error> {
        Ok(())
    }
}

impl ContractMethodResponse for AccountId {
    fn parse(rsp: Vec<u8>) -> Result<Self, Error> {
        Self::try_from_slice(&rsp).map_err(Into::into)
    }
}

impl ContractMethodResponse for Address {
    fn parse(rsp: Vec<u8>) -> Result<Self, Error> {
        Self::try_from_slice(&rsp).map_err(Into::into)
    }
}

impl ContractMethodResponse for Option<EthGas> {
    fn parse(rsp: Vec<u8>) -> Result<Self, Error> {
        borsh::from_slice(&rsp).map_err(Into::into)
    }
}

impl ContractMethodResponse for Option<Address> {
    fn parse(rsp: Vec<u8>) -> Result<Self, Error> {
        borsh::from_slice(&rsp).map_err(Into::into)
    }
}

impl ContractMethodResponse for u128 {
    fn parse(rsp: Vec<u8>) -> Result<Self, Error> {
        borsh::from_slice(&rsp).map_err(Into::into)
    }
}

impl ContractMethodResponse for u64 {
    fn parse(rsp: Vec<u8>) -> Result<Self, Error> {
        let mut buffer = [0; 8];
        buffer.copy_from_slice(&rsp);
        Ok(Self::from_le_bytes(buffer))
    }
}

impl ContractMethodResponse for u32 {
    fn parse(rsp: Vec<u8>) -> Result<Self, Error> {
        let mut buffer = [0; 4];
        buffer.copy_from_slice(&rsp);
        Ok(Self::from_le_bytes(buffer))
    }
}

impl ContractMethodResponse for SubmitResult {
    fn parse(rsp: Vec<u8>) -> Result<Self, Error> {
        Self::try_from_slice(&rsp).map_err(Into::into)
    }
}

impl ContractMethodResponse for Wei {
    fn parse(value: Vec<u8>) -> Result<Self, Error> {
        Self::from_eth(U256::from_big_endian(&value)).ok_or_else(|| {
            {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Failed to convert bytes to WeiU256",
                )
            }
            .into()
        })
    }
}

impl ContractMethodResponse for Option<SiloParamsArgs> {
    fn parse(value: Vec<u8>) -> Result<Self, Error> {
        borsh::from_slice(&value).map_err(Into::into)
    }
}

impl ContractMethodResponse for WhitelistStatusArgs {
    fn parse(value: Vec<u8>) -> Result<Self, Error> {
        borsh::from_slice(&value).map_err(Into::into)
    }
}

impl ContractMethodResponse for Erc20Metadata {
    fn parse(value: Vec<u8>) -> Result<Self, Error> {
        serde_json::from_slice(&value)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e).into())
    }
}

impl ContractMethodResponse for PausedMask {
    fn parse(value: Vec<u8>) -> Result<Self, Error> {
        Self::try_from_slice(value.as_slice()).map_err(Into::into)
    }
}

impl ContractMethodResponse for H256 {
    fn parse(value: Vec<u8>) -> Result<Self, Error> {
        Ok(Self::from_slice(&value))
    }
}

impl ContractMethodResponse for serde_json::Value {
    fn parse(value: Vec<u8>) -> Result<Self, Error> {
        serde_json::from_slice(&value)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e).into())
    }
}

impl ContractMethodResponse for StorageBalance {
    fn parse(value: Vec<u8>) -> Result<Self, Error> {
        serde_json::from_slice(&value).map_err(Into::into)
    }
}

impl ContractMethodResponse for TransactionStatus {
    fn parse(value: Vec<u8>) -> Result<Self, super::error::Error> {
        Self::try_from_slice(&value).map_err(Into::into)
    }
}
