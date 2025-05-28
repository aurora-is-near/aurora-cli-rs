use std::{collections::HashMap, fmt::Debug, vec};

use near_crypto::Signer;
use near_jsonrpc_client::{
    AsUrl, JsonRpcClient, MethodCallResult,
    errors::{JsonRpcError, JsonRpcServerError},
    methods::{
        self,
        broadcast_tx_async::RpcBroadcastTxAsyncRequest,
        broadcast_tx_commit::RpcBroadcastTxCommitRequest,
        tx::{
            RpcTransactionError, RpcTransactionResponse, RpcTransactionStatusRequest,
            TransactionInfo,
        },
    },
};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::{
    action::{Action, FunctionCallAction},
    errors::InvalidTxError,
    hash::CryptoHash,
    transaction::SignedTransaction,
    types::{AccountId, BlockReference, Finality, Gas, Nonce},
    views::{AccessKeyView, BlockView, FinalExecutionOutcomeView, QueryRequest, TxExecutionStatus},
};
use near_token::NearToken;
use tokio::sync::Mutex;

use super::Result;
use super::error::Error;

pub struct RpcClient {
    client: JsonRpcClient,
    access_key_nonces: Mutex<HashMap<(AccountId, near_crypto::PublicKey), u64>>,
}

impl RpcClient {
    pub(crate) fn new<U: AsUrl>(url: U, api_key: Option<String>) -> Result<Self> {
        let connector = JsonRpcClient::new_client();
        let mut client = connector.connect(url);
        if let Some(api_key) = api_key {
            let api_key = near_jsonrpc_client::auth::ApiKey::new(api_key)?;
            client = client.header(api_key);
        }

        Ok(Self {
            client,
            access_key_nonces: Mutex::new(HashMap::new()),
        })
    }

    pub(crate) async fn query<M>(&self, method: M) -> MethodCallResult<M::Response, M::Error>
    where
        M: methods::RpcMethod + Debug + Send + Sync,
        M::Response: Debug + Send,
        M::Error: Debug + Send,
    {
        self.client.call(&method).await
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn call(
        &self,
        signer: &Signer,
        contract_id: &AccountId,
        method_name: String,
        args: Vec<u8>,
        gas: Gas,
        deposit: NearToken,
        priority_fee: u64,
    ) -> Result<FinalExecutionOutcomeView> {
        self.send_tx(
            signer,
            contract_id,
            FunctionCallAction {
                args,
                method_name,
                gas,
                deposit: deposit.as_yoctonear(),
            }
            .into(),
            priority_fee,
        )
        .await
    }

    pub(crate) async fn send_batch_tx(
        &self,
        signer: &Signer,
        receiver_id: &AccountId,
        actions: Vec<Action>,
        priority_fee: u64,
    ) -> Result<FinalExecutionOutcomeView> {
        let cache_key = (signer.get_account_id(), signer.public_key());
        let (block_hash, nonce) = self.fetch_tx_nonce(&cache_key).await?;

        send_tx(
            self,
            &cache_key,
            SignedTransaction::from_actions(
                nonce,
                signer.get_account_id(),
                receiver_id.clone(),
                signer,
                actions,
                block_hash,
                priority_fee,
            ),
        )
        .await
    }

    pub(crate) async fn send_batch_tx_async(
        &self,
        signer: &Signer,
        receiver_id: &AccountId,
        actions: Vec<Action>,
        priority_fee: u64,
    ) -> Result<CryptoHash> {
        let cache_key = (signer.get_account_id(), signer.public_key());
        let (block_hash, nonce) = self.fetch_tx_nonce(&cache_key).await?;

        self.query(RpcBroadcastTxAsyncRequest {
            signed_transaction: SignedTransaction::from_actions(
                nonce,
                signer.get_account_id(),
                receiver_id.clone(),
                signer,
                actions,
                block_hash,
                priority_fee,
            ),
        })
        .await
        .map_err(Into::into)
    }

    pub(crate) async fn status(
        &self,
        hash: &CryptoHash,
        sender: &AccountId,
        wait_until: Option<TxExecutionStatus>,
    ) -> Result<RpcTransactionResponse> {
        let status = self
            .client
            .call(RpcTransactionStatusRequest {
                transaction_info: TransactionInfo::TransactionId {
                    tx_hash: *hash,
                    sender_account_id: sender.clone(),
                },
                wait_until: wait_until.unwrap_or(TxExecutionStatus::Final),
            })
            .await?;

        Ok(status)
    }

    async fn query_broadcast_tx(
        &self,
        method: &RpcBroadcastTxCommitRequest,
    ) -> MethodCallResult<FinalExecutionOutcomeView, RpcTransactionError> {
        self.client.call(method).await
    }

    async fn send_tx(
        &self,
        signer: &Signer,
        receiver_id: &AccountId,
        action: Action,
        priority_fee: u64,
    ) -> Result<FinalExecutionOutcomeView> {
        self.send_batch_tx(signer, receiver_id, vec![action], priority_fee)
            .await
    }

    async fn view_block(&self, block_ref: Option<BlockReference>) -> Result<BlockView> {
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
        let mut nonces = self.access_key_nonces.lock().await;

        if let Some(next_nonce_ref) = nonces.get_mut(cache_key) {
            let current_nonce = *next_nonce_ref;
            *next_nonce_ref += 1;

            // Fetch the latest block_hash since the previous one is probably invalid
            // for a new transaction.
            let block = self.view_block(Some(Finality::Final.into())).await?;
            let block_hash = block.header.hash;
            Ok((block_hash, current_nonce))
        } else {
            let (account_id, public_key) = cache_key;
            let (access_key, block_hash) = self
                .access_key(account_id.clone(), public_key.clone())
                .await?;

            // The nonce from the access key is the last used nonce.
            // The nonce for the current transaction should be last_used_nonce + 1.
            let current_nonce = access_key.nonce + 1;

            // Store the nonce for the *next* transaction in the cache.
            nonces.insert(cache_key.clone(), current_nonce + 1);
            drop(nonces);

            Ok((block_hash, current_nonce))
        }
    }

    async fn access_key(
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
            kind => Err(Error::UnexpectedQueryResponseKind(Box::new(kind))),
        }
    }
}

async fn send_tx(
    client: &RpcClient,
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
        let mut nonces = client.access_key_nonces.lock().await;
        nonces.remove(cache_key);
    }

    result.map_err(Into::into)
}

impl Clone for RpcClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            access_key_nonces: Mutex::new(HashMap::new()),
        }
    }
}
