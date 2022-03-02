use aurora_engine_transactions::EthTransactionKind;

pub enum EthMethod {
    GetTransactionReceipt([u8; 32]),
    SendRawTransaction(Box<EthTransactionKind>),
}

impl EthMethod {
    pub fn name(&self) -> &'static str {
        match &self {
            Self::GetTransactionReceipt(_) => "eth_getTransactionReceipt",
            Self::SendRawTransaction(_) => "eth_sendRawTransaction",
        }
    }

    pub fn create_params(&self) -> Vec<String> {
        match &self {
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
