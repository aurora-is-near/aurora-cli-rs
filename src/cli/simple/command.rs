use aurora_engine::parameters::{GetStorageAtArgs, InitCallArgs, NewCallArgs, TransactionStatus};
use aurora_engine_sdk::types::near_account_to_evm_address;
use aurora_engine_types::parameters::engine::SubmitResult;
use aurora_engine_types::types::Address;
use aurora_engine_types::{types::Wei, H256, U256};
use borsh::{BorshDeserialize, BorshSerialize};
use near_primitives::views::FinalExecutionStatus;
use std::{path::Path, str::FromStr};

use crate::client::TransactionOutcome;
use crate::utils::secret_key_from_hex;
use crate::{
    client::Client,
    utils,
    utils::{hex_to_address, hex_to_vec},
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
    let address = hex_to_vec(&address)?;
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
    let address = hex_to_vec(&address)?;
    let result = client.near().view_call("get_balance", address).await?;
    let balance = U256::from_big_endian(&result.result).low_u64();
    println!("{balance}");

    Ok(())
}

/// Return a hex code of the smart contract.
pub async fn get_code(client: Client, address: String) -> anyhow::Result<()> {
    let address = hex_to_vec(&address)?;
    let result = client.near().view_call("get_code", address).await?;
    let code = hex::encode(result.result);
    println!("0x{code}");

    Ok(())
}

/// Deploy Aurora EVM smart contract.
pub async fn deploy_aurora<P: AsRef<Path> + Send>(client: Client, path: P) -> anyhow::Result<()> {
    let code = std::fs::read(path)?;
    let result = match client.near().deploy_contract(code).await {
        Ok(outcome) => match outcome.status {
            FinalExecutionStatus::SuccessValue(_) => {
                "Aurora EVM has been deployed successfully".to_string()
            }
            FinalExecutionStatus::Failure(e) => format!("Error while deployed Aurora EVM: {e}"),
            _ => "Error: Bad transaction status".to_string(),
        },
        Err(e) => format!("{e:?}"),
    };
    println!("{result}");

    Ok(())
}

/// Initialize Aurora EVM smart contract.
pub async fn init(
    client: Client,
    chain_id: u64,
    owner_id: Option<String>,
    bridge_prover: Option<String>,
    upgrade_delay_blocks: Option<u64>,
    custodian_address: Option<String>,
    ft_metadata_path: Option<String>,
) -> anyhow::Result<()> {
    let to_account_id = |id: Option<String>| {
        id.map_or_else(
            || {
                client
                    .near()
                    .engine_account_id
                    .to_string()
                    .parse()
                    .map_err(|e| anyhow::anyhow!("{e}"))
            },
            |id| id.parse().map_err(|e| anyhow::anyhow!("{e}")),
        )
    };

    let owner_id = to_account_id(owner_id)?;
    let prover_id = to_account_id(bridge_prover)?;

    // Init Aurora EVM.
    let aurora_init_args = NewCallArgs {
        chain_id: H256::from_low_u64_be(chain_id).into(),
        owner_id,
        bridge_prover_id: prover_id.clone(),
        upgrade_delay_blocks: upgrade_delay_blocks.unwrap_or_default(),
    }
    .try_to_vec()?;

    let eth_connector_init_args = InitCallArgs {
        prover_account: prover_id,
        eth_custodian_address: custodian_address.map_or_else(
            || Address::default().encode(),
            |address| address.trim_start_matches("0x").to_string(),
        ),
        metadata: utils::ft_metadata::parse_ft_metadata(
            ft_metadata_path.and_then(|path| std::fs::read_to_string(path).ok()),
        )?,
    }
    .try_to_vec()?;

    let batch = vec![
        ("new".to_string(), aurora_init_args),
        ("new_eth_connector".to_string(), eth_connector_init_args),
    ];

    match client.near().contract_call_batch(batch).await?.status {
        FinalExecutionStatus::Failure(e) => {
            anyhow::bail!("Error while initialized Aurora EVM: {e}")
        }
        FinalExecutionStatus::Started | FinalExecutionStatus::NotStarted => {
            anyhow::bail!("Error while initialized Aurora EVM: Bad status of the transaction")
        }
        FinalExecutionStatus::SuccessValue(_) => {}
    }

    println!("Aurora EVM have been initialized successfully");

    Ok(())
}

/// Deploy EVM byte code.
pub async fn deploy_evm_code(client: Client, code: String, sk: Option<&str>) -> anyhow::Result<()> {
    let sk = sk
        .ok_or_else(|| anyhow::anyhow!("Deploy EVM code requires Aurora secret key"))
        .and_then(secret_key_from_hex)?;
    let code = hex::decode(code)?;

    let result = client
        .near()
        .send_aurora_transaction(&sk, None, Wei::zero(), code)
        .await?;
    let output = match result.status {
        FinalExecutionStatus::NotStarted | FinalExecutionStatus::Started => {
            anyhow::bail!("Error while deploying EVM code: Bad status of the transaction")
        }
        FinalExecutionStatus::Failure(e) => {
            anyhow::bail!("Error while deploying EVM code: {e}")
        }
        FinalExecutionStatus::SuccessValue(ref bytes) => {
            let result = SubmitResult::try_from_slice(bytes)?;
            if let TransactionStatus::Succeed(bytes) = result.status {
                format!(
                    "Contract has been deployed to address: 0x{} successfully",
                    hex::encode(bytes)
                )
            } else {
                format!("Transaction reverted: {result:?}")
            }
        }
    };

    println!("{output}");

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
    sk: Option<&str>,
) -> anyhow::Result<()> {
    let sk = sk
        .ok_or_else(|| anyhow::anyhow!("Call contract requires Aurora secret key"))
        .and_then(secret_key_from_hex)?;
    let target = hex_to_address(&address)?;
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
    let address = hex_to_address(&address)?;
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

pub fn key_pair(random: bool, seed: Option<u64>) -> anyhow::Result<()> {
    let (address, sk) = utils::gen_key_pair(random, seed)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "address": format!("0x{}", address.encode()),
            "secret_key": hex::encode(sk.serialize()),
        }))?
    );

    Ok(())
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
