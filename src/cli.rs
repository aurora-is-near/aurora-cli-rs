use clap::{Parser, Subcommand};
use std::str::FromStr;

#[derive(Parser)]
pub struct Cli {
    #[clap(short, long)]
    pub network: Option<Network>,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    GetResult {
        tx_hash_hex: String,
    },
    Transfer {
        #[clap(short, long)]
        source_private_key_hex: String,
        #[clap(short, long)]
        target_addr_hex: String,
        #[clap(short, long)]
        amount: String,
    },
    ProcessTxData {
        #[clap(subcommand)]
        action: ProcessTxAction,
        input_files_list_path: String,
    },
}

#[derive(Subcommand)]
pub enum ProcessTxAction {
    NearGasVsEvmGas,
    AverageGasProfile,
    GasDistribution,
    OutcomeDistribution,
    FilterTo { target_addr_hex: String },
}

#[derive(Debug)]
pub enum Network {
    Mainnet,
}

impl Default for Network {
    fn default() -> Self {
        Self::Mainnet
    }
}

impl FromStr for Network {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mainnet" => Ok(Self::Mainnet),
            _ => Err("Unrecognized network name"),
        }
    }
}
