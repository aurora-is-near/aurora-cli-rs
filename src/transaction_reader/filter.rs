use crate::transaction_reader::{FlatTxStatus, TxData, TxStatus};
use aurora_engine_transactions::EthTransactionKind;
use aurora_engine_types::types::Address;

pub trait Filter {
    fn pass(&self, data: &TxData) -> bool;
}

pub struct None;
impl Filter for None {
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
            .map(|a| a == &self.0)
            .unwrap_or(false)
    }
}

pub struct And<F1, F2> {
    f1: F1,
    f2: F2,
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
    pub fn new(f1: F1, f2: F2) -> Self {
        Self { f1, f2 }
    }
}
impl<F1: Filter, F2: Filter> Filter for Or<F1, F2> {
    fn pass(&self, data: &TxData) -> bool {
        self.f1.pass(data) || self.f2.pass(data)
    }
}
