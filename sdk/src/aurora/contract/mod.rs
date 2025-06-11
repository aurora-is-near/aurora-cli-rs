use aurora_engine_types::types::{Address, EthGas};
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
        borsh::from_slice(&rsp).map_err(Into::into)
    }
}

impl ContractMethodResponse for u32 {
    fn parse(rsp: Vec<u8>) -> Result<Self, Error> {
        borsh::from_slice(&rsp).map_err(Into::into)
    }
}
