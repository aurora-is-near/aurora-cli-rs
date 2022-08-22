use crate::eth_method::EthMethod;
use aurora_engine::parameters::{SubmitResult, TransactionStatus};
use aurora_engine_transactions::{legacy::TransactionLegacy, EthTransactionKind};
use aurora_engine_types::{
    types::{Address, Wei},
    H256, U256,
};
use borsh::{BorshDeserialize, BorshSerialize};
use near_jsonrpc_client::AsUrl;
use near_primitives::views;
use secp256k1::SecretKey;
use serde::{Deserialize, Serialize};

const NEAR_TRANSACTION_KEY: &str = "nearTransactionHash";

type NearQueryError =
    near_jsonrpc_client::errors::JsonRpcError<near_jsonrpc_primitives::types::query::RpcQueryError>;
type NearCallError = near_jsonrpc_client::errors::JsonRpcError<
    near_jsonrpc_client::methods::broadcast_tx_commit::RpcTransactionError,
>;

pub struct AuroraClient<T> {
    inner: reqwest::Client,
    aurora_rpc: T,
    near_client: near_jsonrpc_client::JsonRpcClient,
    engine_account_id: String,
    signer_key_path: Option<String>,
}

impl<T: AsRef<str>> AuroraClient<T> {
    pub fn new<U: AsUrl>(
        aurora_rpc: T,
        near_rpc: U,
        engine_account_id: String,
        signer_key_path: Option<String>,
    ) -> Self {
        let inner = reqwest::Client::new();
        let near_client = near_jsonrpc_client::JsonRpcClient::connect(near_rpc);
        Self {
            inner,
            aurora_rpc,
            near_client,
            engine_account_id,
            signer_key_path,
        }
    }

    pub async fn request<'a, 'b, U: Serialize>(
        &self,
        request: &Web3JsonRequest<'a, 'b, U>,
    ) -> Result<Web3JsonResponse<serde_json::Value>, ClientError> {
        let resp = self
            .inner
            .post(self.aurora_rpc.as_ref())
            .json(request)
            .send()
            .await?;
        // TODO: parse information from headers too (eg x-request-id)
        // println!("{:?}", resp.headers());
        let full = resp.bytes().await?;
        serde_json::from_slice(&full).map_err(|_| {
            let text = match String::from_utf8_lossy(&full) {
                std::borrow::Cow::Owned(s) => s,
                std::borrow::Cow::Borrowed(s) => s.to_owned(),
            };
            ClientError::InvalidJson(text)
        })
    }

    pub async fn get_nonce(&self, address: Address) -> Result<U256, ClientError> {
        let method = EthMethod::GetTransactionCount(address);
        let request = Web3JsonRequest::from_method(1, &method);
        let response = self.request(&request).await?;

        if let Some(e) = response.error {
            return Err(e.into());
        }

        let value = response.result.as_ref().and_then(|v| v.as_str()).unwrap();
        Ok(U256::from_str_radix(value, 16).unwrap())
    }

    pub async fn get_chain_id(&self) -> Result<u64, ClientError> {
        let method = EthMethod::GetChainId;
        let request = Web3JsonRequest::from_method(1, &method);
        let response = self.request(&request).await?;

        if let Some(e) = response.error {
            return Err(e.into());
        }

        let value = response.result.as_ref().and_then(|v| v.as_str()).unwrap();
        Ok(value.parse().unwrap())
    }

    pub async fn eth_transaction(
        &self,
        target: Option<Address>,
        amount: Wei,
        signer: &SecretKey,
        chain_id: u64,
        nonce: U256,
        data: Vec<u8>,
    ) -> Result<H256, ClientError> {
        let tx = TransactionLegacy {
            nonce,
            gas_price: U256::zero(),
            gas_limit: U256::from(u64::MAX),
            to: target,
            value: amount,
            data,
        };
        let signed_tx =
            EthTransactionKind::Legacy(crate::utils::sign_transaction(tx, chain_id, signer));
        let method = EthMethod::SendRawTransaction(Box::new(signed_tx));
        let request = Web3JsonRequest::from_method(1, &method);
        let response = self.request(&request).await?;

        if let Some(e) = response.error {
            return Err(e.into());
        }

        let tx_hash = response.result.as_ref().and_then(|v| v.as_str()).unwrap();
        let tx_hash_bytes = tx_hash
            .strip_prefix("0x")
            .and_then(|x| hex::decode(x).ok())
            .unwrap();
        Ok(H256::from_slice(&tx_hash_bytes))
    }

    pub async fn get_transaction_outcome(
        &self,
        tx_hash: H256,
        relayer: &str,
    ) -> Result<TransactionOutcome, ClientError> {
        let method = EthMethod::GetTransactionReceipt(tx_hash);
        let request = Web3JsonRequest::from_method(1, &method);
        let response = self.request(&request).await?;

        if let Some(e) = response.error {
            return Err(e.into());
        }

        let response_value = response
            .result
            .as_ref()
            .ok_or(ClientError::AuroraTransactionNotFound(tx_hash))?;
        let near_tx_value = response_value
            .as_object()
            .ok_or_else(|| ClientError::NotJsonObject(response_value.clone()))?
            .get(NEAR_TRANSACTION_KEY)
            .ok_or_else(|| ClientError::ResponseKeyNotFound(String::from(NEAR_TRANSACTION_KEY)))?;
        let near_tx_str = near_tx_value
            .as_str()
            .ok_or_else(|| ClientError::NotJsonString(near_tx_value.clone()))?;
        // println!("{}", near_tx_str);
        let near_tx_hex = near_tx_str.strip_prefix("0x").unwrap_or(near_tx_str);
        let near_tx_hash = hex::decode(near_tx_hex)?;

        self.get_near_transaction_outcome(H256::from_slice(&near_tx_hash), relayer)
            .await
    }

    pub async fn get_near_transaction_outcome(
        &self,
        near_tx_hash: H256,
        relayer: &str,
    ) -> Result<TransactionOutcome, ClientError> {
        let tx_status_request = near_jsonrpc_client::methods::tx::RpcTransactionStatusRequest {
            transaction_info:
                near_jsonrpc_primitives::types::transactions::TransactionInfo::TransactionId {
                    hash: near_tx_hash.as_bytes().try_into().unwrap(),
                    account_id: relayer.parse().unwrap(),
                },
        };
        let near_tx_status = self.near_client.call(tx_status_request).await.unwrap();
        match near_tx_status.status {
            near_primitives::views::FinalExecutionStatus::SuccessValue(result) => {
                let result_bytes = base64::decode(result).unwrap();
                let result = SubmitResult::try_from_slice(&result_bytes).unwrap();
                Ok(TransactionOutcome::Result(result))
            }
            near_primitives::views::FinalExecutionStatus::Failure(e) => {
                Ok(TransactionOutcome::Failure(e))
            }
            _ => unreachable!(),
        }
    }

    pub async fn get_nep141_from_erc20(&self, erc20: Address) -> Result<String, ClientError> {
        let result = self
            .near_view_call("get_nep141_from_erc20".into(), erc20.as_bytes().to_vec())
            .await?;
        Ok(String::from_utf8_lossy(&result.result).into_owned())
    }

    pub async fn get_bridge_prover(&self) -> Result<String, ClientError> {
        let result = self
            .near_view_call("get_bridge_prover".into(), Vec::new())
            .await?;
        Ok(String::from_utf8_lossy(&result.result).into_owned())
    }

    pub async fn view_contract_call(
        &self,
        sender: Address,
        target: Address,
        amount: Wei,
        input: Vec<u8>,
    ) -> Result<TransactionStatus, ClientError> {
        let args = aurora_engine::parameters::ViewCallArgs {
            sender,
            address: target,
            amount: amount.to_bytes(),
            input,
        };
        let result = self
            .near_view_call("view".into(), args.try_to_vec().unwrap())
            .await?;
        let status = TransactionStatus::try_from_slice(&result.result).unwrap();
        Ok(status)
    }

    async fn near_view_call(
        &self,
        method_name: String,
        args: Vec<u8>,
    ) -> Result<views::CallResult, ClientError> {
        let request = near_jsonrpc_primitives::types::query::RpcQueryRequest {
            block_reference: near_primitives::types::Finality::Final.into(),
            request: near_primitives::views::QueryRequest::CallFunction {
                account_id: self.engine_account_id.parse().unwrap(),
                method_name,
                args: args.into(),
            },
        };
        let response = self.near_client.call(request).await?;

        match response.kind {
            near_jsonrpc_primitives::types::query::QueryResponseKind::CallResult(result) => {
                Ok(result)
            }
            _ => unreachable!(),
        }
    }

    pub async fn near_contract_call(
        &self,
        method_name: String,
        args: Vec<u8>,
    ) -> Result<views::FinalExecutionOutcomeView, ClientError> {
        let path = self
            .signer_key_path
            .as_ref()
            .map(std::path::Path::new)
            .expect("Signer path must be provided to use this functionality");
        let signer = near_crypto::InMemorySigner::from_file(path);
        let request = near_jsonrpc_primitives::types::query::RpcQueryRequest {
            block_reference: near_primitives::types::Finality::Final.into(),
            request: near_primitives::views::QueryRequest::ViewAccessKey {
                account_id: signer.account_id.clone(),
                public_key: signer.public_key.clone(),
            },
        };
        let response = self.near_client.call(request).await?;
        let block_hash = response.block_hash;
        let nonce = match response.kind {
            near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKey(k) => k.nonce + 1,
            _ => unreachable!(),
        };
        let request =
            near_jsonrpc_client::methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
                signed_transaction: near_primitives::transaction::SignedTransaction::call(
                    nonce,
                    signer.account_id.clone(),
                    self.engine_account_id.parse().unwrap(),
                    &signer,
                    0,
                    method_name,
                    args,
                    300_000_000_000_000,
                    block_hash,
                ),
            };
        let response = self.near_client.call(request).await?;

        Ok(response)
    }
}

#[derive(Debug)]
pub enum TransactionOutcome {
    Result(SubmitResult),
    Failure(near_primitives::errors::TxExecutionError),
}

#[derive(Debug, Serialize)]
pub struct Web3JsonRequest<'method, 'version, T> {
    jsonrpc: &'version str,
    method: &'method str,
    id: u32,
    params: T,
}

impl<'a> Web3JsonRequest<'a, 'static, Vec<String>> {
    pub fn from_method(id: u32, method: &'a EthMethod) -> Self {
        Self {
            jsonrpc: "2.0",
            method: method.name(),
            id,
            params: method.create_params(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Web3JsonResponse<T> {
    jsonrpc: String,
    id: u32,
    result: Option<T>,
    error: Option<Web3JsonResponseError>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Web3JsonResponseError {
    code: i64,
    data: serde_json::Value,
    message: String,
}

#[derive(Debug)]
pub enum ClientError {
    AuroraTransactionNotFound(H256),
    InvalidHex(hex::FromHexError),
    InvalidJson(String),
    ResponseKeyNotFound(String),
    NotJsonObject(serde_json::Value),
    NotJsonString(serde_json::Value),
    Rpc(Web3JsonResponseError),
    NearRpc(NearQueryError),
    NearContractCall(NearCallError),
    Reqwest(reqwest::Error),
}

impl From<hex::FromHexError> for ClientError {
    fn from(e: hex::FromHexError) -> Self {
        Self::InvalidHex(e)
    }
}

impl From<reqwest::Error> for ClientError {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

impl From<Web3JsonResponseError> for ClientError {
    fn from(e: Web3JsonResponseError) -> Self {
        Self::Rpc(e)
    }
}

impl From<NearQueryError> for ClientError {
    fn from(e: NearQueryError) -> Self {
        Self::NearRpc(e)
    }
}

impl From<NearCallError> for ClientError {
    fn from(e: NearCallError) -> Self {
        Self::NearContractCall(e)
    }
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("ClientError({})", self))
    }
}

impl std::error::Error for ClientError {}
