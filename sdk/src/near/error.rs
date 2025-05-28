use std::fmt::Debug;

use near_jsonrpc_client::errors::{JsonRpcError, JsonRpcServerError};
use near_jsonrpc_client::methods;
use near_jsonrpc_client::methods::query::RpcQueryError;
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("RPC error querying block: {0}")]
    RpcBlockError(#[from] JsonRpcError<methods::block::RpcBlockError>),

    #[error("RPC error querying access key: {0}")]
    RpcQueryError(#[from] JsonRpcError<methods::query::RpcQueryError>),

    #[error("RPC error broadcasting transaction: {0}")]
    RpcBroadcastTxError(#[from] JsonRpcError<methods::tx::RpcTransactionError>),

    #[error("RPC error broadcasting transaction async: {0}")]
    RpcBroadcastTxAsyncError(
        #[from] JsonRpcError<methods::broadcast_tx_async::RpcBroadcastTxAsyncError>,
    ),

    #[error("API Key error: {0}")]
    ApiKeyError(#[from] near_jsonrpc_client::header::InvalidHeaderValue),

    #[error("Data conversion error: {0}")]
    DataConversionError(#[from] DataConversionError),

    #[error("Unexpected query response kind: {0:?}")]
    UnexpectedQueryResponseKind(Box<QueryResponseKind>),
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::DataConversionError(DataConversionError::Json(err))
    }
}

impl From<borsh::io::Error> for Error {
    fn from(err: borsh::io::Error) -> Self {
        Self::DataConversionError(DataConversionError::Borsh(err))
    }
}

impl From<RpcQueryError> for Error {
    fn from(err: RpcQueryError) -> Self {
        Self::RpcQueryError(JsonRpcError::ServerError(JsonRpcServerError::HandlerError(
            err,
        )))
    }
}

#[derive(Debug, Error)]
pub enum DataConversionError {
    #[error(transparent)]
    Borsh(#[from] borsh::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
