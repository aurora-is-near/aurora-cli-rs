pub enum EthMethod {
    GetTransactionReceipt([u8; 32]),
}

impl EthMethod {
    pub fn name(&self) -> &str {
        match &self {
            Self::GetTransactionReceipt(_) => "eth_getTransactionReceipt",
        }
    }

    pub fn create_params(&self) -> Vec<String> {
        match &self {
            Self::GetTransactionReceipt(tx_hash) => {
                vec![format!("0x{}", hex::encode(tx_hash))]
            }
        }
    }
}
