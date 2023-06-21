use aurora_engine::parameters::{
    GetStorageAtArgs, InitCallArgs, NewCallArgs, NewCallArgsV2, PausePrecompilesCallArgs,
    SetOwnerArgs, TransactionStatus,
};
use aurora_engine::xcc::FundXccArgs;
use aurora_engine_sdk::types::near_account_to_evm_address;
use aurora_engine_types::account_id::AccountId;
use aurora_engine_types::parameters::engine::SubmitResult;
use aurora_engine_types::types::Address;
use aurora_engine_types::{types::Wei, H256, U256};
use borsh::{BorshDeserialize, BorshSerialize};
use near_primitives::views::{CallResult, FinalExecutionStatus};
use serde_json::Value;
use std::fmt::{Display, Formatter};
use std::{path::Path, str::FromStr};

use crate::{
    client::Client,
    utils::{self, hex_to_address, hex_to_arr, hex_to_vec, near_to_yocto, secret_key_from_hex},
};

pub mod silo;

/// Return `chain_id` of the current network.
pub async fn get_chain_id(client: Client) -> anyhow::Result<()> {
    get_value::<U256>(client, "get_chain_id", None).await
}

/// Return version of the Aurora EVM.
pub async fn get_version(client: Client) -> anyhow::Result<()> {
    get_value::<String>(client, "get_version", None).await
}

/// Return owner of the Aurora EVM.
pub async fn get_owner(client: Client) -> anyhow::Result<()> {
    get_value::<String>(client, "get_owner", None).await
}

/// Return bridge prover of the Aurora EVM.
pub async fn get_bridge_prover(client: Client) -> anyhow::Result<()> {
    get_value::<String>(client, "get_bridge_prover", None).await
}

/// Return nonce for the address.
pub async fn get_nonce(client: Client, address: String) -> anyhow::Result<()> {
    let address = hex_to_vec(&address)?;
    get_value::<U256>(client, "get_nonce", Some(address)).await
}

/// Return a height, after which an upgrade could be done.
pub async fn get_upgrade_index(client: Client) -> anyhow::Result<()> {
    get_value::<u64>(client, "get_upgrade_index", None).await
}

/// Return ETH balance of the address.
pub async fn get_balance(client: Client, address: String) -> anyhow::Result<()> {
    let address = hex_to_vec(&address)?;
    get_value::<U256>(client, "get_balance", Some(address)).await
}

/// Return a hex code of the smart contract.
pub async fn get_code(client: Client, address: String) -> anyhow::Result<()> {
    let address = hex_to_vec(&address)?;
    get_value::<HexString>(client, "get_code", Some(address)).await
}

/// Return a block hash of the specified height.
pub async fn get_block_hash(client: Client, height: u64) -> anyhow::Result<()> {
    let height = height.to_le_bytes().to_vec();
    get_value::<HexString>(client, "get_block_hash", Some(height)).await
}

/// Deploy Aurora EVM smart contract.
pub async fn deploy_aurora<P: AsRef<Path> + Send>(client: Client, path: P) -> anyhow::Result<()> {
    let code = std::fs::read(path)?;
    let result = match client.near().deploy_contract(code).await {
        Ok(outcome) => match outcome.status {
            FinalExecutionStatus::SuccessValue(_) => {
                "Aurora EVM has been deployed successfully".to_string()
            }
            FinalExecutionStatus::Failure(e) => format!("Error while deployed Aurora EVM: {e}"),
            _ => "Error: Bad transaction status".to_string(),
        },
        Err(e) => format!("{e:?}"),
    };
    println!("{result}");

    Ok(())
}

/// Initialize Aurora EVM smart contract.
pub async fn init(
    client: Client,
    chain_id: u64,
    owner_id: Option<String>,
    bridge_prover: Option<String>,
    upgrade_delay_blocks: Option<u64>,
    custodian_address: Option<String>,
    ft_metadata_path: Option<String>,
) -> anyhow::Result<()> {
    let owner_id = to_account_id(owner_id, &client)?;
    let prover_id = to_account_id(bridge_prover, &client)?;

    let aurora_init_args = NewCallArgs::V2(NewCallArgsV2 {
        chain_id: H256::from_low_u64_be(chain_id).into(),
        owner_id,
        upgrade_delay_blocks: upgrade_delay_blocks.unwrap_or_default(),
    })
    .try_to_vec()?;

    let eth_connector_init_args = InitCallArgs {
        prover_account: prover_id,
        eth_custodian_address: custodian_address.map_or_else(
            || Address::default().encode(),
            |address| address.trim_start_matches("0x").to_string(),
        ),
        metadata: utils::ft_metadata::parse_ft_metadata(
            ft_metadata_path.and_then(|path| std::fs::read_to_string(path).ok()),
        )?,
    }
    .try_to_vec()?;

    let batch = vec![
        ("new".to_string(), aurora_init_args),
        ("new_eth_connector".to_string(), eth_connector_init_args),
    ];

    match client.near().contract_call_batch(batch).await?.status {
        FinalExecutionStatus::Failure(e) => {
            anyhow::bail!("Error while initializing Aurora EVM: {e}")
        }
        FinalExecutionStatus::Started | FinalExecutionStatus::NotStarted => {
            anyhow::bail!("Error while initializing Aurora EVM: Bad status of the transaction")
        }
        FinalExecutionStatus::SuccessValue(_) => {}
    }

    println!("Aurora EVM has been initialized successfully");

    Ok(())
}

/// Deploy EVM byte code.
pub async fn deploy_evm_code(
    client: Client,
    code: String,
    abi_path: Option<String>,
    args: Option<String>,
    sk: Option<&str>,
) -> anyhow::Result<()> {
    let sk = sk
        .ok_or_else(|| anyhow::anyhow!("Deploy EVM code requires Aurora secret key"))
        .and_then(secret_key_from_hex)?;
    let input =
        if let Some((abi_path, args)) = abi_path.and_then(|path| args.map(|args| (path, args))) {
            let contract = utils::abi::read_contract(abi_path)?;
            let constructor = contract
                .constructor()
                .ok_or_else(|| anyhow::anyhow!("No constructor definition"))?;
            let args: Value = serde_json::from_str(&args)?;
            let tokens = utils::abi::parse_args(&constructor.inputs, &args)?;
            let code = hex::decode(code)?;
            constructor.encode_input(code, &tokens)?
        } else {
            hex::decode(code)?
        };

    let result = client
        .near()
        .send_aurora_transaction(&sk, None, Wei::zero(), input)
        .await?;
    let output = match result.status {
        FinalExecutionStatus::NotStarted | FinalExecutionStatus::Started => {
            anyhow::bail!("Error while deploying EVM code: Bad status of the transaction")
        }
        FinalExecutionStatus::Failure(e) => {
            anyhow::bail!("Error while deploying EVM code: {e}")
        }
        FinalExecutionStatus::SuccessValue(ref bytes) => {
            let result = SubmitResult::try_from_slice(bytes)?;
            if let TransactionStatus::Succeed(bytes) = result.status {
                format!(
                    "Contract has been deployed to address: 0x{} successfully",
                    hex::encode(bytes)
                )
            } else {
                format!("Transaction reverted: {result:?}")
            }
        }
    };

    println!("{output}");

    Ok(())
}

/// Creates new NEAR account.
pub async fn create_account(
    client: Client,
    account: &str,
    initial_balance: f64,
) -> anyhow::Result<()> {
    match client.near().create_account(account, initial_balance).await {
        Ok(result) => println!("{result}"),
        Err(e) => eprintln!("{e:?}"),
    }

    Ok(())
}

/// View new NEAR account.
pub async fn view_account(client: Client, account: &str) -> anyhow::Result<()> {
    match client.near().view_account(account).await {
        Ok(result) => println!("{result}"),
        Err(e) => eprintln!("{e:?}"),
    }

    Ok(())
}

/// Read-only call of the EVM smart contract.
pub async fn view_call(
    client: Client,
    address: String,
    function: String,
    args: Option<String>,
    abi_path: String,
) -> anyhow::Result<()> {
    let target = hex_to_address(&address)?;
    let contract = utils::abi::read_contract(abi_path)?;
    let func = contract.function(&function)?;
    let args: Value = args.map_or(Ok(Value::Null), |args| serde_json::from_str(&args))?;
    let tokens = utils::abi::parse_args(&func.inputs, &args)?;
    let input = func.encode_input(&tokens)?;
    let result = client
        .near()
        .view_contract_call(Address::default(), target, Wei::zero(), input)
        .await?;

    if let TransactionStatus::Succeed(bytes) = result {
        let parsed_output = func.decode_output(&bytes)?;
        let result = parsed_output
            .iter()
            .map(ethabi::Token::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        println!("{result}");
    } else {
        println!("{result:?}");
    }

    Ok(())
}

/// Modifying call of the EVM smart contract.
pub async fn call(
    client: Client,
    address: String,
    function: String,
    args: Option<String>,
    abi_path: String,
    value: Option<String>,
    sk: Option<&str>,
) -> anyhow::Result<()> {
    let sk = sk
        .ok_or_else(|| anyhow::anyhow!("Call contract requires Aurora secret key"))
        .and_then(secret_key_from_hex)?;
    let target = hex_to_address(&address)?;
    let contract = utils::abi::read_contract(abi_path)?;
    let func = contract.function(&function)?;
    let args: Value = args.map_or(Ok(Value::Null), |args| serde_json::from_str(&args))?;
    let tokens = utils::abi::parse_args(&func.inputs, &args)?;
    let input = func.encode_input(&tokens)?;
    let amount = value
        .and_then(|a| U256::from_dec_str(&a).ok())
        .map_or_else(Wei::zero, Wei::new);

    let result = client
        .near()
        .send_aurora_transaction(&sk, Some(target), amount, input)
        .await?;
    let (gas, status) = match result.status {
        FinalExecutionStatus::NotStarted | FinalExecutionStatus::Started => {
            anyhow::bail!("Error while calling EVM transaction: Bad status of the transaction")
        }
        FinalExecutionStatus::Failure(e) => {
            anyhow::bail!("Error while calling EVM transaction: {e}")
        }
        FinalExecutionStatus::SuccessValue(bytes) => {
            let result = SubmitResult::try_from_slice(&bytes)?;
            let status = match result.status {
                TransactionStatus::Succeed(_) => "successful",
                TransactionStatus::Revert(_) => "reverted",
                TransactionStatus::OutOfGas => "out_of_gas",
                TransactionStatus::OutOfFund => "out_of_fund",
                TransactionStatus::OutOfOffset => "out_of_offset",
                TransactionStatus::CallTooDeep => "call_too_deep",
            };

            (result.gas_used, status)
        }
    };

    println!("Aurora transaction status: {status}, gas used: {gas}");

    Ok(())
}

/// Stage code for delayed upgrade.
pub async fn stage_upgrade<P: AsRef<Path> + Send>(client: Client, path: P) -> anyhow::Result<()> {
    let code = std::fs::read(path)?;

    ContractCall {
        method: "stage_upgrade",
        success_message: "The code has been saved for staged upgrade successfully",
        error_message: "Error while staging code for upgrade",
    }
    .proceed(client, code)
    .await
}

/// Deploy staged upgrade.
pub async fn deploy_upgrade(client: Client) -> anyhow::Result<()> {
    ContractCall {
        method: "deploy_upgrade",
        success_message: "The upgrade has been applied successfully",
        error_message: "Error while deploying upgrade",
    }
    .proceed(client, vec![])
    .await
}

/// Updates the bytecode for user's router contracts.
pub async fn factory_update(client: Client, path: String) -> anyhow::Result<()> {
    let code = std::fs::read(path)?;

    ContractCall {
        method: "factory_update",
        success_message: "The bytecode of user's router contract has been updated successfully",
        error_message: "Error while updating the bytecode of user's router contract",
    }
    .proceed(client, code)
    .await
}

/// Sets the address for the `wNEAR` ERC-20 contract
pub async fn factory_set_wnear_address(client: Client, address: String) -> anyhow::Result<()> {
    let args: [u8; 20] = hex_to_arr(&address)?;

    ContractCall {
        method: "factory_set_wnear_address",
        success_message: "The wnear address has been set successfully",
        error_message: "Error while upgrading wnear address",
    }
    .proceed(client, args.to_vec())
    .await
}

/// Create and/or fund an XCC sub-account directly
pub async fn fund_xcc_sub_account(
    client: Client,
    target: String,
    account_id: Option<String>,
    deposit: f64,
) -> anyhow::Result<()> {
    let args = FundXccArgs {
        target: hex_to_address(&target)?,
        wnear_account_id: account_id
            .map(|id| id.parse().map_err(|e| anyhow::anyhow!("{e}")))
            .transpose()?,
    }
    .try_to_vec()?;

    ContractCall {
        method: "fund_xcc_sub_account",
        success_message: "The XCC sub-account has been funded successfully",
        error_message: "Error while funding XCC sub-account",
    }
    .proceed_with_deposit(client, args, deposit)
    .await
}

/// Set a new owner of the Aurora EVM.
pub async fn set_owner(client: Client, account_id: String) -> anyhow::Result<()> {
    let args = SetOwnerArgs {
        new_owner: account_id.parse().map_err(|e| anyhow::anyhow!("{e}"))?,
    }
    .try_to_vec()?;

    ContractCall {
        method: "set_owner",
        success_message: "The owner has been changed successfully",
        error_message: "Error while setting a new owner",
    }
    .proceed(client, args)
    .await
}

/// Register relayer address.
pub async fn register_relayer(client: Client, address: String) -> anyhow::Result<()> {
    let args = hex_to_vec(&address)?;

    ContractCall {
        method: "register_relayer",
        success_message: "The new relayer has been registered successfully",
        error_message: "Error while registering a new relayer",
    }
    .proceed(client, args)
    .await
}

/// Return value in storage for key at address.
pub async fn get_storage_at(client: Client, address: String, key: String) -> anyhow::Result<()> {
    let address = hex_to_address(&address)?;
    let key = H256::from_str(&key)?;
    let input = GetStorageAtArgs {
        address,
        key: key.0,
    }
    .try_to_vec()?;

    get_value::<H256>(client, "get_storage_at", Some(input)).await
}

/// Return EVM address from NEAR account.
pub fn encode_address(account: &str) {
    let result = near_account_to_evm_address(account.as_bytes()).encode();
    println!("0x{result}");
}

/// Return an address and corresponding private key in JSON format.
pub fn key_pair(random: bool, seed: Option<u64>) -> anyhow::Result<()> {
    let (address, sk) = utils::gen_key_pair(random, seed)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "address": format!("0x{}", address.encode()),
            "secret_key": hex::encode(sk.serialize()),
        }))?
    );

    Ok(())
}

/// Pause precompiles with mask.
pub async fn pause_precompiles(client: Client, mask: u32) -> anyhow::Result<()> {
    let args = PausePrecompilesCallArgs { paused_mask: mask }.try_to_vec()?;

    ContractCall {
        method: "pause_precompiles",
        success_message: "The precompiles have been paused successfully",
        error_message: "Error while pausing precompiles",
    }
    .proceed(client, args)
    .await
}

/// Resume precompiles with mask.
pub async fn resume_precompiles(client: Client, mask: u32) -> anyhow::Result<()> {
    let args = PausePrecompilesCallArgs { paused_mask: mask }.try_to_vec()?;

    ContractCall {
        method: "resume_precompiles",
        success_message: "The precompiles have been resumed successfully",
        error_message: "Error while resuming precompiles",
    }
    .proceed(client, args)
    .await
}

/// Return paused precompiles.
pub async fn paused_precompiles(client: Client) -> anyhow::Result<()> {
    get_value::<u32>(client, "paused_precompiles", None).await
}

async fn get_value<T: FromCallResult + Display>(
    client: Client,
    method_name: &str,
    args: Option<Vec<u8>>,
) -> anyhow::Result<()> {
    let result = client
        .near()
        .view_call(method_name, args.unwrap_or_default())
        .await?;
    let output = T::from_result(result)?;
    println!("{output}");

    Ok(())
}

fn to_account_id(id: Option<String>, client: &Client) -> anyhow::Result<AccountId> {
    id.map_or_else(
        || {
            client
                .near()
                .engine_account_id
                .to_string()
                .parse()
                .map_err(|e| anyhow::anyhow!("{e}"))
        },
        |id| id.parse().map_err(|e| anyhow::anyhow!("{e}")),
    )
}

struct HexString(String);

trait FromCallResult {
    fn from_result(result: CallResult) -> anyhow::Result<Self>
    where
        Self: Sized;
}

impl FromCallResult for H256 {
    fn from_result(result: CallResult) -> anyhow::Result<Self> {
        Ok(Self::from_slice(&result.result))
    }
}

impl FromCallResult for U256 {
    fn from_result(result: CallResult) -> anyhow::Result<Self> {
        Ok(Self::from_big_endian(&result.result))
    }
}

impl FromCallResult for u64 {
    fn from_result(result: CallResult) -> anyhow::Result<Self> {
        let mut buffer = [0; 8];
        buffer.copy_from_slice(&result.result);
        Ok(Self::from_le_bytes(buffer))
    }
}

impl FromCallResult for u32 {
    fn from_result(result: CallResult) -> anyhow::Result<Self> {
        let mut buffer = [0; 4];
        buffer.copy_from_slice(&result.result);
        Ok(Self::from_le_bytes(buffer))
    }
}

impl FromCallResult for String {
    fn from_result(result: CallResult) -> anyhow::Result<Self> {
        let output = Self::from_utf8(result.result)?;
        Ok(output.trim().to_string())
    }
}

impl FromCallResult for HexString {
    fn from_result(result: CallResult) -> anyhow::Result<Self> {
        Ok(Self(hex::encode(result.result)))
    }
}

impl Display for HexString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", self.0)
    }
}

struct ContractCall<'a> {
    method: &'a str,
    success_message: &'a str,
    error_message: &'a str,
}

impl ContractCall<'_> {
    async fn proceed(&self, client: Client, args: Vec<u8>) -> anyhow::Result<()> {
        self.proceed_with_deposit(client, args, 0.0).await
    }

    async fn proceed_with_deposit(
        &self,
        client: Client,
        args: Vec<u8>,
        deposit: f64,
    ) -> anyhow::Result<()> {
        let yocto = near_to_yocto(deposit);
        let result = client
            .near()
            .contract_call_with_deposit(self.method, args, yocto)
            .await?;

        match result.status {
            FinalExecutionStatus::NotStarted | FinalExecutionStatus::Started => {
                anyhow::bail!("{}: Bad transaction status", self.error_message)
            }
            FinalExecutionStatus::Failure(e) => {
                anyhow::bail!("{}: {e}", self.error_message)
            }
            FinalExecutionStatus::SuccessValue(_) => {
                println!("{}", self.success_message);
            }
        }

        Ok(())
    }
}
