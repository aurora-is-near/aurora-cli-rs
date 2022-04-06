//! Helpful functions for analyzing JSON data obtained from NEAR (e.g. via `tx` JSON RPC method).
//! `paths: Vec<String>` gives the list of (absolute) paths for all the files to include in the analysis.

use aurora_engine::parameters::SubmitResult;
use aurora_engine_transactions::{EthTransactionKind, NormalizedEthTransaction};
use aurora_engine_types::types::Address;
use borsh::BorshDeserialize;
use std::collections::HashMap;
use tokio::fs;
use tokio::sync::mpsc;

pub async fn get_near_gas_vs_evm_gas(paths: Vec<String>) {
    let mut data_points = Vec::new();

    let (send_channel, mut receive_channel) = mpsc::unbounded_channel();
    let tasks: Vec<tokio::task::JoinHandle<_>> = paths
        .into_iter()
        .map(|path| {
            let local_channel = send_channel.clone();
            tokio::task::spawn(async move {
                let value = read_file(path.as_str()).await;
                if let Some(TxStatus::Executed(result)) = get_tx_status(&value) {
                    let evm_gas = result.gas_used;
                    let near_gas = get_gas_burnt(&value).unwrap();
                    local_channel.send((evm_gas, near_gas)).unwrap();
                }
            })
        })
        .collect();
    drop(send_channel);

    while let Some(data_point) = receive_channel.recv().await {
        data_points.push(data_point);
    }

    for t in tasks {
        t.await.unwrap_or_else(|e| println!("ERROR {:?}", e));
    }

    for (x, y) in data_points {
        println!("{} {}", x, y);
    }
}

pub async fn get_average_gas_profile(paths: Vec<String>) {
    let mut total_profile: HashMap<String, u128> = HashMap::new();
    let mut count: u128 = 0;

    let (send_channel, mut receive_channel) = mpsc::unbounded_channel();
    let tasks: Vec<tokio::task::JoinHandle<_>> = paths
        .into_iter()
        .map(|path| {
            let local_channel = send_channel.clone();
            tokio::task::spawn(async move {
                let value = read_file(path.as_str()).await;
                if let Some(status) = get_tx_status(&value) {
                    let status = status.flatten();
                    if status == FlatTxStatus::Succeeded {
                        let gas = get_gas_profile(&value).unwrap_or_default();
                        //if gas.get("TOTAL").unwrap() > &200_000_000_000_000 {
                        local_channel.send(gas).unwrap();
                        //}
                    }
                }
            })
        })
        .collect();
    drop(send_channel);

    while let Some(gas) = receive_channel.recv().await {
        count += 1;
        for (k, v) in gas {
            *total_profile.entry(k).or_insert(0) += v;
        }
    }

    for t in tasks {
        t.await.unwrap_or_else(|e| println!("ERROR {:?}", e));
    }

    let mut average_profile: Vec<(String, u128)> = total_profile
        .into_iter()
        .map(|(k, v)| (k, v / count))
        .collect();
    average_profile.sort_unstable_by(|(_, v1), (_, v2)| v2.cmp(v1));

    for (k, v) in average_profile {
        println!("{} {}", k, v);
    }
}

pub async fn count_transactions_by_gas(paths: Vec<String>) {
    let mut counts: HashMap<u128, usize> = HashMap::new();
    const BUCKET_SIZE: u128 = 10_000_000_000_000;

    for i in 0..31 {
        counts.insert(i * BUCKET_SIZE, 0);
    }

    let (send_channel, mut receive_channel) = mpsc::unbounded_channel();
    let tasks: Vec<tokio::task::JoinHandle<_>> = paths
        .into_iter()
        .map(|path| {
            let local_channel = send_channel.clone();
            tokio::task::spawn(async move {
                let value = read_file(path.as_str()).await;
                if let Some(status) = get_tx_status(&value) {
                    let status = status.flatten();
                    if status == FlatTxStatus::Succeeded || status == FlatTxStatus::GasLimit {
                        let gas = get_gas_burnt(&value).unwrap_or_default();
                        local_channel.send(gas).unwrap();
                    }
                }
            })
        })
        .collect();
    drop(send_channel);

    while let Some(gas) = receive_channel.recv().await {
        let bucket = (gas / BUCKET_SIZE) * BUCKET_SIZE;
        *counts.get_mut(&bucket).unwrap() += 1;
    }

    for t in tasks {
        t.await.unwrap_or_else(|e| println!("ERROR {:?}", e));
    }

    for i in 0..31 {
        let bucket = i * BUCKET_SIZE;
        let count = counts.get(&bucket).unwrap();
        println!("{} {}", bucket / 1_000_000_000_000, count);
    }
}

pub async fn count_transactions_by_type(paths: Vec<String>) {
    let mut counts: HashMap<FlatTxStatus, usize> = {
        let init_data = [
            (FlatTxStatus::Succeeded, 0usize),
            (FlatTxStatus::Reverted, 0),
            (FlatTxStatus::GasLimit, 0),
            (FlatTxStatus::IncorrectNonce, 0),
            (FlatTxStatus::Other, 0),
        ];
        init_data.into_iter().collect()
    };
    let (send_channel, mut receive_channel) = mpsc::unbounded_channel();

    let tasks: Vec<tokio::task::JoinHandle<_>> = paths
        .into_iter()
        .map(|path| {
            let local_channel = send_channel.clone();
            tokio::task::spawn(async move {
                let value = read_file(path.as_str()).await;
                if let Some(status) = get_tx_status(&value) {
                    let flat_status = status.flatten();
                    local_channel.send(flat_status).unwrap();
                }
            })
        })
        .collect();

    drop(send_channel);
    while let Some(status) = receive_channel.recv().await {
        *counts.get_mut(&status).unwrap() += 1;
    }

    for t in tasks {
        t.await.unwrap_or_else(|e| println!("ERROR {:?}", e));
    }

    for x in counts.iter() {
        println!("{:?}", x);
    }
}

pub async fn get_txs_to(paths: Vec<String>, to: Address) {
    let tasks: Vec<tokio::task::JoinHandle<_>> = paths
        .into_iter()
        .map(|path| {
            tokio::task::spawn(async move {
                let value = read_file(path.as_str()).await;
                if let Some(tx) = get_eth_tx(&value) {
                    let norm: NormalizedEthTransaction = tx.into();
                    if let Some(tx_to) = norm.to {
                        if tx_to == to {
                            println!("{}", path)
                        }
                    }
                }
            })
        })
        .collect();

    for t in tasks {
        t.await.unwrap_or_else(|e| println!("ERROR {:?}", e));
    }
}

async fn read_file(path: &str) -> serde_json::Value {
    let bytes = match fs::read(path).await {
        Ok(b) => b,
        Err(e) => panic!("ERROR on file {}: {:?}", path, e),
    };
    serde_json::from_slice(&bytes).unwrap()
}

fn get_gas_profile(value: &serde_json::Value) -> Option<HashMap<String, u128>> {
    let mut profile_map = HashMap::new();
    let total = get_gas_burnt(value)?;
    let result = value.as_object()?.get("result")?;
    let outcomes = result.as_object()?.get("receipts_outcome")?.as_array()?;
    let outcome = outcomes
        .iter()
        .filter_map(|v| {
            let outcome = v.as_object()?.get("outcome")?.as_object()?;
            let g = outcome.get("gas_burnt")?.as_u64()?;
            if (g as u128) == total {
                Some(outcome)
            } else {
                None
            }
        })
        .next()?;
    let profile = get_recursive(outcome.get("metadata")?, &["gas_profile"])?.as_array()?;
    for entry in profile.iter() {
        let entry = entry.as_object()?;
        let name = entry.get("cost")?.as_str()?;
        let amount = entry.get("gas_used")?.as_str()?;
        profile_map.insert(name.to_owned(), amount.parse().unwrap());
    }
    let profile_total: u128 = profile_map.values().sum();
    profile_map.insert("OTHER".into(), total - profile_total);
    profile_map.insert("TOTAL".into(), total);
    Some(profile_map)
}

fn get_eth_tx(value: &serde_json::Value) -> Option<EthTransactionKind> {
    let result = value.as_object()?.get("result")?;
    let transaction = result.as_object()?.get("transaction")?;
    let actions = transaction.as_object()?.get("actions")?;
    for action in actions.as_array()? {
        if let Some(fn_call) = action.as_object().and_then(|a| a.get("FunctionCall")) {
            let args = fn_call.as_object()?.get("args")?.as_str()?;
            let bytes = base64::decode(args).ok()?;
            return bytes.as_slice().try_into().ok();
        }
    }
    None
}

fn get_tx_status(value: &serde_json::Value) -> Option<TxStatus> {
    let result = value.as_object()?.get("result")?;
    let status = result.as_object()?.get("status")?.as_object()?;
    match status.get("Failure") {
        Some(failed_status) => {
            let message = get_recursive(
                failed_status,
                &["ActionError", "kind", "FunctionCallError", "ExecutionError"],
            )?
            .as_str()?;
            if message.contains("ERR_INCORRECT_NONCE") {
                Some(TxStatus::IncorrectNonce)
            } else if message.contains("Exceeded the maximum amount of gas") {
                Some(TxStatus::GasLimit)
            } else {
                Some(TxStatus::Other(message.to_owned()))
            }
        }
        None => {
            let success_b64 = status.get("SuccessValue")?.as_str()?;
            let success_bytes = base64::decode(success_b64).ok()?;
            let result = SubmitResult::try_from_slice(success_bytes.as_slice()).ok()?;
            Some(TxStatus::Executed(result))
        }
    }
}

fn get_gas_burnt(value: &serde_json::Value) -> Option<u128> {
    let result = value.as_object()?.get("result")?;
    let outcomes = result.as_object()?.get("receipts_outcome")?.as_array()?;
    let max_burnt = outcomes
        .iter()
        .filter_map(|v| {
            let g = get_recursive(v, &["outcome", "gas_burnt"])?;
            g.as_u64()
        })
        .max()?;
    Some(max_burnt as u128)
}

fn get_recursive<'a, 'b>(
    v: &'a serde_json::Value,
    path: &'b [&str],
) -> Option<&'a serde_json::Value> {
    if path.is_empty() {
        return Some(v);
    }

    let field = v.as_object()?.get(path[0])?;
    get_recursive(field, &path[1..])
}

#[derive(Debug)]
enum TxStatus {
    Executed(SubmitResult),
    GasLimit,
    IncorrectNonce,
    Other(String),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum FlatTxStatus {
    Succeeded,
    Reverted,
    GasLimit,
    IncorrectNonce,
    Other,
}

impl TxStatus {
    fn flatten(self) -> FlatTxStatus {
        match self {
            Self::Executed(result) => match result.status {
                aurora_engine::parameters::TransactionStatus::Succeed(_) => FlatTxStatus::Succeeded,
                _ => FlatTxStatus::Reverted,
            },
            Self::GasLimit => FlatTxStatus::GasLimit,
            Self::IncorrectNonce => FlatTxStatus::IncorrectNonce,
            Self::Other(_) => FlatTxStatus::Other,
        }
    }
}
