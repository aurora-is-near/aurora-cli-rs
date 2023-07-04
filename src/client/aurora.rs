use aurora_engine_transactions::{legacy::TransactionLegacy, EthTransactionKind};
use aurora_engine_types::{
    account_id::AccountId,
    types::{Address, Wei},
    H256, U256,
};
use libsecp256k1::SecretKey;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Formatter;
use thiserror::Error;

use crate::eth_method::EthMethod;

use super::{ClientError, TransactionOutcome};

const NEAR_TRANSACTION_KEY: &str = "nearTransactionHash";

pub struct AuroraClient {
    inner: reqwest::Client,
    url: String,
    near_url: String,
    engine_account_id: AccountId,
}

impl AuroraClient {
    #[allow(clippy::used_underscore_binding)]
    pub fn new(url: &str, near_url: &str, engine_account: &str) -> Self {
        let inner = reqwest::Client::new();
        Self {
            inner,
            url: url.to_string(),
            near_url: near_url.to_string(),
            engine_account_id: engine_account.parse().expect("couldn't parse engine id"),
        }
    }

    pub async fn request<'a, S: Serialize + Send + Sync>(
        &self,
        request: &Web3JsonRequest<'a, S>,
    ) -> anyhow::Result<Web3JsonResponse<Value>> {
        let resp = self.inner.post(&self.url).json(request).send().await?;
        // TODO: parse information from headers too (eg x-request-id)
        let full = resp.bytes().await?;

        serde_json::from_slice(&full).map_err(|_| {
            let text = match String::from_utf8_lossy(&full) {
                std::borrow::Cow::Owned(s) => s,
                std::borrow::Cow::Borrowed(s) => s.to_owned(),
            };
            ClientError::InvalidJson(text).into()
        })
    }

    pub async fn get_nonce(&self, address: Address) -> anyhow::Result<U256> {
        let method = EthMethod::GetTransactionCount(address);
        let request = Web3JsonRequest::from_method(1, &method);
        let response = self.request(&request).await?;

        if let Some(e) = response.error {
            return Err(e.into());
        }

        let value = response.result.as_ref().and_then(Value::as_str).unwrap();
        Ok(U256::from_str_radix(value, 16).unwrap())
    }

    #[cfg(feature = "simple")]
    pub async fn get_balance(&self, address: Address) -> anyhow::Result<U256> {
        let method = EthMethod::GetBalance(address);
        let request = Web3JsonRequest::from_method(1, &method);
        let response = self.request(&request).await?;

        if let Some(e) = response.error {
            return Err(e.into());
        }

        let value = response.result.as_ref().and_then(Value::as_str).unwrap();
        Ok(U256::from_str_radix(value, 16).unwrap())
    }

    #[cfg(feature = "simple")]
    pub async fn get_code(&self, address: Address) -> anyhow::Result<String> {
        let method = EthMethod::GetCode(address);
        let request = Web3JsonRequest::from_method(1, &method);
        let response = self.request(&request).await?;

        if let Some(e) = response.error {
            return Err(e.into());
        }

        Ok(response
            .result
            .as_ref()
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap())
    }

    pub async fn get_chain_id(&self) -> anyhow::Result<u64> {
        let method = EthMethod::GetChainId;
        let request = Web3JsonRequest::from_method(1, &method);
        let response = self.request(&request).await?;

        if let Some(e) = response.error {
            return Err(e.into());
        }

        let value = response.result.as_ref().and_then(Value::as_str).unwrap();
        Ok(value.parse().unwrap())
    }

    pub(crate) async fn send_and_wait_transaction(
        &self,
        sk: &SecretKey,
        to: Option<Address>,
        amount: Wei,
        input: Vec<u8>,
    ) -> anyhow::Result<()> {
        let source = crate::utils::address_from_secret_key(sk)?;
        println!("FROM {source:?}");

        let nonce = self.get_nonce(source).await?;
        let chain_id = self.get_chain_id().await?;
        let tx_hash = self
            .send_eth_transaction(to, amount, sk, chain_id, nonce, input)
            .await?;

        // Wait for the RPC to pick up the transaction
        loop {
            match self.get_transaction_outcome(tx_hash).await {
                Ok(result) => {
                    println!("{result:?}");
                    break;
                }
                Err(e) => match e.downcast_ref::<ClientError>() {
                    Some(ClientError::AuroraTransactionNotFound(_)) => continue,
                    _ => anyhow::bail!(e),
                },
            }
        }

        Ok(())
    }

    /// Send Aurora Engine transaction.
    pub async fn send_eth_transaction(
        &self,
        target: Option<Address>,
        amount: Wei,
        signer: &SecretKey,
        chain_id: u64,
        nonce: U256,
        data: Vec<u8>,
    ) -> anyhow::Result<H256> {
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
            return Err(ClientError::AuroraRpc(e).into());
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
    ) -> anyhow::Result<TransactionOutcome> {
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
        let near_rx_hex = near_tx_str.strip_prefix("0x").unwrap_or(near_tx_str);
        let near_receipt_id = hex::decode(near_rx_hex)?;
        let near_client =
            super::NearClient::new(&self.near_url, self.engine_account_id.as_ref(), None, false);

        near_client
            .get_receipt_outcome(near_receipt_id.as_slice().try_into().unwrap())
            .await
    }
}

#[derive(Debug, Serialize)]
pub struct Web3JsonRequest<'a, T> {
    jsonrpc: &'static str,
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
            params: method.params(),
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

#[derive(Debug, Error, Deserialize)]
#[allow(dead_code)]
pub struct Web3JsonResponseError {
    code: i64,
    data: Value,
    message: String,
}

impl std::fmt::Display for Web3JsonResponseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("code: {}, msg: {}", self.code, self.message))
    }
}
