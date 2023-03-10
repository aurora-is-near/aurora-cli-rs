use aurora_engine::parameters::{GetStorageAtArgs, NewCallArgs, TransactionStatus};
use aurora_engine_sdk::types::near_account_to_evm_address;
use aurora_engine_types::account_id::AccountId;
use aurora_engine_types::{types::Wei, H256, U256};
use borsh::{BorshDeserialize, BorshSerialize};
use near_primitives::views::FinalExecutionStatus;
use std::path::Path;
use std::str::FromStr;

use crate::client::TransactionOutcome;
use crate::{
    client::Client,
    utils::{address_from_hex, secret_key_from_file},
};

pub async fn get_chain_id(client: Client) -> anyhow::Result<()> {
    get_number(client, "get_chain_id", None).await
}

pub async fn get_version(client: Client) -> anyhow::Result<()> {
    get_string(client, "get_version", None).await
}

pub async fn get_owner(client: Client) -> anyhow::Result<()> {
    get_string(client, "get_owner", None).await
}

pub async fn get_bridge_prover(client: Client) -> anyhow::Result<()> {
    get_string(client, "get_bridge_prover", None).await
}

pub async fn get_nonce(client: Client, address: String) -> anyhow::Result<()> {
    let address = address_from_hex(&address)?.as_bytes().to_vec();
    get_number(client, "get_nonce", Some(address)).await
}

pub async fn get_upgrade_index(client: Client) -> anyhow::Result<()> {
    // TODO: Check for correctness.
    let index = client
        .near()
        .view_call("get_upgrade_index", vec![])
        .await
        .map(|result| u64::try_from_slice(&result.result).unwrap_or_default())
        .unwrap_or_default();
    println!("{index}");

    Ok(())
}

/// Return ETH balance of the address.
pub async fn get_balance(client: Client, address: String) -> anyhow::Result<()> {
    let address = address_from_hex(&address)?.as_bytes().to_vec();
    let result = client.near().view_call("get_balance", address).await?;
    let balance = U256::from_big_endian(&result.result).low_u64();
    println!("{balance}");

    Ok(())
}

/// Return a hex code of the smart contract.
pub async fn get_code(client: Client, address: String) -> anyhow::Result<()> {
    let address = address_from_hex(&address)?.as_bytes().to_vec();
    let result = client.near().view_call("get_code", address).await?;
    let code = hex::encode(result.result);
    println!("0x{code}");

    Ok(())
}

/// Initialize Aurora EVM smart contract.
pub async fn init(
    client: Client,
    chain_id: u64,
    owner_id: Option<String>,
    bridge_prover: Option<String>,
    upgrade_delay_blocks: Option<u64>,
) -> anyhow::Result<()> {
    let args = NewCallArgs {
        chain_id: H256::from_low_u64_be(chain_id).into(),
        owner_id: owner_id
            .and_then(|id| AccountId::try_from(id).ok())
            .unwrap_or_default(),
        bridge_prover_id: bridge_prover
            .and_then(|id| AccountId::try_from(id).ok())
            .unwrap_or_default(),
        upgrade_delay_blocks: upgrade_delay_blocks.unwrap_or_default(),
    }
    .try_to_vec()?;

    let result = client.near().contract_call("new", args).await;

    println!("{result:?}");

    Ok(())
}

pub async fn deploy_evm_code(
    client: Client,
    code: String,
    path_to_sk: Option<&str>,
) -> anyhow::Result<()> {
    let key = path_to_sk.ok_or_else(|| anyhow::anyhow!("operation requires secret key"))?;
    let sk = secret_key_from_file(key)?;
    let code = hex::decode(code)?;

    let result = client
        .near()
        .send_aurora_transaction(&sk, None, Wei::zero(), code)
        .await;
    let output = match result {
        Ok(result) => match result.status {
            FinalExecutionStatus::NotStarted => "not_tarted".to_string(),
            FinalExecutionStatus::Started => "started".to_string(),
            FinalExecutionStatus::Failure(_) => "failure".to_string(),
            FinalExecutionStatus::SuccessValue(_) => format!(
                "code has been deployed successfully, tx hash: {}",
                result.transaction.hash
            ),
        },
        Err(e) => format!("Deploying code error: {e}"),
    };

    println!("{output}");

    Ok(())
}

pub async fn deploy_aurora<P: AsRef<Path> + Send>(client: Client, path: P) -> anyhow::Result<()> {
    let code = std::fs::read(path)?;

    match client.near().deploy_contract(code).await {
        Ok(result) => println!("{result}"),
        Err(e) => eprintln!("{e:?}"),
    }

    Ok(())
}

/// Creates new NEAR's account.
pub async fn create_account(
    client: Client,
    account: &str,
    initial_balance: f64,
) -> anyhow::Result<()> {
    match client.near().create_account(account, initial_balance).await {
        Ok(result) => println!("{result}"),
        Err(e) => eprintln!("{e:?}"),
    }

    Ok(())
}

/// View new NEAR's account.
pub async fn view_account(client: Client, account: &str) -> anyhow::Result<()> {
    match client.near().view_account(account).await {
        Ok(result) => println!("{result}"),
        Err(e) => eprintln!("{e:?}"),
    }

    Ok(())
}

pub async fn call(
    client: Client,
    address: String,
    func_hash: String,
    path_to_sk: Option<&str>,
) -> anyhow::Result<()> {
    let key = path_to_sk.ok_or_else(|| anyhow::anyhow!("operation requires secret key"))?;
    let sk = secret_key_from_file(key)?;
    let target = address_from_hex(&address)?;
    let func = hex::decode(func_hash)?;

    let result = client
        .near()
        .send_aurora_transaction(&sk, Some(target), Wei::zero(), func)
        .await;
    let output = match result {
        Ok(result) => match result.status {
            FinalExecutionStatus::NotStarted => "not_tarted".to_string(),
            FinalExecutionStatus::Started => "started".to_string(),
            FinalExecutionStatus::Failure(_) => "failure".to_string(),
            FinalExecutionStatus::SuccessValue(_) => {
                let outcome = client
                    .near()
                    .get_receipt_outcome(result.transaction_outcome.id)
                    .await?;
                match outcome {
                    TransactionOutcome::Result(result) => match result.status {
                        TransactionStatus::Succeed(data) => {
                            format!("transaction successful, result: {data:?}")
                        }
                        _ => String::from_utf8_lossy(result.status.as_ref()).to_string(),
                    },
                    TransactionOutcome::Failure(e) => format!("bad outcome: {e}"),
                }
                // format!(
                //     "success, tx hash: {}, {}",
                //     result.transaction.hash.to_string(),
                // )
            }
        },
        Err(e) => format!("Deploying code error: {e}"),
    };

    println!("{output}");

    Ok(())
}

pub async fn stage_upgrade(client: Client) -> anyhow::Result<()> {
    let _result = client.near().view_call("stage_upgrade", vec![]).await?;

    Ok(())
}

pub async fn deploy_upgrade(client: Client) -> anyhow::Result<()> {
    let _result = client.near().view_call("deploy_upgrade", vec![]).await?;

    Ok(())
}

pub async fn get_storage_at(client: Client, address: String, key: String) -> anyhow::Result<()> {
    let address = address_from_hex(&address)?;
    let key = H256::from_str(&key)?;
    let input = GetStorageAtArgs {
        address,
        key: key.0,
    };
    let result = client
        .near()
        .view_call("get_storage_at", input.try_to_vec()?)
        .await?;
    let storage = H256::from_slice(&result.result);

    println!("{storage}");

    Ok(())
}

pub fn encode_address(account: &str) {
    let result = near_account_to_evm_address(account.as_bytes()).encode();
    println!("0x{result}");
}

async fn get_number(
    client: Client,
    method_name: &str,
    args: Option<Vec<u8>>,
) -> anyhow::Result<()> {
    let result = client
        .near()
        .view_call(method_name, args.unwrap_or_default())
        .await?;
    let output = U256::from_big_endian(&result.result);
    println!("{output}");

    Ok(())
}

async fn get_string(
    client: Client,
    method_name: &str,
    args: Option<Vec<u8>>,
) -> anyhow::Result<()> {
    let result = client
        .near()
        .view_call(method_name, args.unwrap_or_default())
        .await?;
    let output = String::from_utf8(result.result)?;
    println!("{}", output.trim());

    Ok(())
}
