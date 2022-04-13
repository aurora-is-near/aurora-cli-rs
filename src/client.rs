use crate::eth_method::EthMethod;
use aurora_engine::parameters::SubmitResult;
use aurora_engine_transactions::{legacy::TransactionLegacy, EthTransactionKind};
use aurora_engine_types::{
    types::{Address, Wei},
    H256, U256,
};
use borsh::BorshDeserialize;
use near_jsonrpc_client::AsUrl;
use near_primitives::views;
use secp256k1::SecretKey;
use serde::{Deserialize, Serialize};

const NEAR_TRANSACTION_KEY: &str = "nearTransactionHash";

type NearQueryError =
    near_jsonrpc_client::errors::JsonRpcError<near_jsonrpc_primitives::types::query::RpcQueryError>;

pub struct AuroraClient<T> {
    inner: reqwest::Client,
    aurora_rpc: T,
    near_client: near_jsonrpc_client::JsonRpcClient,
}

impl<T: AsRef<str>> AuroraClient<T> {
    pub fn new<U: AsUrl>(aurora_rpc: T, near_rpc: U) -> Self {
        let inner = reqwest::Client::new();
        let near_client = near_jsonrpc_client::JsonRpcClient::connect(near_rpc);
        Self {
            inner,
            aurora_rpc,
            near_client,
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
        let near_tx_hex = near_tx_str.strip_prefix("0x").unwrap_or(near_tx_str);
        let near_tx_hash = hex::decode(near_tx_hex)?;

        let tx_status_request = near_jsonrpc_client::methods::tx::RpcTransactionStatusRequest {
            transaction_info:
                near_jsonrpc_primitives::types::transactions::TransactionInfo::TransactionId {
                    hash: near_tx_hash.as_slice().try_into().unwrap(),
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

    async fn near_view_call(
        &self,
        method_name: String,
        args: Vec<u8>,
    ) -> Result<views::CallResult, ClientError> {
        let request = near_jsonrpc_primitives::types::query::RpcQueryRequest {
            block_reference: near_primitives::types::Finality::Final.into(),
            request: near_primitives::views::QueryRequest::CallFunction {
                account_id: "aurora".parse().unwrap(),
                method_name,
                args: args.into(),
            },
        };
        let response = self.near_client.call(request).await?;

        match response.kind {
            near_jsonrpc_primitives::types::query::QueryResponseKind::CallResult(result) => {
                Ok(result)
            }
            near_jsonrpc_primitives::types::query::QueryResponseKind::LegacyError(e) => Err(
                ClientError::NearRpc(near_jsonrpc_client::errors::JsonRpcError::ServerError(
                    near_jsonrpc_client::errors::JsonRpcServerError::InternalError {
                        info: Some(e.error),
                    },
                )),
            ),
            _ => unreachable!(),
        }
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

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("ClientError({})", self))
    }
}

impl std::error::Error for ClientError {}
