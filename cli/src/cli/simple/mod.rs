use aurora_engine_types::account_id::AccountId;
use aurora_engine_types::public_key::{KeyType, PublicKey};
use clap::{Parser, Subcommand, ValueEnum};
use near_primitives::hash::CryptoHash;
use shadow_rs::shadow;
use std::str::FromStr;
use std::sync::LazyLock;

use crate::utils::near_to_yocto;

pub mod command;

static VERSION: LazyLock<String> = LazyLock::new(|| {
    shadow!(build);
    format!("{}-{}", build::PKG_VERSION, build::SHORT_COMMIT)
});

fn get_version() -> &'static str {
    VERSION.as_str()
}

/// Simple command line interface for communication with Aurora Engine
#[derive(Parser)]
#[command(author, long_about = None)]
#[command(version = get_version())]
pub struct Cli {
    /// NEAR network ID
    #[arg(long, value_enum, default_value_t = Network::Localnet)]
    pub network: Network,
    /// Aurora EVM account
    #[arg(long, value_name = "ACCOUNT_ID", default_value = "aurora")]
    pub engine: String,
    /// The way output of a command would be formatted
    #[arg(long, default_value = "plain")]
    pub output_format: OutputFormat,
    /// Path to file with NEAR account id and secret key in JSON format
    #[arg(long)]
    pub near_key_path: Option<String>,
    /// NEAR block height to use for getting data
    #[arg(long)]
    pub block_height: Option<u64>,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create new NEAR account
    CreateAccount {
        /// `AccountId`
        #[arg(long, short)]
        account: String,
        /// Initial account balance in NEAR
        #[arg(long, short)]
        balance: f64,
    },
    /// View NEAR account
    ViewAccount {
        /// `AccountId`
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
    GetNonce { address: String },
    /// Return block hash of the specified height
    GetBlockHash { height: u64 },
    /// Return smart contract's code for contract address
    GetCode { address: String },
    /// Return balance for address
    GetBalance { address: String },
    /// Return a height for a staged upgrade
    GetUpgradeIndex,
    /// Return Aurora EVM version
    GetVersion,
    /// Return Aurora EVM owner
    GetOwner,
    /// Set a new owner of Aurora EVM
    SetOwner { account_id: String },
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
    RegisterRelayer { address: String },
    /// Start hashchain
    StartHashchain {
        /// Height of the block to start the hashchain
        #[arg(long)]
        block_height: u64,
        /// Hashchain of the block to start the hashchain
        #[arg(long)]
        block_hashchain: String,
    },
    /// Pause contract
    PauseContract,
    /// Resume contract
    ResumeContract,
    /// Pause precompiles
    PausePrecompiles { mask: u32 },
    /// Resume precompiles
    ResumePrecompiles { mask: u32 },
    /// Return paused precompiles
    PausedPrecompiles,
    /// Updates the bytecode for user's router contracts
    FactoryUpdate { path: String },
    /// Return the address of the `wNEAR` ERC-20 contract
    FactoryGetWnearAddress,
    /// Sets the address for the `wNEAR` ERC-20 contract
    FactorySetWnearAddress { address: String },
    /// Create and/or fund an XCC sub-account directly
    FundXccSubAccount {
        /// Address of the target
        target: String,
        /// Wnear Account Id
        wnear_account_id: Option<String>,
        /// Attached deposit in NEAR
        deposit: f64,
    },
    /// Upgrade contract with provided code
    Upgrade { path: String },
    /// Stage a new code for upgrade
    StageUpgrade { path: String },
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
    /// Call a method of the smart contract
    Call {
        /// Address of the smart contract
        #[arg(long)]
        address: String,
        /// Input data of the EVM transaction encoded in hex
        #[arg(long)]
        input: Option<String>,
        /// Attached value in EVM transaction
        #[arg(long)]
        value: Option<u128>,
        /// From `account_id`
        #[arg(long, value_parser = parse_account_id)]
        from: Option<AccountId>,
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
        /// Sender address
        #[arg(long)]
        from: String,
        /// Path to ABI of the contract
        #[arg(long)]
        abi_path: String,
    },
    /// Call a modified method of the smart contract
    Submit {
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
    EncodeAddress { account: String },
    /// Return Public and Secret ED25519 keys
    KeyPair {
        /// Random
        #[arg(long, default_value = "false")]
        random: bool,
        /// From seed
        #[arg(long)]
        seed: Option<u64>,
    },
    /// Return randomly generated NEAR key for `AccountId`
    GenerateNearKey {
        /// `AccountId`
        account_id: String,
        /// Key type: ed25519 or secp256k1
        key_type: KeyType,
    },
    /// Return fixed gas
    GetFixedGas,
    /// Set fixed gas
    SetFixedGas {
        /// Fixed gas in `EthGas`.
        cost: u64,
    },
    /// Return Silo params
    GetSiloParams,
    /// Set SILO params.
    SetSiloParams {
        /// Fixed gas in `EthGas`.
        #[arg(long, short)]
        gas: u64,
        /// Fallback EVM address.
        #[arg(long, short)]
        fallback_address: String,
    },
    /// Disable SILO mode.
    DisableSiloMode,
    /// Return a status of the whitelist
    GetWhitelistStatus {
        /// Kind of the whitelist.
        kind: String,
    },
    /// Set a status for the whitelist
    SetWhitelistStatus {
        /// Kind of the whitelist.
        #[arg(long)]
        kind: String,
        /// Status of the whitelist, 0/1.
        #[arg(long)]
        status: u8,
    },
    /// Add entry into the whitelist
    AddEntryToWhitelist {
        /// Kind of the whitelist.
        #[arg(long)]
        kind: String,
        /// Entry for adding to the whitelist.
        #[arg(long)]
        entry: String,
    },
    /// Add entries into the whitelist
    AddEntryToWhitelistBatch {
        /// Path to JSON file with array of entries.
        path: String,
    },
    /// Remove the entry from the whitelist
    RemoveEntryFromWhitelist {
        /// Kind of the whitelist.
        #[arg(long)]
        kind: String,
        /// Entry for removing from the whitelist.
        #[arg(long)]
        entry: String,
    },
    /// Set relayer key manager
    SetKeyManager {
        /// `AccountId` of the key manager
        #[arg(value_parser = parse_account_id)]
        account_id: Option<AccountId>,
    },
    /// Add relayer public key
    AddRelayerKey {
        /// Public key
        #[arg(long)]
        public_key: PublicKey,
        /// Allowance
        #[arg(long)]
        allowance: f64,
    },
    /// Remove relayer public key
    RemoveRelayerKey {
        /// Public key
        public_key: PublicKey,
    },
    /// Get delay for upgrade in blocks
    GetUpgradeDelayBlocks,
    /// Set delay for upgrade in blocks
    SetUpgradeDelayBlocks {
        /// Number blocks
        blocks: u64,
    },
    /// Get ERC-20 from NEP-141
    GetErc20FromNep141 {
        /// Account id of NEP-141
        account_id: String,
    },
    /// Get NEP-141 from ERC-20
    GetNep141FromErc20 {
        /// Address for ERC-20
        address: String,
    },
    /// Get ERC-20 metadata
    GetErc20Metadata {
        /// Address or account id of the ERC-20 contract
        erc20_id: String,
    },
    /// Set ERC-20 metadata
    SetErc20Metadata {
        /// Address or account id of the ERC-20 contract
        #[arg(long)]
        erc20_id: String,
        /// Name of the token
        #[arg(long)]
        name: String,
        /// Symbol of the token
        #[arg(long)]
        symbol: String,
        /// Decimals of the token
        #[arg(long)]
        decimals: u8,
    },
    /// Mirror ERC-20 token
    MirrorErc20Token {
        /// Account of contract where ERC-20 has been deployed
        #[arg(long)]
        contract_id: String,
        /// Account of corresponding NEP-141
        #[arg(long)]
        nep141: String,
    },
    /// Set eth connector account id
    SetEthConnectorContractAccount {
        /// Account id of eth connector
        #[arg(long)]
        account_id: String,
        /// Serialization type in withdraw method
        #[arg(long)]
        withdraw_ser: Option<WithdrawSerialization>,
    },
    /// Get eth connector account id
    GetEthConnectorContractAccount,
    /// Set eth connector data
    SetEthConnectorContractData {
        /// Prover account id
        #[arg(long)]
        prover_id: String,
        /// Custodian ETH address
        #[arg(long)]
        custodian_address: String,
        /// Path to the file with the metadata of the fungible token
        #[arg(long)]
        ft_metadata_path: String,
    },
    /// Set eth connector paused flags
    SetPausedFlags {
        /// Pause mask
        mask: u8,
    },
    /// Get eth connector paused flags
    GetPausedFlags,
    /// Get transaction status
    TransactionStatus {
        /// Transaction hash
        #[arg(long)]
        hash: CryptoHash,
        /// Wait until the transaction is in the `wait_until` state
        #[arg(long, default_value_t = command::WaitUntil::Final)]
        wait_until: command::WaitUntil,
    },

    /// Add relayer
    AddRelayer {
        #[arg(long)]
        deposit: f64,
        #[arg(long)]
        full_access_pub_key: near_crypto::PublicKey,
        #[arg(long)]
        function_call_pub_key: near_crypto::PublicKey,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Network {
    Localnet,
    Mainnet,
    Testnet,
}

#[derive(Default, Clone)]
pub enum OutputFormat {
    #[default]
    Plain,
    Json,
    Toml,
}

impl FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "plain" => Ok(Self::Plain),
            "json" => Ok(Self::Json),
            "toml" => Ok(Self::Toml),
            _ => anyhow::bail!("unknown output format: {s}"),
        }
    }
}

#[derive(Clone)]
pub enum WithdrawSerialization {
    Borsh,
    Json,
}

impl FromStr for WithdrawSerialization {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "borsh" => Ok(Self::Borsh),
            "json" => Ok(Self::Json),
            _ => anyhow::bail!("unknown withdraw serialization type: {s}"),
        }
    }
}

pub async fn run(args: Cli) -> anyhow::Result<()> {
    let near_rpc = parse_near_rpc(&args.network)?;
    let client = crate::client::Client::new(&near_rpc, &args.engine, args.near_key_path);
    let context = crate::client::Context::new(client, args.output_format, args.block_height);

    match args.command {
        Command::GetChainId => command::get_chain_id(context).await?,
        Command::GetVersion => command::get_version(context).await?,
        Command::GetOwner => command::get_owner(context).await?,
        Command::SetOwner { account_id } => command::set_owner(context, account_id).await?,
        Command::RegisterRelayer { address } => command::register_relayer(context, address).await?,
        Command::StartHashchain {
            block_height,
            block_hashchain,
        } => command::start_hashchain(context, block_height, block_hashchain).await?,
        Command::PauseContract => command::pause_contract(context).await?,
        Command::ResumeContract => command::resume_contract(context).await?,
        Command::GetBridgeProver => command::get_bridge_prover(context).await?,
        Command::GetNonce { address } => command::get_nonce(context, address).await?,
        Command::GetCode { address } => command::get_code(context, address).await?,
        Command::GetBalance { address } => command::get_balance(context, address).await?,
        Command::GetBlockHash { height } => command::get_block_hash(context, height).await?,
        Command::Submit {
            address,
            function,
            args,
            abi_path,
            value,
            aurora_secret_key,
        } => {
            command::submit(
                context,
                address,
                function,
                args,
                abi_path,
                value,
                aurora_secret_key.as_deref(),
            )
            .await?;
        }
        Command::Call {
            address,
            input,
            value,
            from,
        } => command::call(context, address, input, value, from).await?,
        Command::ViewCall {
            address,
            function,
            args,
            from,
            abi_path,
        } => command::view_call(context, address, function, args, from, abi_path).await?,
        Command::PausePrecompiles { mask } => command::pause_precompiles(context, mask).await?,
        Command::ResumePrecompiles { mask } => command::resume_precompiles(context, mask).await?,
        Command::PausedPrecompiles => command::paused_precompiles(context).await?,
        Command::GetUpgradeIndex => command::get_upgrade_index(context).await?,
        Command::FactoryUpdate { path } => command::factory_update(context, path).await?,
        Command::FactoryGetWnearAddress => command::factory_get_wnear_address(context).await?,
        Command::FactorySetWnearAddress { address } => {
            command::factory_set_wnear_address(context, address).await?;
        }
        Command::FundXccSubAccount {
            target,
            wnear_account_id,
            deposit,
        } => {
            command::fund_xcc_sub_account(context, target, wnear_account_id, deposit).await?;
        }
        Command::Upgrade { path } => command::upgrade(context, path).await?,
        Command::StageUpgrade { path } => command::stage_upgrade(context, path).await?,
        Command::DeployUpgrade => command::deploy_upgrade(context).await?,
        Command::GetStorageAt { address, key } => {
            command::get_storage_at(context, address, key).await?;
        }
        Command::Deploy {
            code,
            abi_path,
            args,
            aurora_secret_key,
        } => {
            command::deploy_evm_code(context, code, abi_path, args, aurora_secret_key.as_deref())
                .await?;
        }
        Command::DeployAurora { path } => command::deploy_aurora(context, path).await?,
        Command::CreateAccount { account, balance } => {
            command::create_account(context, &account, balance).await?;
        }
        Command::ViewAccount { account } => command::view_account(context, &account).await?,
        Command::Init {
            chain_id,
            owner_id,
            bridge_prover_id,
            upgrade_delay_blocks,
            custodian_address,
            ft_metadata_path,
        } => {
            command::init(
                context,
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
        Command::GenerateNearKey {
            account_id,
            key_type,
        } => command::gen_near_key(&account_id, key_type)?,
        // Silo Specific Methods
        Command::GetFixedGas => command::silo::get_fixed_gas_cost(context).await?,
        Command::SetFixedGas { cost } => {
            command::silo::set_fixed_gas(context, cost).await?;
        }
        Command::GetSiloParams => command::silo::get_silo_params(context).await?,
        Command::SetSiloParams {
            gas,
            fallback_address,
        } => {
            command::silo::set_silo_params(context, gas, fallback_address).await?;
        }
        Command::DisableSiloMode => {
            command::silo::disable_silo_mode(context).await?;
        }
        Command::GetWhitelistStatus { kind } => {
            command::silo::get_whitelist_status(context, kind).await?;
        }
        Command::SetWhitelistStatus { kind, status } => {
            command::silo::set_whitelist_status(context, kind, status).await?;
        }
        Command::AddEntryToWhitelist { kind, entry } => {
            command::silo::add_entry_to_whitelist(context, kind, entry).await?;
        }
        Command::AddEntryToWhitelistBatch { path } => {
            command::silo::add_entry_to_whitelist_batch(context, path).await?;
        }
        Command::RemoveEntryFromWhitelist { kind, entry } => {
            command::silo::remove_entry_from_whitelist(context, kind, entry).await?;
        }
        Command::SetKeyManager { account_id } => {
            command::set_key_manager(context, account_id).await?;
        }
        Command::AddRelayerKey {
            public_key,
            allowance,
        } => {
            command::add_relayer_key(context, public_key, allowance).await?;
        }
        Command::RemoveRelayerKey { public_key } => {
            command::remove_relayer_key(context, public_key).await?;
        }
        Command::GetUpgradeDelayBlocks => {
            command::get_upgrade_delay_blocks(context).await?;
        }
        Command::SetUpgradeDelayBlocks { blocks } => {
            command::set_upgrade_delay_blocks(context, blocks).await?;
        }
        Command::GetErc20FromNep141 { account_id } => {
            command::get_erc20_from_nep141(context, account_id).await?;
        }
        Command::GetNep141FromErc20 { address } => {
            command::get_nep141_from_erc20(context, address).await?;
        }
        Command::GetErc20Metadata { erc20_id } => {
            command::get_erc20_metadata(context, erc20_id).await?;
        }
        Command::SetErc20Metadata {
            erc20_id,
            name,
            symbol,
            decimals,
        } => {
            command::set_erc20_metadata(context, erc20_id, name, symbol, decimals).await?;
        }
        Command::MirrorErc20Token {
            contract_id,
            nep141,
        } => {
            command::mirror_erc20_token(context, contract_id, nep141).await?;
        }
        Command::SetEthConnectorContractAccount {
            account_id,
            withdraw_ser,
        } => {
            command::set_eth_connector_contract_account(context, account_id, withdraw_ser).await?;
        }
        Command::GetEthConnectorContractAccount => {
            command::get_eth_connector_contract_account(context).await?;
        }
        Command::SetEthConnectorContractData {
            prover_id,
            custodian_address,
            ft_metadata_path,
        } => {
            command::set_eth_connector_contract_data(
                context,
                prover_id,
                custodian_address,
                ft_metadata_path,
            )
            .await?;
        }
        Command::SetPausedFlags { mask } => {
            command::set_paused_flags(context, mask).await?;
        }
        Command::GetPausedFlags => {
            command::get_paused_flags(context).await?;
        }
        Command::TransactionStatus { hash, wait_until } => {
            command::transaction_status(context, hash, wait_until).await?;
        }
        Command::AddRelayer {
            deposit,
            full_access_pub_key,
            function_call_pub_key,
        } => {
            command::add_relayer(
                context,
                near_to_yocto(deposit),
                full_access_pub_key,
                function_call_pub_key,
            )
            .await?;
        }
    }

    Ok(())
}

fn parse_account_id(arg: &str) -> anyhow::Result<AccountId> {
    arg.parse().map_err(|e| anyhow::anyhow!("{e}"))
}

fn parse_near_rpc(network: &Network) -> anyhow::Result<String> {
    std::env::var("NEAR_RPC_URL").or_else(|_| {
        let endpoint = match network {
            Network::Mainnet => super::NEAR_MAINNET_ENDPOINT,
            Network::Testnet => super::NEAR_TESTNET_ENDPOINT,
            Network::Localnet => super::NEAR_LOCAL_ENDPOINT,
        };

        Ok(endpoint.to_string())
    })
}
