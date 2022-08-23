use clap::{Parser, Subcommand};

pub mod process_tx_data;

#[derive(Parser)]
pub struct Cli {
    #[clap(short, long)]
    pub config_path: Option<String>,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    GetResult {
        tx_hash_hex: String,
    },
    GetNearResult {
        receipt_id_b58: String,
    },
    Transfer {
        #[clap(short, long)]
        target_addr_hex: String,
        #[clap(short, long)]
        amount: String,
    },
    ContractCall {
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
        input_data_hex: String,
    },
    GetNep141 {
        erc_20_address_hex: String,
    },
    GetBridgeProver,
    ProcessTxData {
        #[clap(subcommand)]
        action: process_tx_data::ProcessTxAction,
        input_files_list_path: String,
    },
    FactoryUpdate {
        #[clap(short, long)]
        wasm_bytes_path: String,
    },
}
