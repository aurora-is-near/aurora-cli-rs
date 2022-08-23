use clap::{Parser, Subcommand};
use std::str::FromStr;

#[derive(Parser)]
pub struct Cli {
    #[clap(short, long)]
    pub network: Option<Network>,
    #[clap(short, long)]
    pub api_key_path: Option<String>,
    #[clap(short, long)]
    pub engine_account_id: Option<String>,
    #[clap(short, long)]
    pub signer_key_path: Option<String>,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    GetResult {
        tx_hash_hex: String,
    },
    GetNearResult {
        tx_hash_b58: String,
        relayer: Option<String>,
    },
    Transfer {
        #[clap(short, long)]
        source_private_key_hex: String,
        #[clap(short, long)]
        target_addr_hex: String,
        #[clap(short, long)]
        amount: String,
    },
    ContractCall {
        #[clap(short, long)]
        source_private_key_hex: String,
        #[clap(short, long)]
        target_addr_hex: String,
        #[clap(short, long)]
        amount: Option<String>,
        #[clap(short, long)]
        input_data_hex: String,
    },
    ContractView {
        #[clap(short, long)]
        sender_addr_hex: Option<String>,
        #[clap(short, long)]
        target_addr_hex: String,
        #[clap(short, long)]
        amount: Option<String>,
        #[clap(short, long)]
        input_data_hex: String,
    },
    Xcc {
        #[clap(short, long)]
        source_private_key_hex: String,
        #[clap(short, long)]
        target_near_account: String,
        #[clap(short, long)]
        method_name: String,
        #[clap(short, long)]
        json_args: Option<String>,
        #[clap(long)]
        json_args_stdin: Option<bool>,
        #[clap(short, long)]
        deposit_yocto: Option<String>,
        #[clap(short, long)]
        attached_gas: Option<String>,
    },
    Deploy {
        #[clap(short, long)]
        source_private_key_hex: String,
        #[clap(short, long)]
        input_data_hex: String,
    },
    GetNep141 {
        erc_20_address_hex: String,
    },
    GetBridgeProver,
    ProcessTxData {
        #[clap(subcommand)]
        action: ProcessTxAction,
        input_files_list_path: String,
    },
    FactoryUpdate {
        #[clap(short, long)]
        wasm_bytes_path: String,
    },
}

#[derive(Subcommand)]
pub enum ProcessTxAction {
    NearGasVsEvmGas,
    AverageGasProfile {
        min_near_gas: Option<u128>,
    },
    GasDistribution,
    OutcomeDistribution,
    FilterTo {
        target_addr_hex: String,
    },
    FilterGasRange {
        #[clap(long)]
        min_near: Option<u128>,
        #[clap(long)]
        min_evm: Option<u64>,
        #[clap(long)]
        max_near: Option<u128>,
        #[clap(long)]
        max_evm: Option<u64>,
    },
    FromToGasUsed,
}

#[derive(Debug)]
pub enum Network {
    Mainnet,
    Testnet,
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
            "testnet" => Ok(Self::Testnet),
            _ => Err("Unrecognized network name"),
        }
    }
}
