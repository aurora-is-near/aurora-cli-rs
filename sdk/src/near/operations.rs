use near_crypto::{InMemorySigner, KeyFile, PublicKey, Signer};
use near_gas::NearGas;
use near_primitives::{
    account::AccessKey,
    action::{
        AddKeyAction, CreateAccountAction, DeleteAccountAction, DeleteKeyAction,
        DeployContractAction, DeployGlobalContractAction, FunctionCallAction, StakeAction,
        TransferAction, UseGlobalContractAction,
    },
    hash::CryptoHash,
    transaction::Action,
    types::AccountId,
    views::FinalExecutionOutcomeView,
};
use near_token::NearToken;

use crate::near::types::{GlobalContractDeployMode, GlobalContractIdentifier};

pub const MAX_GAS: NearGas = NearGas::from_tgas(300);

pub(crate) const DEFAULT_CALL_FN_GAS: NearGas = NearGas::from_tgas(10);
pub(crate) const DEFAULT_CALL_DEPOSIT: NearToken = NearToken::from_near(0);
pub(crate) const DEFAULT_PRIORITY_FEE: u64 = 0;

use super::Result;
use super::rpc_client::RpcClient;

pub struct Function {
    pub(crate) args: Vec<u8>,
    deposit: NearToken,
    gas: NearGas,
    pub(crate) name: String,
}

impl Function {
    /// Initialize a new instance of [`Function`], tied to a specific function on a
    /// contract that lives directly on a contract we've specified in [`Transaction`].
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            args: vec![],
            deposit: DEFAULT_CALL_DEPOSIT,
            gas: DEFAULT_CALL_FN_GAS,
            name: name.into(),
        }
    }

    /// Provide the arguments for the call. These args are serialized bytes from either
    /// a JSON or Borsh serializable set of arguments. To use the more specific versions
    /// with better quality of life, use `args_json` or `args_borsh`.
    #[must_use]
    pub fn args(mut self, args: Vec<u8>) -> Self {
        self.args = args;
        self
    }

    /// Similar to `args`, specify an argument that is JSON serializable and can be
    /// accepted by the equivalent contract. Recommend to use something like
    /// `serde_json::json!` macro to easily serialize the arguments.
    ///
    /// # Errors
    ///
    /// The method will return an error if the serialization of the arguments fails.
    pub fn args_json<U: serde::Serialize>(mut self, args: U) -> Result<Self> {
        self.args = serde_json::to_vec(&args)?;
        Ok(self)
    }

    /// Similar to `args`, specify an argument that is borsh serializable and can be
    /// accepted by the equivalent contract.
    /// # Errors
    ///
    /// The method will return an error if the serialization of the arguments fails.
    pub fn args_borsh<U: borsh::BorshSerialize>(mut self, args: U) -> Result<Self> {
        self.args = borsh::to_vec(&args)?;
        Ok(self)
    }

    /// Specify the amount of tokens to be deposited where `deposit` is the amount of
    /// tokens in yocto near.
    #[must_use]
    pub const fn deposit(mut self, deposit: NearToken) -> Self {
        self.deposit = deposit;
        self
    }

    /// Specify the amount of gas to be used.
    #[must_use]
    pub const fn gas(mut self, gas: u64) -> Self {
        self.gas = NearGas::from_gas(gas);
        self
    }

    /// Use the maximum amount of gas possible to perform this function call into the contract.
    #[must_use]
    pub const fn max_gas(self) -> Self {
        self.gas(MAX_GAS.as_gas())
    }
}

pub struct Transaction<'a> {
    client: &'a RpcClient,
    signer: Signer,
    receiver_id: AccountId,
    actions: Vec<Action>,
    priority_fee: u64,
}

impl<'a> Transaction<'a> {
    pub(crate) const fn new(client: &'a RpcClient, signer: Signer, receiver_id: AccountId) -> Self {
        Self {
            client,
            signer,
            receiver_id,
            actions: vec![],
            priority_fee: DEFAULT_PRIORITY_FEE,
        }
    }

    /// Adds a key to the `receiver_id`'s account, where the public key can be used
    /// later to delete the same key.
    #[must_use]
    pub fn add_key(mut self, public_key: PublicKey, access_key: AccessKey) -> Self {
        self.actions.push(
            AddKeyAction {
                public_key,
                access_key,
            }
            .into(),
        );

        self
    }

    /// Call into the `receiver_id`'s contract with the specific function arguments.
    #[must_use]
    pub fn call(mut self, function: Function) -> Self {
        self.actions
            .push(Action::FunctionCall(Box::new(FunctionCallAction {
                method_name: function.name.to_string(),
                args: function.args,
                deposit: function.deposit.as_yoctonear(),
                gas: function.gas.as_gas(),
            })));

        self
    }

    /// Create a new account with the account id being `receiver_id`.
    #[must_use]
    pub fn create_account(mut self) -> Self {
        self.actions.push(CreateAccountAction {}.into());
        self
    }

    /// Deletes the `receiver_id`'s account. The beneficiary specified by
    /// `beneficiary_id` will receive the funds of the account deleted.
    #[must_use]
    pub fn delete_account(mut self, beneficiary_id: &AccountId) -> Self {
        self.actions.push(
            DeleteAccountAction {
                beneficiary_id: beneficiary_id.clone(),
            }
            .into(),
        );
        self
    }

    /// Deletes a key from the `receiver_id`'s account, where the public key is
    /// associated with the access key to be deleted.
    #[must_use]
    pub fn delete_key(mut self, pk: PublicKey) -> Self {
        self.actions.push(DeleteKeyAction { public_key: pk }.into());
        self
    }

    /// Deploy contract code of WASM bytes to the `receiver_id`'s account.
    #[must_use]
    pub fn deploy(mut self, code: &[u8]) -> Self {
        self.actions
            .push(DeployContractAction { code: code.into() }.into());
        self
    }

    /// Deploy a global contract code of WASM bytes to the `receiver_id`'s account
    /// with the given `deploy_mode`.
    #[must_use]
    pub fn deploy_global_contract(
        mut self,
        code: &[u8],
        deploy_mode: GlobalContractDeployMode,
    ) -> Self {
        self.actions.push(
            DeployGlobalContractAction {
                code: code.into(),
                deploy_mode: deploy_mode.into(),
            }
            .into(),
        );
        self
    }

    /// Use a global contract for the `receiver_id`'s account.
    #[must_use]
    pub fn use_global_contract(mut self, contract_identifier: GlobalContractIdentifier) -> Self {
        self.actions.push(Action::UseGlobalContract(
            UseGlobalContractAction {
                contract_identifier: contract_identifier.into(),
            }
            .into(),
        ));
        self
    }

    /// An action which stakes the signer's tokens and sets a validator public key.
    #[must_use]
    pub fn stake(mut self, stake: NearToken, public_key: PublicKey) -> Self {
        self.actions.push(
            StakeAction {
                stake: stake.as_yoctonear(),
                public_key,
            }
            .into(),
        );
        self
    }

    /// Specify the priority fee for the transaction.
    #[must_use]
    pub const fn priority_fee(mut self, priority_fee: u64) -> Self {
        self.priority_fee = priority_fee;
        self
    }

    /// Transfer `deposit` amount from `signer`'s account into `receiver_id`'s account.
    #[must_use]
    pub fn transfer(mut self, deposit: NearToken) -> Self {
        self.actions.push(
            TransferAction {
                deposit: deposit.as_yoctonear(),
            }
            .into(),
        );
        self
    }

    pub fn signer_id(mut self, id: &AccountId) -> Self {
        let key_file: KeyFile = self.signer.into(); // a hack to access the secret key
        self.signer = InMemorySigner::from_secret_key(id.clone(), key_file.secret_key);
        self
    }

    /// Executes the transaction, sending all queued actions to the network.
    ///
    /// Waits for the transaction to be finalized and returns the final outcome.
    pub async fn transact(self) -> Result<FinalExecutionOutcomeView> {
        self.client
            .send_batch_tx(
                &self.signer,
                &self.receiver_id,
                self.actions,
                self.priority_fee,
            )
            .await
    }

    /// Executes the transaction asynchronously.
    ///
    /// Sends all queued actions to the network and returns immediately with the transaction hash.
    /// It does not wait for the transaction to be finalized.
    pub async fn transact_async(self) -> Result<CryptoHash> {
        self.client
            .send_batch_tx_async(
                &self.signer,
                &self.receiver_id,
                self.actions,
                self.priority_fee,
            )
            .await
    }
}

/// Similar to a [`Transaction`], but more specific to making a call into a contract.
/// Note, only one call can be made per `CallTransaction`.
pub struct CallTransaction<'a> {
    client: &'a RpcClient,
    signer: Signer,
    contract_id: AccountId,
    function: Function,
    priority_fee: u64,
}

impl<'a> CallTransaction<'a> {
    pub(crate) fn new<F: Into<String>>(
        client: &'a RpcClient,
        contract_id: AccountId,
        signer: Signer,
        function: F,
    ) -> Self {
        Self {
            client,
            signer,
            contract_id,
            function: Function::new(function),
            priority_fee: DEFAULT_PRIORITY_FEE,
        }
    }

    /// Provide the arguments for the call. These args are serialized bytes from either
    /// a JSON or Borsh serializable set of arguments. To use the more specific versions
    /// with better quality of life, use `args_json` or `args_borsh`.
    #[must_use]
    pub fn args(mut self, args: Vec<u8>) -> Self {
        self.function = self.function.args(args);
        self
    }

    /// Similar to `args`, specify an argument that is JSON serializable and can be
    /// accepted by the equivalent contract. Recommend to use something like
    /// `serde_json::json!` macro to easily serialize the arguments.
    pub fn args_json<U: serde::Serialize>(mut self, args: U) -> Result<Self> {
        self.function = self.function.args_json(args)?;
        Ok(self)
    }

    /// Similar to `args`, specify an argument that is borsh serializable and can be
    /// accepted by the equivalent contract.
    pub fn args_borsh<U: borsh::BorshSerialize>(mut self, args: U) -> Result<Self> {
        self.function = self.function.args_borsh(args)?;
        Ok(self)
    }

    /// Specify the amount of tokens to be deposited where `deposit` is the amount of
    /// tokens in yocto near.
    #[must_use]
    pub fn deposit(mut self, deposit: NearToken) -> Self {
        self.function = self.function.deposit(deposit);
        self
    }

    /// Specify the amount of gas to be used where `gas` is the amount of gas in yocto near.
    #[must_use]
    pub fn gas(mut self, gas: u64) -> Self {
        self.function = self.function.gas(gas);
        self
    }

    /// Use the maximum amount of gas possible to perform this transaction.
    #[must_use]
    pub fn max_gas(self) -> Self {
        self.gas(MAX_GAS.as_gas())
    }

    /// Specify the signer of the transaction.
    #[must_use]
    pub fn signer(mut self, signer: Signer) -> Self {
        self.signer = signer;
        self
    }

    /// Specify the priority fee for the transaction.
    #[must_use]
    pub const fn priority_fee(mut self, priority_fee: u64) -> Self {
        self.priority_fee = priority_fee;
        self
    }

    pub fn signer_id(mut self, id: &AccountId) -> Self {
        let key_file: KeyFile = self.signer.into(); // a hack to access the secret key
        self.signer = InMemorySigner::from_secret_key(id.clone(), key_file.secret_key);
        self
    }

    /// Executes the function call transaction.
    ///
    /// Waits for the transaction to be finalized and returns the final outcome.
    pub async fn transact(self) -> Result<FinalExecutionOutcomeView> {
        self.client
            .call(
                &self.signer,
                &self.contract_id,
                self.function.name.to_string(),
                self.function.args,
                self.function.gas.as_gas(),
                self.function.deposit,
                self.priority_fee,
            )
            .await
    }

    /// Executes the function call transaction asynchronously.
    ///
    /// Sends the transaction to the network and returns immediately with the transaction hash.
    /// It does not wait for the transaction to be finalized.
    pub async fn transact_async(self) -> Result<CryptoHash> {
        self.client
            .send_batch_tx_async(
                &self.signer,
                &self.contract_id,
                vec![
                    FunctionCallAction {
                        args: self.function.args,
                        method_name: self.function.name,
                        gas: self.function.gas.as_gas(),
                        deposit: self.function.deposit.as_yoctonear(),
                    }
                    .into(),
                ],
                self.priority_fee,
            )
            .await
    }
}
