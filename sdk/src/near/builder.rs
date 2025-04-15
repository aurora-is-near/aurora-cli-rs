use crate::near::Client;
use near_jsonrpc_client::{JsonRpcClient, NEAR_MAINNET_RPC_URL, NEAR_TESTNET_RPC_URL};
use std::time::Duration;

use super::broadcast::{self, Broadcast};

const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

pub struct ClientBuilder {
    url: String,
    read_timeout: Duration,
    connect_timeout: Duration,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            url: NEAR_MAINNET_RPC_URL.to_string(),
            read_timeout: DEFAULT_READ_TIMEOUT,
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
        }
    }

    pub fn mainnet(mut self) -> Self {
        self.url = NEAR_MAINNET_RPC_URL.to_string();
        self
    }

    pub fn testnet(mut self) -> Self {
        self.url = NEAR_TESTNET_RPC_URL.to_string();
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }

    pub fn with_read_timeout(mut self, timeout: Duration) -> Self {
        self.read_timeout = timeout;
        self
    }

    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    pub fn build_sync(self) -> anyhow::Result<Client<broadcast::Sync>> {
        self.build()
    }

    pub fn build_async(self) -> anyhow::Result<Client<broadcast::Async>> {
        self.build()
    }

    fn build<B: Broadcast>(self) -> anyhow::Result<Client<B>> {
        let headers = reqwest::header::HeaderMap::from_iter([(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        )]);
        let client = reqwest::Client::builder()
            .timeout(self.read_timeout)
            .connect_timeout(self.connect_timeout)
            .default_headers(headers)
            .build()
            .map(JsonRpcClient::with)
            .expect("couldn't create json rpc client");
        let client = client.connect(self.url);

        Ok(Client::<B> {
            client,
            _broadcast: std::marker::PhantomData,
        })
    }
}
