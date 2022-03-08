use crate::eth_method::EthMethod;
use aurora_engine_transactions::{legacy::TransactionLegacy, EthTransactionKind};
use aurora_engine_types::{
    types::{Address, Wei},
    H256, U256,
};
use secp256k1::SecretKey;
use serde::{Deserialize, Serialize};

pub struct AuroraClient<T> {
    inner: reqwest::Client,
    rpc: T,
}

impl<T: AsRef<str>> AuroraClient<T> {
    pub fn new(rpc: T) -> Self {
        let inner = reqwest::Client::new();
        Self { inner, rpc }
    }

    pub async fn request<'a, 'b, U: Serialize>(
        &self,
        request: &Web3JsonRequest<'a, 'b, U>,
    ) -> Result<Web3JsonResponse<serde_json::Value>, ClientError> {
        let resp = self
            .inner
            .post(self.rpc.as_ref())
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

    pub async fn transfer(
        &self,
        target: Address,
        amount: Wei,
        signer: &SecretKey,
        chain_id: u64,
        nonce: U256,
    ) -> Result<H256, ClientError> {
        let tx = TransactionLegacy {
            nonce,
            gas_price: U256::zero(),
            gas_limit: U256::from(u64::MAX),
            to: Some(target),
            value: amount,
            data: Vec::new(),
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
pub struct Web3JsonResponse<T> {
    jsonrpc: String,
    id: u32,
    result: Option<T>,
    error: Option<Web3JsonResponseError>,
}

#[derive(Debug, Deserialize)]
pub struct Web3JsonResponseError {
    code: i64,
    data: serde_json::Value,
    message: String,
}

#[derive(Debug)]
pub enum ClientError {
    InvalidJson(String),
    Rpc(Web3JsonResponseError),
    Reqwest(reqwest::Error),
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

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("ClientError({})", self))
    }
}

impl std::error::Error for ClientError {}
