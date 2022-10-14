use aurora_engine::fungible_token::FungibleReferenceHash;
use aurora_engine_types::types::Address;
use borsh::{BorshSerialize, BorshDeserialize};


#[derive(BorshSerialize)]
pub struct GetStorageAtInput {
    pub address: Address,
    pub key: Vec<u8>,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, PartialEq, Eq)]
pub struct FungibleTokenMetadata {
    pub spec: String,
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub reference: Option<String>,
    pub reference_hash: Option<FungibleReferenceHash>,
    pub decimals: u8,
}