mod client;
mod eth_method;
mod utils;

const MAINNET_ENDPOINT: &str = "https://mainnet.aurora.dev/";

use client::{AuroraClient, Web3JsonRequest};
use eth_method::EthMethod;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AuroraClient::new(MAINNET_ENDPOINT);
    let tx = "bcb429aa180ef52f7c47efcef0a06b89e14f7a1b83316ee8e565c093adb532ca";
    let method = EthMethod::GetTransactionReceipt(utils::hex_to_arr32(tx));
    let request = Web3JsonRequest::from_method(1, &method);
    let resp = client.request(&request).await?;
    println!("{:#?}", resp);

    Ok(())
}
