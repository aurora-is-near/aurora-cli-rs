use crate::{
    transaction_reader::{self, aggregator, filter},
    utils,
};
use clap::Subcommand;
use std::sync::Arc;

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

pub async fn execute_command(action: ProcessTxAction, input_files_list_path: String) {
    let paths_contents = tokio::fs::read_to_string(input_files_list_path)
        .await
        .unwrap();
    let paths: Vec<String> = paths_contents
        .split('\n')
        .filter_map(|line| {
            if line.is_empty() {
                None
            } else {
                Some(line.to_string())
            }
        })
        .collect();

    match action {
        ProcessTxAction::AverageGasProfile { min_near_gas } => {
            let f1 = filter::MatchFlatStatus(transaction_reader::FlatTxStatus::Succeeded);
            match min_near_gas {
                None => {
                    let f = Arc::new(f1);
                    transaction_reader::process_data::<aggregator::AverageGasProfile, _>(paths, &f)
                        .await;
                }
                Some(min_gas) => {
                    let f2 = filter::MinNearGasUsed(min_gas);
                    let f = Arc::new(filter::And::new(f1, f2));
                    transaction_reader::process_data::<aggregator::AverageGasProfile, _>(paths, &f)
                        .await;
                }
            }
        }
        ProcessTxAction::FilterTo { target_addr_hex } => {
            let to = utils::hex_to_address(&target_addr_hex).unwrap();
            let f = Arc::new(filter::EthTxTo(to));
            transaction_reader::process_data::<aggregator::Echo, _>(paths, &f).await;
        }
        ProcessTxAction::GasDistribution => {
            let f1 = filter::MatchFlatStatus(transaction_reader::FlatTxStatus::Succeeded);
            let f2 = filter::MatchFlatStatus(transaction_reader::FlatTxStatus::GasLimit);
            let f = Arc::new(filter::Or::new(f1, f2));
            transaction_reader::process_data::<aggregator::GroupByGas, _>(paths, &f).await;
        }
        ProcessTxAction::NearGasVsEvmGas => {
            let f = Arc::new(filter::StatusExecuted);
            transaction_reader::process_data::<aggregator::GasComparison, _>(paths, &f).await;
        }
        ProcessTxAction::OutcomeDistribution => {
            let f = Arc::new(filter::NoFilter);
            transaction_reader::process_data::<aggregator::GroupByFlatStatus, _>(paths, &f).await;
        }
        ProcessTxAction::FilterGasRange {
            min_near,
            min_evm,
            max_near,
            max_evm,
        } => {
            let f = Arc::new(filter::GeneralGasFilter {
                min_near,
                min_evm,
                max_near,
                max_evm,
            });
            transaction_reader::process_data::<aggregator::Echo, _>(paths, &f).await;
        }
        ProcessTxAction::FromToGasUsed => {
            let f = Arc::new(filter::NoFilter);
            transaction_reader::process_data::<aggregator::FromToGasUsage, _>(paths, &f).await;
        }
    }
}
