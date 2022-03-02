use crate::eth_method::EthMethod;
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
    ) -> Result<Web3JsonResponse<serde_json::Value>, reqwest::Error> {
        let resp = self
            .inner
            .post(self.rpc.as_ref())
            .json(request)
            .send()
            .await?;
        Ok(resp.json().await.unwrap())
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
    result: T,
}
