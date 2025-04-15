use anyhow::Ok;
use broadcast::Broadcast;
pub use builder::ClientBuilder;
use near_crypto::PublicKey;
use near_jsonrpc_client::JsonRpcClient;
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_jsonrpc_primitives::types::transactions::RpcTransactionResponse;
use near_primitives::transaction::SignedTransaction;
use near_primitives::types::AccountId;
use near_primitives::views;
use near_primitives::{hash::CryptoHash, views::CallResult};
use operations::{CallTransaction, Transaction, ViewTransaction};

pub mod broadcast;
pub mod builder;
pub mod operations;
pub mod response;

#[derive(Debug, Clone)]
pub struct Client<B: Broadcast> {
    client: JsonRpcClient,

    _broadcast: std::marker::PhantomData<B>,
}

impl<B: Broadcast> Client<B> {
    /// Make a view call to the engine contract.
    pub async fn view(&self, args: ViewTransaction) -> anyhow::Result<CallResult> {
        let request = near_jsonrpc_client::methods::query::RpcQueryRequest {
            block_reference: near_primitives::types::Finality::Final.into(),
            request: near_primitives::views::QueryRequest::CallFunction {
                account_id: args.contract_id,
                method_name: args.function.method,
                args: args.function.args?.into(),
            },
        };
        let response = self.client.call(request).await?;

        match response.kind {
            QueryResponseKind::CallResult(result) => Ok(result),
            _ => anyhow::bail!("Wrong response type"),
        }
    }

    pub async fn view_account(&self, account_id: AccountId) -> anyhow::Result<views::AccountView> {
        let request = near_jsonrpc_primitives::types::query::RpcQueryRequest {
            block_reference: near_primitives::types::Finality::Final.into(),
            request: views::QueryRequest::ViewAccount { account_id },
        };

        let response = self.client.call(request).await?;

        match response.kind {
            QueryResponseKind::ViewAccount(account_view) => Ok(account_view),
            _ => anyhow::bail!("Wrong response type"),
        }
    }

    pub async fn tx_status(
        &self,
        sender_id: AccountId,
        tx_hash: CryptoHash,
    ) -> anyhow::Result<RpcTransactionResponse> {
        let tx_status_request = near_jsonrpc_client::methods::tx::RpcTransactionStatusRequest {
            transaction_info:
                near_jsonrpc_primitives::types::transactions::TransactionInfo::TransactionId {
                    tx_hash,
                    sender_account_id: sender_id,
                },
            wait_until: near_primitives::views::TxExecutionStatus::Executed,
        };

        self.client
            .call(tx_status_request)
            .await
            .map_err(Into::into)
    }

    pub async fn broadcast(&self, tx: Transaction) -> anyhow::Result<B::Output> {
        let (block_hash, nonce) = self
            .get_nonce(
                tx.signer.account_id.clone(),
                tx.signer.secret_key.public_key(),
            )
            .await?;
        let nonce = tx.nonce.unwrap_or(nonce);

        let signed_tx = SignedTransaction::from_actions(
            nonce,
            tx.signer.account_id.clone(),
            tx.receiver_id,
            &tx.signer.into(),
            tx.actions?,
            block_hash,
            0,
        );

        B::broadcast_tx(&self.client, signed_tx).await
    }

    pub async fn get_nonce(
        &self,
        account_id: AccountId,
        public_key: PublicKey,
    ) -> anyhow::Result<(CryptoHash, u64)> {
        let request = near_jsonrpc_primitives::types::query::RpcQueryRequest {
            block_reference: near_primitives::types::Finality::Final.into(),
            request: views::QueryRequest::ViewAccessKey {
                account_id,
                public_key,
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

    pub async fn call(&self, args: CallTransaction) -> anyhow::Result<B::Output> {
        Ok(self.broadcast(args.into()).await?)
    }

    pub fn into_async(self) -> Client<broadcast::Async> {
        Client::<broadcast::Async> {
            client: self.client,
            _broadcast: std::marker::PhantomData,
        }
    }

    pub fn into_sync(self) -> Client<broadcast::Sync> {
        Client::<broadcast::Sync> {
            client: self.client,
            _broadcast: std::marker::PhantomData,
        }
    }
}
