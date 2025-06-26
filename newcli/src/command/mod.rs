use std::{fs, path::PathBuf};

use aurora_sdk_rs::{
    aurora::{
        self, H256, U256,
        parameters::{
            connector::{FungibleTokenMetadata, WithdrawSerializeType},
            silo::WhitelistKind,
        },
        types::{Address, EthGas, Wei},
    },
    near::{
        crypto::{KeyType, PublicKey},
        primitives::types::AccountId,
        token::NearToken,
    },
};
use clap::Subcommand;

use crate::{
    cli::Cli,
    common,
    context::{self, Context},
};

mod near;

#[derive(Clone, Subcommand)]
pub enum Command {
    /// Create new NEAR account
    CreateAccount {
        /// `AccountId`
        #[arg(long, short)]
        account: AccountId,
        /// Initial account balance in NEAR
        #[arg(long, short)]
        balance: NearToken,
    },
    /// View NEAR account
    ViewAccount {
        /// `AccountId`
        account: AccountId,
    },
    /// Deploy Aurora EVM smart contract
    DeployAurora {
        /// Path to the WASM file
        path: PathBuf,
    },
    /// Initialize Aurora EVM and ETH connector
    Init {
        /// Chain ID
        #[arg(long, default_value_t = 1313161556)]
        chain_id: u64,
        /// Owner of the Aurora EVM
        #[arg(long)]
        owner_id: AccountId,
        /// Account of the bridge prover
        #[arg(long)]
        bridge_prover_id: AccountId,
        /// How many blocks after staging upgrade can deploy it
        #[arg(long)]
        upgrade_delay_blocks: Option<u64>,
        /// Custodian ETH address
        #[arg(long, value_parser = parse_address)]
        custodian_address: Option<Address>,
        /// Path to the file with the metadata of the fungible token
        #[arg(long, value_parser = parse_ft_metadata_path)]
        ft_metadata_path: FungibleTokenMetadata,
    },
    /// Return chain id of the network
    GetChainId,
    /// Return next nonce for address
    GetNonce {
        #[arg(short, long, value_parser = parse_address)]
        address: Address,
    },
    /// Return block hash of the specified height
    GetBlockHash { height: u64 },
    /// Return smart contract's code for contract address
    GetCode {
        #[arg(short, long, value_parser = parse_address)]
        address: Address,
    },
    /// Return balance for address
    GetBalance {
        #[arg(short, long, value_parser = parse_address)]
        address: Address,
    },
    /// Return a height for a staged upgrade
    GetUpgradeIndex,
    /// Return Aurora EVM version
    GetVersion,
    /// Return Aurora EVM owner
    GetOwner,
    /// Set a new owner of Aurora EVM
    SetOwner { account_id: AccountId },
    /// Return bridge prover
    GetBridgeProver,
    /// Return a value from storage at address with key
    GetStorageAt {
        #[arg(short, long, value_parser = parse_address)]
        address: Address,
        #[arg(short, long)]
        key: H256,
    },
    /// Register relayer address
    RegisterRelayer {
        #[arg(short, long, value_parser = parse_address)]
        address: Address,
    },
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
    FactorySetWnearAddress {
        #[arg(value_parser = parse_address)]
        address: Address,
    },
    /// Create and/or fund an XCC sub-account directly
    FundXccSubAccount {
        /// Address of the target
        #[arg(value_parser = parse_address)]
        target: Address,
        /// Wnear Account Id
        wnear_account_id: Option<AccountId>,
        /// Attached deposit in NEAR
        deposit: NearToken,
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
        #[arg(long, value_parser = parse_address)]
        address: Address,
        /// Input data of the EVM transaction encoded in hex
        #[arg(long)]
        input: Option<String>,
        /// Attached value in EVM transaction
        #[arg(long)]
        value: Option<u128>,
        /// From `account_id`
        #[arg(long)]
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
        #[arg(long, short, value_parser = parse_address)]
        address: Address,
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
        #[arg(long, value_parser = parse_wei)]
        value: Wei,
        /// Aurora EVM secret key
        #[arg(long)]
        aurora_secret_key: String,
    },
    /// Encode address
    EncodeAddress { account: AccountId },
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
        account_id: AccountId,
        /// Key type: ed25519 or secp256k1
        key_type: KeyType,
    },
    /// Return fixed gas
    GetFixedGas,
    /// Set fixed gas
    SetFixedGas {
        /// Fixed gas in `EthGas`.
        #[arg(value_parser = parse_eth_gas)]
        cost: EthGas,
    },
    /// Return Silo params
    GetSiloParams,
    /// Set SILO params.
    SetSiloParams {
        /// Fixed gas in `EthGas`.
        #[arg(long, short, value_parser = parse_eth_gas)]
        gas: EthGas,
        /// Fallback EVM address.
        #[arg(long, short, value_parser = parse_address)]
        fallback_address: Address,
    },
    /// Disable SILO mode.
    DisableSiloMode,
    /// Return a status of the whitelist
    GetWhitelistStatus {
        /// Kind of the whitelist.
        #[arg(value_parser = parse_whitelist_kind)]
        kind: WhitelistKind,
    },
    /// Set a status for the whitelist
    SetWhitelistStatus {
        /// Kind of the whitelist.
        #[arg(value_parser = parse_whitelist_kind)]
        kind: WhitelistKind,
        /// Status of the whitelist, 0/1.
        #[arg(long)]
        status: u8,
    },
    /// Add entry into the whitelist
    AddEntryToWhitelist {
        /// Kind of the whitelist.
        #[arg(long, value_parser = parse_whitelist_kind)]
        kind: WhitelistKind,
        /// Entry for adding to the whitelist.
        #[arg(long)]
        entry: String,
    },
    /// Remove the entry from the whitelist
    RemoveEntryFromWhitelist {
        /// Kind of the whitelist.
        #[arg(long, value_parser = parse_whitelist_kind)]
        kind: WhitelistKind,
        /// Entry for removing from the whitelist.
        #[arg(long)]
        entry: String,
    },
    /// Set relayer key manager
    SetKeyManager {
        /// `AccountId` of the key manager
        account_id: Option<AccountId>,
    },
    /// Add relayer public key
    AddRelayerKey {
        /// Public key
        #[arg(long)]
        public_key: PublicKey,
        /// Allowance
        #[arg(long)]
        allowance: NearToken,
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
        account_id: AccountId,
    },
    /// Get NEP-141 from ERC-20
    GetNep141FromErc20 {
        /// Address for ERC-20
        #[arg(value_parser = parse_address)]
        address: Address,
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
        contract_id: AccountId,
        /// Account of corresponding NEP-141
        #[arg(long)]
        nep141: AccountId,
    },
    /// Set eth connector account id
    SetEthConnectorContractAccount {
        /// Account id of eth connector
        #[arg(long)]
        account_id: AccountId,
        /// Serialization type in withdraw method
        #[arg(long, value_parser = parse_withdraw_serialize_type, default_value = "borsh")]
        withdraw_ser: Option<WithdrawSerializeType>,
    },
    /// Get eth connector account id
    GetEthConnectorContractAccount,
    /// Set eth connector data
    SetEthConnectorContractData {
        /// Prover account id
        #[arg(long)]
        prover_id: AccountId,
        /// Custodian ETH address
        #[arg(long, value_parser = parse_address)]
        custodian_address: Address,
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
    /// Add relayer
    AddRelayer {
        #[arg(long)]
        deposit: NearToken,
        #[arg(long)]
        full_access_pub_key: PublicKey,
        #[arg(long)]
        function_call_pub_key: PublicKey,
    },
}

pub async fn run(cli: Cli) -> anyhow::Result<()> {
    let near_client =
        aurora_sdk_rs::near::client::Client::new(cli.network.rpc_url(), None, cli.signer()?)?;
    let aurora_client = aurora::client::Client::new(near_client);

    let context = &Context {
        cli,
        client: aurora_client,
    };

    match context.cli.command.clone() {
        Command::CreateAccount { account, balance } => {
            let outcome = near::create_account(context, account, balance).await?;
            println!("Account created: {:?}", outcome);
        }
        Command::ViewAccount { ref account } => {
            let view = near::view_account(context, account).await?;
            println!("Account view: {:?}", view);
        }
        Command::DeployAurora { ref path } => {
            let wasm = std::fs::read(path)?;
            let outcome = near::deploy_aurora(context, wasm).await?;
            println!("Aurora deployed: {:?}", outcome);
        }
        Command::Init {
            chain_id,
            owner_id,
            bridge_prover_id,
            upgrade_delay_blocks,
            custodian_address,
            ft_metadata_path,
        } => {
            near::init(
                context,
                chain_id,
                owner_id,
                bridge_prover_id,
                upgrade_delay_blocks,
                custodian_address,
                ft_metadata_path,
            )
            .await?;

            println!("Aurora EVM and ETH connector initialized successfully");
        }
        Command::GetChainId => {
            let id = near::get_chain_id(context).await?;
            println!("Chain ID: {}", id);
        }
        Command::GetNonce { address } => {
            let nonce = near::get_nonce(context, address).await?;
            println!("Nonce: {}", nonce);
        }
        Command::GetBlockHash { height } => {
            let hash = near::get_block_hash(context, height).await?;
            println!("Block hash: {}", hash);
        }
        Command::GetCode { address } => {
            let code = near::get_code(context, address).await?;
            println!("Code: 0x{:?}", code);
        }
        Command::GetBalance { address } => {
            let balance = near::get_balance(context, address).await?;
            println!("Balance: {}", balance);
        }
        Command::GetUpgradeIndex => {
            let index = near::get_upgrade_index(&context).await?;
            println!("Upgrade index: {}", index);
        }
        Command::GetVersion => {
            let version = near::get_version(&context).await?;
            println!("Version: {}", version);
        }
        Command::GetOwner => {
            let owner = near::get_owner(&context).await?;
            println!("Owner: {}", owner);
        }
        Command::SetOwner { account_id } => {
            near::set_owner(&context, account_id).await?;
            println!("Owner set successfully");
        }
        Command::GetBridgeProver => {
            let prover = near::get_bridge_prover(&context).await?;
            println!("Bridge prover: {}", prover);
        }
        Command::GetStorageAt { address, key } => {
            near::get_storage_at(context, address, key).await?;
        }
        Command::RegisterRelayer { address } => {
            near::register_relayer(context, address).await?;
        }
        Command::StartHashchain {
            block_height,
            block_hashchain,
        } => {
            near::start_hashchain(context, block_height, block_hashchain).await?;
            println!("Hashchain started successfully");
        }
        Command::PauseContract => {
            near::pause_contract(context).await?;
            println!("Contract paused successfully");
        }
        Command::ResumeContract => {
            near::resume_contract(context).await?;
            println!("Contract resumed successfully");
        }
        Command::PausePrecompiles { mask } => {
            near::pause_precompiles(context, mask).await?;
            println!("Precompiles paused successfully");
        }
        Command::ResumePrecompiles { mask } => {
            near::resume_precompiles(context, mask).await?;
            println!("Precompiles resumed successfully");
        }
        Command::PausedPrecompiles => {
            let mask = near::paused_precompiles(context).await?;
            println!("Paused precompiles: {mask}");
        }
        Command::FactoryUpdate { path } => {
            let code = std::fs::read(path)?;
            near::factory_update(context, code).await?;
            println!("Factory updated successfully");
        }
        Command::FactoryGetWnearAddress => {
            let address = near::factory_get_wnear_address(context).await?;
            println!("WNEAR address: {}", address);
        }
        Command::FactorySetWnearAddress { address } => {
            near::factory_set_wnear_address(context, address).await?;
            println!("WNEAR address set successfully");
        }
        Command::FundXccSubAccount {
            target,
            wnear_account_id,
            deposit,
        } => {
            near::fund_xcc_sub_account(context, target, wnear_account_id, deposit).await?;
            println!("XCC sub-account funded successfully");
        }
        Command::Upgrade { path } => {
            let code = std::fs::read(path)?;
            near::upgrade(context, code).await?;
            println!("Contract upgraded successfully");
        }
        Command::StageUpgrade { path } => {
            let code = std::fs::read(path)?;
            near::stage_upgrade(context, code).await?;
            println!("Contract staged successfully");
        }
        Command::DeployUpgrade => {
            near::deploy_upgrade(context).await?;
            println!("Upgrade deployed successfully");
        }
        Command::Deploy {
            code: _,
            args: _,
            abi_path: _,
            aurora_secret_key: _,
        } => todo!(),
        Command::Call {
            address: _,
            input: _,
            value: _,
            from: _,
        } => {
            todo!()
        }
        Command::ViewCall {
            address: _,
            function: _,
            args: _,
            from: _,
            abi_path: _,
        } => {
            todo!()
        }
        Command::Submit {
            address,
            function,
            args,
            abi_path,
            value,
            aurora_secret_key,
        } => {
            let result = near::submit(
                &context,
                address,
                function,
                args,
                abi_path,
                value.into(),
                aurora_secret_key,
            )
            .await?;
            println!("Submit result: {:?}", result);
        }
        Command::EncodeAddress { ref account } => {
            let addr = near::encode_to_address(account);
            println!("Encoded address: {:?}", addr);
        }
        Command::KeyPair { random, seed } => {
            let (addr, sk) = near::gen_key_pair(random, seed)?;
            println!("Key pair - Address: {:?}, SecretKey: {:?}", addr, sk);
        }
        Command::GenerateNearKey {
            account_id: _,
            key_type,
        } => {
            let key = near::gen_near_key_pair(key_type)?;
            println!("Generated NEAR key: {:?}", key);
        }
        Command::GetFixedGas => {
            let gas = near::get_fixed_gas(&context).await?;
            println!("Fixed gas: {:?}", gas);
        }
        Command::SetFixedGas { cost } => {
            // Assuming EthGas can be constructed from a u64 cost.
            near::set_fixed_gas(&context, Some(cost.into())).await?;
            println!("Fixed gas set successfully");
        }
        Command::GetSiloParams => {
            let params = near::get_silo_params(&context).await?;
            println!("Silo params: {:?}", params);
        }
        Command::SetSiloParams {
            gas,
            fallback_address,
        } => {
            let params = aurora_sdk_rs::aurora::parameters::silo::SiloParamsArgs {
                fixed_gas: gas,
                erc20_fallback_address: fallback_address,
            };
            near::set_silo_params(&context, Some(params)).await?;
            println!("Silo params set");
        }
        Command::DisableSiloMode => {
            near::set_silo_params(&context, None).await?;
            println!("Silo mode disabled");
        }
        Command::GetWhitelistStatus { kind } => {
            let status = near::get_whitelist_status(&context, kind).await?;
            println!("Whitelist status: {:?}", status);
        }
        Command::SetWhitelistStatus { kind, status } => {
            near::set_whitelist_status(&context, kind, status != 0).await?;
            println!("Whitelist status set");
        }
        Command::AddEntryToWhitelist { kind, entry } => {
            near::add_entry_to_whitelist(&context, kind, entry).await?;
            println!("Entry added to whitelist");
        }
        Command::RemoveEntryFromWhitelist { kind, entry } => {
            near::remove_entry_from_whitelist(&context, kind, entry).await?;
            println!("Entry removed from whitelist");
        }
        Command::SetKeyManager { account_id } => {
            near::set_key_manager(&context, account_id).await?;
            println!("Key manager set");
        }
        Command::AddRelayerKey {
            public_key,
            allowance,
        } => {
            let outcome = near::add_relayer_key(&context, public_key, allowance).await?;
            println!("Relayer key added: {:?}", outcome);
        }
        Command::RemoveRelayerKey { public_key } => {
            near::remove_relayer_key(&context, public_key).await?;
            println!("Relayer key removed");
        }
        Command::GetUpgradeDelayBlocks => {
            let blocks = near::get_upgrade_delay_blocks(&context).await?;
            println!("Upgrade delay blocks: {}", blocks);
        }
        Command::SetUpgradeDelayBlocks { blocks } => {
            near::set_upgrade_delay_blocks(&context, blocks).await?;
            println!("Upgrade delay blocks set");
        }
        Command::GetErc20FromNep141 { account_id } => {
            let erc20 = near::get_erc20_from_nep141(&context, account_id).await?;
            println!("ERC20 from NEP141: {}", erc20);
        }
        Command::GetNep141FromErc20 { address } => {
            let acc_id = near::get_nep141_from_erc20(&context, address).await?;
            println!("NEP141 from ERC20: {}", acc_id);
        }
        Command::GetErc20Metadata { erc20_id } => {
            let meta = near::get_erc20_metadata(&context, erc20_id).await?;
            println!("ERC20 metadata: {:?}", meta);
        }
        Command::SetErc20Metadata {
            erc20_id,
            name,
            symbol,
            decimals,
        } => {
            let metadata = aurora_sdk_rs::aurora::parameters::connector::Erc20Metadata {
                name,
                symbol,
                decimals,
            };
            near::set_erc20_metadata(&context, erc20_id, metadata).await?;
            println!("ERC20 metadata set");
        }
        Command::MirrorErc20Token {
            contract_id,
            nep141,
        } => {
            let addr = near::mirror_erc20_token(&context, contract_id, nep141).await?;
            println!("Mirrored ERC20 token address: {:?}", addr);
        }
        Command::SetEthConnectorContractAccount {
            account_id,
            withdraw_ser: _,
        } => {
            near::set_eth_connector_contract_account(&context, account_id).await?;
            println!("Eth connector contract account set");
        }
        Command::GetEthConnectorContractAccount => {
            let account = near::get_eth_connector_contract_account(&context).await?;
            println!("Eth connector contract account: {}", account);
        }
        Command::SetEthConnectorContractData {
            prover_id,
            custodian_address,
            ft_metadata_path,
        } => {
            near::set_eth_connector_contract_data(
                &context,
                prover_id,
                custodian_address,
                ft_metadata_path,
            )
            .await?;
            println!("Eth connector contract data set");
        }
        Command::SetPausedFlags { mask } => {
            near::set_paused_flags(&context, mask).await?;
            println!("Paused flags set");
        }
        Command::GetPausedFlags => {
            let flags = near::get_paused_flags(&context).await?;
            println!("Paused flags: {:?}", flags);
        }
        Command::AddRelayer {
            deposit,
            full_access_pub_key,
            function_call_pub_key,
        } => {
            let outcome = near::add_relayer(
                &context,
                deposit,
                full_access_pub_key,
                function_call_pub_key,
            )
            .await?;
            println!("Relayer added: {:?}", outcome);
        }
    }

    Ok(())
}

fn parse_address(s: &str) -> anyhow::Result<Address> {
    Address::decode(s).map_err(|e| anyhow::anyhow!("Invalid address: {s}, error: {e}"))
}

fn parse_wei(s: &str) -> anyhow::Result<Wei> {
    U256::from_str_radix(s, 10)
        .map_err(|s| anyhow::anyhow!("Invalid wei value: {s}"))
        .and_then(|u| Wei::from_eth(u).ok_or(anyhow::anyhow!("Wei overflow")))
}

fn parse_whitelist_kind(s: &str) -> anyhow::Result<WhitelistKind> {
    match s.to_lowercase().as_str() {
        "admin" => Ok(WhitelistKind::Admin),
        "evm-admin" => Ok(WhitelistKind::EvmAdmin),
        "account" => Ok(WhitelistKind::Account),
        "address" => Ok(WhitelistKind::Address),
        _ => Err(anyhow::anyhow!("Invalid whitelist kind: {s}")),
    }
}

fn parse_withdraw_serialize_type(s: &str) -> anyhow::Result<WithdrawSerializeType> {
    match s.to_lowercase().as_str() {
        "json" => Ok(WithdrawSerializeType::Json),
        "borsh" => Ok(WithdrawSerializeType::Borsh),
        _ => Err(anyhow::anyhow!("Invalid withdraw serialize type: {s}")),
    }
}

fn parse_eth_gas(s: &str) -> anyhow::Result<EthGas> {
    s.parse::<u64>()
        .map(EthGas::new)
        .map_err(|e| anyhow::anyhow!("Invalid EthGas value: {s}, error: {e}"))
}

fn parse_ft_metadata_path(s: &str) -> anyhow::Result<FungibleTokenMetadata> {
    common::parse_ft_metadata(std::fs::read_to_string(s).ok())
}
