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

#[cfg(feature = "advanced")]
impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        std::fs::File::open(path)
            .map_err(Into::into)
            .and_then(|reader| serde_json::from_reader(reader).map_err(Into::into))
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let serialized = serde_json::to_string_pretty(&self)?;
        std::fs::write(path, serialized)?;
        Ok(())
    }

    pub fn get_evm_secret_key(&self) -> anyhow::Result<&str> {
        self.evm_secret_key.as_deref().ok_or_else(|| {
            anyhow::anyhow!("evm_secret_key must be given in config to use this feature")
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    Mainnet,
    Testnet,
    Custom {
        near_rpc: String,
        aurora_rpc: String,
    },
}
