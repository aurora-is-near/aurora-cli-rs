use clap::{Parser, Subcommand};
use std::str::FromStr;

pub mod command;

/// Simple command line interface for communication with Aurora Engine
#[derive(Parser)]
#[command(author, version, long_about = None)]
pub struct Cli {
    /// NEAR network ID
    #[arg(long, default_value = "localnet")]
    pub network: Network,
    /// Aurora EVM account
    #[arg(long, value_name = "ACCOUNT_ID", default_value = "aurora")]
    pub engine: String,
    /// Path to file with NEAR account id and secret key in JSON format
    #[arg(long)]
    pub near_key_path: Option<String>,
    /// Aurora API key
    #[arg(long)]
    pub aurora_api_key: Option<String>,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Deploy Aurora EVM smart contract
    DeployAurora {
        #[arg(action)]
        path: String,
    },
    /// Create new NEAR account
    CreateAccount {
        #[arg(long)]
        account: String,
        #[arg(long)]
        balance: f64,
    },
    /// View new NEAR account
    ViewAccount {
        #[arg(action)]
        account: String,
    },
    /// Initialize Aurora EVM and ETH connector
    Init {
        /// Chain ID
        #[arg(long, default_value = "1313161556")]
        chain_id: u64,
        /// Owner of the Aurora EVM
        #[arg(long)]
        owner_id: Option<String>,
        /// Account of the bridge prover
        #[arg(long)]
        bridge_prover_id: Option<String>,
        /// How many blocks after staging upgrade can deploy it
        #[arg(long)]
        upgrade_delay_blocks: Option<u64>,
        /// Custodian ETH address
        #[arg(long)]
        custodian_address: Option<String>,
        /// Path to the file with the metadata of the fungible token
        #[arg(long)]
        ft_metadata_path: Option<String>,
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
        #[arg(action)]
        address: String,
    },
    /// Return smart contract's code for contract address
    GetCode {
        #[arg(action, value_name = "address")]
        address: String,
    },
    /// Return balance for address
    GetBalance {
        #[arg(action)]
        address: String,
    },
    /// Call method of a smart contract
    Call {
        #[arg(long)]
        address: String,
        #[arg(long)]
        function: String,
        #[arg(long)]
        input: String,
        /// Aurora EVM secret key
        #[arg(long)]
        aurora_secret_key: Option<String>,
    },
    /// Return a value from storage at address with key
    GetStorageAt {
        #[arg(long)]
        address: String,
        #[arg(long)]
        key: String,
    },
    /// Deploy EVM smart contract's code in hex
    DeployEvmCode {
        /// Code in HEX to deploy
        #[arg(long)]
        code: String,
        /// Aurora EVM secret key
        #[arg(long)]
        aurora_secret_key: Option<String>,
    },
    /// Encode address
    EncodeAddress {
        /// Account ID
        account: String,
    },
    /// Return Public and Secret ED25519 keys
    KeyPair {
        /// Random
        #[arg(long, default_value = "false")]
        random: bool,
        /// From seed
        #[arg(long)]
        seed: Option<u64>,
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
    let client = crate::client::Client::new(near_rpc, &args.engine, args.near_key_path);

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
            address,
            function,
            input: _,
            aurora_secret_key,
        } => command::call(client, address, function, aurora_secret_key.as_deref()).await?,
        Command::StageUpgrade => command::stage_upgrade(client).await?,
        Command::DeployUpgrade => command::deploy_upgrade(client).await?,
        Command::GetStorageAt { address, key } => {
            command::get_storage_at(client, address, key).await?;
        }
        Command::DeployEvmCode {
            code,
            aurora_secret_key,
        } => {
            command::deploy_evm_code(client, code, aurora_secret_key.as_deref()).await?;
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
            custodian_address,
            ft_metadata_path,
        } => {
            command::init(
                client,
                chain_id,
                owner_id,
                bridge_prover_id,
                upgrade_delay_blocks,
                custodian_address,
                ft_metadata_path,
            )
            .await?;
        }
        Command::EncodeAddress { account } => command::encode_address(&account),
        Command::KeyPair { random, seed } => command::key_pair(random, seed)?,
    }

    Ok(())
}
