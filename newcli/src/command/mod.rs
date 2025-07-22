use std::path::PathBuf;

use aurora_sdk_rs::{
    aurora::{
        self, H256, U256, abi, ethabi,
        parameters::{
            connector::{FungibleTokenMetadata, WithdrawSerializeType},
            engine::{SubmitResult, TransactionStatus},
            silo::WhitelistKind,
        },
        types::{Address, Balance, EthGas, NEP141Wei, Wei, Yocto},
    },
    near::{
        crypto::{KeyType, PublicKey},
        primitives::{borsh::BorshDeserialize, types::AccountId, views::FinalExecutionStatus},
        token::NearToken,
    },
};
use clap::Subcommand;

use crate::{cli::Cli, common, output, result_object};
use crate::{common::output::CommandResult, context::Context};

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
        path: String,
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
        path: String,
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
            handle_create_account(context, account, balance).await
        }
        Command::ViewAccount { ref account } => handle_view_account(context, account).await,
        Command::DeployAurora { ref path } => handle_deploy_aurora(context, path).await,
        Command::Init {
            chain_id,
            owner_id,
            bridge_prover_id,
            upgrade_delay_blocks,
            custodian_address,
            ft_metadata_path,
        } => {
            handle_init(
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
        Command::GetChainId => handle_get_chain_id(context).await,
        Command::GetNonce { address } => handle_get_nonce(context, address).await,
        Command::GetBlockHash { height } => handle_get_block_hash(context, height).await,
        Command::GetCode { address } => handle_get_code(context, address).await,
        Command::GetBalance { address } => handle_get_balance(context, address).await,
        Command::GetUpgradeIndex => handle_get_upgrade_index(context).await,
        Command::GetVersion => handle_get_version(context).await,
        Command::GetOwner => handle_get_owner(context).await,
        Command::SetOwner { account_id } => handle_set_owner(context, account_id).await,
        Command::GetBridgeProver => handle_get_bridge_prover(context).await,
        Command::GetStorageAt { address, key } => {
            handle_get_storage_at(context, address, key).await
        }
        Command::RegisterRelayer { address } => handle_register_relayer(context, address).await,
        Command::StartHashchain {
            block_height,
            block_hashchain,
        } => handle_start_hashchain(context, block_height, block_hashchain).await,
        Command::PauseContract => handle_pause_contract(context).await,
        Command::ResumeContract => handle_resume_contract(context).await,
        Command::PausePrecompiles { mask } => handle_pause_precompiles(context, mask).await,
        Command::ResumePrecompiles { mask } => handle_resume_precompiles(context, mask).await,
        Command::PausedPrecompiles => handle_paused_precompiles(context).await,
        Command::FactoryUpdate { path } => handle_factory_update(context, path).await,
        Command::FactoryGetWnearAddress => handle_factory_get_wnear_address(context).await,
        Command::FactoryUpdateAddressVersion { address, version } => {
            handle_factory_update_address_version(context, address, version).await
        }
        Command::FactorySetWnearAddress { address } => {
            handle_factory_set_wnear_address(context, address).await
        }
        Command::FundXccSubAccount {
            target,
            wnear_account_id,
            deposit,
        } => handle_fund_xcc_sub_account(context, target, wnear_account_id, deposit).await,
        Command::Upgrade { path } => handle_upgrade(context, path).await,
        Command::StageUpgrade { path } => handle_stage_upgrade(context, path).await,
        Command::DeployUpgrade => handle_deploy_upgrade(context).await,
        Command::Deploy {
            code,
            args,
            abi_path,
            aurora_secret_key,
        } => handle_deploy(context, code, args, abi_path, aurora_secret_key).await,
        Command::Call {
            address,
            input,
            value,
            from,
        } => handle_call(context, address, input, value, from).await,
        Command::ViewCall {
            address,
            function,
            args,
            from,
            abi_path,
        } => handle_view_call(context, address, function, args, from, abi_path).await,
        Command::Submit {
            address,
            function,
            args,
            abi_path,
            value,
            aurora_secret_key,
        } => {
            handle_submit(
                context,
                address,
                function,
                args,
                abi_path,
                value,
                aurora_secret_key,
            )
            .await
        }
        Command::EncodeAddress { ref account } => handle_encode_address(context, account).await,
        Command::KeyPair { random, seed } => handle_key_pair(context, random, seed).await,
        Command::GenerateNearKey {
            account_id: _,
            key_type,
        } => handle_generate_near_key(context, key_type).await,
        Command::GetFixedGas => handle_get_fixed_gas(context).await,
        Command::SetFixedGas { cost } => handle_set_fixed_gas(context, cost).await,
        Command::GetSiloParams => handle_get_silo_params(context).await,
        Command::SetSiloParams {
            gas,
            fallback_address,
        } => handle_set_silo_params(context, gas, fallback_address).await,
        Command::DisableSiloMode => handle_disable_silo_mode(context).await,
        Command::GetWhitelistStatus { kind } => handle_get_whitelist_status(context, kind).await,
        Command::SetWhitelistStatus { kind, status } => {
            handle_set_whitelist_status(context, kind, status).await
        }
        Command::AddEntryToWhitelist { kind, entry } => {
            handle_add_entry_to_whitelist(context, kind, entry).await
        }
        Command::RemoveEntryFromWhitelist { kind, entry } => {
            handle_remove_entry_from_whitelist(context, kind, entry).await
        }
        Command::SetKeyManager { account_id } => handle_set_key_manager(context, account_id).await,
        Command::AddRelayerKey {
            public_key,
            allowance,
        } => handle_add_relayer_key(context, public_key, allowance).await,
        Command::RemoveRelayerKey { public_key } => {
            handle_remove_relayer_key(context, public_key).await
        }
        Command::GetUpgradeDelayBlocks => handle_get_upgrade_delay_blocks(context).await,
        Command::SetUpgradeDelayBlocks { blocks } => {
            handle_set_upgrade_delay_blocks(context, blocks).await
        }
        Command::GetErc20FromNep141 { account_id } => {
            handle_get_erc20_from_nep141(context, account_id).await
        }
        Command::GetNep141FromErc20 { address } => {
            handle_get_nep141_from_erc20(context, address).await
        }
        Command::GetErc20Metadata { erc20_id } => {
            handle_get_erc20_metadata(context, erc20_id).await
        }
        Command::SetErc20Metadata {
            erc20_id,
            name,
            symbol,
            decimals,
        } => handle_set_erc20_metadata(context, erc20_id, name, symbol, decimals).await,
        Command::MirrorErc20Token {
            contract_id,
            nep141,
        } => handle_mirror_erc20_token(context, contract_id, nep141).await,
        Command::SetEthConnectorContractAccount {
            account_id,
            withdraw_ser: _,
        } => handle_set_eth_connector_contract_account(context, account_id).await,
        Command::GetEthConnectorContractAccount => {
            handle_get_eth_connector_contract_account(context).await
        }
        Command::SetEthConnectorContractData {
            prover_id,
            custodian_address,
            ft_metadata_path,
        } => {
            handle_set_eth_connector_contract_data(
                context,
                prover_id,
                custodian_address,
                ft_metadata_path,
            )
            .await
        }
        Command::SetPausedFlags { mask } => handle_set_paused_flags(context, mask).await,
        Command::GetPausedFlags => handle_get_paused_flags(context).await,
        Command::AddRelayer {
            deposit,
            full_access_pub_key,
            function_call_pub_key,
        } => handle_add_relayer(context, deposit, full_access_pub_key, function_call_pub_key).await,
        Command::WithdrawWnearToRouter { address, amount } => {
            handle_withdraw_wnear_to_router(context, address, amount).await
        }
        Command::MirrorErc20TokenCallback {
            contract_id,
            nep141,
        } => handle_mirror_erc20_token_callback(context, contract_id, nep141).await,
        Command::GetLatestHashchain => handle_get_latest_hashchain(context).await,
        Command::FtTotalSupply => handle_ft_total_supply(context).await,
        Command::FtBalanceOf { account_id } => handle_ft_balance_of(context, account_id).await,
        Command::FtBalanceOfEth { address } => handle_ft_balance_of_eth(context, address).await,
        Command::FtTransfer {
            receiver_id,
            amount,
            memo,
        } => handle_ft_transfer(context, receiver_id, amount, memo).await,
        Command::FtTransferCall {
            receiver_id,
            amount,
            memo,
            msg,
        } => handle_ft_transfer_call(context, receiver_id, amount, memo, msg).await,
        Command::FtOnTransfer {
            sender_id,
            amount,
            msg,
        } => handle_ft_on_transfer(context, sender_id, amount, msg).await,
        Command::DeployErc20Token { nep141 } => handle_deploy_erc20_token(context, nep141).await,
        Command::StorageDeposit {
            account_id,
            registration_only,
        } => handle_storage_deposit(context, account_id, registration_only).await,
        Command::StorageUnregister { force } => handle_storage_unregister(context, force).await,
        Command::StorageWithdraw { amount } => handle_storage_withdraw(context, amount).await,
        Command::StorageBalanceOf { account_id } => {
            handle_storage_balance_of(context, account_id).await
        }
    }
}

// Command handler functions

async fn handle_create_account(
    context: &Context,
    account: AccountId,
    balance: NearToken,
) -> anyhow::Result<()> {
    let account_str = account.to_string();
    let outcome = near::create_account(context, account, balance).await?;
    output!(
        &context.cli.output_format,
        result_object!("status" => "created", "account" => account_str, "outcome" => format!("{:?}", outcome))
    );
    Ok(())
}

async fn handle_view_account(context: &Context, account: &AccountId) -> anyhow::Result<()> {
    let view = near::view_account(context, account).await?;
    output!(
        &context.cli.output_format,
        result_object!("account" => account.to_string(), "view" => format!("{:?}", view))
    );
    Ok(())
}

async fn handle_deploy_aurora(context: &Context, path: &PathBuf) -> anyhow::Result<()> {
    let wasm = std::fs::read(path)?;
    let outcome = near::deploy_aurora(context, wasm).await?;
    output!(
        &context.cli.output_format,
        result_object!("status" => "deployed", "path" => path.display().to_string(), "outcome" => format!("{:?}", outcome))
    );
    Ok(())
}

async fn handle_init(
    context: &Context,
    chain_id: u64,
    owner_id: AccountId,
    bridge_prover_id: AccountId,
    upgrade_delay_blocks: Option<u64>,
    custodian_address: Option<Address>,
    ft_metadata_path: FungibleTokenMetadata,
) -> anyhow::Result<()> {
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

    output!(
        &context.cli.output_format,
        CommandResult::success("Aurora EVM and ETH connector initialized successfully")
    );
    Ok(())
}

async fn handle_get_chain_id(context: &Context) -> anyhow::Result<()> {
    let id = near::get_chain_id(context).await?;
    output!(&context.cli.output_format, result_object!("chain_id" => id));
    Ok(())
}

async fn handle_get_nonce(context: &Context, address: Address) -> anyhow::Result<()> {
    let nonce = near::get_nonce(context, address).await?;
    output!(
        &context.cli.output_format,
        result_object!("address" => format!("{:?}", address), "nonce" => nonce)
    );
    Ok(())
}

async fn handle_get_block_hash(context: &Context, height: u64) -> anyhow::Result<()> {
    let hash = near::get_block_hash(context, height).await?;
    output!(
        &context.cli.output_format,
        result_object!("height" => height, "hash" => hash.to_string())
    );
    Ok(())
}

async fn handle_get_code(context: &Context, address: Address) -> anyhow::Result<()> {
    let code = near::get_code(context, address).await?;
    output!(
        &context.cli.output_format,
        result_object!("address" => format!("{:?}", address), "code" => format!("0x{:?}", hex::encode(code)))
    );
    Ok(())
}

async fn handle_get_balance(context: &Context, address: Address) -> anyhow::Result<()> {
    let balance = near::get_balance(context, address).await?;
    output!(
        &context.cli.output_format,
        result_object!("address" => format!("{:?}", address), "balance" => balance.to_string())
    );
    Ok(())
}

async fn handle_get_upgrade_index(context: &Context) -> anyhow::Result<()> {
    let index = near::get_upgrade_index(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("upgrade_index" => index)
    );
    Ok(())
}

async fn handle_get_version(context: &Context) -> anyhow::Result<()> {
    let version = near::get_version(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("version" => version)
    );
    Ok(())
}

async fn handle_get_owner(context: &Context) -> anyhow::Result<()> {
    let owner = near::get_owner(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("owner" => owner.to_string())
    );
    Ok(())
}

async fn handle_set_owner(context: &Context, account_id: AccountId) -> anyhow::Result<()> {
    near::set_owner(context, account_id).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Owner set successfully")
    );
    Ok(())
}

async fn handle_get_bridge_prover(context: &Context) -> anyhow::Result<()> {
    let prover = near::get_bridge_prover(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("bridge_prover" => prover.to_string())
    );
    Ok(())
}

async fn handle_get_storage_at(
    context: &Context,
    address: Address,
    key: H256,
) -> anyhow::Result<()> {
    near::get_storage_at(context, address, key).await?;
    Ok(())
}

async fn handle_register_relayer(context: &Context, address: Address) -> anyhow::Result<()> {
    near::register_relayer(context, address).await?;
    Ok(())
}

async fn handle_start_hashchain(
    context: &Context,
    block_height: u64,
    block_hashchain: String,
) -> anyhow::Result<()> {
    near::start_hashchain(context, block_height, block_hashchain).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Hashchain started successfully")
    );
    Ok(())
}

async fn handle_pause_contract(context: &Context) -> anyhow::Result<()> {
    near::pause_contract(context).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Contract paused successfully")
    );
    Ok(())
}

async fn handle_resume_contract(context: &Context) -> anyhow::Result<()> {
    near::resume_contract(context).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Contract resumed successfully")
    );
    Ok(())
}

async fn handle_pause_precompiles(context: &Context, mask: u32) -> anyhow::Result<()> {
    near::pause_precompiles(context, mask).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Precompiles paused successfully")
    );
    Ok(())
}

async fn handle_resume_precompiles(context: &Context, mask: u32) -> anyhow::Result<()> {
    near::resume_precompiles(context, mask).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Precompiles resumed successfully")
    );
    Ok(())
}

async fn handle_paused_precompiles(context: &Context) -> anyhow::Result<()> {
    let mask = near::paused_precompiles(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("paused_precompiles_mask" => mask)
    );
    Ok(())
}

async fn handle_factory_update(context: &Context, path: String) -> anyhow::Result<()> {
    let code = std::fs::read(path)?;
    near::factory_update(context, code).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Factory updated successfully")
    );
    Ok(())
}

async fn handle_factory_get_wnear_address(context: &Context) -> anyhow::Result<()> {
    let address = near::factory_get_wnear_address(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("wnear_address" => format!("{:?}", address))
    );
    Ok(())
}

async fn handle_factory_update_address_version(
    context: &Context,
    address: Address,
    version: u32,
) -> anyhow::Result<()> {
    near::factory_update_address_version(context, address, version).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Factory address version updated successfully")
    );
    Ok(())
}

async fn handle_factory_set_wnear_address(
    context: &Context,
    address: Address,
) -> anyhow::Result<()> {
    near::factory_set_wnear_address(context, address).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("WNEAR address set successfully")
    );
    Ok(())
}

async fn handle_fund_xcc_sub_account(
    context: &Context,
    target: Address,
    wnear_account_id: Option<AccountId>,
    deposit: NearToken,
) -> anyhow::Result<()> {
    near::fund_xcc_sub_account(context, target, wnear_account_id, deposit).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("XCC sub-account funded successfully")
    );
    Ok(())
}

async fn handle_upgrade(context: &Context, path: String) -> anyhow::Result<()> {
    let code = std::fs::read(path)?;
    near::upgrade(context, code).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Contract upgraded successfully")
    );
    Ok(())
}

async fn handle_stage_upgrade(context: &Context, path: String) -> anyhow::Result<()> {
    let code = std::fs::read(path)?;
    near::stage_upgrade(context, code).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Contract staged successfully")
    );
    Ok(())
}

async fn handle_deploy_upgrade(context: &Context) -> anyhow::Result<()> {
    near::deploy_upgrade(context).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Upgrade deployed successfully")
    );
    Ok(())
}

async fn handle_deploy(
    context: &Context,
    code: String,
    args: Option<String>,
    abi_path: Option<PathBuf>,
    aurora_secret_key: String,
) -> anyhow::Result<()> {
    let input =
        if let Some((abi_path, args)) = abi_path.and_then(|path| args.map(|args| (path, args))) {
            let contract = abi::read_contract(abi_path)?;
            let constructor = contract
                .constructor()
                .ok_or_else(|| anyhow::anyhow!("Constructor not found in ABI"))?;
            let args: serde_json::Value = serde_json::from_str(&args)?;
            let tokens = abi::parse_args(&constructor.inputs, &args)?;
            let code = hex::decode(code)?;
            constructor.encode_input(code, &tokens)?
        } else {
            hex::decode(code)?
        };

    let result = near::deploy(context, input, aurora_secret_key).await?;
    output!(
        &context.cli.output_format,
        result_object!("deploy_result" => format!("{:?}", result.status))
    );
    Ok(())
}

async fn handle_call(
    context: &Context,
    address: Address,
    input: Option<String>,
    value: Option<u128>,
    from: Option<AccountId>,
) -> anyhow::Result<()> {
    let outcome = near::call(context, address, input, value, from).await?;
    match outcome.status {
        FinalExecutionStatus::NotStarted | FinalExecutionStatus::Started => {}
        FinalExecutionStatus::Failure(failure) => {
            output!(
                &context.cli.output_format,
                result_object!("status" => "failure", "error" => failure.to_string())
            );
        }
        FinalExecutionStatus::SuccessValue(result) => {
            let submit_result = SubmitResult::try_from_slice(&result)?;
            match submit_result.status {
                TransactionStatus::Succeed(_) => {
                    output!(
                        &context.cli.output_format,
                        result_object!("status" => "success")
                    );
                }
                TransactionStatus::Revert(_) => {
                    output!(
                        &context.cli.output_format,
                        result_object!("status" => "reverted")
                    );
                }
                _ => {
                    output!(
                        &context.cli.output_format,
                        result_object!("status" => "failed")
                    );
                }
            }
        }
    }
    Ok(())
}

async fn handle_view_call(
    context: &Context,
    address: Address,
    function: String,
    args: Option<String>,
    from: Address,
    abi_path: String,
) -> anyhow::Result<()> {
    let contract = abi::read_contract(abi_path)?;
    let tokens = near::view_call(context, address, function, args, from, contract).await?;

    let result = tokens
        .iter()
        .map(ethabi::Token::to_string)
        .collect::<Vec<_>>();

    output!(
        &context.cli.output_format,
        result_object!("address" => format!("{:?}", address), "result" => result)
    );
    Ok(())
}

async fn handle_submit(
    context: &Context,
    address: Address,
    function: String,
    args: Option<String>,
    abi_path: String,
    value: Wei,
    aurora_secret_key: String,
) -> anyhow::Result<()> {
    let result = near::submit(
        context,
        address,
        function,
        args,
        abi_path,
        value,
        aurora_secret_key,
    )
    .await?;
    output!(
        &context.cli.output_format,
        result_object!("submit_result" => format!("{:?}", result))
    );
    Ok(())
}

async fn handle_encode_address(context: &Context, account: &AccountId) -> anyhow::Result<()> {
    let addr = near::encode_to_address(account);
    output!(
        &context.cli.output_format,
        result_object!("account" => account.to_string(), "encoded_address" => format!("{:?}", addr))
    );
    Ok(())
}

async fn handle_key_pair(context: &Context, random: bool, seed: Option<u64>) -> anyhow::Result<()> {
    let (addr, sk) = near::gen_key_pair(random, seed)?;
    output!(
        &context.cli.output_format,
        result_object!("address" => format!("{:?}", addr), "secret_key" => format!("{:?}", sk))
    );
    Ok(())
}

async fn handle_generate_near_key(context: &Context, key_type: KeyType) -> anyhow::Result<()> {
    let key = near::gen_near_key_pair(key_type)?;
    output!(
        &context.cli.output_format,
        result_object!("generated_near_key" => format!("{:?}", key))
    );
    Ok(())
}

async fn handle_get_fixed_gas(context: &Context) -> anyhow::Result<()> {
    let gas = near::get_fixed_gas(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("fixed_gas" => format!("{:?}", gas))
    );
    Ok(())
}

async fn handle_set_fixed_gas(context: &Context, cost: EthGas) -> anyhow::Result<()> {
    near::set_fixed_gas(context, Some(cost)).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Fixed gas set successfully")
    );
    Ok(())
}

async fn handle_get_silo_params(context: &Context) -> anyhow::Result<()> {
    let params = near::get_silo_params(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("silo_params" => format!("{:?}", params))
    );
    Ok(())
}

async fn handle_set_silo_params(
    context: &Context,
    gas: EthGas,
    fallback_address: Address,
) -> anyhow::Result<()> {
    let params = aurora_sdk_rs::aurora::parameters::silo::SiloParamsArgs {
        fixed_gas: gas,
        erc20_fallback_address: fallback_address,
    };
    near::set_silo_params(context, Some(params)).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Silo params set")
    );
    Ok(())
}

async fn handle_disable_silo_mode(context: &Context) -> anyhow::Result<()> {
    near::set_silo_params(context, None).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Silo mode disabled")
    );
    Ok(())
}

async fn handle_get_whitelist_status(context: &Context, kind: WhitelistKind) -> anyhow::Result<()> {
    let status = near::get_whitelist_status(context, kind).await?;
    output!(
        &context.cli.output_format,
        result_object!("whitelist_kind" => format!("{:?}", kind), "status" => format!("{:?}", status))
    );
    Ok(())
}

async fn handle_set_whitelist_status(
    context: &Context,
    kind: WhitelistKind,
    status: u8,
) -> anyhow::Result<()> {
    near::set_whitelist_status(context, kind, status != 0).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Whitelist status set")
    );
    Ok(())
}

async fn handle_add_entry_to_whitelist(
    context: &Context,
    kind: WhitelistKind,
    entry: String,
) -> anyhow::Result<()> {
    near::add_entry_to_whitelist(context, kind, entry).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Entry added to whitelist")
    );
    Ok(())
}

async fn handle_remove_entry_from_whitelist(
    context: &Context,
    kind: WhitelistKind,
    entry: String,
) -> anyhow::Result<()> {
    near::remove_entry_from_whitelist(context, kind, entry).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Entry removed from whitelist")
    );
    Ok(())
}

async fn handle_set_key_manager(
    context: &Context,
    account_id: Option<AccountId>,
) -> anyhow::Result<()> {
    near::set_key_manager(context, account_id).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Key manager set")
    );
    Ok(())
}

async fn handle_add_relayer_key(
    context: &Context,
    public_key: PublicKey,
    allowance: NearToken,
) -> anyhow::Result<()> {
    let outcome = near::add_relayer_key(context, public_key, allowance).await?;
    output!(
        &context.cli.output_format,
        result_object!("relayer_key_added" => format!("{:?}", outcome))
    );
    Ok(())
}

async fn handle_remove_relayer_key(context: &Context, public_key: PublicKey) -> anyhow::Result<()> {
    near::remove_relayer_key(context, public_key).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Relayer key removed")
    );
    Ok(())
}

async fn handle_get_upgrade_delay_blocks(context: &Context) -> anyhow::Result<()> {
    let blocks = near::get_upgrade_delay_blocks(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("upgrade_delay_blocks" => blocks)
    );
    Ok(())
}

async fn handle_set_upgrade_delay_blocks(context: &Context, blocks: u64) -> anyhow::Result<()> {
    near::set_upgrade_delay_blocks(context, blocks).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Upgrade delay blocks set")
    );
    Ok(())
}

async fn handle_get_erc20_from_nep141(
    context: &Context,
    account_id: AccountId,
) -> anyhow::Result<()> {
    let account_str = account_id.to_string();
    let erc20 = near::get_erc20_from_nep141(context, account_id).await?;
    output!(
        &context.cli.output_format,
        result_object!("nep141_account" => account_str, "erc20_address" => format!("{:?}", erc20))
    );
    Ok(())
}

async fn handle_get_nep141_from_erc20(context: &Context, address: Address) -> anyhow::Result<()> {
    let acc_id = near::get_nep141_from_erc20(context, address).await?;
    output!(
        &context.cli.output_format,
        result_object!("erc20_address" => format!("{:?}", address), "nep141_account" => acc_id.to_string())
    );
    Ok(())
}

async fn handle_get_erc20_metadata(context: &Context, erc20_id: String) -> anyhow::Result<()> {
    let erc20_id_str = erc20_id.clone();
    let meta = near::get_erc20_metadata(context, erc20_id).await?;
    output!(
        &context.cli.output_format,
        result_object!("erc20_id" => erc20_id_str, "metadata" => format!("{:?}", meta))
    );
    Ok(())
}

async fn handle_set_erc20_metadata(
    context: &Context,
    erc20_id: String,
    name: String,
    symbol: String,
    decimals: u8,
) -> anyhow::Result<()> {
    let metadata = aurora_sdk_rs::aurora::parameters::connector::Erc20Metadata {
        name,
        symbol,
        decimals,
    };
    near::set_erc20_metadata(context, erc20_id, metadata).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("ERC20 metadata set")
    );
    Ok(())
}

async fn handle_mirror_erc20_token(
    context: &Context,
    contract_id: AccountId,
    nep141: AccountId,
) -> anyhow::Result<()> {
    let addr = near::mirror_erc20_token(context, contract_id, nep141).await?;
    output!(
        &context.cli.output_format,
        result_object!("mirrored_erc20_token_address" => format!("{:?}", addr))
    );
    Ok(())
}

async fn handle_set_eth_connector_contract_account(
    context: &Context,
    account_id: AccountId,
) -> anyhow::Result<()> {
    near::set_eth_connector_contract_account(context, account_id).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Eth connector contract account set")
    );
    Ok(())
}

async fn handle_get_eth_connector_contract_account(context: &Context) -> anyhow::Result<()> {
    let account = near::get_eth_connector_contract_account(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("eth_connector_contract_account" => account.to_string())
    );
    Ok(())
}

async fn handle_set_eth_connector_contract_data(
    context: &Context,
    prover_id: AccountId,
    custodian_address: Address,
    ft_metadata_path: String,
) -> anyhow::Result<()> {
    near::set_eth_connector_contract_data(context, prover_id, custodian_address, ft_metadata_path)
        .await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Eth connector contract data set")
    );
    Ok(())
}

async fn handle_set_paused_flags(context: &Context, mask: u8) -> anyhow::Result<()> {
    near::set_paused_flags(context, mask).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Paused flags set")
    );
    Ok(())
}

async fn handle_get_paused_flags(context: &Context) -> anyhow::Result<()> {
    let flags = near::get_paused_flags(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("paused_flags" => format!("{:?}", flags))
    );
    Ok(())
}

async fn handle_add_relayer(
    context: &Context,
    deposit: NearToken,
    full_access_pub_key: PublicKey,
    function_call_pub_key: PublicKey,
) -> anyhow::Result<()> {
    let outcome =
        near::add_relayer(context, deposit, full_access_pub_key, function_call_pub_key).await?;
    output!(
        &context.cli.output_format,
        result_object!("relayer_added" => format!("{:?}", outcome))
    );
    Ok(())
}

async fn handle_withdraw_wnear_to_router(
    context: &Context,
    address: Address,
    amount: NearToken,
) -> anyhow::Result<()> {
    near::withdraw_wnear_to_router(context, address, amount).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("wNEAR withdrawn to router successfully")
    );
    Ok(())
}

async fn handle_mirror_erc20_token_callback(
    context: &Context,
    contract_id: AccountId,
    nep141: AccountId,
) -> anyhow::Result<()> {
    near::mirror_erc20_token_callback(context, contract_id, nep141).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("ERC-20 token mirrored successfully")
    );
    Ok(())
}

async fn handle_get_latest_hashchain(context: &Context) -> anyhow::Result<()> {
    let latest_hashchain = near::get_latest_hashchain(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("latest_hashchain" => latest_hashchain.to_string())
    );
    Ok(())
}

async fn handle_ft_total_supply(context: &Context) -> anyhow::Result<()> {
    let total_supply = near::ft_total_supply(context).await?;
    output!(
        &context.cli.output_format,
        result_object!("total_supply" => format!("{:?}", total_supply))
    );
    Ok(())
}

async fn handle_ft_balance_of(context: &Context, account_id: AccountId) -> anyhow::Result<()> {
    let balance = near::ft_balance_of(context, account_id.clone()).await?;
    output!(
        &context.cli.output_format,
        result_object!("account_id" => account_id.to_string(), "balance" => format!("{:?}", balance))
    );
    Ok(())
}

async fn handle_ft_balance_of_eth(context: &Context, address: Address) -> anyhow::Result<()> {
    let balance = near::ft_balance_of_eth(context, address).await?;
    output!(
        &context.cli.output_format,
        result_object!("address" => address.encode(), "balance" => format!("{:?}", balance))
    );
    Ok(())
}

async fn handle_ft_transfer(
    context: &Context,
    receiver_id: AccountId,
    amount: NearToken,
    memo: Option<String>,
) -> anyhow::Result<()> {
    near::ft_transfer(
        context,
        receiver_id,
        NEP141Wei::new(amount.as_yoctonear()),
        memo,
    )
    .await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("FT transfer executed successfully")
    );
    Ok(())
}

async fn handle_ft_transfer_call(
    context: &Context,
    receiver_id: AccountId,
    amount: NearToken,
    memo: Option<String>,
    msg: String,
) -> anyhow::Result<()> {
    near::ft_transfer_call(
        context,
        receiver_id,
        NEP141Wei::new(amount.as_yoctonear()),
        memo,
        msg,
    )
    .await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("FT transfer call executed successfully")
    );
    Ok(())
}

async fn handle_ft_on_transfer(
    context: &Context,
    sender_id: AccountId,
    amount: NearToken,
    msg: String,
) -> anyhow::Result<()> {
    near::ft_on_transfer(context, sender_id, Balance::new(amount.as_yoctonear()), msg).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("FT on transfer executed successfully")
    );
    Ok(())
}

async fn handle_deploy_erc20_token(context: &Context, nep141: AccountId) -> anyhow::Result<()> {
    let address = near::deploy_erc20_token(context, nep141).await?;
    output!(
        &context.cli.output_format,
        result_object!("deployed_erc20_token_address" => format!("{:?}", address))
    );
    Ok(())
}

async fn handle_storage_deposit(
    context: &Context,
    account_id: Option<AccountId>,
    registration_only: Option<bool>,
) -> anyhow::Result<()> {
    near::storage_deposit(context, account_id, registration_only).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Storage deposit executed successfully")
    );
    Ok(())
}

async fn handle_storage_unregister(context: &Context, force: bool) -> anyhow::Result<()> {
    near::storage_unregister(context, force).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Storage unregistered successfully")
    );
    Ok(())
}

async fn handle_storage_withdraw(
    context: &Context,
    amount: Option<NearToken>,
) -> anyhow::Result<()> {
    near::storage_withdraw(context, amount.map(|n| Yocto::new(n.as_yoctonear()))).await?;
    output!(
        &context.cli.output_format,
        CommandResult::success("Storage withdrawn successfully")
    );
    Ok(())
}

async fn handle_storage_balance_of(context: &Context, account_id: AccountId) -> anyhow::Result<()> {
    let balance = near::storage_balance_of(context, account_id.clone()).await?;
    output!(
        &context.cli.output_format,
        result_object!("account_id" => account_id.to_string(), "storage_balance" => format!("{:?}", balance))
    );
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
