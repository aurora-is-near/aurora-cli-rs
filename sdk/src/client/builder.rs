use crate::client::Client;
use near_jsonrpc_client::{JsonRpcClient, NEAR_MAINNET_RPC_URL, NEAR_TESTNET_RPC_URL};
use near_primitives::types::AccountId;
use std::path::PathBuf;
use std::time::Duration;

const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

pub struct ClientBuilder {
    url: String,
    read_timeout: Duration,
    connect_timeout: Duration,
    engine_account_id: Option<AccountId>,
    secret_key_path: Option<PathBuf>,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            url: NEAR_MAINNET_RPC_URL.to_string(),
            read_timeout: DEFAULT_READ_TIMEOUT,
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            engine_account_id: None,
            secret_key_path: None,
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

    pub fn with_read_timeout(mut self, timeout: u64) -> Self {
        self.read_timeout = Duration::from_secs(timeout);
        self
    }

    pub fn with_connect_timeout(mut self, timeout: u64) -> Self {
        self.connect_timeout = Duration::from_secs(timeout);
        self
    }

    pub fn with_engine_account_id(mut self, engine_account_id: impl AsRef<str>) -> Self {
        self.engine_account_id = Some(engine_account_id.as_ref().parse().unwrap());
        self
    }

    pub fn build(self) -> Client {
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

        Client {
            client,
            engine_account_id: self.engine_account_id,
            signer_key_path: self.secret_key_path,
        }
    }
}
