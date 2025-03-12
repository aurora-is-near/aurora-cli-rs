use crate::client::response::{FromBytes, Response};
use anyhow::Ok;
use broadcast::Broadcast;
pub use builder::ClientBuilder;
use near_crypto::InMemorySigner;
use near_jsonrpc_client::JsonRpcClient;
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_jsonrpc_primitives::types::transactions::RpcTransactionResponse;
use near_primitives::action::Action;
use near_primitives::hash::CryptoHash;
use near_primitives::transaction::SignedTransaction;
use near_primitives::types::AccountId;
use near_primitives::views;

pub mod broadcast;
pub mod builder;
pub mod response;

#[derive(Debug, Clone)]
pub struct Client<B: Broadcast> {
    client: JsonRpcClient,
    engine: AccountId,
    signer: InMemorySigner,

    _broadcast: std::marker::PhantomData<B>,
}

impl<B: Broadcast> Client<B> {
    /// Make a view call to the engine contract.
    pub async fn view<T: FromBytes>(
        &self,
        method: &str,
        args: Option<Vec<u8>>,
    ) -> anyhow::Result<Response<T>> {
        let request = near_jsonrpc_client::methods::query::RpcQueryRequest {
            block_reference: near_primitives::types::Finality::Final.into(),
            request: near_primitives::views::QueryRequest::CallFunction {
                account_id: self.engine.clone(),
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
                    sender_account_id: self.engine.clone(),
                },
            wait_until: near_primitives::views::TxExecutionStatus::Executed,
        };

        self.client
            .call(tx_status_request)
            .await
            .map_err(Into::into)
    }

    pub async fn broadcast(&self, actions: Vec<Action>) -> anyhow::Result<B::Output> {
        let (block_hash, nonce) = self.get_nonce().await?;

        let signed_tx = SignedTransaction::from_actions(
            nonce,
            self.signer.account_id.clone(),
            self.engine.as_str().parse()?,
            &self.signer.clone().into(),
            actions,
            block_hash,
            0,
        );

        B::broadcast_tx(&self.client, signed_tx).await
    }

    pub async fn get_nonce(&self) -> anyhow::Result<(CryptoHash, u64)> {
        let request = near_jsonrpc_primitives::types::query::RpcQueryRequest {
            block_reference: near_primitives::types::Finality::Final.into(),
            request: views::QueryRequest::ViewAccessKey {
                account_id: self.signer.account_id.clone(),
                public_key: self.signer.public_key.clone(),
            },
        };
        let response = self.client.call(request).await?;
        let block_hash = response.block_hash;
        let nonce = match response.kind {
            QueryResponseKind::AccessKey(k) => k.nonce + 1,
            _ => anyhow::bail!("Wrong response kind: {:?}", response.kind),
        };

        Ok((block_hash, nonce))
    }

    pub fn with_engine(self, engine: AccountId) -> Self {
        Self { engine, ..self }
    }

    pub fn with_signer(self, signer: InMemorySigner) -> Self {
        Self { signer, ..self }
    }

    pub fn with_account_id(self, account_id: AccountId) -> Self {
        Self {
            signer: InMemorySigner {
                account_id,
                public_key: self.signer.public_key,
                secret_key: self.signer.secret_key,
            },
            ..self
        }
    }

    pub fn to_async(self) -> Client<broadcast::Async> {
        Client::<broadcast::Async> {
            client: self.client,
            engine: self.engine,
            signer: self.signer,
            _broadcast: std::marker::PhantomData,
        }
    }

    pub fn to_sync(self) -> Client<broadcast::Sync> {
        Client::<broadcast::Sync> {
            client: self.client,
            engine: self.engine,
            signer: self.signer,
            _broadcast: std::marker::PhantomData,
        }
    }
}
