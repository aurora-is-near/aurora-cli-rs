use crate::{
    client::{AuroraClient, ClientError},
    config::Config,
    utils,
};
use aurora_engine_types::{
    types::{Address, Wei},
    U256,
};
use clap::Subcommand;

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
    GetResult {
        tx_hash_hex: String,
    },
    Call {
        #[clap(short, long)]
        sender_addr_hex: Option<String>,
        #[clap(short, long)]
        target_addr_hex: String,
        #[clap(short, long)]
        amount: Option<String>,
        #[clap(short, long)]
        input_data_hex: String,
    },
    GetNep141 {
        erc_20_address_hex: String,
    },
    GetErc20 {
        nep_141_account: String,
    },
    GetBridgeProver,
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

pub async fn execute_command<T: AsRef<str>>(
    command: Command,
    client: &AuroraClient<T>,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Command::Read { subcommand } => match subcommand {
            ReadCommand::GetResult { tx_hash_hex } => {
                let tx_hash =
                    aurora_engine_types::H256::from_slice(&hex::decode(tx_hash_hex).unwrap());
                let outcome = client.get_transaction_outcome(tx_hash).await?;
                println!("{:?}", outcome);
            }
            ReadCommand::Call {
                sender_addr_hex,
                target_addr_hex,
                amount,
                input_data_hex,
            } => {
                let target = Address::decode(&target_addr_hex).unwrap();
                let sender = sender_addr_hex
                    .map(|x| Address::decode(&x).unwrap())
                    .unwrap_or_default();
                let amount = amount
                    .as_ref()
                    .map(|a| Wei::new(U256::from_dec_str(a).unwrap()))
                    .unwrap_or_else(Wei::zero);
                let input = hex::decode(input_data_hex)?;

                let result = client
                    .view_contract_call(sender, target, amount, input)
                    .await
                    .unwrap();
                println!("{:?}", result);
            }
            ReadCommand::GetNep141 { erc_20_address_hex } => {
                let erc20 = Address::decode(&erc_20_address_hex).unwrap();
                match client.get_nep141_from_erc20(erc20).await {
                    Ok(nep_141_account) => println!("{}", nep_141_account),
                    Err(e) => {
                        let error_msg = format!("{:?}", e);
                        if error_msg.contains("ERC20_NOT_FOUND") {
                            println!("No NEP-141 account associated with {}", erc_20_address_hex);
                        } else {
                            panic!("{}", error_msg);
                        }
                    }
                };
            }
            ReadCommand::GetErc20 { nep_141_account } => {
                println!("{:?}", client.get_erc20_from_nep141(&nep_141_account).await);
            }
            ReadCommand::GetBridgeProver => {
                println!("{:?}", client.get_bridge_prover().await);
            }
        },
        Command::Write { subcommand } => match subcommand {
            WriteCommand::Deploy { input_data_hex } => {
                let source_private_key_hex = config.get_evm_secret_key();
                let sk_bytes = utils::hex_to_arr32(source_private_key_hex)?;
                let sk = secp256k1::SecretKey::parse(&sk_bytes).unwrap();
                let input = hex::decode(input_data_hex)?;
                send_transaction(client, &sk, None, Wei::zero(), input).await?;
            }
            WriteCommand::Transfer {
                target_addr_hex,
                amount,
            } => {
                let source_private_key_hex = config.get_evm_secret_key();
                let sk_bytes = utils::hex_to_arr32(source_private_key_hex)?;
                let sk = secp256k1::SecretKey::parse(&sk_bytes).unwrap();
                let target = Address::decode(&target_addr_hex).unwrap();
                let amount = Wei::new(U256::from_dec_str(&amount).unwrap());
                send_transaction(client, &sk, Some(target), amount, Vec::new()).await?;
            }
            WriteCommand::Call {
                target_addr_hex,
                amount,
                input_data_hex,
            } => {
                let source_private_key_hex = config.get_evm_secret_key();
                let sk_bytes = utils::hex_to_arr32(source_private_key_hex)?;
                let sk = secp256k1::SecretKey::parse(&sk_bytes).unwrap();
                let target = Address::decode(&target_addr_hex).unwrap();
                let amount = amount
                    .as_ref()
                    .map(|a| Wei::new(U256::from_dec_str(a).unwrap()))
                    .unwrap_or_else(Wei::zero);
                let input = hex::decode(input_data_hex)?;
                send_transaction(client, &sk, Some(target), amount, input).await?;
            }
        },
    }
    Ok(())
}

async fn send_transaction<T: AsRef<str>>(
    client: &AuroraClient<T>,
    sk: &secp256k1::SecretKey,
    to: Option<Address>,
    amount: Wei,
    input: Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = utils::address_from_secret_key(sk);
    println!("FROM {:?}", source);

    let nonce = client.get_nonce(source).await?;
    let chain_id = client.get_chain_id().await?;

    let tx_hash = client
        .eth_transaction(to, amount, sk, chain_id, nonce, input)
        .await
        .unwrap();

    // Wait for the RPC to pick up the transaction
    loop {
        match client.get_transaction_outcome(tx_hash).await {
            Ok(result) => {
                println!("{:?}", result);
                break;
            }
            Err(ClientError::AuroraTransactionNotFound(_)) => {
                continue;
            }
            Err(other) => return Err(Box::new(other) as Box<dyn std::error::Error>),
        }
    }

    Ok(())
}
