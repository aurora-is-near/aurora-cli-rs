use aurora_sdk_rs::read::ReadClient;
use aurora_sdk_rs::ClientBuilder;

#[tokio::main]
async fn main() {
    let client = ClientBuilder::new()
        .with_engine_account_id("aurora")
        .build();
    let response = client.get_chain_id().await.unwrap();
    println!("chain id: {:?}", response);

    let owner = client.get_owner().await.unwrap();
    println!("owner: {:?}", owner);

    let version = client.get_version().await.unwrap();
    println!("version: {:?}", version);

    let nonce = client
        .get_nonce("0xdC2a061b5c68F97F96a2064792DeAE0F79c78C40")
        .await
        .unwrap();
    println!("nonce: {:?}", nonce);

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
}
