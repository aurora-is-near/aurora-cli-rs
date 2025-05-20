use std::str::FromStr;

use aurora_sdk_rs::{aurora::ContractMethod, near::client::Client};
use near_crypto::Signer;
use near_token::NearToken;
use near_workspaces::{Account, Contract, Worker, network::Sandbox};

pub async fn setup_sandbox() -> anyhow::Result<(Worker<Sandbox>, Client, Contract, Account)> {
    let worker = near_workspaces::sandbox().await?;

    let wasm_path = "tests/res/nep141.wasm";
    let wasm = std::fs::read(wasm_path).map_err(|e|
        anyhow::anyhow!("Failed to read the WASM file at {wasm_path}: {e}. Please ensure the file exists and the path is correct.")
    )?;

    let contract = worker.dev_deploy(&wasm).await?;

    // Most NEP-141 contracts require initialization.
    // Example: Initialize with owner_id and total_supply.
    // Adjust the method name ("new") and arguments based on your specific contract.
    let owner_account = worker.root_account()?;
    // Define total_supply using NearToken, then convert to string for JSON args
    let total_supply = NearToken::from_near(1);
    let outcome = contract
        .call("new_default_meta") // Common init function, adjust if needed
        .args_json(serde_json::json!({
            "owner_id": owner_account.id(),
            "total_supply": total_supply.as_yoctonear().to_string(),
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let root = worker.root_account()?; // Use root signer for the workspace client
    let client = Client::new(
        worker.rpc_addr().as_str(),
        None,
        signer_from_secret(root.id(), root.secret_key()),
    )?;

    // Return the owner account along with worker, client and contract
    Ok((worker, client, contract, owner_account))
}

fn signer_from_secret(
    account_id: &near_workspaces::AccountId,
    sk: &near_workspaces::types::SecretKey,
) -> near_crypto::InMemorySigner {
    let signer = near_crypto::InMemorySigner::from_secret_key(
        account_id.to_owned(),
        near_crypto::SecretKey::from_str(&sk.to_string()).unwrap(),
    );

    match signer {
        Signer::Empty(_) => panic!("Signer should not be empty"),
        Signer::InMemory(signer) => signer,
    }
}

struct FtTotalSupply {}

impl ContractMethod for FtTotalSupply {
    type Response = String;

    fn method_name(&self) -> &str {
        "ft_total_supply"
    }
}

#[tokio::test]
async fn test_total_supply() -> anyhow::Result<()> {
    let (_, client, contract, _) = setup_sandbox().await?;
    let aurora_client = aurora_sdk_rs::aurora::client::Client::new(client);
    let total_supply = aurora_client.call(contract.id(), FtTotalSupply {}).await?;

    assert_eq!(
        total_supply,
        NearToken::from_near(1).as_yoctonear().to_string()
    );
    Ok(())
}
