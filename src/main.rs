mod client;
mod eth_method;
mod utils;

const AURORA_MAINNET_ENDPOINT: &str = "https://mainnet.aurora.dev/";
const NEAR_MAINNET_ENDPOINT: &str = "https://archival-rpc.mainnet.near.org";

use aurora_engine_types::types::{Address, Wei};
use client::{AuroraClient, ClientError};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AuroraClient::new(AURORA_MAINNET_ENDPOINT, NEAR_MAINNET_ENDPOINT);
    let sk = secp256k1::SecretKey::parse(&[1; 32]).unwrap();
    let source = utils::address_from_secret_key(&sk);
    let target = Address::decode("cc5a584f545b2ca3ebacc1346556d1f5b82b8fc6").unwrap();
    let nonce = client.get_nonce(source).await?;
    let chain_id = client.get_chain_id().await?;
    let tx_hash = client
        .transfer(target, Wei::zero(), &sk, chain_id, nonce)
        .await
        .unwrap();

    // Wait for the RPC to pick up the transaction
    loop {
        match client
            .get_transaction_outcome(tx_hash, "relay.aurora")
            .await
        {
            Ok(result) => {
                println!("{:?}", result);
                break;
            }
            Err(ClientError::AuroraTransactionNotFound(_)) => {
                continue;
            }
            Err(other) => return Err(Box::new(other) as Box<dyn std::error::Error>),
        }
    }

    Ok(())
}
