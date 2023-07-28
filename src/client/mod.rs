#[cfg(feature = "simple")]
use aurora_engine_types::account_id::AccountId;
#[cfg(feature = "advanced")]
use aurora_engine_types::parameters::engine::SubmitResult;
#[cfg(feature = "advanced")]
use aurora_engine_types::H256;
#[cfg(feature = "advanced")]
use thiserror::Error;

#[cfg(feature = "advanced")]
pub use aurora::AuroraClient;
pub use near::NearClient;

#[cfg(feature = "advanced")]
mod aurora;
mod near;

#[cfg(feature = "advanced")]
type NearQueryError =
    near_jsonrpc_client::errors::JsonRpcError<near_jsonrpc_primitives::types::query::RpcQueryError>;
#[cfg(feature = "advanced")]
type NearCallError = near_jsonrpc_client::errors::JsonRpcError<
    near_jsonrpc_client::methods::broadcast_tx_commit::RpcTransactionError,
>;

#[cfg(feature = "simple")]
pub struct Client {
    near_rpc: String,
    #[cfg(feature = "advanced")]
    aurora_rpc: String,
    engine_account_id: AccountId,
    signer_key_path: Option<String>,
}

#[cfg(feature = "simple")]
impl Client {
    pub fn new(near_rpc: &str, engine_account: &str, signer_key_path: Option<String>) -> Self {
        Self {
            near_rpc: near_rpc.to_string(),
            engine_account_id: engine_account.parse().expect("wrong engine account format"),
            signer_key_path,
        }
    }

    pub fn near(&self) -> NearClient {
        NearClient::new(
            &self.near_rpc,
            self.engine_account_id.as_ref(),
            self.signer_key_path.clone(),
        )
    }
}

#[cfg(feature = "advanced")]
#[derive(Debug)]
pub enum TransactionOutcome {
    Result(SubmitResult),
    Failure(near_primitives::errors::TxExecutionError),
}

#[cfg(feature = "advanced")]
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Transaction with hash {0} not found")]
    AuroraTransactionNotFound(H256),
    #[error("Invalid hash")]
    InvalidHex(#[from] hex::FromHexError),
    #[error("Invalid json: {0}")]
    InvalidJson(String),
    #[error("response key not found: {0}")]
    ResponseKeyNotFound(String),
    #[error("Wrong json object: {0:?}")]
    NotJsonObject(serde_json::Value),
    #[error("Wrong json string: {0:?}")]
    NotJsonString(serde_json::Value),
    #[error("Aurora RPC error: {0}")]
    AuroraRpc(#[from] aurora::Web3JsonResponseError),
    #[error("NEAR RPC error")]
    NearRpc(#[from] NearQueryError),
    #[error("NEAR contract call error: {0}")]
    NearContractCall(#[from] NearCallError),
    #[error("Reqwest client error: {0}")]
    Reqwest(#[from] reqwest::Error),
}
