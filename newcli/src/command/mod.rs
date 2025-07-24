use std::path::PathBuf;

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

use crate::{cli::Cli, common, context::Context};

mod handlers;
mod near;

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Create a new NEAR account
    CreateAccount {
        /// `AccountId`
        #[arg(long, short)]
        account: AccountId,
        /// Initial account balance in NEAR
        #[arg(long, short, value_parser = parse_near_token)]
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
    GetBlockHash {
        height: u64,
    },
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
    SetOwner {
        account_id: AccountId,
    },
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
        path: PathBuf,
    },
    /// Return the address of the `wNEAR` ERC-20 contract
    FactoryGetWnearAddress,
    /// Sets the address for the `wNEAR` ERC-20 contract
    FactorySetWnearAddress {
        #[arg(value_parser = parse_address)]
        address: Address,
    },
    FactoryUpdateAddressVersion {
        #[arg(value_parser = parse_address)]
        address: Address,
        version: u32,
    },
    /// Create and/or fund an XCC sub-account directly
    FundXccSubAccount {
        /// Address of the target
        #[arg(value_parser = parse_address)]
        target: Address,
        /// Wnear Account Id
        wnear_account_id: Option<AccountId>,
        /// Attached deposit in NEAR
        #[arg(value_parser = parse_near_token)]
        deposit: NearToken,
    },
    /// Upgrade contract with provided code
    Upgrade {
        path: PathBuf,
    },
    /// Stage a new code for upgrade
    StageUpgrade {
        path: PathBuf,
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
        abi_path: Option<PathBuf>,
        /// Aurora EVM secret key
        #[arg(long)]
        aurora_secret_key: String,
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
        #[arg(long, short, value_parser = parse_address)]
        address: Address,
        /// Name of the function to call
        #[arg(long, short)]
        function: String,
        /// Arguments with values in JSON
        #[arg(long)]
        args: Option<String>,
        /// Sender address
        #[arg(long, value_parser = parse_address)]
        from: Address,
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
    EncodeAddress {
        account: AccountId,
    },
    /// Return Public and Secret ED25519 keys
    KeyPair {
        /// Random
        #[arg(long, default_value = "false", conflicts_with = "seed")]
        random: bool,
        /// From seed
        #[arg(long, conflicts_with = "random")]
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
        #[arg(long, value_parser = parse_near_token)]
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
        /// Account of the contract where ERC-20 has been deployed
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
        #[arg(long, value_parser = parse_near_token)]
        deposit: NearToken,
        #[arg(long)]
        full_access_pub_key: PublicKey,
        #[arg(long)]
        function_call_pub_key: PublicKey,
    },
    WithdrawWnearToRouter {
        #[arg(long, value_parser = parse_address)]
        address: Address,
        #[arg(value_parser = parse_near_token)]
        amount: NearToken,
    },
    MirrorErc20TokenCallback {
        #[arg(long)]
        contract_id: AccountId,
        #[arg(long)]
        nep141: AccountId,
    },
    GetLatestHashchain,
    FtTotalSupply,
    FtBalanceOf {
        #[arg(long)]
        account_id: AccountId,
    },
    FtBalanceOfEth {
        #[arg(long, value_parser = parse_address)]
        address: Address,
    },
    FtTransfer {
        #[arg(long)]
        receiver_id: AccountId,
        #[arg(long, value_parser = parse_near_token)]
        amount: NearToken,
        #[arg(long)]
        memo: Option<String>,
    },
    FtTransferCall {
        #[arg(long)]
        receiver_id: AccountId,
        #[arg(long, value_parser = parse_near_token)]
        amount: NearToken,
        #[arg(long)]
        memo: Option<String>,
        #[arg(long)]
        msg: String,
    },
    FtOnTransfer {
        #[arg(long)]
        sender_id: AccountId,
        #[arg(long, value_parser = parse_near_token)]
        amount: NearToken,
        #[arg(long)]
        msg: String,
    },
    DeployErc20Token {
        #[arg(long)]
        nep141: AccountId,
    },
    StorageDeposit {
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    },
    StorageUnregister {
        #[arg(long, default_value = "false")]
        force: bool,
    },
    StorageWithdraw {
        #[arg(long, value_parser = parse_near_token)]
        amount: Option<NearToken>,
    },
    StorageBalanceOf {
        #[arg(long)]
        account_id: AccountId,
    },
}

pub async fn run(cli: Cli) -> anyhow::Result<()> {
    println!("Running command: {:?}", cli.command);

    let context = {
        let near_client =
            aurora_sdk_rs::near::client::Client::new(cli.network.rpc_url(), None, cli.signer()?)?;
        let aurora_client = aurora::client::Client::new(near_client);

        &Context {
            cli,
            client: aurora_client,
        }
    };

    match context.cli.command.clone() {
        Command::CreateAccount { account, balance } => {
            handlers::create_account(context, account, balance).await
        }
        Command::ViewAccount { ref account } => handlers::view_account(context, account).await,
        Command::DeployAurora { ref path } => handlers::deploy_aurora(context, path).await,
        Command::Init {
            chain_id,
            owner_id,
            bridge_prover_id,
            upgrade_delay_blocks,
            custodian_address,
            ft_metadata_path,
        } => {
            handlers::init(
                context,
                chain_id,
                owner_id,
                bridge_prover_id,
                upgrade_delay_blocks,
                custodian_address,
                ft_metadata_path,
            )
            .await
        }
        Command::GetChainId => {
            handlers::get_chain_id(context).await;
        }
        Command::GetNonce { address } => {
            handlers::get_nonce(context, address).await;
        }
        Command::GetBlockHash { height } => {
            handlers::get_block_hash(context, height).await;
        }
        Command::GetCode { address } => {
            handlers::get_code(context, address).await;
        }
        Command::GetBalance { address } => {
            handlers::get_balance(context, address).await;
        }
        Command::GetUpgradeIndex => {
            handlers::get_upgrade_index(context).await;
        }
        Command::GetVersion => {
            handlers::get_version(context).await;
        }
        Command::GetOwner => {
            handlers::get_owner(context).await;
        }
        Command::SetOwner { account_id } => handlers::set_owner(context, account_id).await,
        Command::GetBridgeProver => {
            handlers::get_bridge_prover(context).await;
        }
        Command::GetStorageAt { address, key } => {
            handlers::get_storage_at(context, address, key).await
        }
        Command::RegisterRelayer { address } => handlers::register_relayer(context, address).await,
        Command::StartHashchain {
            block_height,
            block_hashchain,
        } => handlers::start_hashchain(context, block_height, block_hashchain).await,
        Command::PauseContract => {
            handlers::pause_contract(context).await;
        }
        Command::ResumeContract => {
            handlers::resume_contract(context).await;
        }
        Command::PausePrecompiles { mask } => {
            handlers::pause_precompiles(context, mask).await;
        }
        Command::ResumePrecompiles { mask } => {
            handlers::resume_precompiles(context, mask).await;
        }
        Command::PausedPrecompiles => {
            handlers::paused_precompiles(context).await;
        }
        Command::FactoryUpdate { path } => handlers::factory_update(context, path).await,
        Command::FactoryGetWnearAddress => handlers::factory_get_wnear_address(context).await,
        Command::FactoryUpdateAddressVersion { address, version } => {
            handlers::factory_update_address_version(context, address, version).await
        }
        Command::FactorySetWnearAddress { address } => {
            handlers::factory_set_wnear_address(context, address).await
        }
        Command::FundXccSubAccount {
            target,
            wnear_account_id,
            deposit,
        } => handlers::fund_xcc_sub_account(context, target, wnear_account_id, deposit).await,
        Command::Upgrade { path } => handlers::upgrade(context, path).await,
        Command::StageUpgrade { path } => handlers::stage_upgrade(context, path).await,
        Command::DeployUpgrade => handlers::deploy_upgrade(context).await,
        Command::Deploy {
            code,
            args,
            abi_path,
            aurora_secret_key,
        } => handlers::deploy(context, code, args, abi_path, aurora_secret_key).await?,
        Command::Call {
            address,
            input,
            value,
            from,
        } => handlers::call(context, address, input, value, from).await?,
        Command::ViewCall {
            address,
            function,
            args,
            from,
            abi_path,
        } => handlers::view_call(context, address, function, args, from, abi_path).await?,
        Command::Submit {
            address,
            function,
            args,
            abi_path,
            value,
            aurora_secret_key,
        } => {
            handlers::submit(
                context,
                address,
                function,
                args,
                abi_path,
                value,
                aurora_secret_key,
            )
            .await?
        }
        Command::EncodeAddress { ref account } => {
            handlers::encode_address(context, account).await?
        }
        Command::KeyPair { random, seed } => handlers::key_pair(context, random, seed).await?,
        Command::GenerateNearKey {
            account_id: _,
            key_type,
        } => handlers::generate_near_key(context, key_type).await,
        Command::GetFixedGas => handlers::get_fixed_gas(context).await,

        Command::SetFixedGas { cost } => handlers::set_fixed_gas(context, cost).await,
        Command::GetSiloParams => handlers::get_silo_params(context).await,

        Command::SetSiloParams {
            gas,
            fallback_address,
        } => handlers::set_silo_params(context, gas, fallback_address).await,
        Command::DisableSiloMode => handlers::disable_silo_mode(context).await,
        Command::GetWhitelistStatus { kind } => handlers::get_whitelist_status(context, kind).await,
        Command::SetWhitelistStatus { kind, status } => {
            handlers::set_whitelist_status(context, kind, status).await
        }
        Command::AddEntryToWhitelist { kind, entry } => {
            handlers::add_entry_to_whitelist(context, kind, entry).await
        }
        Command::RemoveEntryFromWhitelist { kind, entry } => {
            handlers::remove_entry_from_whitelist(context, kind, entry).await
        }
        Command::SetKeyManager { account_id } => {
            handlers::set_key_manager(context, account_id).await
        }
        Command::AddRelayerKey {
            public_key,
            allowance,
        } => handlers::add_relayer_key(context, public_key, allowance).await,
        Command::RemoveRelayerKey { public_key } => {
            handlers::remove_relayer_key(context, public_key).await
        }
        Command::GetUpgradeDelayBlocks => handlers::get_upgrade_delay_blocks(context).await,
        Command::SetUpgradeDelayBlocks { blocks } => {
            handlers::set_upgrade_delay_blocks(context, blocks).await
        }
        Command::GetErc20FromNep141 { account_id } => {
            handlers::get_erc20_from_nep141(context, account_id).await
        }
        Command::GetNep141FromErc20 { address } => {
            handlers::get_nep141_from_erc20(context, address).await
        }
        Command::GetErc20Metadata { erc20_id } => {
            handlers::get_erc20_metadata(context, erc20_id).await
        }
        Command::SetErc20Metadata {
            erc20_id,
            name,
            symbol,
            decimals,
        } => handlers::set_erc20_metadata(context, erc20_id, name, symbol, decimals).await,
        Command::MirrorErc20Token {
            contract_id,
            nep141,
        } => handlers::mirror_erc20_token(context, contract_id, nep141).await,
        Command::SetEthConnectorContractAccount {
            account_id,
            withdraw_ser: _,
        } => handlers::set_eth_connector_contract_account(context, account_id).await,
        Command::GetEthConnectorContractAccount => {
            handlers::get_eth_connector_contract_account(context).await
        }
        Command::SetEthConnectorContractData {
            prover_id,
            custodian_address,
            ft_metadata_path,
        } => {
            handlers::set_eth_connector_contract_data(
                context,
                prover_id,
                custodian_address,
                ft_metadata_path,
            )
            .await
        }
        Command::SetPausedFlags { mask } => handlers::set_paused_flags(context, mask).await,
        Command::GetPausedFlags => handlers::get_paused_flags(context).await,
        Command::AddRelayer {
            deposit,
            full_access_pub_key,
            function_call_pub_key,
        } => {
            handlers::add_relayer(context, deposit, full_access_pub_key, function_call_pub_key)
                .await
        }
        Command::WithdrawWnearToRouter { address, amount } => {
            handlers::withdraw_wnear_to_router(context, address, amount).await
        }
        Command::MirrorErc20TokenCallback {
            contract_id,
            nep141,
        } => handlers::mirror_erc20_token_callback(context, contract_id, nep141).await,
        Command::GetLatestHashchain => handlers::get_latest_hashchain(context).await,
        Command::FtTotalSupply => handlers::ft_total_supply(context).await,
        Command::FtBalanceOf { account_id } => handlers::ft_balance_of(context, account_id).await,
        Command::FtBalanceOfEth { address } => handlers::ft_balance_of_eth(context, address).await,
        Command::FtTransfer {
            receiver_id,
            amount,
            memo,
        } => handlers::ft_transfer(context, receiver_id, amount, memo).await,
        Command::FtTransferCall {
            receiver_id,
            amount,
            memo,
            msg,
        } => handlers::ft_transfer_call(context, receiver_id, amount, memo, msg).await,
        Command::FtOnTransfer {
            sender_id,
            amount,
            msg,
        } => handlers::ft_on_transfer(context, sender_id, amount, msg).await,
        Command::DeployErc20Token { nep141 } => handlers::deploy_erc20_token(context, nep141).await,
        Command::StorageDeposit {
            account_id,
            registration_only,
        } => handlers::storage_deposit(context, account_id, registration_only).await,
        Command::StorageUnregister { force } => handlers::storage_unregister(context, force).await,
        Command::StorageWithdraw { amount } => handlers::storage_withdraw(context, amount).await,
        Command::StorageBalanceOf { account_id } => {
            handlers::storage_balance_of(context, account_id).await
        }
    };

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

fn parse_near_token(s: &str) -> anyhow::Result<NearToken> {
    s.parse::<u128>()
        .map(NearToken::from_yoctonear)
        .map_err(|e| anyhow::anyhow!("Invalid NearToken value: {s}, error: {e}"))
}

fn parse_eth_gas(s: &str) -> anyhow::Result<EthGas> {
    s.parse::<u64>()
        .map(EthGas::new)
        .map_err(|e| anyhow::anyhow!("Invalid EthGas value: {s}, error: {e}"))
}

fn parse_ft_metadata_path(s: &str) -> anyhow::Result<FungibleTokenMetadata> {
    common::parse_ft_metadata(std::fs::read_to_string(s).ok())
}
