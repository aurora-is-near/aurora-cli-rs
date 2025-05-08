use near_primitives::hash::CryptoHash;
use near_primitives::types::AccountId;

pub enum GlobalContractDeployMode {
    CodeHash,
    AccountId,
}

impl From<GlobalContractDeployMode> for near_primitives::action::GlobalContractDeployMode {
    fn from(mode: GlobalContractDeployMode) -> Self {
        match mode {
            GlobalContractDeployMode::CodeHash => Self::CodeHash,
            GlobalContractDeployMode::AccountId => Self::AccountId,
        }
    }
}

pub enum GlobalContractIdentifier {
    AccountId(AccountId),
    CodeHash(CryptoHash),
}

impl From<GlobalContractIdentifier> for near_primitives::action::GlobalContractIdentifier {
    fn from(value: GlobalContractIdentifier) -> Self {
        match value {
            GlobalContractIdentifier::AccountId(account_id) => Self::AccountId(account_id),
            GlobalContractIdentifier::CodeHash(code_hash) => Self::CodeHash(code_hash),
        }
    }
}
