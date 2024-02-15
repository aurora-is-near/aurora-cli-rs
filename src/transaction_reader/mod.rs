//! Helpful functions for analyzing JSON data obtained from NEAR (e.g. via `tx` JSON RPC method).
//! `paths: Vec<String>` gives the list of (absolute) paths for all the files to include in the analysis.

use aurora_engine_transactions::EthTransactionKind;
use aurora_engine_types::borsh::BorshDeserialize;
use aurora_engine_types::parameters::engine::{SubmitResult, TransactionStatus};
use serde_json::Value;
use std::{collections::HashMap, path::Path, sync::Arc};
use tokio::fs;

pub mod aggregator;
pub mod filter;

use aggregator::Aggregator;
use filter::Filter;

pub struct ParsedTx {
    path: String,
    data: TxData,
}

pub struct TxData {
    status: TxStatus,
    gas_profile: HashMap<String, u128>,
    eth_tx: Option<EthTransactionKind>,
}

impl TxData {
    pub fn from_value(value: &Value) -> Option<Self> {
        let status = get_tx_status(value)?;
        let gas_profile = get_gas_profile(value)?;
        let eth_tx = get_eth_tx(value);

        Some(Self {
            status,
            gas_profile,
            eth_tx,
        })
    }
}

pub async fn process_data<A, F>(paths: Vec<String>, filter: &Arc<F>)
where
    A: Aggregator + Send,
    A::Input: std::fmt::Debug + Send + 'static,
    F: Filter + Send + Sync + 'static,
{
    let (send_channel, aggregator) = A::create();

    let read_tasks: Vec<tokio::task::JoinHandle<_>> = paths
        .into_iter()
        .map(|path| {
            let local_channel = send_channel.clone();
            let local_filter = Arc::clone(filter);
            tokio::task::spawn(async move {
                let value = read_file(&path).await;
                match TxData::from_value(&value) {
                    None => println!("ERROR failed to read tx data for {}", &path),
                    Some(data) => {
                        if local_filter.pass(&data) {
                            let tx = ParsedTx { path, data };
                            let input = A::pre_process(&tx);
                            local_channel.send(input).unwrap();
                        }
                    }
                }
            })
        })
        .collect();
    drop(send_channel);
    let agg_task = aggregator.start();

    for t in read_tasks {
        t.await.unwrap_or_else(|e| println!("ERROR {e:?}"));
    }

    let aggregator = agg_task.await.unwrap();
    aggregator.finish();
}

async fn read_file<P: AsRef<Path> + Send>(path: P) -> serde_json::Value {
    let path = path.as_ref();
    let bytes = match fs::read(path).await {
        Ok(b) => b,
        Err(e) => panic!("ERROR on file {}: {e:?}", path.display()),
    };
    match serde_json::from_slice(&bytes) {
        Ok(x) => x,
        Err(e) => panic!("ERROR on file {}: {e:?}", path.display()),
    }
}

fn get_gas_profile(value: &Value) -> Option<HashMap<String, u128>> {
    let mut profile_map = HashMap::new();
    let total = get_gas_burnt(value)?;
    let result = value.as_object()?.get("result")?;
    let outcomes = result.as_object()?.get("receipts_outcome")?.as_array()?;
    let outcome = outcomes.iter().find_map(|v| {
        let outcome = v.as_object()?.get("outcome")?.as_object()?;
        let g = outcome.get("gas_burnt")?.as_u64()?;
        if u128::from(g) == total {
            Some(outcome)
        } else {
            None
        }
    })?;
    let profile = get_recursive(outcome.get("metadata")?, &["gas_profile"])?.as_array()?;
    for entry in profile {
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

fn get_eth_tx(value: &Value) -> Option<EthTransactionKind> {
    let result = value.as_object()?.get("result")?;
    let transaction = result.as_object()?.get("transaction")?;
    let actions = transaction.as_object()?.get("actions")?;
    for action in actions.as_array()? {
        if let Some(fn_call) = action.as_object().and_then(|a| a.get("FunctionCall")) {
            let args = fn_call.as_object()?.get("args")?.as_str()?;
            let bytes = aurora_engine_sdk::base64::decode(args).ok()?;
            return bytes.as_slice().try_into().ok();
        }
    }
    None
}

fn get_tx_status(value: &Value) -> Option<TxStatus> {
    let result = value.as_object()?.get("result")?;
    let status = result.as_object()?.get("status")?.as_object()?;

    if let Some(failed_status) = status.get("Failure") {
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
    } else {
        let success_b64 = status.get("SuccessValue")?.as_str()?;
        let success_bytes = aurora_engine_sdk::base64::decode(success_b64).ok()?;
        let result = SubmitResult::try_from_slice(success_bytes.as_slice()).ok()?;
        Some(TxStatus::Executed(result))
    }
}

fn get_gas_burnt(value: &Value) -> Option<u128> {
    let result = value.as_object()?.get("result")?;
    let outcomes = result.as_object()?.get("receipts_outcome")?.as_array()?;
    let max_burnt = outcomes
        .iter()
        .filter_map(|v| {
            let g = get_recursive(v, &["outcome", "gas_burnt"])?;
            g.as_u64()
        })
        .max()?;
    Some(u128::from(max_burnt))
}

fn get_recursive<'a>(v: &'a Value, path: &[&str]) -> Option<&'a Value> {
    if path.is_empty() {
        Some(v)
    } else {
        let field = v.as_object()?.get(path[0])?;
        get_recursive(field, &path[1..])
    }
}

#[derive(Debug)]
enum TxStatus {
    Executed(SubmitResult),
    GasLimit,
    IncorrectNonce,
    Other(String),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum FlatTxStatus {
    Succeeded,
    Reverted,
    GasLimit,
    IncorrectNonce,
    Other,
}

impl TxStatus {
    const fn flatten(&self) -> FlatTxStatus {
        match self {
            Self::Executed(result) => match result.status {
                TransactionStatus::Succeed(_) => FlatTxStatus::Succeeded,
                _ => FlatTxStatus::Reverted,
            },
            Self::GasLimit => FlatTxStatus::GasLimit,
            Self::IncorrectNonce => FlatTxStatus::IncorrectNonce,
            Self::Other(_) => FlatTxStatus::Other,
        }
    }
}
