use aurora_sdk_rs::near;
use near_crypto::{InMemorySigner, Signer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let signer = signer()?;
    let workspace = near::workspace::Workspace::new("some-addr", None, signer.clone())?;

    let hash = workspace
        .call(&"c.aurora".parse()?, "")
        .signer(signer.clone())
        .transact_async()
        .await?;

    println!("Transaction hash: {hash:?}");

    Ok(())
}

fn signer() -> anyhow::Result<InMemorySigner> {
    let signer = std::env::var("NEAR_KEY_PATH")
        .ok()
        .as_ref()
        .map(std::path::Path::new)
        .ok_or_else(|| {
            anyhow::anyhow!("Path to the key file must be provided to use this functionality")
        })
        .and_then(|path| InMemorySigner::from_file(path).map_err(Into::into))?;

    match signer {
        Signer::Empty(_) => panic!("Signer must not be empty"),
        Signer::InMemory(signer) => Ok(signer),
    }
}
