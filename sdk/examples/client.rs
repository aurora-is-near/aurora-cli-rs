use std::time::Duration;

use aurora_sdk_rs::{
    client::broadcast::{Async, Sync},
    read::ReadClient,
    ClientBuilder,
};
use near_crypto::{InMemorySigner, KeyType};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = ClientBuilder::new("aurora".parse().unwrap(), signer().unwrap())
        .with_connect_timeout(Duration::from_secs(30))
        .with_read_timeout(Duration::from_secs(30))
        .build::<Async>()?;
    let response = client.get_chain_id().await.unwrap();
    println!("chain id: {:?}", response);

    let owner = client.get_owner().await.unwrap();
    println!("owner: {:?}", owner);

    let version = client.get_version().await.unwrap();
    println!("version: {:?}", version);

    let nonce = client.get_nonce().await.unwrap();
    println!("nonce: {:?}", nonce);

    let client = client.switch::<Sync>();

    let client = client.with_engine("some.aurora".parse().unwrap());

    let client = client.with_signer(InMemorySigner::from_random(
        "some.random".parse().unwrap(),
        KeyType::ED25519,
    ));

    let response = client
        .get_transaction_status(
            "3dbhsJA7eDGFPsQRqWk77DaCCS48j29kdjXpk8h2nvuy"
                .parse()
                .unwrap(),
        )
        .await
        .unwrap();

    println!(
        "response: {}",
        serde_json::to_string_pretty(&response).unwrap()
    );

    Ok(())
}

fn signer() -> anyhow::Result<InMemorySigner> {
    std::env::var("NEAR_KEY_PATH")
        .ok()
        .as_ref()
        .map(std::path::Path::new)
        .ok_or_else(|| {
            anyhow::anyhow!("Path to the key file must be provided to use this functionality")
        })
        .and_then(|path| InMemorySigner::from_file(path).map_err(Into::into))
}
