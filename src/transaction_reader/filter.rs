use crate::transaction_reader::{FlatTxStatus, TxData, TxStatus};
use aurora_engine_transactions::EthTransactionKind;
use aurora_engine_types::types::Address;

pub trait Filter {
    fn pass(&self, data: &TxData) -> bool;
}

pub struct NoFilter;

impl Filter for NoFilter {
    fn pass(&self, _data: &TxData) -> bool {
        true
    }
}

pub struct StatusExecuted;

impl Filter for StatusExecuted {
    fn pass(&self, data: &TxData) -> bool {
        matches!(data.status, TxStatus::Executed(_))
    }
}

pub struct MatchFlatStatus(pub FlatTxStatus);

impl Filter for MatchFlatStatus {
    fn pass(&self, data: &TxData) -> bool {
        data.status.flatten() == self.0
    }
}

pub struct MinNearGasUsed(pub u128);

impl Filter for MinNearGasUsed {
    fn pass(&self, data: &TxData) -> bool {
        data.gas_profile
            .get("TOTAL")
            .map_or(false, |total| total >= &self.0)
    }
}

pub struct MaxNearGasUsed(pub u128);

impl Filter for MaxNearGasUsed {
    fn pass(&self, data: &TxData) -> bool {
        data.gas_profile
            .get("TOTAL")
            .map_or(false, |total| total <= &self.0)
    }
}

pub struct MinEvmGasUsed(pub u64);

impl Filter for MinEvmGasUsed {
    fn pass(&self, data: &TxData) -> bool {
        match &data.status {
            TxStatus::Executed(submit_result) => submit_result.gas_used >= self.0,
            _ => false,
        }
    }
}

pub struct MaxEvmGasUsed(pub u64);

impl Filter for MaxEvmGasUsed {
    fn pass(&self, data: &TxData) -> bool {
        match &data.status {
            TxStatus::Executed(submit_result) => submit_result.gas_used <= self.0,
            _ => false,
        }
    }
}

pub struct GeneralGasFilter {
    pub min_near: Option<u128>,
    pub min_evm: Option<u64>,
    pub max_near: Option<u128>,
    pub max_evm: Option<u64>,
}

impl Filter for GeneralGasFilter {
    fn pass(&self, data: &TxData) -> bool {
        let Some(near_gas_used) = data.gas_profile.get("TOTAL") else { return false };
        let evm_gas_used = match &data.status {
            TxStatus::Executed(submit_result) => &submit_result.gas_used,
            _ => return false,
        };

        self.min_near.as_ref().map_or(true, |g| near_gas_used >= g)
            && self.min_evm.as_ref().map_or(true, |g| evm_gas_used >= g)
            && self.max_near.as_ref().map_or(true, |g| near_gas_used <= g)
            && self.max_evm.as_ref().map_or(true, |g| evm_gas_used <= g)
    }
}

pub struct EthTxTo(pub Address);

impl Filter for EthTxTo {
    fn pass(&self, data: &TxData) -> bool {
        data.eth_tx
            .as_ref()
            .and_then(|eth_tx| match eth_tx {
                EthTransactionKind::Legacy(tx) => tx.transaction.to.as_ref(),
                EthTransactionKind::Eip2930(t) => t.transaction.to.as_ref(),
                EthTransactionKind::Eip1559(t) => t.transaction.to.as_ref(),
            })
            .map_or(false, |a| a == &self.0)
    }
}

pub struct And<F1, F2> {
    f1: F1,
    f2: F2,
}

impl<F1, F2> And<F1, F2> {
    pub const fn new(f1: F1, f2: F2) -> Self {
        Self { f1, f2 }
    }
}

impl<F1: Filter, F2: Filter> Filter for And<F1, F2> {
    fn pass(&self, data: &TxData) -> bool {
        self.f1.pass(data) && self.f2.pass(data)
    }
}

pub struct Or<F1, F2> {
    f1: F1,
    f2: F2,
}

impl<F1, F2> Or<F1, F2> {
    pub const fn new(f1: F1, f2: F2) -> Self {
        Self { f1, f2 }
    }
}

impl<F1: Filter, F2: Filter> Filter for Or<F1, F2> {
    fn pass(&self, data: &TxData) -> bool {
        self.f1.pass(data) || self.f2.pass(data)
    }
}
