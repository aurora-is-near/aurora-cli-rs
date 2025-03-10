use crate::client::response::{FromBytes, Response};
pub use builder::ClientBuilder;
use near_jsonrpc_client::JsonRpcClient;
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_jsonrpc_primitives::types::transactions::RpcTransactionResponse;
use near_primitives::hash::CryptoHash;
use near_primitives::types::AccountId;
use std::path::PathBuf;

mod builder;
pub mod response;

#[derive(Debug)]
pub struct Client {
    client: JsonRpcClient,
    engine_account_id: Option<AccountId>,
    signer_key_path: Option<PathBuf>,
}

impl Client {
    /// Make a view call to the engine contract.
    pub async fn view<T: FromBytes>(
        &self,
        method: &str,
        args: Option<Vec<u8>>,
    ) -> anyhow::Result<Response<T>> {
        let request = near_jsonrpc_client::methods::query::RpcQueryRequest {
            block_reference: near_primitives::types::Finality::Final.into(),
            request: near_primitives::views::QueryRequest::CallFunction {
                account_id: self
                    .engine_account_id
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("Missing engine account id"))?,
                method_name: method.to_string(),
                args: args.unwrap_or_default().into(),
            },
        };
        let response = self.client.call(request).await?;

        match response.kind {
            QueryResponseKind::CallResult(result) => result.try_into(),
            _ => anyhow::bail!("Wrong response type"),
        }
    }

    pub async fn tx_status(&self, tx_hash: CryptoHash) -> anyhow::Result<RpcTransactionResponse> {
        let tx_status_request = near_jsonrpc_client::methods::tx::RpcTransactionStatusRequest {
            transaction_info:
                near_jsonrpc_primitives::types::transactions::TransactionInfo::TransactionId {
                    tx_hash,
                    sender_account_id: self.engine_account_id.clone().unwrap(),
                },
            wait_until: near_primitives::views::TxExecutionStatus::Executed,
        };

        self.client
            .call(tx_status_request)
            .await
            .map_err(Into::into)
    }
}
