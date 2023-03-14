use clap::{Parser, Subcommand};
use std::str::FromStr;

pub mod command;

/// Simple command line interface for communication with Aurora Engine
#[derive(Parser)]
#[command(author, version, long_about = None)]
pub struct Cli {
    /// NEAR network ID
    #[arg(long, value_name = "network", default_value = "localnet")]
    pub network: Network,
    /// Aurora EVM account
    #[arg(long, value_name = "account", default_value = "aurora")]
    pub engine: String,
    /// Path to file with NEAR account id and secret key in JSON format
    #[arg(long)]
    pub near_secret_file: Option<String>,
    /// Path to file with Aurora EVM secret key
    #[arg(long)]
    pub aurora_secret_key: Option<String>,
    /// Aurora API key
    #[arg(long)]
    pub aurora_api_key: Option<String>,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Init Aurora EVM
    Init {
        /// Chain ID
        chain_id: u64,
        /// Owner of the Aurora EVM
        owner_id: Option<String>,
        /// Account of the bridge prover
        bridge_prover_id: Option<String>,
        /// How many blocks after staging upgrade can deploy it
        upgrade_delay_blocks: Option<u64>,
    },
    /// Return chain ID
    GetChainId,
    /// Upgrade index
    GetUpgradeIndex,
    /// Return Aurora EVM version
    GetVersion,
    /// Return Aurora EVM owner
    GetOwner,
    /// Return bridge prover
    GetBridgeProver,
    /// Stage upgrade
    StageUpgrade,
    /// Deploy upgrade
    DeployUpgrade,
    /// Return next nonce for address
    GetNonce {
        #[arg(action, value_name = "address")]
        address: String,
    },
    /// Return smart contract's code for contract address
    GetCode {
        #[arg(action, value_name = "address")]
        address: String,
    },
    /// Return balance for address
    GetBalance {
        #[arg(action, value_name = "address")]
        address: String,
    },
    /// Call method of a smart contract
    Call {
        #[arg(action, value_name = "address")]
        address: String,
        #[arg(action, value_name = "function")]
        function: String,
        #[arg(action, value_name = "input")]
        input: String,
    },
    /// Return a value from storage at address with key
    GetStorageAt {
        #[arg(action, value_name = "address")]
        address: String,
        #[arg(action, value_name = "key")]
        key: String,
    },
    /// Deploy EVM smart contract's code in hex
    DeployEvmCode {
        #[arg(action, value_name = "code")]
        code: String,
    },
    /// Deploy Aurora EVM smart contract
    DeployAurora {
        #[arg(action, value_name = "path")]
        path: String,
    },
    /// Create new NEAR's account.
    CreateAccount {
        #[arg(action, value_name = "account")]
        account: String,
        #[arg(action, value_name = "balance")]
        balance: f64,
    },
    /// View new NEAR's account.
    ViewAccount {
        #[arg(action, value_name = "account")]
        account: String,
    },
    /// Encode address
    EncodeAddress {
        /// Account ID
        account: String,
    },
}

#[derive(Clone)]
pub enum Network {
    Localnet,
    Mainnet,
    Testnet,
}

impl FromStr for Network {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "localnet" => Ok(Self::Localnet),
            "mainnet" => Ok(Self::Mainnet),
            "testnet" => Ok(Self::Testnet),
            _ => anyhow::bail!("unknown network"),
        }
    }
}

pub async fn run(args: Cli) -> anyhow::Result<()> {
    let near_rpc = match args.network {
        Network::Mainnet => super::NEAR_MAINNET_ENDPOINT,
        Network::Testnet => super::NEAR_TESTNET_ENDPOINT,
        Network::Localnet => super::NEAR_LOCAL_ENDPOINT,
    };
    let client = crate::client::Client::new(near_rpc, &args.engine, args.near_secret_file);

    match args.command {
        Command::GetChainId => command::get_chain_id(client).await?,
        Command::GetUpgradeIndex => command::get_upgrade_index(client).await?,
        Command::GetVersion => command::get_version(client).await?,
        Command::GetOwner => command::get_owner(client).await?,
        Command::GetBridgeProver => command::get_bridge_prover(client).await?,
        Command::GetNonce { address } => command::get_nonce(client, address).await?,
        Command::GetCode { address } => command::get_code(client, address).await?,
        Command::GetBalance { address } => command::get_balance(client, address).await?,
        Command::Call {
            address, function, ..
        } => command::call(client, address, function, args.aurora_secret_key.as_deref()).await?,
        Command::StageUpgrade => command::stage_upgrade(client).await?,
        Command::DeployUpgrade => command::deploy_upgrade(client).await?,
        Command::GetStorageAt { address, key } => {
            command::get_storage_at(client, address, key).await?;
        }
        Command::DeployEvmCode { code } => {
            command::deploy_evm_code(client, code, args.aurora_secret_key.as_deref()).await?;
        }
        Command::DeployAurora { path } => command::deploy_aurora(client, path).await?,
        Command::CreateAccount { account, balance } => {
            command::create_account(client, &account, balance).await?;
        }
        Command::ViewAccount { account } => command::view_account(client, &account).await?,
        Command::Init {
            chain_id,
            owner_id,
            bridge_prover_id,
            upgrade_delay_blocks,
        } => {
            command::init(
                client,
                chain_id,
                owner_id,
                bridge_prover_id,
                upgrade_delay_blocks,
            )
            .await?;
        }
        Command::EncodeAddress { account } => command::encode_address(&account),
    }

    Ok(())
}
