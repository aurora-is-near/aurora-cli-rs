use std::{
    collections::HashMap,
    fmt::Debug,
    sync::atomic::{AtomicU64, Ordering},
    vec,
};

use aurora_engine_types::types::NearGas;
use near_crypto::{InMemorySigner, Signer};
use near_jsonrpc_client::{
    errors::{JsonRpcError, JsonRpcServerError},
    methods::{
        self, broadcast_tx_async::RpcBroadcastTxAsyncRequest,
        broadcast_tx_commit::RpcBroadcastTxCommitRequest, tx::RpcTransactionError,
    },
    JsonRpcClient, MethodCallResult,
};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::{
    action::{Action, FunctionCallAction},
    errors::InvalidTxError,
    hash::CryptoHash,
    transaction::SignedTransaction,
    types::{AccountId, BlockReference, Finality, Nonce},
    views::{AccessKeyView, BlockView, FinalExecutionOutcomeView, QueryRequest},
};
use near_token::NearToken;
use tokio::sync::RwLock;

use super::error::Error;
use super::operations::DEFAULT_PRIORITY_FEE;
use super::Result;

pub(crate) struct Client {
    client: JsonRpcClient,

    pub(crate) access_key_nonces: RwLock<HashMap<(AccountId, near_crypto::PublicKey), AtomicU64>>,
}

impl Client {
    pub(crate) fn new(addr: &str, api_key: Option<String>) -> Result<Self> {
        let connector = JsonRpcClient::new_client();
        let mut client = connector.connect(addr);
        if let Some(api_key) = api_key {
            let api_key = near_jsonrpc_client::auth::ApiKey::new(api_key)?;
            client = client.header(api_key);
        }

        Ok(Self {
            client,
            access_key_nonces: RwLock::new(HashMap::new()),
        })
    }

    pub(crate) async fn query_broadcast_tx(
        &self,
        method: &RpcBroadcastTxCommitRequest,
    ) -> MethodCallResult<FinalExecutionOutcomeView, RpcTransactionError> {
        self.client.call(method).await.map_err(Into::into)
    }

    pub(crate) async fn query<M>(&self, method: M) -> MethodCallResult<M::Response, M::Error>
    where
        M: methods::RpcMethod + Debug + Send + Sync,
        M::Response: Debug + Send,
        M::Error: Debug + Send,
    {
        self.client.call(&method).await
    }

    async fn send_tx(
        &self,
        signer: &InMemorySigner,
        receiver_id: &AccountId,
        action: Action,
    ) -> Result<FinalExecutionOutcomeView> {
        self.send_batch_tx(signer, receiver_id, vec![action]).await
    }

    pub(crate) async fn view_block(&self, block_ref: Option<BlockReference>) -> Result<BlockView> {
        let block_reference = block_ref.unwrap_or_else(|| Finality::None.into());
        let block_view = self
            .query(&methods::block::RpcBlockRequest { block_reference })
            .await?;

        Ok(block_view)
    }

    async fn fetch_tx_nonce(
        &self,
        cache_key: &(AccountId, near_crypto::PublicKey),
    ) -> Result<(CryptoHash, Nonce)> {
        let nonces = self.access_key_nonces.read().await;
        if let Some(nonce) = nonces.get(cache_key) {
            let nonce = nonce.fetch_add(1, Ordering::SeqCst);
            drop(nonces);

            // Fetch latest block_hash since the previous one is now invalid for new transactions:
            let block = self.view_block(Some(Finality::Final.into())).await?;
            let block_hash = block.header.hash;
            Ok((block_hash, nonce + 1))
        } else {
            drop(nonces);

            let (account_id, public_key) = cache_key;
            let (access_key, block_hash) = self
                .access_key(account_id.clone(), public_key.clone())
                .await?;

            // case where multiple writers end up at the same lock acquisition point and tries
            // to overwrite the cached value that a previous writer already wrote.
            let nonce = self
                .access_key_nonces
                .write()
                .await
                .entry(cache_key.clone())
                .or_insert_with(|| AtomicU64::new(access_key.nonce + 1))
                .fetch_max(access_key.nonce + 1, Ordering::SeqCst)
                .max(access_key.nonce + 1);

            Ok((block_hash, nonce))
        }
    }

    pub(crate) async fn access_key(
        &self,
        account_id: AccountId,
        public_key: near_crypto::PublicKey,
    ) -> Result<(AccessKeyView, CryptoHash)> {
        let query_resp = self
            .query(&methods::query::RpcQueryRequest {
                block_reference: Finality::None.into(),
                request: QueryRequest::ViewAccessKey {
                    account_id,
                    public_key,
                },
            })
            .await?;

        match query_resp.kind {
            QueryResponseKind::AccessKey(access_key) => Ok((access_key, query_resp.block_hash)),
            kind => Err(Error::UnexpectedQueryResponseKind(kind)),
        }
    }

    pub(crate) async fn send_batch_tx(
        &self,
        signer: &InMemorySigner,
        receiver_id: &AccountId,
        actions: Vec<Action>,
    ) -> Result<FinalExecutionOutcomeView> {
        let cache_key = (signer.account_id.clone(), signer.secret_key.public_key());

        let (block_hash, nonce) = self.fetch_tx_nonce(&cache_key).await?;
        send_tx(
            self,
            &cache_key,
            SignedTransaction::from_actions(
                nonce,
                signer.account_id.clone(),
                receiver_id.clone(),
                &Signer::InMemory(signer.clone()),
                actions.clone(),
                block_hash,
                DEFAULT_PRIORITY_FEE,
            ),
        )
        .await
    }

    pub(crate) async fn send_batch_tx_async(
        &self,
        signer: &InMemorySigner,
        receiver_id: &AccountId,
        actions: Vec<Action>,
    ) -> Result<CryptoHash> {
        let cache_key = (signer.account_id.clone(), signer.secret_key.public_key());
        let (block_hash, nonce) = self.fetch_tx_nonce(&cache_key).await?;

        self.query(RpcBroadcastTxAsyncRequest {
            signed_transaction: SignedTransaction::from_actions(
                nonce,
                signer.account_id.clone(),
                receiver_id.clone(),
                &Signer::InMemory(signer.clone()),
                actions,
                block_hash,
                DEFAULT_PRIORITY_FEE,
            ),
        })
        .await
        .map_err(Into::into)
    }

    pub(crate) async fn call(
        &self,
        signer: &InMemorySigner,
        contract_id: &AccountId,
        method_name: String,
        args: Vec<u8>,
        gas: NearGas,
        deposit: NearToken,
    ) -> Result<FinalExecutionOutcomeView> {
        self.send_tx(
            signer,
            contract_id,
            FunctionCallAction {
                args,
                method_name,
                gas: gas.as_u64(),
                deposit: deposit.as_yoctonear(),
            }
            .into(),
        )
        .await
    }
}

async fn send_tx(
    client: &Client,
    cache_key: &(AccountId, near_crypto::PublicKey),
    tx: SignedTransaction,
) -> Result<FinalExecutionOutcomeView> {
    let result = client
        .query_broadcast_tx(&RpcBroadcastTxCommitRequest {
            signed_transaction: tx,
        })
        .await;

    // InvalidNonce, cached nonce is potentially very far behind, so invalidate it.
    if let Err(JsonRpcError::ServerError(JsonRpcServerError::HandlerError(
        RpcTransactionError::InvalidTransaction {
            context: InvalidTxError::InvalidNonce { .. },
            ..
        },
    ))) = &result
    {
        let mut nonces = client.access_key_nonces.write().await;
        nonces.remove(cache_key);
    }

    result.map_err(Into::into)
}
