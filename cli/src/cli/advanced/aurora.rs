#[cfg(feature = "simple")]
use aurora_engine_types::types::Address;
use aurora_engine_types::{U256, types::Wei};
use clap::Subcommand;

use crate::{client::AuroraClient, config::Config, utils};

#[derive(Subcommand)]
pub enum Command {
    Read {
        #[clap(subcommand)]
        subcommand: ReadCommand,
    },
    Write {
        #[clap(subcommand)]
        subcommand: WriteCommand,
    },
}

#[derive(Subcommand)]
pub enum ReadCommand {
    GetResult { tx_hash_hex: String },
}

#[derive(Subcommand)]
pub enum WriteCommand {
    Deploy {
        input_data_hex: String,
    },
    Transfer {
        #[clap(short, long)]
        target_addr_hex: String,
        #[clap(short, long)]
        amount: String,
    },
    Call {
        #[clap(short, long)]
        target_addr_hex: String,
        #[clap(short, long)]
        amount: Option<String>,
        #[clap(short, long)]
        input_data_hex: String,
    },
}

pub async fn execute_command(
    command: Command,
    client: &AuroraClient,
    config: &Config,
) -> anyhow::Result<()> {
    match command {
        // Command::Benchmark
        Command::Read { subcommand } => match subcommand {
            ReadCommand::GetResult { tx_hash_hex } => {
                let tx_hash = aurora_engine_types::H256::from_slice(&hex::decode(tx_hash_hex)?);
                let outcome = client.get_transaction_outcome(tx_hash).await?;
                println!("{outcome:?}");
            }
        },
        Command::Write { subcommand } => match subcommand {
            WriteCommand::Deploy { input_data_hex } => {
                let secret_key_hex = config.get_evm_secret_key()?;
                let secret_key = utils::secret_key_from_hex(secret_key_hex)?;
                let input = utils::hex_to_vec(&input_data_hex)?;
                client
                    .send_and_wait_transaction(&secret_key, None, Wei::zero(), input)
                    .await?;
            }
            WriteCommand::Transfer {
                target_addr_hex,
                amount,
            } => {
                let secret_key_hex = config.get_evm_secret_key()?;
                let secret_key = utils::secret_key_from_hex(secret_key_hex)?;
                let target = utils::hex_to_address(&target_addr_hex)?;
                let amount = Wei::new(U256::from_dec_str(&amount).unwrap());
                client
                    .send_and_wait_transaction(&secret_key, Some(target), amount, Vec::new())
                    .await?;
            }
            WriteCommand::Call {
                target_addr_hex,
                amount,
                input_data_hex,
            } => {
                let secret_key_hex = config.get_evm_secret_key()?;
                let secret_key = utils::secret_key_from_hex(secret_key_hex)?;
                let target = utils::hex_to_address(&target_addr_hex)?;
                let amount = amount
                    .as_ref()
                    .map_or_else(Wei::zero, |a| Wei::new(U256::from_dec_str(a).unwrap()));
                let input = utils::hex_to_vec(&input_data_hex)?;
                client
                    .send_and_wait_transaction(&secret_key, Some(target), amount, input)
                    .await?;
            }
        },
    }
    Ok(())
}
