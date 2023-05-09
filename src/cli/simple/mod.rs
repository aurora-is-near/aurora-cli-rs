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
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create new NEAR account
    CreateAccount {
        /// AccountId
        #[arg(long, short)]
        account: String,
        /// Initial account balance in NEAR  
        #[arg(long, short)]
        balance: f64,
    },
    /// View NEAR account
    ViewAccount {
        /// AccountId
        account: String,
    },
    /// Deploy Aurora EVM smart contract
    DeployAurora {
        /// Path to the WASM file
        path: String,
    },
    /// Initialize Aurora EVM and ETH connector
    Init {
        /// Chain ID
        #[arg(long, default_value_t = 1313161556)]
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
    /// Return chain id of the network
    GetChainId,
    /// Return next nonce for address
    GetNonce {
        address: String,
    },
    /// Return block hash of the specified height
    GetBlockHash {
        height: u64,
    },
    /// Return smart contract's code for contract address
    GetCode {
        address: String,
    },
    /// Return balance for address
    GetBalance {
        address: String,
    },
    /// Return a height for a staged upgrade
    GetUpgradeIndex,
    /// Return Aurora EVM version
    GetVersion,
    /// Return Aurora EVM owner
    GetOwner,
    /// Set a new owner of Aurora EVM
    SetOwner {
        account_id: String,
    },
    /// Return bridge prover
    GetBridgeProver,
    /// Return a value from storage at address with key
    GetStorageAt {
        #[arg(short, long)]
        address: String,
        #[arg(short, long)]
        key: String,
    },
    /// Register relayer address
    RegisterRelayer {
        address: String,
    },
    /// Pause precompiles
    PausePrecompiles {
        mask: u32,
    },
    /// Resume precompiles
    ResumePrecompiles {
        mask: u32,
    },
    /// Return paused precompiles
    PausedPrecompiles,
    /// Updates the bytecode for user's router contracts
    FactoryUpdate {
        path: String,
    },
    /// Sets the address for the `wNEAR` ERC-20 contract
    FactorySetWnearAddress {
        address: String,
    },
    /// Create and/or fund an XCC sub-account directly
    FundXccSubAccount {
        /// Address of the target
        target: String,
        /// Wnear Account Id
        wnear_account_id: Option<String>,
    },
    /// Stage a new code for upgrade
    StageUpgrade {
        path: String,
    },
    /// Deploy staged upgrade
    DeployUpgrade,
    /// Deploy EVM smart contract's code in hex
    Deploy {
        /// Code in HEX to deploy
        #[arg(long)]
        code: String,
        /// Constructor arguments with values in JSON
        #[arg(long)]
        args: Option<String>,
        /// Path to ABI of the contract
        #[arg(long)]
        abi_path: Option<String>,
        /// Aurora EVM secret key
        #[arg(long)]
        aurora_secret_key: Option<String>,
    },
    /// Call a view method of the smart contract
    ViewCall {
        /// Address of the smart contract
        #[arg(long, short)]
        address: String,
        /// Name of the function to call
        #[arg(long, short)]
        function: String,
        /// Arguments with values in JSON
        #[arg(long)]
        args: Option<String>,
        /// Path to ABI of the contract
        #[arg(long)]
        abi_path: String,
    },
    /// Call a modified method of the smart contract
    Call {
        /// Address of the smart contract
        #[arg(long, short)]
        address: String,
        /// Name of the function to call
        #[arg(long, short)]
        function: String,
        /// Arguments with values in JSON
        #[arg(long)]
        args: Option<String>,
        /// Path to ABI of the contract
        #[arg(long)]
        abi_path: String,
        /// Value sending in EVM transaction
        #[arg(long)]
        value: Option<String>,
        /// Aurora EVM secret key
        #[arg(long)]
        aurora_secret_key: Option<String>,
    },
    /// Encode address
    EncodeAddress {
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
    // Silo Specific Methods
    GetFixedGasCost,
    SetFixedGasCost {
        cost: u64,
    },
    GetWhitelistStatus {
        kind: String,
    },
    SetWhitelistStatus {
        kind: String,
        status: bool,
    },
    AddEntryToWhitelist {
        whitelistArgs: String,
        kind: String,
        address: String,
    },
    AddEntryToWhitelistBatch {
        whitelistArgs: Vec<String>,
        kind: Vec<String>,
        address: Vec<String>,
    },
    RemoveEntryFromWhitelist {
        whitelistArgs: String,
        kind: String,
        address: String,
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
        Command::GetVersion => command::get_version(client).await?,
        Command::GetOwner => command::get_owner(client).await?,
        Command::SetOwner { account_id } => command::set_owner(client, account_id).await?,
        Command::RegisterRelayer { address } => command::register_relayer(client, address).await?,
        Command::GetBridgeProver => command::get_bridge_prover(client).await?,
        Command::GetNonce { address } => command::get_nonce(client, address).await?,
        Command::GetCode { address } => command::get_code(client, address).await?,
        Command::GetBalance { address } => command::get_balance(client, address).await?,
        Command::GetBlockHash { height } => command::get_block_hash(client, height).await?,
        Command::Call {
            address,
            function,
            args,
            abi_path,
            value,
            aurora_secret_key,
        } => {
            command::call(
                client,
                address,
                function,
                args,
                abi_path,
                value,
                aurora_secret_key.as_deref(),
            )
            .await?;
        }
        Command::ViewCall {
            address,
            function,
            args,
            abi_path,
        } => command::view_call(client, address, function, args, abi_path).await?,
        Command::PausePrecompiles { mask } => command::pause_precompiles(client, mask).await?,
        Command::ResumePrecompiles { mask } => command::resume_precompiles(client, mask).await?,
        Command::PausedPrecompiles => command::paused_precompiles(client).await?,
        Command::GetUpgradeIndex => command::get_upgrade_index(client).await?,
        Command::FactoryUpdate { path } => command::factory_update(client, path).await?,
        Command::FactorySetWnearAddress { address } => {
            command::factory_set_wnear_address(client, address).await?;
        }
        Command::FundXccSubAccount {
            target,
            wnear_account_id,
        } => {
            command::fund_xcc_sub_account(client, target, wnear_account_id).await?;
        }
        Command::StageUpgrade { path } => command::stage_upgrade(client, path).await?,
        Command::DeployUpgrade => command::deploy_upgrade(client).await?,
        Command::GetStorageAt { address, key } => {
            command::get_storage_at(client, address, key).await?;
        }
        Command::Deploy {
            code,
            abi_path,
            args,
            aurora_secret_key,
        } => {
            command::deploy_evm_code(client, code, abi_path, args, aurora_secret_key.as_deref())
                .await?;
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
        // Silo Specific Methods
        Command::GetFixedGasCost => command::get_fixed_gas_cost(client).await?,
        Command::SetFixedGasCost { cost } => command::set_fixed_gas_cost(client, cost).await?,
        Command::GetWhitelistStatus { kind } => command::get_whitelist_status(client, kind).await?,
        Command::SetWhitelistStatus { kind, status } => {
            command::set_whitelist_status(client, kind, status).await?
        }
        Command::AddEntryToWhitelist {
            whitelistArgs,
            kind,
            address,
        } => command::add_entity_to_whitelist(client, whitelistArgs, kind, address).await?,
        Command::AddEntryToWhitelistBatch {
            whitelistArgs,
            kind,
            address,
        } => {
            assert!(whitelistArgs.len() == kind.len() && kind.len() == address.len());
            command::add_entry_to_whitelist_batch(client, whitelistArgs, kind, address).await?
        }
        Command::RemoveEntryFromWhitelist {
            whitelistArgs,
            kind,
            address,
        } => command::remove_entry_from_whitelist(client, whitelistArgs, kind, address).await?,
    }

    Ok(())
}
