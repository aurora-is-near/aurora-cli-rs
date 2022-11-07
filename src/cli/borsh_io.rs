use aurora_engine::{fungible_token::FungibleReferenceHash, admin_controlled::PausedMask};
use aurora_engine_types::types::{Address, RawU256, NEP141Wei};
use borsh::{BorshSerialize, BorshDeserialize};
use near_primitives::types::{AccountId, Balance};
use serde::{Serialize, Deserialize};


#[derive(BorshSerialize)]
pub struct GetStorageAtInput {
    pub address: Address,
    pub key: Vec<u8>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct AccountBalance {
    pub address: Address,
    pub balance: RawU256,
}

#[derive(Serialize, Deserialize)]
pub struct AccountBalanceSerde {
    pub address: Address,
    pub balance: RawU256,
}

/// Borsh-encoded parameters for the `begin_chain` function.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct BeginChainArgs {
    pub chain_id: RawU256,
    pub genesis_alloc: Vec<AccountBalance>,
}

/// Borsh-encoded parameters for the `begin_block` function.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct BeginBlockArgs {
    /// The current block's hash (for replayer use).
    pub hash: RawU256,
    /// The current block's beneficiary address.
    pub coinbase: Address,
    /// The current block's timestamp (in seconds since the Unix epoch).
    pub timestamp: RawU256,
    /// The current block's number (the genesis block is number zero).
    pub number: RawU256,
    /// The current block's difficulty.
    pub difficulty: RawU256,
    /// The current block's gas limit.
    pub gaslimit: RawU256,
}

/// Borsh-encoded parameters for `deploy_erc20_token` function.
#[derive(BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq, Clone)]
pub struct DeployErc20TokenArgs {
    pub nep141: AccountId,
}

/// withdraw NEAR eth-connector call args
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct WithdrawCallArgs {
    pub recipient_address: Address,
    pub amount: NEP141Wei,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct PauseEthConnectorCallArgs {
    pub paused_mask: PausedMask,
}

/// Borsh-encoded parameters for the `ft_transfer_call` function
/// for regular NEP-141 tokens.
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct NEP141FtOnTransferArgs {
    pub sender_id: AccountId,
    /// Balance can be for Eth on Near and for Eth to Aurora
    /// `ft_on_transfer` can be called with arbitrary NEP-141 tokens attached, therefore we do not specify a particular type Wei.
    pub amount: Balance,
    pub msg: String,
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