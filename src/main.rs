mod client;
mod eth_method;
mod utils;

const MAINNET_ENDPOINT: &str = "https://mainnet.aurora.dev/";

use aurora_engine_types::types::{Address, Wei};
use client::{AuroraClient, Web3JsonRequest};
use eth_method::EthMethod;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AuroraClient::new(MAINNET_ENDPOINT);
    let sk = secp256k1::SecretKey::parse(&[1; 32]).unwrap();
    let source = utils::address_from_secret_key(&sk);
    let target = Address::decode("cc5a584f545b2ca3ebacc1346556d1f5b82b8fc6").unwrap();
    let nonce = client.get_nonce(source).await.unwrap();
    let chain_id = client.get_chain_id().await.unwrap();
    let tx_hash = client
        .transfer(target, Wei::zero(), &sk, chain_id, nonce)
        .await
        .unwrap();
    let method = EthMethod::GetTransactionReceipt(tx_hash);
    let request = Web3JsonRequest::from_method(1, &method);
    let resp = client.request(&request).await?;
    println!("{:#?}", resp);

    Ok(())
}
