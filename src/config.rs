use serde::{Deserialize, Serialize};
#[cfg(feature = "advanced")]
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub network: Network,
    pub engine_account_id: String,
    pub aurora_api_key: Option<String>,
    pub near_key_path: Option<String>,
    pub evm_secret_key: Option<String>,
}

impl Config {
    #[cfg(feature = "advanced")]
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        std::fs::File::open(path)
            .map_err(Into::into)
            .and_then(|reader| serde_json::from_reader(reader).map_err(Into::into))
    }

    #[cfg(feature = "advanced")]
    pub fn get_evm_secret_key(&self) -> anyhow::Result<&str> {
        self.evm_secret_key.as_deref().ok_or_else(|| {
            anyhow::anyhow!("evm_secret_key must be given in config to use this feature")
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    Mainnet,
    Testnet,
}
