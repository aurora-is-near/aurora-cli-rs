use aurora_engine_types::parameters::connector::FungibleTokenMetadata;
use aurora_sdk_rs::near::operations::Function;
use aurora_sdk_rs::near::query::JsonIntoResult;
use aurora_sdk_rs::near::{client::Client, operations::MAX_GAS};
use helpers::{setup_sandbox, signer_from_secret};
use near_contract_standards::storage_management::StorageBalanceBounds;
use near_primitives::views::{AccessKeyList, AccessKeyPermissionView};
use near_workspaces::types::NearToken;

mod helpers;

#[tokio::test]
async fn test_view_call_sandbox() -> anyhow::Result<()> {
    let (_, client, contract, _) = setup_sandbox().await?;

    // Call the ft_metadata view function
    let metadata: FungibleTokenMetadata = client
        .view(contract.id(), "ft_metadata")
        .await?
        .into_result()?;

    // Assertions using extracted values
    assert_eq!(metadata.spec, "ft-1.0.0");
    assert_eq!(metadata.name, "Example NEAR fungible token");
    assert_eq!(metadata.symbol, "EXAMPLE");
    assert_eq!(metadata.decimals, 24);

    Ok(())
}

#[tokio::test]
async fn test_transaction_sandbox() -> anyhow::Result<()> {
    let (worker, _, contract, owner_account) = setup_sandbox().await?;

    let receiver_account = worker.dev_create_account().await?;

    let client = Client::new(
        worker.rpc_addr().as_str(),
        None,
        signer_from_secret(owner_account.id(), owner_account.secret_key())?,
    )?;

    // Query storage balance bounds
    let bounds: StorageBalanceBounds = client
        .view(contract.id(), "storage_balance_bounds")
        .await?
        .into_result()?;
    let minimum_deposit = bounds.min;

    // Define amounts using NearToken
    let initial_total_supply = NearToken::from_near(1);
    let transfer_amount = NearToken::from_millinear(10); // 0.1 NEAR
    let one_yocto = NearToken::from_yoctonear(1);

    // Register the receiver account by calling storage_deposit
    let storage_outcome = client
        .call(contract.id(), "storage_deposit")
        .args_json(serde_json::json!({
            "account_id": receiver_account.id(),
            "registration_only": true
        }))?
        .deposit(minimum_deposit)
        .max_gas()
        .priority_fee(100)
        .transact()
        .await?;
    storage_outcome.assert_success();

    // Perform the ft_transfer call
    let outcome = client
        .call(contract.id(), "ft_transfer")
        .args_json(serde_json::json!({
            "receiver_id": receiver_account.id(),
            "amount": transfer_amount.as_yoctonear().to_string()
        }))?
        .deposit(one_yocto)
        .max_gas()
        .transact()
        .await?;
    outcome.assert_success();

    // Verify balances using NearToken
    // Owner balance check
    let owner_balance: NearToken = client
        .view(contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": owner_account.id()
        }))?
        .await?
        .into_result()?;
    // Perform arithmetic with NearToken
    let expected_owner_balance = initial_total_supply.checked_sub(transfer_amount).unwrap();
    assert_eq!(owner_balance, expected_owner_balance);

    // Receiver balance check
    let receiver_balance: NearToken = client
        .view(contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": receiver_account.id()
        }))?
        .await?
        .into_result()?;
    assert_eq!(receiver_balance, transfer_amount);

    Ok(())
}

#[tokio::test]
async fn test_ft_balance_of_sandbox() -> anyhow::Result<()> {
    let (worker, client, contract, owner_account) = setup_sandbox().await?;

    let expected_balance = NearToken::from_near(1);

    let balance: NearToken = client
        .view(contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": owner_account.id()
        }))?
        .await?
        .into_result()?;

    assert_eq!(balance, expected_balance);

    // Check the balance of a different, empty account
    let other_account = worker.dev_create_account().await?;
    let balance_other: NearToken = client
        .view(contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": other_account.id()
        }))?
        .await?
        .into_result()?;
    assert_eq!(balance_other, NearToken::from_yoctonear(0));

    Ok(())
}

#[tokio::test]
async fn test_view_access_keys_sandbox() -> anyhow::Result<()> {
    let (_, client, _, owner_account) = setup_sandbox().await?;

    // Get the owner account's public key for comparison
    let owner_public_key = owner_account.secret_key().public_key();

    // Call view_access_keys for the owner account
    let access_key_list: AccessKeyList = client.view_access_keys(owner_account.id()).await?;

    // Assertions
    assert!(
        !access_key_list.keys.is_empty(),
        "Access key list should not be empty for the owner account"
    );

    // Find the owner's key in the list and check its type
    let owner_key_info = access_key_list
        .keys
        .iter()
        .find(|key_info| key_info.public_key.to_string() == owner_public_key.to_string())
        .expect("Owner's public key not found in access key list");

    // Assert that the owner key has FullAccess permission
    assert!(matches!(
        owner_key_info.access_key.permission,
        AccessKeyPermissionView::FullAccess
    ));

    println!(
        "Successfully verified access key for: {}",
        owner_account.id()
    );
    println!("Key: {owner_key_info:?}");

    Ok(())
}

#[tokio::test]
async fn test_batch_transaction_sandbox() -> anyhow::Result<()> {
    let (worker, _, contract, owner_account) = setup_sandbox().await?;

    // Create a receiver account that needs registration and tokens
    let receiver_account = worker.dev_create_account().await?;

    // Initialize Workspace with the owner's signer who pays for the tx and owns the tokens
    let client = Client::new(
        worker.rpc_addr().as_str(),
        None,
        signer_from_secret(owner_account.id(), owner_account.secret_key())?,
    )?;

    // --- Get Minimum Deposit for Storage ---
    let bounds: StorageBalanceBounds = client
        .view(contract.id(), "storage_balance_bounds")
        .await?
        .into_result()?;
    let minimum_deposit = bounds.min;

    // --- Define Transfer Details ---
    let transfer_amount = NearToken::from_millinear(5); // 0.05 NEAR
    let one_yocto = NearToken::from_yoctonear(1);

    // --- Build and Send Batch Transaction ---
    // The batch targets the contract where the actions (calls) will happen.
    // The signer (owner_account) pays for the gas and provides attached deposits.
    let batch_outcome = client
        .batch(contract.id()) // Target the NEP-141 contract
        // Action 1: Call storage_deposit for the receiver
        .call(
            Function::new("storage_deposit")
                .args_json(serde_json::json!({
                    "account_id": receiver_account.id(),
                    "registration_only": true
                }))?
                .deposit(minimum_deposit) // Attach NEAR deposit for storage cost
                .gas(MAX_GAS.as_gas() / 2),
        )
        // Action 2: Call ft_transfer from owner to receiver
        .call(
            Function::new("ft_transfer")
                .args_json(serde_json::json!({
                    "receiver_id": receiver_account.id(),
                    "amount": transfer_amount.as_yoctonear().to_string()
                }))?
                .deposit(one_yocto) // Attach 1 yoctoNEAR for the transfer standard
                .gas(MAX_GAS.as_gas() / 2),
        )
        .priority_fee(1000)
        .transact()
        .await?;

    batch_outcome.assert_success();

    // --- Verify Receiver's Balance ---
    let receiver_balance: NearToken = client
        .view(contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": receiver_account.id()
        }))?
        .await?
        .into_result()?;

    assert_eq!(
        receiver_balance, transfer_amount,
        "Receiver balance should match the transferred amount after batch"
    );

    Ok(())
}
