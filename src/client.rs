use crate::eth_method::EthMethod;
use aurora_engine::parameters::{SubmitResult, TransactionStatus};
use aurora_engine_transactions::{legacy::TransactionLegacy, EthTransactionKind};
use aurora_engine_types::{
    types::{Address, Wei},
    H256, U256,
};
use borsh::{BorshDeserialize, BorshSerialize};
use libsecp256k1::SecretKey;
use near_jsonrpc_client::AsUrl;
use near_primitives::{transaction::Action, views};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const NEAR_TRANSACTION_KEY: &str = "nearTransactionHash";

type NearQueryError =
    near_jsonrpc_client::errors::JsonRpcError<near_jsonrpc_primitives::types::query::RpcQueryError>;
type NearCallError = near_jsonrpc_client::errors::JsonRpcError<
    near_jsonrpc_client::methods::broadcast_tx_commit::RpcTransactionError,
>;

pub struct AuroraClient {
    inner: reqwest::Client,
    aurora_rpc: String,
    near_client: near_jsonrpc_client::JsonRpcClient,
    engine_account_id: String,
    signer_key_path: Option<String>,
}

impl AuroraClient {
    pub fn new<U: AsUrl>(
        aurora_rpc: String,
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

    pub async fn request<'a, U: Serialize + Send + Sync>(
        &self,
        request: &Web3JsonRequest<'a, U>,
    ) -> Result<Web3JsonResponse<Value>, ClientError> {
        let resp = self
            .inner
            .post(&self.aurora_rpc)
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

        let value = response.result.as_ref().and_then(Value::as_str).unwrap();
        Ok(U256::from_str_radix(value, 16).unwrap())
    }

    pub async fn get_chain_id(&self) -> Result<u64, ClientError> {
        let method = EthMethod::GetChainId;
        let request = Web3JsonRequest::from_method(1, &method);
        let response = self.request(&request).await?;

        if let Some(e) = response.error {
            return Err(e.into());
        }

        let value = response.result.as_ref().and_then(Value::as_str).unwrap();
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

        let tx_hash = response.result.as_ref().and_then(Value::as_str).unwrap();
        let tx_hash_bytes = tx_hash
            .strip_prefix("0x")
            .and_then(|x| hex::decode(x).ok())
            .unwrap();
        Ok(H256::from_slice(&tx_hash_bytes))
    }

    pub async fn get_transaction_outcome(
        &self,
        tx_hash: H256,
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
        let near_rx_hex = near_tx_str.strip_prefix("0x").unwrap_or(near_tx_str);
        let near_receipt_id = hex::decode(near_rx_hex)?;

        self.get_near_receipt_outcome(near_receipt_id.as_slice().try_into().unwrap())
            .await
    }

    pub async fn get_near_receipt_outcome(
        &self,
        near_receipt_id: near_primitives::hash::CryptoHash,
    ) -> Result<TransactionOutcome, ClientError> {
        let mut receipt_id = near_receipt_id;
        let receiver_id: near_primitives::types::AccountId =
            self.engine_account_id.parse().unwrap();
        loop {
            let block_hash = {
                let request = near_jsonrpc_client::methods::block::RpcBlockRequest {
                    block_reference: near_primitives::types::Finality::Final.into(),
                };
                let response = self.near_client.call(request).await.unwrap();
                response.header.hash
            };
            let request = near_jsonrpc_client::methods::light_client_proof::RpcLightClientExecutionProofRequest {
                id: near_primitives::types::TransactionOrReceiptId::Receipt { receipt_id, receiver_id: receiver_id.clone() },
                light_client_head: block_hash,
            };
            let response = self.near_client.call(request).await.unwrap();
            match response.outcome_proof.outcome.status {
                views::ExecutionStatusView::SuccessValue(result) => {
                    let result = SubmitResult::try_from_slice(&result).unwrap();
                    break Ok(TransactionOutcome::Result(result));
                }
                views::ExecutionStatusView::Failure(e) => {
                    break Ok(TransactionOutcome::Failure(e));
                }
                views::ExecutionStatusView::SuccessReceiptId(id) => {
                    println!("Intermediate receipt_id: {id:?}");
                    receipt_id = id;
                }
                views::ExecutionStatusView::Unknown => {
                    panic!("Unknown receipt_id: {near_receipt_id:?}")
                }
            }
        }
    }

    pub async fn get_nep141_from_erc20(&self, erc20: Address) -> Result<String, ClientError> {
        let result = self
            .near_view_call("get_nep141_from_erc20".into(), erc20.as_bytes().to_vec())
            .await?;
        Ok(String::from_utf8_lossy(&result.result).into_owned())
    }

    pub async fn get_erc20_from_nep141(&self, nep141: &str) -> Result<Address, ClientError> {
        let args = aurora_engine::parameters::GetErc20FromNep141CallArgs {
            nep141: nep141.parse().unwrap(),
        };
        let result = self
            .near_view_call("get_erc20_from_nep141".into(), args.try_to_vec().unwrap())
            .await?;
        Ok(Address::try_from_slice(&result.result).unwrap())
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

    pub async fn near_view_call(
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

    pub async fn near_deploy_contract(
        &self,
        wasm_code: Vec<u8>,
    ) -> Result<views::FinalExecutionOutcomeView, ClientError> {
        self.near_broadcast_tx(
            vec![Action::DeployContract(
                near_primitives::transaction::DeployContractAction { code: wasm_code },
            )],
            None,
        )
        .await
    }

    pub async fn near_contract_call(
        &self,
        method_name: String,
        args: Vec<u8>,
    ) -> Result<views::FinalExecutionOutcomeView, ClientError> {
        self.near_broadcast_tx(
            vec![Action::FunctionCall(
                near_primitives::transaction::FunctionCallAction {
                    method_name,
                    args,
                    gas: 300_000_000_000_000,
                    deposit: 0,
                },
            )],
            None,
        )
        .await
    }

    pub async fn near_contract_call_with_nonce(
        &self,
        method_name: String,
        args: Vec<u8>,
        nonce_override: u64,
    ) -> Result<views::FinalExecutionOutcomeView, ClientError> {
        self.near_broadcast_tx(
            vec![Action::FunctionCall(
                near_primitives::transaction::FunctionCallAction {
                    method_name,
                    args,
                    gas: 300_000_000_000_000,
                    deposit: 0,
                },
            )],
            Some(nonce_override),
        )
        .await
    }

    async fn near_broadcast_tx(
        &self,
        actions: Vec<Action>,
        nonce_override: Option<u64>,
    ) -> Result<views::FinalExecutionOutcomeView, ClientError> {
        let path = self
            .signer_key_path
            .as_ref()
            .map(std::path::Path::new)
            .expect("Signer path must be provided to use this functionality");
        let signer = crate::utils::read_key_file(path).unwrap();
        let request = near_jsonrpc_primitives::types::query::RpcQueryRequest {
            block_reference: near_primitives::types::Finality::Final.into(),
            request: near_primitives::views::QueryRequest::ViewAccessKey {
                account_id: signer.account_id.clone(),
                public_key: signer.public_key.clone(),
            },
        };
        let response = self.near_client.call(request).await?;
        let block_hash = response.block_hash;
        let nonce = nonce_override.unwrap_or_else(|| match response.kind {
            near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKey(k) => k.nonce + 1,
            _ => unreachable!(),
        });
        let request =
            near_jsonrpc_client::methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
                signed_transaction: near_primitives::transaction::SignedTransaction::from_actions(
                    nonce,
                    signer.account_id.clone(),
                    self.engine_account_id.parse().unwrap(),
                    &signer,
                    actions,
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
pub struct Web3JsonRequest<'a, T> {
    jsonrpc: &'a str,
    method: &'a str,
    id: u32,
    params: T,
}

impl<'a> Web3JsonRequest<'a, Vec<String>> {
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
    data: Value,
    message: String,
}

#[derive(Debug)]
pub enum ClientError {
    AuroraTransactionNotFound(H256),
    InvalidHex(hex::FromHexError),
    InvalidJson(String),
    ResponseKeyNotFound(String),
    NotJsonObject(Value),
    NotJsonString(Value),
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
        f.write_fmt(format_args!("ClientError({self})"))
    }
}

impl std::error::Error for ClientError {}
