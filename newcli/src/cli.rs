use std::{path::PathBuf, str::FromStr};

use aurora_sdk_rs::near::{
    crypto::{EmptySigner, InMemorySigner, Signer},
    primitives::types::AccountId,
};
use clap::{Parser, ValueEnum, command};

use crate::command::Command;

#[derive(Debug, Clone, ValueEnum)]
pub enum Network {
    Localnet,
    Mainnet,
    Testnet,
}
impl Network {
    pub fn rpc_url(&self) -> &str {
        match self {
            Network::Localnet => "http://localhost:3030",
            Network::Mainnet => "https://rpc.mainnet.near.org",
            Network::Testnet => "https://rpc.testnet.near.org",
        }
    }
}

#[derive(Parser)]
#[command(author, long_about = None)]
pub struct Cli {
    /// Near network ID
    #[arg(long, value_enum, default_value_t = Network::Mainnet)]
    pub network: Network,
    /// Aurora EVM account
    #[arg(long, value_name = "ACCOUNT_ID", default_value = "aurora")]
    pub engine: AccountId,
    /// The way output of a command would be formatted
    #[arg(long, default_value = "plain")]
    pub output_format: OutputFormat,
    /// Path to file with NEAR account id and secret key in JSON format
    #[arg(long)]
    pub near_key_path: Option<PathBuf>,
    /// Block height to use for the view command
    #[arg(short, long)]
    pub block_height: Option<u64>,
    #[clap(subcommand)]
    pub command: Command,
}

impl Cli {
    pub(crate) fn signer(&self) -> anyhow::Result<Signer> {
        let path = self
            .near_key_path
            .clone()
            .or(std::env::var("NEAR_KEY_PATH").ok().map(PathBuf::from));

        Ok(path
            .map(|p| InMemorySigner::from_file(&p))
            .transpose()?
            .unwrap_or_else(|| Signer::Empty(EmptySigner::new())))
    }

    pub(crate) fn root_contract_id(&self) -> anyhow::Result<AccountId> {
        let account = match self.network {
            Network::Testnet => "testnet",
            Network::Mainnet => "near",
            Network::Localnet => {
                anyhow::bail!("Account creation is only supported for mainnet or testnet")
            }
        };

        account.parse().map_err(Into::into)
    }
}

#[derive(Default, Clone, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Plain,
    Json,
}

impl FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "plain" => Ok(Self::Plain),
            "json" => Ok(Self::Json),
            _ => anyhow::bail!("unknown output format: {s}"),
        }
    }
}
