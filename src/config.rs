use std::path::Path;
use std::{fs, io};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub network: Network,
    pub engine_account_id: String,
    pub aurora_api_key: Option<String>,
    pub near_key_path: Option<String>,
    pub evm_secret_key: Option<String>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let reader = fs::File::open(path)?;
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }

    pub fn get_evm_secret_key(&self) -> &str {
        self.evm_secret_key
            .as_deref()
            .expect("evm_secret_key must be given in config to use this feature")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    Mainnet,
    Testnet,
    Custom {
        near_rpc: String,
        aurora_rpc: String,
    },
}
