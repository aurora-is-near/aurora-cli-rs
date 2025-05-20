use aurora_sdk_rs::near::client::Client;
use aurora_sdk_rs::near::operations::Function;
use helpers::{setup_sandbox, signer_from_secret};
use near_primitives::views::{AccessKeyList, AccessKeyPermissionView};
use near_workspaces::types::NearToken;

mod helpers;

#[tokio::test]
async fn test_view_call_sandbox() -> anyhow::Result<()> {
    let (_, client, contract, _) = setup_sandbox().await?;

    // Call the ft_metadata view function
    let result = client.view(contract.id(), "ft_metadata").await?;

    // Deserialize into serde_json::Value
    let metadata_val: serde_json::Value = serde_json::from_slice(&result.result)?;

    // Extract values manually from the JSON Value
    let spec = metadata_val["spec"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'spec' in metadata"))?;
    let name = metadata_val["name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'name' in metadata"))?;
    let symbol = metadata_val["symbol"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'symbol' in metadata"))?;
    let decimals = metadata_val["decimals"]
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'decimals' in metadata"))?
        as u8;
    // Optional fields can be handled similarly if needed
    // let icon = metadata_val["icon"].as_str();

    // Assertions using extracted values
    assert_eq!(spec, "ft-1.0.0");
    assert_eq!(name, "Example NEAR fungible token");
    assert_eq!(symbol, "EXAMPLE");
    assert_eq!(decimals, 24);

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
    let bounds_result = client.view(contract.id(), "storage_balance_bounds").await?;
    // Deserialize into serde_json::Value
    let bounds_val: serde_json::Value = serde_json::from_slice(&bounds_result.result)?;
    // Extract min bound string
    let min_bound_str = bounds_val["min"].as_str().ok_or_else(|| {
        anyhow::anyhow!("Missing or invalid 'min' in storage_balance_bounds response")
    })?;
    // Parse the min bound string to u128, then create NearToken
    let min_bound_u128 = min_bound_str.parse::<u128>().map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse min bound string '{}' to u128: {}",
            min_bound_str,
            e
        )
    })?;
    let minimum_deposit = NearToken::from_yoctonear(min_bound_u128);

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
    let owner_balance_result = client
        .view(contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": owner_account.id()
        }))?
        .await?;
    let owner_balance_str: String = serde_json::from_slice(&owner_balance_result.result)?;
    let owner_balance_u128 = owner_balance_str.parse::<u128>().map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse owner balance string '{}' to u128: {}",
            owner_balance_str,
            e
        )
    })?;
    let owner_balance = NearToken::from_yoctonear(owner_balance_u128);
    // Perform arithmetic with NearToken
    let expected_owner_balance = initial_total_supply.checked_sub(transfer_amount).unwrap();
    assert_eq!(owner_balance, expected_owner_balance);

    // Receiver balance check
    let receiver_balance_result = client
        .view(contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": receiver_account.id()
        }))?
        .await?;
    let receiver_balance_str: String = serde_json::from_slice(&receiver_balance_result.result)?;
    let receiver_balance_u128 = receiver_balance_str.parse::<u128>().map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse receiver balance string '{}' to u128: {}",
            receiver_balance_str,
            e
        )
    })?;
    let receiver_balance = NearToken::from_yoctonear(receiver_balance_u128);
    assert_eq!(receiver_balance, transfer_amount);

    Ok(())
}

#[tokio::test]
async fn test_ft_balance_of_sandbox() -> anyhow::Result<()> {
    let (worker, client, contract, owner_account) = setup_sandbox().await?;

    let expected_balance = NearToken::from_near(1);

    let result = client
        .view(contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": owner_account.id()
        }))?
        .await?;

    // Deserialize the balance string, parse to u128, then create NearToken
    let balance_str: String = serde_json::from_slice(&result.result)?;
    let balance_u128 = balance_str.parse::<u128>().map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse balance string '{}' to u128: {}",
            balance_str,
            e
        )
    })?;
    let balance = NearToken::from_yoctonear(balance_u128);

    assert_eq!(balance, expected_balance);

    // Check the balance of a different, empty account
    let other_account = worker.dev_create_account().await?;
    let result_other = client
        .view(contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": other_account.id()
        }))?
        .await?;
    let balance_other_str: String = serde_json::from_slice(&result_other.result)?;
    let balance_other_u128 = balance_other_str.parse::<u128>().map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse other balance string '{}' to u128: {}",
            balance_other_str,
            e
        )
    })?;
    let balance_other = NearToken::from_yoctonear(balance_other_u128);
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
    println!("Key: {:?}", owner_key_info);

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
    let bounds_result = client.view(contract.id(), "storage_balance_bounds").await?;
    let bounds_val: serde_json::Value = serde_json::from_slice(&bounds_result.result)?;
    let min_bound_str = bounds_val["min"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing min bound"))?;
    let min_bound_u128 = min_bound_str.parse::<u128>()?;
    let minimum_deposit = NearToken::from_yoctonear(min_bound_u128);

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
                .deposit(minimum_deposit), // Attach NEAR deposit for storage cost
        )
        // Action 2: Call ft_transfer from owner to receiver
        .call(
            Function::new("ft_transfer")
                .args_json(serde_json::json!({
                    "receiver_id": receiver_account.id(),
                    "amount": transfer_amount.as_yoctonear().to_string()
                }))?
                .deposit(one_yocto), // Attach 1 yoctoNEAR for the transfer standard
        )
        .priority_fee(1000)
        .transact()
        .await?;

    batch_outcome.assert_success();

    // --- Verify Receiver's Balance ---
    let receiver_balance_result = client
        .view(contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": receiver_account.id()
        }))?
        .await?;
    let receiver_balance_str: String = serde_json::from_slice(&receiver_balance_result.result)?;
    let receiver_balance_u128 = receiver_balance_str.parse::<u128>()?;
    let receiver_balance = NearToken::from_yoctonear(receiver_balance_u128);

    assert_eq!(
        receiver_balance, transfer_amount,
        "Receiver balance should match the transferred amount after batch"
    );

    Ok(())
}
