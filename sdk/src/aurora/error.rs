use std::io;

use aurora_engine_types::types::address::error::AddressError;
use near_primitives::errors::TxExecutionError;
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Execution is not started")]
    ExecutionNotStarted,
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Execution(#[from] TxExecutionError),
    #[error(transparent)]
    Silo(#[from] SiloError),
    #[error(transparent)]
    Near(#[from] crate::near::error::Error),
}

impl From<AddressError> for Error {
    fn from(e: AddressError) -> Self {
        match e {
            AddressError::FailedDecodeHex => {
                io::Error::new(io::ErrorKind::InvalidData, "Failed to decode hex")
            }
            AddressError::IncorrectLength => {
                io::Error::new(io::ErrorKind::InvalidData, "Incorrect length")
            }
        }
        .into()
    }
}

#[derive(Debug, thiserror::Error, Deserialize)]
pub enum SiloError {
    #[error("ERR_CALL_TOO_DEEP")]
    CallTooDeep,
    #[error("ERR_OUT_OF_FUNDS")]
    OutOfFunds,
    #[error("ERR_OUT_OF_GAS")]
    OutOfGas,
    #[error("ERR_OUT_OF_OFFSET")]
    OutOfOffset,
    #[error("ERR_REVERT")]
    Revert,
    #[error("ERR_NOT_A_JSON_TYPE")]
    NotAJsonType,
    #[error("ERR_JSON_MISSING_VALUE")]
    JsonMissingValue,
    #[error("ERR_FAILED_PARSE_U8")]
    FailedParseU8,
    #[error("ERR_FAILED_PARSE_U64")]
    FailedParseU64,
    #[error("ERR_FAILED_PARSE_U128")]
    FailedParseU128,
    #[error("ERR_FAILED_PARSE_BOOL")]
    FailedParseBool,
    #[error("ERR_FAILED_PARSE_STRING")]
    FailedParseString,
    #[error("ERR_FAILED_PARSE_ARRAY")]
    FailedParseArray,
    #[error("ERR_EXPECTED_STRING_GOT_NUMBER")]
    ExpectedStringGotNumber,
    #[error("ERR_OUT_OF_RANGE_U8")]
    OutOfRangeU8,
    #[error("ERR_OUT_OF_RANGE_U128")]
    OutOfRangeU128,
    #[error("ERR_PROMISE_COUNT")]
    PromiseCount,
    #[error("ERR_REFUND_FAILURE")]
    RefundFailure,
    #[error("ERR_NOT_ALLOWED:TOO_EARLY")]
    NotAllowedTooEarly,
    #[error("ERR_PROMISE_FAILED")]
    PromiseFailed,
    #[error("ERR_VERIFY_PROOF")]
    VerifyProof,
    #[error("ERR_INVALID_UPGRADE")]
    InvalidUpgrade,
    #[error("ERR_NO_UPGRADE")]
    NoUpgrade,
    #[error("ERR_NOT_ALLOWED")]
    NotAllowed,
    #[error("ERR_NOT_OWNER")]
    NotOwner,
    #[error("ERR_PAUSED")]
    Paused,
    #[error("ERR_FT_PAUSED")]
    FtPaused,
    #[error("ERR_RUNNING")]
    Running,
    #[error("ERR_SERIALIZE")]
    Serialize,
    #[error("ERR_PROMISE_ENCODING")]
    PromiseEncoding,
    #[error("ERR_ARGS")]
    Args,
    #[error("ERR_VALUE_CONVERSION")]
    ValueConversion,
    #[error("ERR_BORSH_DESERIALIZE")]
    BorShDeserialize,
    #[error("ERR_JSON_DESERIALIZE")]
    JsonDeserialize,
    #[error("ERR_META_TX_PARSE")]
    MetaTxParse,
    #[error("ERR_STACK_UNDERFLOW")]
    StackUnderflow,
    #[error("ERR_STACK_OVERFLOW")]
    StackOverflow,
    #[error("ERR_INVALID_JUMP")]
    InvalidJump,
    #[error("ERR_INVALID_RANGE")]
    InvalidRange,
    #[error("ERR_DESIGNATED_INVALID")]
    DesignatedInvalid,
    #[error("ERR_CREATE_COLLISION")]
    CreateCollision,
    #[error("ERR_CREATE_CONTRACT_LIMIT")]
    CreateContractLimit,
    #[error("ERR_INVALID_OPCODE")]
    InvalidOpcode,
    #[error("ERR_OUT_OF_FUND")]
    OutOfFund,
    #[error("ERR_CREATE_EMPTY")]
    CreateEmpty,
    #[error("ERR_MAX_NONCE")]
    MaxNonce,
    #[error("ERR_NOT_SUPPORTED")]
    NotSupported,
    #[error("ERR_UNHANDLED_INTERRUPT")]
    UnhandledInterrupt,
    #[error("ERR_INCORRECT_NONCE")]
    IncorrectNonce,
    #[error("ERR_INVALID_CHAIN_ID")]
    InvalidChainId,
    #[error("ERR_INVALID_ECDSA_SIGNATURE")]
    InvalidEcdsaSignature,
    #[error("ERR_INTRINSIC_GAS")]
    IntrinsicGas,
    #[error("ERR_MAX_PRIORITY_FEE_GREATER")]
    MaxPriorityFeeGreater,
    #[error("ERR_GAS_OVERFLOW")]
    GasOverflow,
    #[error("ERR_FIXED_GAS_OVERFLOW")]
    FixedGasOverflow,
    #[error("ERR_BALANCE_OVERFLOW")]
    BalanceOverflow,
    #[error("ERR_GAS_ETH_AMOUNT_OVERFLOW")]
    GasEthAmountOverflow,
    #[error("ERR_PARSE_ADDRESS")]
    ParseAddress,
    #[error("ERR_STATE_NOT_FOUND")]
    StateNotFound,
    #[error("ERR_STATE_CORRUPTED")]
    StateCorrupted,
    #[error("ERR_CONNECTOR_STORAGE_KEY_NOT_FOUND")]
    ConnectorStorageKeyNotFound,
    #[error("ERR_FAILED_DESERIALIZE_CONNECTOR_DATA")]
    FailedDeserializeConnectorData,
    #[error("ERR_PROOF_EXIST")]
    ProofExist,
    #[error("ERR_WRONG_EVENT_ADDRESS")]
    WrongEventAddress,
    #[error("ERR_CONTRACT_INITIALIZED")]
    ContractInitialized,
    #[error("ERR_RLP_FAILED")]
    RlpFailed,
    #[error("ERR_PARSE_ARGS")]
    ParseArgs,
    #[error("ERR_PARSE_DEPOSIT_EVENT")]
    ParseDepositEvent,
    #[error("ERR_PARSE_WITHDRAW_EVENT")]
    ParseWithdrawEvent,
    #[error("ERR_INVALID_EVENT_MESSAGE_FORMAT")]
    InvalidEventMessageFormat,
    #[error("ERR_INVALID_SENDER")]
    InvalidSender,
    #[error("ERR_INVALID_AMOUNT")]
    InvalidAmount,
    #[error("ERR_INVALID_FEE")]
    InvalidFee,
    #[error("ERR_INVALID_ON_TRANSFER_MESSAGE_FORMAT")]
    InvalidOnTransferMessageFormat,
    #[error("ERR_INVALID_ON_TRANSFER_MESSAGE_HEX")]
    InvalidOnTransferMessageHex,
    #[error("ERR_INVALID_ON_TRANSFER_MESSAGE_DATA")]
    InvalidOnTransferMessageData,
    #[error("ERR_INVALID_ACCOUNT_ID")]
    InvalidAccountId,
    #[error("ERR_OVERFLOW_NUMBER")]
    OverflowNumber,
    #[error("ERR_TOTAL_SUPPLY_OVERFLOW")]
    TotalSupplyOverflow,
    #[error("ERR_NOT_ENOUGH_BALANCE")]
    NotEnoughBalance,
    #[error("ERR_TOTAL_SUPPLY_UNDERFLOW")]
    TotalSupplyUnderflow,
    #[error("ERR_ZERO_AMOUNT")]
    ZeroAmount,
    #[error("ERR_SENDER_EQUALS_RECEIVER")]
    SenderEqualsReceiver,
    #[error("ERR_ACCOUNT_NOT_REGISTERED")]
    AccountNotRegistered,
    #[error("ERR_NO_AVAILABLE_BALANCE")]
    NoAvailableBalance,
    #[error("ERR_ATTACHED_DEPOSIT_NOT_ENOUGH")]
    AttachedDepositNotEnough,
    #[error("ERR_FAILED_UNREGISTER_ACCOUNT_POSITIVE_BALANCE")]
    FailedUnregisterAccountPositiveBalance,
    #[error("ERR_SAME_OWNER")]
    SameOwner,
    #[error("ERR_SAME_KEY_MANAGER")]
    SameKeyManager,
    #[error("ERR_FUNCTION_CALL_KEY_NOT_FOUND")]
    FunctionCallKeyNotFound,
    #[error("ERR_KEY_MANAGER_IS_NOT_SET")]
    KeyManagerIsNotSet,
    #[error("ERR_ACCOUNTS_COUNTER_OVERFLOW")]
    AccountsCounterOverflow,
    #[error("ERR_DECODING_TOKEN")]
    DecodingToken,
    #[error("ERR_GETTING_TOKEN")]
    GettingToken,
    #[error("ERR_WRONG_TOKEN_TYPE")]
    WrongTokenType,
    #[error("ERR_TOKEN_NO_VALUE")]
    TokenNoValue,
    #[error("ERR_NOT_ENOUGH_BALANCE_FOR_FEE")]
    NotEnoughBalanceForFee,
    #[error("ERR_GETTING_ERC20_FROM_NEP141")]
    GettingErc20FromNep141,
    #[error("ERR_ALLOWED_IN_SILO_MODE_ONLY")]
    AllowedInSiloModeOnly,
    #[error("ERR_INVALID_NEP141_ACCOUNT_ID")]
    InvalidNep141AccountId,
    #[error("ERR_NEP141_NOT_FOUND")]
    Nep141NotFound,
    #[error("ERR_NEP141_TOKEN_ALREADY_REGISTERED")]
    Nep141TokenAlreadyRegistered,
    #[error("ERR_REJECT_CALL_WITH_CODE")]
    RejectCallWithCode,
    #[error("ERR_UNKNOWN")]
    Unknown(String),
}

impl From<String> for SiloError {
    #[allow(clippy::too_many_lines)]
    fn from(s: String) -> Self {
        match s.as_str() {
            "ERR_CALL_TOO_DEEP" => Self::CallTooDeep,
            "ERR_OUT_OF_FUNDS" => Self::OutOfFunds,
            "ERR_OUT_OF_GAS" => Self::OutOfGas,
            "ERR_OUT_OF_OFFSET" => Self::OutOfOffset,
            "ERR_REVERT" => Self::Revert,
            "ERR_NOT_A_JSON_TYPE" => Self::NotAJsonType,
            "ERR_JSON_MISSING_VALUE" => Self::JsonMissingValue,
            "ERR_FAILED_PARSE_U8" => Self::FailedParseU8,
            "ERR_FAILED_PARSE_U64" => Self::FailedParseU64,
            "ERR_FAILED_PARSE_U128" => Self::FailedParseU128,
            "ERR_FAILED_PARSE_BOOL" => Self::FailedParseBool,
            "ERR_FAILED_PARSE_STRING" => Self::FailedParseString,
            "ERR_FAILED_PARSE_ARRAY" => Self::FailedParseArray,
            "ERR_EXPECTED_STRING_GOT_NUMBER" => Self::ExpectedStringGotNumber,
            "ERR_OUT_OF_RANGE_U8" => Self::OutOfRangeU8,
            "ERR_OUT_OF_RANGE_U128" => Self::OutOfRangeU128,
            "ERR_PROMISE_COUNT" => Self::PromiseCount,
            "ERR_REFUND_FAILURE" => Self::RefundFailure,
            "ERR_NOT_ALLOWED:TOO_EARLY" => Self::NotAllowedTooEarly,
            "ERR_PROMISE_FAILED" => Self::PromiseFailed,
            "ERR_VERIFY_PROOF" => Self::VerifyProof,
            "ERR_INVALID_UPGRADE" => Self::InvalidUpgrade,
            "ERR_NO_UPGRADE" => Self::NoUpgrade,
            "ERR_NOT_ALLOWED" => Self::NotAllowed,
            "ERR_NOT_OWNER" => Self::NotOwner,
            "ERR_PAUSED" => Self::Paused,
            "ERR_FT_PAUSED" => Self::FtPaused,
            "ERR_RUNNING" => Self::Running,
            "ERR_SERIALIZE" => Self::Serialize,
            "ERR_PROMISE_ENCODING" => Self::PromiseEncoding,
            "ERR_ARGS" => Self::Args,
            "ERR_VALUE_CONVERSION" => Self::ValueConversion,
            "ERR_BORSH_DESERIALIZE" => Self::BorShDeserialize,
            "ERR_JSON_DESERIALIZE" => Self::JsonDeserialize,
            "ERR_META_TX_PARSE" => Self::MetaTxParse,
            "ERR_STACK_UNDERFLOW" => Self::StackUnderflow,
            "ERR_STACK_OVERFLOW" => Self::StackOverflow,
            "ERR_INVALID_JUMP" => Self::InvalidJump,
            "ERR_INVALID_RANGE" => Self::InvalidRange,
            "ERR_DESIGNATED_INVALID" => Self::DesignatedInvalid,
            "ERR_CREATE_COLLISION" => Self::CreateCollision,
            "ERR_CREATE_CONTRACT_LIMIT" => Self::CreateContractLimit,
            "ERR_INVALID_OPCODE" => Self::InvalidOpcode,
            "ERR_OUT_OF_FUND" => Self::OutOfFund,
            "ERR_CREATE_EMPTY" => Self::CreateEmpty,
            "ERR_MAX_NONCE" => Self::MaxNonce,
            "ERR_NOT_SUPPORTED" => Self::NotSupported,
            "ERR_UNHANDLED_INTERRUPT" => Self::UnhandledInterrupt,
            "ERR_INCORRECT_NONCE" => Self::IncorrectNonce,
            "ERR_INVALID_CHAIN_ID" => Self::InvalidChainId,
            "ERR_INVALID_ECDSA_SIGNATURE" => Self::InvalidEcdsaSignature,
            "ERR_INTRINSIC_GAS" => Self::IntrinsicGas,
            "ERR_MAX_PRIORITY_FEE_GREATER" => Self::MaxPriorityFeeGreater,
            "ERR_GAS_OVERFLOW" => Self::GasOverflow,
            "ERR_FIXED_GAS_OVERFLOW" => Self::FixedGasOverflow,
            "ERR_BALANCE_OVERFLOW" => Self::BalanceOverflow,
            "ERR_GAS_ETH_AMOUNT_OVERFLOW" => Self::GasEthAmountOverflow,
            "ERR_PARSE_ADDRESS" => Self::ParseAddress,
            "ERR_STATE_NOT_FOUND" => Self::StateNotFound,
            "ERR_STATE_CORRUPTED" => Self::StateCorrupted,
            "ERR_CONNECTOR_STORAGE_KEY_NOT_FOUND" => Self::ConnectorStorageKeyNotFound,
            "ERR_FAILED_DESERIALIZE_CONNECTOR_DATA" => Self::FailedDeserializeConnectorData,
            "ERR_PROOF_EXIST" => Self::ProofExist,
            "ERR_WRONG_EVENT_ADDRESS" => Self::WrongEventAddress,
            "ERR_CONTRACT_INITIALIZED" => Self::ContractInitialized,
            "ERR_RLP_FAILED" => Self::RlpFailed,
            "ERR_PARSE_ARGS" => Self::ParseArgs,
            "ERR_PARSE_DEPOSIT_EVENT" => Self::ParseDepositEvent,
            "ERR_PARSE_WITHDRAW_EVENT" => Self::ParseWithdrawEvent,
            "ERR_INVALID_EVENT_MESSAGE_FORMAT" => Self::InvalidEventMessageFormat,
            "ERR_INVALID_SENDER" => Self::InvalidSender,
            "ERR_INVALID_AMOUNT" => Self::InvalidAmount,
            "ERR_INVALID_FEE" => Self::InvalidFee,
            "ERR_INVALID_ON_TRANSFER_MESSAGE_FORMAT" => Self::InvalidOnTransferMessageFormat,
            "ERR_INVALID_ON_TRANSFER_MESSAGE_HEX" => Self::InvalidOnTransferMessageHex,
            "ERR_INVALID_ON_TRANSFER_MESSAGE_DATA" => Self::InvalidOnTransferMessageData,
            "ERR_INVALID_ACCOUNT_ID" => Self::InvalidAccountId,
            "ERR_OVERFLOW_NUMBER" => Self::OverflowNumber,
            "ERR_TOTAL_SUPPLY_OVERFLOW" => Self::TotalSupplyOverflow,
            "ERR_NOT_ENOUGH_BALANCE" => Self::NotEnoughBalance,
            "ERR_TOTAL_SUPPLY_UNDERFLOW" => Self::TotalSupplyUnderflow,
            "ERR_ZERO_AMOUNT" => Self::ZeroAmount,
            "ERR_SENDER_EQUALS_RECEIVER" => Self::SenderEqualsReceiver,
            "ERR_ACCOUNT_NOT_REGISTERED" => Self::AccountNotRegistered,
            "ERR_NO_AVAILABLE_BALANCE" => Self::NoAvailableBalance,
            "ERR_ATTACHED_DEPOSIT_NOT_ENOUGH" => Self::AttachedDepositNotEnough,
            "ERR_FAILED_UNREGISTER_ACCOUNT_POSITIVE_BALANCE" => {
                Self::FailedUnregisterAccountPositiveBalance
            }
            "ERR_SAME_OWNER" => Self::SameOwner,
            "ERR_SAME_KEY_MANAGER" => Self::SameKeyManager,
            "ERR_FUNCTION_CALL_KEY_NOT_FOUND" => Self::FunctionCallKeyNotFound,
            "ERR_KEY_MANAGER_IS_NOT_SET" => Self::KeyManagerIsNotSet,
            "ERR_ACCOUNTS_COUNTER_OVERFLOW" => Self::AccountsCounterOverflow,
            "ERR_DECODING_TOKEN" => Self::DecodingToken,
            "ERR_GETTING_TOKEN" => Self::GettingToken,
            "ERR_WRONG_TOKEN_TYPE" => Self::WrongTokenType,
            "ERR_TOKEN_NO_VALUE" => Self::TokenNoValue,
            "ERR_NOT_ENOUGH_BALANCE_FOR_FEE" => Self::NotEnoughBalanceForFee,
            "ERR_GETTING_ERC20_FROM_NEP141" => Self::GettingErc20FromNep141,
            "ERR_ALLOWED_IN_SILO_MODE_ONLY" => Self::AllowedInSiloModeOnly,
            "ERR_INVALID_NEP141_ACCOUNT_ID" => Self::InvalidNep141AccountId,
            "ERR_NEP141_NOT_FOUND" => Self::Nep141NotFound,
            "ERR_NEP141_TOKEN_ALREADY_REGISTERED" => Self::Nep141TokenAlreadyRegistered,
            "ERR_REJECT_CALL_WITH_CODE" => Self::RejectCallWithCode,
            _ => Self::Unknown(s),
        }
    }
}
