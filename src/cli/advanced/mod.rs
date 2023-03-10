use clap::{Parser, Subcommand};

pub mod aurora;
pub mod erc20;
pub mod near;
pub mod process_tx_data;
pub mod solidity;

/// Advanced command line interface for communication with Aurora Engine
#[derive(Parser)]
#[command(author, version, long_about = None)]
pub struct Cli {
    /// Path to config file
    #[clap(short, long)]
    pub config_path: Option<String>,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Commands for communication with Aurora Engine
    Aurora {
        #[clap(subcommand)]
        subcommand: aurora::Command,
    },
    /// Commands for communication with NEAR node
    Near {
        #[clap(subcommand)]
        subcommand: near::Command,
    },
    /// Transaction operations
    ProcessTxData {
        #[clap(subcommand)]
        action: process_tx_data::ProcessTxAction,
        input_files_list_path: String,
    },
}
