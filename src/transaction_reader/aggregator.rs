use crate::transaction_reader::{FlatTxStatus, ParsedTx, TxStatus};
use aurora_engine_types::types::Address;
use std::collections::HashMap;
use tokio::sync::mpsc;

pub trait Aggregator: Sized {
    type Input;

    fn create() -> (mpsc::UnboundedSender<Self::Input>, Self);
    fn start(self) -> tokio::task::JoinHandle<Self>;
    fn pre_process(tx: &ParsedTx) -> Self::Input;
    fn finish(self);
}

type FromToGasUsageEntry = (Address, Option<Address>, u128, u64, u128);
pub struct FromToGasUsage {
    entries: Vec<FromToGasUsageEntry>,
    receive_channel: mpsc::UnboundedReceiver<Option<FromToGasUsageEntry>>,
}
impl Aggregator for FromToGasUsage {
    type Input = Option<FromToGasUsageEntry>;

    fn create() -> (mpsc::UnboundedSender<Self::Input>, Self) {
        let (send_channel, receive_channel) = mpsc::unbounded_channel();

        (
            send_channel,
            Self {
                entries: Vec::new(),
                receive_channel,
            },
        )
    }

    fn start(mut self) -> tokio::task::JoinHandle<Self> {
        tokio::task::spawn(async move {
            while let Some(maybe_entry) = self.receive_channel.recv().await {
                if let Some(entry) = maybe_entry {
                    self.entries.push(entry);
                }
            }

            self
        })
    }

    fn pre_process(tx: &ParsedTx) -> Self::Input {
        let eth_tx = tx.data.eth_tx.as_ref()?;
        let norm_tx: aurora_engine_transactions::NormalizedEthTransaction = eth_tx.clone().try_into().ok()?;
        let from = norm_tx.address;
        let to = norm_tx.to;
        let gas_limit = norm_tx.gas_limit.low_u64();
        let gas_price = norm_tx.max_fee_per_gas.low_u128();
        let gas_used = tx.data.gas_profile.get("TOTAL").copied()?;
        Some((from, to, gas_used, gas_limit, gas_price))
    }

    fn finish(self) {
        for (from, to, gas_used, gas_limit, gas_price) in self.entries {
            let to_str = to.map(|t| t.encode()).unwrap_or_default();
            println!(
                "{},{},{:?},{:?},{:?}",
                from.encode(),
                to_str,
                gas_used,
                gas_limit,
                gas_price
            );
        }
    }
}

pub struct GroupByFlatStatus {
    counts: HashMap<FlatTxStatus, usize>,
    receive_channel: mpsc::UnboundedReceiver<FlatTxStatus>,
}
impl Aggregator for GroupByFlatStatus {
    type Input = FlatTxStatus;

    fn create() -> (mpsc::UnboundedSender<Self::Input>, Self) {
        let (send_channel, receive_channel) = mpsc::unbounded_channel();
        let counts: HashMap<FlatTxStatus, usize> = {
            let init_data = [
                (FlatTxStatus::Succeeded, 0usize),
                (FlatTxStatus::Reverted, 0),
                (FlatTxStatus::GasLimit, 0),
                (FlatTxStatus::IncorrectNonce, 0),
                (FlatTxStatus::Other, 0),
            ];
            init_data.into_iter().collect()
        };

        (
            send_channel,
            Self {
                receive_channel,
                counts,
            },
        )
    }

    fn start(mut self) -> tokio::task::JoinHandle<Self> {
        tokio::task::spawn(async move {
            while let Some(status) = self.receive_channel.recv().await {
                *self.counts.get_mut(&status).unwrap() += 1;
            }

            self
        })
    }

    fn pre_process(tx: &ParsedTx) -> Self::Input {
        tx.data.status.flatten()
    }

    fn finish(self) {
        let mut counts: Vec<(FlatTxStatus, usize)> = self.counts.into_iter().collect();
        counts.sort_unstable_by(|(_, v1), (_, v2)| v2.cmp(v1));
        for (status, count) in counts {
            println!("{:?} {}", status, count);
        }
    }
}

pub struct GroupByGas {
    counts: HashMap<u128, usize>,
    receive_channel: mpsc::UnboundedReceiver<u128>,
}
impl GroupByGas {
    const BUCKET_SIZE: u128 = 10_000_000_000_000;
}
impl Aggregator for GroupByGas {
    type Input = u128;

    fn create() -> (mpsc::UnboundedSender<Self::Input>, Self) {
        let (send_channel, receive_channel) = mpsc::unbounded_channel();
        let mut counts = HashMap::new();

        for i in 0..31 {
            counts.insert(i * Self::BUCKET_SIZE, 0);
        }

        (
            send_channel,
            Self {
                receive_channel,
                counts,
            },
        )
    }

    fn start(mut self) -> tokio::task::JoinHandle<Self> {
        tokio::task::spawn(async move {
            while let Some(gas) = self.receive_channel.recv().await {
                let bucket = (gas / Self::BUCKET_SIZE) * Self::BUCKET_SIZE;
                *self.counts.get_mut(&bucket).unwrap() += 1;
            }

            self
        })
    }

    fn pre_process(tx: &ParsedTx) -> Self::Input {
        tx.data
            .gas_profile
            .get("TOTAL")
            .copied()
            .unwrap_or_default()
    }

    fn finish(self) {
        for i in 0..31 {
            let bucket = i * Self::BUCKET_SIZE;
            let count = self.counts.get(&bucket).unwrap();
            println!("{} {}", bucket / 1_000_000_000_000, count);
        }
    }
}

pub struct Echo {
    receive_channel: mpsc::UnboundedReceiver<String>,
}
impl Aggregator for Echo {
    type Input = String;

    fn create() -> (mpsc::UnboundedSender<Self::Input>, Self) {
        let (send_channel, receive_channel) = mpsc::unbounded_channel();

        (send_channel, Self { receive_channel })
    }

    fn start(mut self) -> tokio::task::JoinHandle<Self> {
        tokio::task::spawn(async move {
            while let Some(s) = self.receive_channel.recv().await {
                println!("{}", s);
            }

            self
        })
    }

    fn pre_process(tx: &ParsedTx) -> Self::Input {
        tx.path.clone()
    }

    fn finish(self) {}
}

pub struct AverageGasProfile {
    total_profile: HashMap<String, u128>,
    count: u128,
    receive_channel: mpsc::UnboundedReceiver<HashMap<String, u128>>,
}
impl Aggregator for AverageGasProfile {
    type Input = HashMap<String, u128>;

    fn create() -> (mpsc::UnboundedSender<Self::Input>, Self) {
        let (send_channel, receive_channel) = mpsc::unbounded_channel();
        let total_profile: HashMap<String, u128> = HashMap::new();
        let count: u128 = 0;
        let me = Self {
            receive_channel,
            total_profile,
            count,
        };
        (send_channel, me)
    }

    fn start(mut self) -> tokio::task::JoinHandle<Self> {
        tokio::task::spawn(async move {
            while let Some(gas) = self.receive_channel.recv().await {
                self.count += 1;
                for (k, v) in gas {
                    *self.total_profile.entry(k).or_insert(0) += v;
                }
            }

            self
        })
    }

    fn pre_process(tx: &ParsedTx) -> Self::Input {
        tx.data.gas_profile.clone()
    }

    fn finish(self) {
        let mut average_profile: Vec<(String, u128)> = self
            .total_profile
            .into_iter()
            .map(|(k, v)| (k, v / self.count))
            .collect();
        average_profile.sort_unstable_by(|(_, v1), (_, v2)| v2.cmp(v1));

        for (k, v) in average_profile {
            println!("{} {}", k, v);
        }
    }
}

pub struct GasComparison {
    data_points: Vec<(u64, u128)>,
    receive_channel: mpsc::UnboundedReceiver<(u64, u128)>,
}
impl Aggregator for GasComparison {
    type Input = (u64, u128);

    fn create() -> (mpsc::UnboundedSender<Self::Input>, Self) {
        let (send_channel, receive_channel) = mpsc::unbounded_channel();
        let data_points = Vec::new();
        let me = Self {
            data_points,
            receive_channel,
        };
        (send_channel, me)
    }

    fn start(mut self) -> tokio::task::JoinHandle<Self> {
        tokio::task::spawn(async move {
            while let Some(data_point) = self.receive_channel.recv().await {
                self.data_points.push(data_point);
            }

            self
        })
    }

    fn pre_process(tx: &ParsedTx) -> Self::Input {
        let evm_gas = match &tx.data.status {
            TxStatus::Executed(submit_result) => submit_result.gas_used,
            _ => 0,
        };
        let near_gas = tx
            .data
            .gas_profile
            .get("TOTAL")
            .copied()
            .unwrap_or_default();
        (evm_gas, near_gas)
    }

    fn finish(self) {
        for (x, y) in self.data_points {
            println!("{} {}", x, y);
        }
    }
}

pub struct Pair<A1: Aggregator, A2: Aggregator> {
    a1: Option<A1>,
    a2: Option<A2>,
    receive_channel: mpsc::UnboundedReceiver<(A1::Input, A2::Input)>,
}
impl<A1, A2> Aggregator for Pair<A1, A2>
where
    A1: Aggregator + Send + 'static,
    A2: Aggregator + Send + 'static,
    A1::Input: Send + std::fmt::Debug + 'static,
    A2::Input: Send + std::fmt::Debug + 'static,
{
    type Input = (A1::Input, A2::Input);

    fn create() -> (mpsc::UnboundedSender<Self::Input>, Self) {
        let (send_channel, receive_channel) = mpsc::unbounded_channel();
        let pair = Self {
            a1: None,
            a2: None,
            receive_channel,
        };
        (send_channel, pair)
    }

    fn start(mut self) -> tokio::task::JoinHandle<Self> {
        let (a1_channel, a1) = A1::create();
        let (a2_channel, a2) = A2::create();
        tokio::task::spawn(async move {
            let a1_task = a1.start();
            let a2_task = a2.start();

            while let Some((a1_input, a2_input)) = self.receive_channel.recv().await {
                a1_channel.send(a1_input).unwrap();
                a2_channel.send(a2_input).unwrap();
            }
            drop(a1_channel);
            drop(a2_channel);

            let a1 = a1_task.await.unwrap();
            let a2 = a2_task.await.unwrap();

            Self {
                a1: Some(a1),
                a2: Some(a2),
                receive_channel: self.receive_channel,
            }
        })
    }

    fn pre_process(tx: &ParsedTx) -> Self::Input {
        (A1::pre_process(tx), A2::pre_process(tx))
    }

    fn finish(self) {
        self.a1
            .map(|a1| a1.finish())
            .unwrap_or_else(|| println!("WARN Pair finished with no A1 instance"));
        self.a2
            .map(|a2| a2.finish())
            .unwrap_or_else(|| println!("WARN Pair finished with no A2 instance"));
    }
}
