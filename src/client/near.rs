use aurora_engine_types::borsh::BorshDeserialize;
#[cfg(feature = "advanced")]
use aurora_engine_types::parameters::engine::SubmitResult;
use aurora_engine_types::parameters::engine::TransactionStatus;
use aurora_engine_types::{
    types::{Address, Wei},
    U256,
};
use near_crypto::InMemorySigner;
#[cfg(feature = "simple")]
use near_crypto::PublicKey;
#[cfg(feature = "simple")]
use near_jsonrpc_client::methods::tx::{
    RpcTransactionResponse, RpcTransactionStatusRequest, TransactionInfo,
};
use near_jsonrpc_client::{
    methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest, AsUrl, JsonRpcClient,
};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::transaction::{Action, SignedTransaction};
#[cfg(feature = "simple")]
use near_primitives::views::FinalExecutionStatus;
#[cfg(feature = "simple")]
use near_primitives::views::TxExecutionStatus;
#[cfg(feature = "simple")]
use near_primitives::{
    account::{AccessKey, AccessKeyPermission, FunctionCallPermission},
    action::{AddKeyAction, CreateAccountAction, TransferAction},
};
use near_primitives::{
    hash::CryptoHash, types::AccountId, views, views::FinalExecutionOutcomeView,
};
#[cfg(feature = "simple")]
use std::str::FromStr;
use std::time::Duration;

#[cfg(feature = "advanced")]
use super::TransactionOutcome;
use crate::utils;

// The maximum amount of prepaid NEAR gas required for paying for a transaction.
const NEAR_GAS: u64 = 300_000_000_000_000;
const TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Clone)]
pub struct NearClient {
    client: JsonRpcClient,
    pub engine_account_id: AccountId,
    signer_key_path: Option<String>,
}

impl NearClient {
    pub fn new<U: AsUrl>(url: U, engine_account_id: &str, signer_key_path: Option<String>) -> Self {
        let mut headers = reqwest::header::HeaderMap::with_capacity(2);
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        let client = reqwest::Client::builder()
            .timeout(TIMEOUT)
            .connect_timeout(TIMEOUT)
            .default_headers(headers)
            .build()
            .map(JsonRpcClient::with)
            .expect("couldn't create json rpc client");
        let client = client.connect(url);

        Self {
            client,
            engine_account_id: engine_account_id.parse().unwrap(),
            signer_key_path,
        }
    }

    #[cfg(feature = "advanced")]
    pub async fn get_receipt_outcome(
        &self,
        near_receipt_id: CryptoHash,
    ) -> anyhow::Result<TransactionOutcome> {
        let mut receipt_id = near_receipt_id;
        let receiver_id = &self.engine_account_id;
        loop {
            let block_hash = {
                let request = near_jsonrpc_client::methods::block::RpcBlockRequest {
                    block_reference: near_primitives::types::Finality::Final.into(),
                };
                let response = self.client.call(request).await?;
                response.header.hash
            };
            let request = near_jsonrpc_client::methods::light_client_proof::RpcLightClientExecutionProofRequest {
                id: near_primitives::types::TransactionOrReceiptId::Receipt { receipt_id, receiver_id: receiver_id.clone() },
                light_client_head: block_hash,
            };
            let response = self.client.call(request).await?;

            match response.outcome_proof.outcome.status {
                views::ExecutionStatusView::SuccessValue(result) => {
                    let result = SubmitResult::try_from_slice(&result)?;
                    break Ok(TransactionOutcome::Result(result));
                }
                views::ExecutionStatusView::Failure(e) => {
                    break Ok(TransactionOutcome::Failure(e));
                }
                views::ExecutionStatusView::SuccessReceiptId(id) => {
                    println!("Intermediate receipt_id: {id:?}");
                    receipt_id = id;
                }
                views::ExecutionStatusView::Unknown => {
                    panic!("Unknown receipt_id: {near_receipt_id:?}")
                }
            }
        }
    }

    #[cfg(feature = "advanced")]
    pub async fn get_nep141_from_erc20(&self, erc20: Address) -> anyhow::Result<String> {
        let result = self
            .view_call("get_nep141_from_erc20", erc20.as_bytes().to_vec())
            .await?;
        Ok(String::from_utf8_lossy(&result.result).into_owned())
    }

    #[cfg(feature = "advanced")]
    pub async fn get_erc20_from_nep141(&self, nep141: &str) -> anyhow::Result<Address> {
        let args = aurora_engine_types::parameters::engine::GetErc20FromNep141CallArgs {
            nep141: nep141.parse().unwrap(),
        };
        let result = self
            .view_call("get_erc20_from_nep141", borsh::to_vec(&args)?)
            .await?;

        Address::try_from_slice(&result.result).map_err(|e| anyhow::anyhow!(e))
    }

    #[cfg(feature = "advanced")]
    pub async fn get_bridge_prover(&self) -> anyhow::Result<String> {
        let result = self.view_call("get_bridge_prover", Vec::new()).await?;
        Ok(String::from_utf8_lossy(&result.result).into_owned())
    }

    pub async fn view_contract_call(
        &self,
        sender: Address,
        target: Address,
        amount: Wei,
        input: Vec<u8>,
    ) -> anyhow::Result<TransactionStatus> {
        let args = aurora_engine_types::parameters::engine::ViewCallArgs {
            sender,
            address: target,
            amount: amount.to_bytes(),
            input,
        };
        let result = self.view_call("view", borsh::to_vec(&args)?).await?;
        let status = TransactionStatus::try_from_slice(&result.result)?;
        Ok(status)
    }

    pub async fn view_call(
        &self,
        method_name: &str,
        args: Vec<u8>,
    ) -> anyhow::Result<views::CallResult> {
        let request = near_jsonrpc_client::methods::query::RpcQueryRequest {
            block_reference: near_primitives::types::Finality::Final.into(),
            request: views::QueryRequest::CallFunction {
                account_id: self.engine_account_id.clone(),
                method_name: method_name.to_string(),
                args: args.into(),
            },
        };
        let response = self.client.call(request).await?;

        match response.kind {
            QueryResponseKind::CallResult(result) => Ok(result),
            _ => anyhow::bail!("Wrong response type"),
        }
    }

    #[cfg(feature = "simple")]
    pub async fn view_account(&self, account: &str) -> anyhow::Result<String> {
        let account_id: AccountId = account.parse()?;
        let request = near_jsonrpc_client::methods::query::RpcQueryRequest {
            block_reference: near_primitives::types::BlockReference::Finality(
                near_primitives::types::Finality::Final,
            ),
            request: views::QueryRequest::ViewAccount { account_id },
        };

        let response = self.client.call(request).await?;

        match response.kind {
            QueryResponseKind::ViewAccount(view) => Ok(serde_json::to_string_pretty(&view)?),
            _ => anyhow::bail!("Wrong type response"),
        }
    }

    pub async fn contract_call(
        &self,
        method_name: &str,
        args: Vec<u8>,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        self.contract_call_with_deposit(method_name, args, 0).await
    }

    pub async fn contract_call_with_deposit(
        &self,
        method_name: &str,
        args: Vec<u8>,
        deposit: u128,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        self.near_broadcast_tx(
            vec![Action::FunctionCall(Box::new(
                near_primitives::transaction::FunctionCallAction {
                    method_name: method_name.to_string(),
                    args,
                    gas: NEAR_GAS,
                    deposit,
                },
            ))],
            None,
        )
        .await
    }

    #[cfg(feature = "simple")]
    pub async fn contract_call_batch(
        &self,
        batch: Vec<(String, Vec<u8>)>,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        let batch_with_deposit: Vec<(String, Vec<u8>, u128)> = batch
            .into_iter()
            .map(|(method_name, args)| (method_name, args, 0u128))
            .collect();

        self.contract_call_batch_with_deposit(batch_with_deposit)
            .await
    }

    #[cfg(feature = "simple")]
    pub async fn contract_call_batch_with_deposit(
        &self,
        batch: Vec<(String, Vec<u8>, u128)>,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        let gas = NEAR_GAS / u64::try_from(batch.len())?;
        let actions = batch
            .into_iter()
            .map(|(method_name, args, deposit)| {
                Action::FunctionCall(Box::new(near_primitives::transaction::FunctionCallAction {
                    method_name,
                    args,
                    gas,
                    deposit,
                }))
            })
            .collect();

        self.near_broadcast_tx(actions, None).await
    }

    #[cfg(feature = "advanced")]
    pub async fn contract_call_with_nonce(
        &self,
        method_name: &str,
        args: Vec<u8>,
        nonce_override: u64,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        self.near_broadcast_tx(
            vec![Action::FunctionCall(Box::new(
                near_primitives::transaction::FunctionCallAction {
                    method_name: method_name.to_string(),
                    args,
                    gas: NEAR_GAS,
                    deposit: 0,
                },
            ))],
            Some(nonce_override),
        )
        .await
    }

    #[cfg(feature = "advanced")]
    pub async fn contract_call_from(
        &self,
        method_name: &str,
        args: Vec<u8>,
        from: AccountId,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        self.near_broadcast_tx_from(
            vec![Action::FunctionCall(Box::new(
                near_primitives::transaction::FunctionCallAction {
                    method_name: method_name.to_string(),
                    args,
                    gas: NEAR_GAS,
                    deposit: 0,
                },
            ))],
            from,
            None,
        )
        .await
    }

    async fn near_broadcast_tx(
        &self,
        actions: Vec<Action>,
        nonce_override: Option<u64>,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        let signer = self.signer()?;
        self.near_broadcast_tx_from(actions, signer.account_id.clone(), nonce_override)
            .await
    }

    async fn near_broadcast_tx_from(
        &self,
        actions: Vec<Action>,
        from: AccountId,
        nonce_override: Option<u64>,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        let mut signer = self.signer()?;
        signer.account_id = from.clone();

        let (block_hash, nonce) = self.get_nonce(&signer).await?;
        let nonce = nonce_override.unwrap_or(nonce);

        let request = RpcBroadcastTxCommitRequest {
            signed_transaction: SignedTransaction::from_actions(
                nonce,
                from,
                self.engine_account_id.as_str().parse()?,
                &signer.into(),
                actions,
                block_hash,
                0,
            ),
        };
        let response = self.client.call(request).await?;

        Ok(response)
    }

    /// Creates new NEAR account.
    #[cfg(feature = "simple")]
    pub async fn create_account(&self, account: &str, deposit: f64) -> anyhow::Result<String> {
        let signer = self.signer()?;
        let new_account_id = AccountId::from_str(account)?;
        let is_sub_account = new_account_id.is_sub_account_of(&signer.account_id);
        let new_key_pair = near_crypto::SecretKey::from_random(near_crypto::KeyType::ED25519);
        let (block_hash, nonce) = self.get_nonce(&signer).await?;
        let initial_balance = utils::near_to_yocto(deposit);

        let request = if is_sub_account {
            RpcBroadcastTxCommitRequest {
                signed_transaction: SignedTransaction::create_account(
                    nonce,
                    signer.account_id.clone(),
                    new_account_id,
                    initial_balance,
                    new_key_pair.public_key(),
                    &signer.into(),
                    block_hash,
                ),
            }
        } else {
            let contract_id = self.contract_id()?;
            RpcBroadcastTxCommitRequest {
                signed_transaction: SignedTransaction::call(
                    nonce,
                    signer.account_id.clone(),
                    contract_id,
                    &signer.into(),
                    initial_balance,
                    "create_account".to_string(),
                    serde_json::json!({
                        "new_account_id": new_account_id,
                        "new_public_key": new_key_pair.public_key(),
                    })
                    .to_string()
                    .into_bytes(),
                    NEAR_GAS,
                    block_hash,
                ),
            }
        };

        let response = self.client.call(request).await?;

        match &response.status {
            FinalExecutionStatus::NotStarted => {
                anyhow::bail!("Transaction execution status: not started")
            }
            FinalExecutionStatus::Started => anyhow::bail!("Transaction execution status: started"),
            FinalExecutionStatus::Failure(error) => anyhow::bail!(error.to_string()),
            FinalExecutionStatus::SuccessValue(result) => {
                if String::from_utf8_lossy(result) == "false" {
                    anyhow::bail!(
                        "Error while creating account, tx hash: {}",
                        response.transaction.hash
                    )
                }

                Ok(serde_json::to_string_pretty(&serde_json::json!({
                    "account_id": account,
                    "public_key": new_key_pair.public_key().to_string(),
                    "private_key": new_key_pair.to_string(),
                }))?)
            }
        }
    }

    /// Deploy WASM contract.
    pub async fn deploy_contract(
        &self,
        code: Vec<u8>,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        let signer = self.signer()?;
        let (block_hash, nonce) = self.get_nonce(&signer).await?;
        let request = RpcBroadcastTxCommitRequest {
            signed_transaction: SignedTransaction::from_actions(
                nonce,
                signer.account_id.clone(),
                signer.account_id.clone(),
                &signer.into(),
                vec![Action::DeployContract(
                    near_primitives::transaction::DeployContractAction { code },
                )],
                block_hash,
                0,
            ),
        };

        self.client.call(request).await.map_err(Into::into)
    }

    /// Send Aurora EVM transaction via NEAR network.
    pub async fn send_aurora_transaction(
        &self,
        sk: &libsecp256k1::SecretKey,
        to: Option<Address>,
        amount: Wei,
        input: Vec<u8>,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        let sender_address = utils::address_from_secret_key(sk)?;
        let nonce = {
            let result = self
                .view_call("get_nonce", sender_address.as_bytes().to_vec())
                .await?;
            U256::from_big_endian(&result.result)
        };
        let tx = aurora_engine_transactions::legacy::TransactionLegacy {
            nonce,
            gas_price: U256::zero(),
            gas_limit: U256::from(u64::MAX),
            to,
            value: amount,
            data: input,
        };
        let chain_id = {
            let result = self
                .view_call("get_chain_id", sender_address.as_bytes().to_vec())
                .await?;
            U256::from_big_endian(&result.result).low_u64()
        };
        let signed_tx = aurora_engine_transactions::EthTransactionKind::Legacy(
            utils::sign_transaction(tx, chain_id, sk),
        );
        let result = self.contract_call("submit", (&signed_tx).into()).await?;

        Ok(result)
    }

    pub async fn get_nonce(&self, signer: &InMemorySigner) -> anyhow::Result<(CryptoHash, u64)> {
        let request = near_jsonrpc_primitives::types::query::RpcQueryRequest {
            block_reference: near_primitives::types::Finality::Final.into(),
            request: views::QueryRequest::ViewAccessKey {
                account_id: signer.account_id.clone(),
                public_key: signer.public_key.clone(),
            },
        };
        let response = self.client.call(request).await?;
        let block_hash = response.block_hash;
        let nonce = match response.kind {
            QueryResponseKind::AccessKey(k) => k.nonce + 1,
            _ => anyhow::bail!("Wrong response kind: {:?}", response.kind),
        };

        Ok((block_hash, nonce))
    }

    fn signer(&self) -> anyhow::Result<InMemorySigner> {
        std::env::var("NEAR_KEY_PATH")
            .ok()
            .as_ref()
            .or(self.signer_key_path.as_ref())
            .map(std::path::Path::new)
            .ok_or_else(|| {
                anyhow::anyhow!("Path to the key file must be provided to use this functionality")
            })
            .and_then(utils::read_key_file)
    }

    #[cfg(feature = "simple")]
    fn contract_id(&self) -> anyhow::Result<AccountId> {
        let server_addr = self.client.server_addr();

        let account = if server_addr.contains("testnet.near.org") {
            "testnet"
        } else if server_addr.contains("mainnet.near.org") {
            "near"
        } else {
            anyhow::bail!("Non-sub accounts could be created for mainnet or testnet only");
        };

        let account_id = account.parse()?;
        Ok(account_id)
    }

    #[cfg(feature = "simple")]
    pub async fn transaction_status(
        &self,
        hash: CryptoHash,
        wait_until: TxExecutionStatus,
    ) -> anyhow::Result<RpcTransactionResponse> {
        let signer = self.signer()?;
        let req = RpcTransactionStatusRequest {
            transaction_info: TransactionInfo::TransactionId {
                tx_hash: hash,
                sender_account_id: signer.account_id,
            },
            wait_until,
        };

        let rsp = self.client.call(req).await?;
        Ok(rsp)
    }

    #[cfg(feature = "simple")]
    pub async fn add_relayer(
        &self,
        contract_id: AccountId,
        deposit: u128,
        full_access_key: PublicKey,
        function_call_key: PublicKey,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        let actions = vec![
            Action::CreateAccount(CreateAccountAction {}),
            Action::Transfer(TransferAction { deposit }),
            Action::AddKey(Box::new(AddKeyAction {
                public_key: full_access_key,
                access_key: AccessKey {
                    nonce: 0,
                    permission: AccessKeyPermission::FullAccess,
                },
            })),
            Action::AddKey(Box::new(AddKeyAction {
                public_key: function_call_key,
                access_key: AccessKey {
                    nonce: 0,
                    permission: AccessKeyPermission::FunctionCall(FunctionCallPermission {
                        allowance: None,
                        receiver_id: contract_id.clone().into(),
                        method_names: vec![
                            "submit".to_string(),
                            "submit_with_args".to_string(),
                            "call".to_string(),
                        ],
                    }),
                },
            })),
        ];

        let rsp = self
            .near_broadcast_tx_from(actions, contract_id, None)
            .await?;
        Ok(rsp)
    }
}
