use near_jsonrpc_client::{
    methods::{
        broadcast_tx_async::RpcBroadcastTxAsyncRequest,
        broadcast_tx_commit::RpcBroadcastTxCommitRequest,
    },
    JsonRpcClient,
};
use near_primitives::{
    hash::CryptoHash, transaction::SignedTransaction, views::FinalExecutionOutcomeView,
};

#[async_trait::async_trait]
pub trait Broadcast {
    type Output;

    async fn broadcast_tx(
        client: &JsonRpcClient,
        tx: SignedTransaction,
    ) -> anyhow::Result<Self::Output>;
}

#[derive(Clone)]
pub struct Sync;

#[async_trait::async_trait]
impl Broadcast for Sync {
    type Output = FinalExecutionOutcomeView;

    async fn broadcast_tx(
        client: &JsonRpcClient,
        tx: SignedTransaction,
    ) -> anyhow::Result<Self::Output> {
        let request = RpcBroadcastTxCommitRequest {
            signed_transaction: tx,
        };

        client.call(request).await.map_err(Into::into)
    }
}

#[derive(Clone)]
pub struct Async;

#[async_trait::async_trait]
impl Broadcast for Async {
    type Output = CryptoHash;

    async fn broadcast_tx(
        client: &JsonRpcClient,
        signed_tx: SignedTransaction,
    ) -> anyhow::Result<Self::Output> {
        let request = RpcBroadcastTxAsyncRequest {
            signed_transaction: signed_tx,
        };

        client.call(request).await.map_err(Into::into)
    }
}
