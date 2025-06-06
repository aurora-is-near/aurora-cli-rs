use near_crypto::{PublicKey, Signer};
use near_jsonrpc_client::AsUrl;
use near_jsonrpc_client::methods::tx::RpcTransactionResponse;
use near_primitives::hash::CryptoHash;
use near_primitives::types::AccountId;
use near_primitives::views::TxExecutionStatus;

use super::Result;
use super::operations::{CallTransaction, Function, Transaction};
use super::query::{Query, ViewAccessKey, ViewAccessKeyList, ViewAccount, ViewFunction};
use super::rpc_client::RpcClient;

/// Represents a connection to a NEAR network, allowing interaction with contracts
/// and accounts. Provides methods for sending transactions and querying state.
#[derive(Clone)]
pub struct Client {
    client: RpcClient,
    signer: Signer,
}

impl Client {
    /// Creates a new `Workspace` instance connected to a specific NEAR RPC endpoint.
    ///
    /// # Arguments
    ///
    /// * `addr` - The URL of the NEAR RPC endpoint.
    /// * `api_key` - An optional API key for authenticated access to the RPC endpoint.
    /// * `signer` - The `Signer` used to sign transactions originated from this workspace.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `Workspace` instance or an `error::ApiKeyError` if the client creation fails.
    ///
    /// # Errors
    ///
    /// An error is returned if the URL is invalid or if the API key is invalid.
    pub fn new<U: AsUrl>(url: U, api_key: Option<String>, signer: Signer) -> Result<Self> {
        let client = RpcClient::new(url, api_key)?;
        Ok(Self { client, signer })
    }

    /// Initiates a function call transaction builder.
    ///
    /// This method allows you to construct and send a transaction that calls a method
    /// on a specified smart contract.
    ///
    /// # Arguments
    ///
    /// * `contract_id` - The account ID of the target contract.
    /// * `method` - The name of the method to call on the contract.
    ///
    /// # Returns
    ///
    /// A `CallTransaction` builder instance to configure and execute the call.
    pub fn call<M: Into<String>>(&self, contract_id: &AccountId, method: M) -> CallTransaction {
        CallTransaction::new(
            &self.client,
            contract_id.to_owned(),
            self.signer.clone(),
            method,
        )
    }

    /// Creates a batch transaction builder.
    ///
    /// This allows grouping multiple actions (like function calls, transfers, etc.)
    /// into a single transaction to be executed atomically.
    ///
    /// # Arguments
    ///
    /// * `contract_id` - The account ID for the batch transaction (receiver ID).
    ///
    /// # Returns
    ///
    /// A `Transaction` builder instance to add actions and execute the batch.
    pub fn batch(&self, contract_id: &AccountId) -> Transaction {
        Transaction::new(&self.client, self.signer.clone(), contract_id.to_owned())
    }

    /// Creates a query builder to view the list of access keys for a given account.
    ///
    /// # Arguments
    ///
    /// * `account_id` - The account ID whose access keys are to be viewed.
    ///
    /// # Returns
    ///
    /// A `Query<'_, ViewAccessKeyList>` instance to execute the query.
    pub fn view_access_keys(&self, account_id: &AccountId) -> Query<'_, ViewAccessKeyList> {
        Query::new(
            &self.client,
            ViewAccessKeyList {
                account_id: account_id.to_owned(),
            },
        )
    }

    /// Creates a query builder to view the details of a given account.
    ///
    /// # Arguments
    ///
    /// * `account_id` - The account ID whose details are to be viewed.
    ///
    /// # Returns
    ///
    /// A `Query<'_, ViewAccount>` instance to execute the query.
    pub fn view_account(&self, account_id: &AccountId) -> Query<'_, ViewAccount> {
        Query::new(
            &self.client,
            ViewAccount {
                account_id: account_id.to_owned(),
            },
        )
    }

    /// Creates a query builder to view the details of a specific access key for a given account.
    ///
    /// # Arguments
    ///
    /// * `account_id` - The account ID that owns the access key.
    /// * `public_key` - The public key of the access key to view.
    ///
    /// # Returns
    ///
    /// A `Query<'_, ViewAccessKey>` instance to execute the query.
    pub fn view_access_key(
        &self,
        account_id: &AccountId,
        public_key: &PublicKey,
    ) -> Query<'_, ViewAccessKey> {
        Query::new(
            &self.client,
            ViewAccessKey {
                account_id: account_id.clone(),
                public_key: public_key.clone(),
            },
        )
    }

    /// Creates a query builder to call a view-only function on a contract.
    ///
    /// View functions do not modify the state and do not require gas fees or signing.
    ///
    /// # Arguments
    ///
    /// * `contract_id` - The account ID of the contract to call.
    /// * `method` - The name of the view function to call.
    ///
    /// # Returns
    ///
    /// A `Query<'_, ViewFunction>` instance to configure arguments and execute the view call.
    pub fn view<M: Into<String>>(
        &self,
        contract_id: &AccountId,
        method: M,
    ) -> Query<'_, ViewFunction> {
        Query::new(
            &self.client,
            ViewFunction {
                account_id: contract_id.to_owned(),
                function: Function::new(method.into()),
            },
        )
    }

    pub async fn status(
        &self,
        hash: &CryptoHash,
        sender: Option<&AccountId>,
        wait_until: Option<TxExecutionStatus>,
    ) -> Result<RpcTransactionResponse> {
        let default_acc_id = &self.signer.get_account_id();
        self.client
            .status(hash, sender.unwrap_or(default_acc_id), wait_until)
            .await
    }

    pub const fn signer(&self) -> &Signer {
        &self.signer
    }
}
