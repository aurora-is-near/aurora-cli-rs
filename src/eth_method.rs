use aurora_engine_transactions::EthTransactionKind;
use aurora_engine_types::{types::Address, H256};

pub enum EthMethod {
    GetChainId,
    GetTransactionCount(Address),
    GetTransactionReceipt(H256),
    SendRawTransaction(Box<EthTransactionKind>),
}

impl EthMethod {
    pub const fn name(&self) -> &'static str {
        match &self {
            Self::GetChainId => "net_version",
            Self::GetTransactionCount(_) => "eth_getTransactionCount",
            Self::GetTransactionReceipt(_) => "eth_getTransactionReceipt",
            Self::SendRawTransaction(_) => "eth_sendRawTransaction",
        }
    }

    pub fn create_params(&self) -> Vec<String> {
        match &self {
            Self::GetChainId => Vec::new(),
            Self::GetTransactionCount(address) => {
                vec![format!("0x{}", address.encode())]
            }
            Self::GetTransactionReceipt(tx_hash) => {
                vec![format!("0x{}", hex::encode(tx_hash))]
            }
            Self::SendRawTransaction(tx) => {
                let tx_bytes: Vec<u8> = tx.as_ref().into();
                vec![format!("0x{}", hex::encode(tx_bytes))]
            }
        }
    }
}
