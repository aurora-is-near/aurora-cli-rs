use aurora_engine::parameters::SubmitResult;
#[cfg(feature = "advanced")]
use aurora_engine::parameters::TransactionStatus;
use aurora_engine_types::{
    types::{Address, Wei},
    U256,
};
use borsh::BorshDeserialize;
#[cfg(feature = "advanced")]
use borsh::BorshSerialize;
use near_crypto::InMemorySigner;
use near_jsonrpc_client::{
    methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest, AsUrl, JsonRpcClient,
};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
#[cfg(not(feature = "advanced"))]
use near_primitives::views::FinalExecutionStatus;
use near_primitives::{
    hash::CryptoHash, transaction::SignedTransaction, types::AccountId, views,
    views::FinalExecutionOutcomeView,
};
#[cfg(not(feature = "advanced"))]
use std::str::FromStr;

use super::TransactionOutcome;
use crate::utils;

pub struct NearClient {
    client: JsonRpcClient,
    pub engine_account_id: AccountId,
    signer_key_path: Option<String>,
}

impl NearClient {
    pub fn new<U: AsUrl>(url: U, engine_account_id: &str, signer_key_path: Option<String>) -> Self {
        let client = JsonRpcClient::connect(url);
        Self {
            client,
            engine_account_id: engine_account_id.parse().unwrap(),
            signer_key_path,
        }
    }

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
        let args = aurora_engine::parameters::GetErc20FromNep141CallArgs {
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

    #[cfg(feature = "advanced")]
    pub async fn view_contract_call(
        &self,
        sender: Address,
        target: Address,
        amount: Wei,
        input: Vec<u8>,
    ) -> anyhow::Result<TransactionStatus> {
        let args = aurora_engine::parameters::ViewCallArgs {
            sender,
            address: target,
            amount: amount.to_bytes(),
            input,
        };
        let result = self.view_call("view", args.try_to_vec().unwrap()).await?;
        let status = TransactionStatus::try_from_slice(&result.result).unwrap();
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

    #[cfg(not(feature = "advanced"))]
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
        let signer = self.signer()?;
        let (block_hash, nonce) = self.get_nonce(&signer).await?;
        let request = RpcBroadcastTxCommitRequest {
            signed_transaction: SignedTransaction::call(
                nonce,
                signer.account_id.clone(),
                self.engine_account_id.parse().unwrap(),
                &signer,
                0,
                method_name.into(),
                args,
                300_000_000_000_000,
                block_hash,
            ),
        };
        let response = self.client.call(request).await?;

        Ok(response)
    }

    /// Creates new NEAR's account.
    #[cfg(not(feature = "advanced"))]
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
                    &signer,
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
                    &signer,
                    initial_balance,
                    "create_account".to_string(),
                    serde_json::json!({
                        "new_account_id": new_account_id,
                        "new_public_key": new_key_pair.public_key(),
                    })
                    .to_string()
                    .into_bytes(),
                    300_000_000_000_000,
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
    #[cfg(not(feature = "advanced"))]
    pub async fn deploy_contract(&self, code: Vec<u8>) -> anyhow::Result<String> {
        let signer = self.signer()?;
        let (block_hash, nonce) = self.get_nonce(&signer).await?;
        let request = RpcBroadcastTxCommitRequest {
            signed_transaction: SignedTransaction::from_actions(
                nonce,
                signer.account_id.clone(),
                signer.account_id.clone(),
                &signer,
                vec![near_primitives::transaction::Action::DeployContract(
                    near_primitives::transaction::DeployContractAction { code },
                )],
                block_hash,
            ),
        };
        let response = self.client.call(request).await?;

        match response.status {
            FinalExecutionStatus::NotStarted | FinalExecutionStatus::Started => {
                anyhow::bail!("Bad tx status")
            }
            FinalExecutionStatus::Failure(e) => anyhow::bail!(e),
            FinalExecutionStatus::SuccessValue(_) => {
                Ok("Smart contract has been deployed successfully".to_string())
            }
        }
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

    async fn get_nonce(&self, signer: &InMemorySigner) -> anyhow::Result<(CryptoHash, u64)> {
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
        self.signer_key_path
            .as_ref()
            .map(std::path::Path::new)
            .ok_or_else(|| {
                anyhow::anyhow!("Path to the signer key must be provided to use this functionality")
            })
            .and_then(utils::read_key_file)
    }

    #[cfg(not(feature = "advanced"))]
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
