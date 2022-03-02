use serde::{Deserialize, Serialize};

const MAINNET_ENDPOINT: &str = "https://mainnet.aurora.dev/";

fn hex_to_arr32(h: &str) -> [u8; 32] {
    let mut output = [0u8; 32];
    hex::decode_to_slice(h, &mut output).unwrap();
    output
}

enum EthMethod {
    EthGetTransactionReceipt([u8; 32]),
}

impl EthMethod {
    fn to_params(&self) -> Vec<String> {
        match &self {
            Self::EthGetTransactionReceipt(tx_hash) => {
                vec![format!("0x{}", hex::encode(tx_hash))]
            }
        }
    }
}

impl AsRef<str> for EthMethod {
    fn as_ref(&self) -> &str {
        match &self {
            Self::EthGetTransactionReceipt(_) => "eth_getTransactionReceipt",
        }
    }
}

#[derive(Debug, Serialize)]
struct Web3JsonRequest<'method, 'version, T> {
    jsonrpc: &'version str,
    method: &'method str,
    id: u32,
    params: T,
}

impl<'a> Web3JsonRequest<'a, 'static, Vec<String>> {
    fn from_method(id: u32, method: &'a EthMethod) -> Self {
        Self {
            jsonrpc: "2.0",
            method: method.as_ref(),
            id,
            params: method.to_params(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct Web3JsonResponse<T> {
    jsonrpc: String,
    id: u32,
    result: T,
}

struct AuroraClient<T> {
    inner: reqwest::Client,
    rpc: T,
}

impl<T: AsRef<str>> AuroraClient<T> {
    fn new(rpc: T) -> Self {
        let inner = reqwest::Client::new();
        Self { inner, rpc }
    }

    async fn request<'a, 'b, U: Serialize>(
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AuroraClient::new(MAINNET_ENDPOINT);
    let tx = "bcb429aa180ef52f7c47efcef0a06b89e14f7a1b83316ee8e565c093adb532ca";
    let method = EthMethod::EthGetTransactionReceipt(hex_to_arr32(tx));
    let request = Web3JsonRequest::from_method(1, &method);
    let resp = client.request(&request).await?;
    println!("{:#?}", resp);

    Ok(())
}
