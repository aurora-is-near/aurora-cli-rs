use aurora_engine_types::borsh::{BorshDeserialize, BorshSerialize};
#[cfg(feature = "advanced")]
use aurora_engine_types::parameters::engine::SubmitResult;
use aurora_engine_types::parameters::engine::TransactionStatus;
use aurora_engine_types::{
    types::{Address, Wei},
    U256,
};
use near_crypto::InMemorySigner;
use near_jsonrpc_client::{
    methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest, AsUrl, JsonRpcClient,
};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
#[cfg(feature = "simple")]
use near_primitives::views::FinalExecutionStatus;
use near_primitives::{
    account::{AccessKey, AccessKeyPermission},
    hash::CryptoHash,
    transaction::{
        Action, AddKeyAction, CreateAccountAction, DeployContractAction, FunctionCallAction,
        Transaction, TransferAction,
    },
    types::AccountId,
    views,
    views::FinalExecutionOutcomeView,
};

#[cfg(feature = "simple")]
use std::str::FromStr;

#[cfg(feature = "advanced")]
use super::TransactionOutcome;
use crate::utils;

// The maximum amount of prepaid NEAR gas required for paying for a transaction.
const NEAR_GAS: u64 = 300_000_000_000_000;

pub struct NearClient {
    client: JsonRpcClient,
    pub engine_account_id: AccountId,
    signer_key_path: Option<String>,
    ledger: bool,
}

impl NearClient {
    pub fn new<U: AsUrl>(
        url: U,
        engine_account_id: &str,
        signer_key_path: Option<String>,
        use_ledger: bool,
    ) -> Self {
        let client = JsonRpcClient::connect(url);
        Self {
            client,
            engine_account_id: engine_account_id.parse().unwrap(),
            signer_key_path,
            ledger: use_ledger,
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
            .view_call("get_erc20_from_nep141", args.try_to_vec()?)
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
        let result = self.view_call("view", args.try_to_vec()?).await?;
        let status = TransactionStatus::try_from_slice(&result.result)?;
        Ok(status)
    }

    pub async fn view_call(
        &self,
        method_name: &str,
        args: Vec<u8>,
    ) -> anyhow::Result<views::CallResult> {
        let request = near_jsonrpc_primitives::types::query::RpcQueryRequest {
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
            vec![Action::FunctionCall(
                near_primitives::transaction::FunctionCallAction {
                    method_name: method_name.to_string(),
                    args,
                    gas: NEAR_GAS,
                    deposit,
                },
            )],
            None,
        )
        .await
    }

    #[cfg(feature = "simple")]
    pub async fn contract_call_batch(
        &self,
        batch: Vec<(String, Vec<u8>)>,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        let gas = NEAR_GAS / u64::try_from(batch.len())?;
        let actions = batch
            .into_iter()
            .map(|(method_name, args)| {
                Action::FunctionCall(near_primitives::transaction::FunctionCallAction {
                    method_name,
                    args,
                    gas,
                    deposit: 0,
                })
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
            vec![Action::FunctionCall(
                near_primitives::transaction::FunctionCallAction {
                    method_name: method_name.to_string(),
                    args,
                    gas: NEAR_GAS,
                    deposit: 0,
                },
            )],
            Some(nonce_override),
        )
        .await
    }

    async fn near_broadcast_tx(
        &self,
        actions: Vec<Action>,
        nonce_override: Option<u64>,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        let signer = self.signer()?;
        let (block_hash, nonce) = self.get_nonce(&signer).await?;
        let nonce = nonce_override.unwrap_or(nonce);
        let unsigned_transaction = Transaction {
            signer_id: signer.account_id.clone(),
            public_key: signer.public_key.clone(),
            nonce,
            receiver_id: self.engine_account_id.parse().unwrap(),
            block_hash,
            actions,
        };

        let signed_transaction = if self.ledger {
            utils::sign_near_transaction_with_ledger(unsigned_transaction).unwrap()
        } else {
            unsigned_transaction.sign(&signer)
        };

        let request = RpcBroadcastTxCommitRequest { signed_transaction };

        let response: FinalExecutionOutcomeView = self.client.call(request).await?;

        Ok(response)
    }

    /// Fund NEAR account
    #[cfg(feature = "simple")]
    pub async fn send_money(&self, account: &str, amount: f64) -> anyhow::Result<String> {
        let signer = self.signer()?;
        let receiver_id = AccountId::from_str(account)?;
        let (block_hash, nonce) = self.get_nonce(&signer).await?;
        let deposit = utils::near_to_yocto(amount);

        let unsigned_transaction = Transaction {
            signer_id: signer.account_id.clone(),
            public_key: signer.public_key.clone(),
            nonce,
            receiver_id: receiver_id.clone(),
            block_hash,
            actions: vec![Action::Transfer(TransferAction { deposit })],
        };

        let signed_transaction = if self.ledger {
            utils::sign_near_transaction_with_ledger(unsigned_transaction).unwrap()
        } else {
            unsigned_transaction.sign(&signer)
        };

        let request = RpcBroadcastTxCommitRequest { signed_transaction };
        let response: FinalExecutionOutcomeView = self.client.call(request).await?;

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

                Ok(format!(
                    "Account {receiver_id:?} has received {amount:?}NEAR"
                ))
            }
        }
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
            let unsigned_transaction = Transaction {
                signer_id: signer.account_id.clone(),
                public_key: signer.public_key.clone(),
                nonce,
                receiver_id: new_account_id.clone(),
                block_hash,
                actions: vec![
                    Action::CreateAccount(CreateAccountAction {}),
                    Action::AddKey(AddKeyAction {
                        public_key: new_key_pair.public_key(),
                        access_key: AccessKey {
                            nonce: 0,
                            permission: AccessKeyPermission::FullAccess,
                        },
                    }),
                    Action::Transfer(TransferAction {
                        deposit: initial_balance,
                    }),
                ],
            };

            let signed_transaction = if self.ledger {
                utils::sign_near_transaction_with_ledger(unsigned_transaction).unwrap()
            } else {
                unsigned_transaction.sign(&signer)
            };

            RpcBroadcastTxCommitRequest { signed_transaction }
        } else {
            let contract_id = self.contract_id()?;
            let new_public_key = if self.ledger {
                signer.public_key.clone() // use the ledger public key for named account
            } else {
                new_key_pair.public_key()
            };

            let unsigned_transaction = Transaction {
                signer_id: signer.account_id.clone(),
                public_key: signer.public_key.clone(),
                nonce,
                receiver_id: contract_id,
                block_hash,
                actions: vec![Action::FunctionCall(FunctionCallAction {
                    args: serde_json::json!({
                        "new_account_id": new_account_id,
                        "new_public_key": new_public_key,
                    })
                    .to_string()
                    .into_bytes(),
                    method_name: "create_account".to_string(),
                    gas: NEAR_GAS,
                    deposit: initial_balance,
                })],
            };
            let signed_transaction = if self.ledger {
                utils::sign_near_transaction_with_ledger(unsigned_transaction).unwrap()
            } else {
                unsigned_transaction.sign(&signer)
            };

            RpcBroadcastTxCommitRequest { signed_transaction }
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
        let unsigned_transaction = Transaction {
            signer_id: signer.account_id.clone(),
            public_key: signer.public_key.clone(),
            nonce,
            receiver_id: signer.account_id.clone(),
            block_hash,
            actions: vec![Action::DeployContract(DeployContractAction { code })],
        };

        let signed_transaction = if self.ledger {
            utils::sign_near_transaction_with_ledger(unsigned_transaction).unwrap()
        } else {
            unsigned_transaction.sign(&signer)
        };

        let request = RpcBroadcastTxCommitRequest { signed_transaction };
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
        let nonce: u64 = match response.kind {
            QueryResponseKind::AccessKey(k) => k.nonce + 1,
            _ => anyhow::bail!("Wrong response kind: {:?}", response.kind),
        };

        Ok((block_hash, nonce))
    }

    fn signer(&self) -> anyhow::Result<InMemorySigner> {
        if !self.ledger {
            std::env::var("NEAR_KEY_PATH")
                .ok()
                .as_ref()
                .or(self.signer_key_path.as_ref())
                .map(std::path::Path::new)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Path to the key file must be provided to use this functionality"
                    )
                })
                .and_then(utils::read_key_file)
        } else {
            // use ledger singer!
            utils::read_ledger_keypair()
        }
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
}
