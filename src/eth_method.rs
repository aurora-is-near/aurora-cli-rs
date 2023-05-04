use aurora_engine_transactions::EthTransactionKind;
use aurora_engine_types::{types::Address, H256};

pub enum EthMethod {
    GetChainId,
    #[cfg(feature = "simple")]
    GetBalance(Address),
    #[cfg(feature = "simple")]
    GetCode(Address),
    GetTransactionCount(Address),
    GetTransactionReceipt(H256),
    SendRawTransaction(Box<EthTransactionKind>),
}

impl EthMethod {
    pub const fn name(&self) -> &str {
        match &self {
            Self::GetChainId => "net_version",
            #[cfg(feature = "simple")]
            Self::GetBalance(_) => "eth_getBalance",
            #[cfg(feature = "simple")]
            Self::GetCode(_) => "eth_getCode",
            Self::GetTransactionCount(_) => "eth_getTransactionCount",
            Self::GetTransactionReceipt(_) => "eth_getTransactionReceipt",
            Self::SendRawTransaction(_) => "eth_sendRawTransaction",
        }
    }

    pub fn params(&self) -> Vec<String> {
        match &self {
            Self::GetChainId => Vec::new(),
            #[cfg(feature = "simple")]
            Self::GetBalance(address) | Self::GetCode(address) => {
                vec![format!("0x{}", address.encode())]
            }
            Self::GetTransactionCount(address) => vec![format!("0x{}", address.encode())],
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
