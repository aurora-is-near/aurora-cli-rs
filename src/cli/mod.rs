use clap::{Parser, Subcommand};

pub mod aurora;
pub mod erc20;
pub mod near;
pub mod pausables;
pub mod process_tx_data;
pub mod solidity;

#[derive(Parser)]
pub struct Cli {
    #[clap(short, long)]
    pub config_path: Option<String>,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Aurora {
        #[clap(subcommand)]
        subcommand: aurora::Command,
    },
    Near {
        #[clap(subcommand)]
        subcommand: near::Command,
    },
    ProcessTxData {
        #[clap(subcommand)]
        action: process_tx_data::ProcessTxAction,
        input_files_list_path: String,
    },
}
