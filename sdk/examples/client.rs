use std::{str::FromStr, time::Duration};

use aurora_engine_types::account_id::AccountId;
use aurora_sdk_rs::{
    aurora::{
        self,
        operations::{CallOperation, ViewOperation},
    },
    ClientBuilder,
};
use near_crypto::InMemorySigner;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = ClientBuilder::new()
        .testnet()
        .with_read_timeout(Duration::from_secs(30))
        .with_connect_timeout(Duration::from_secs(30))
        .build_async()?;

    let aurora_cli = aurora::Aurora::new(client, signer()?, AccountId::from_str("aurora")?);
    let call_result = aurora_cli
        .view(aurora::operations::GetLatestReleaseHash {})
        .await?;
    let release_hash = aurora::operations::GetLatestReleaseHash::parse(call_result)?;

    println!("Latest release hash: {}", release_hash);

    let call_result = aurora_cli
        .call(aurora::operations::SetEthConnectorContractAccount {
            deposit: Default::default(),
            contract_account: AccountId::from_str("some_eth_conn")?,
        })
        .await?; // in async mode it's CrpytoHash

    let call_result = aurora_cli
        .into_sync()
        .call(aurora::operations::SetEthConnectorContractAccount {
            deposit: Default::default(),
            contract_account: AccountId::from_str("some_eth_conn")?,
        })
        .await?;

    let some_result = aurora::operations::SetEthConnectorContractAccount::parse(call_result)?;

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
