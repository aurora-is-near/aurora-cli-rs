use aurora_engine_types::types::NearGas;
use near_crypto::PublicKey;
use near_primitives::account::AccessKey;
use near_primitives::action::{
    AddKeyAction, CreateAccountAction, DeleteAccountAction, DeleteKeyAction, DeployContractAction,
    FunctionCallAction, StakeAction, TransferAction,
};
use near_primitives::hash::CryptoHash;
use near_primitives::transaction::Action;
use near_primitives::types::AccountId;
use near_primitives::views::FinalExecutionOutcomeView;
use near_token::NearToken;

const ONE_TERA_GAS: u64 = 10u64.pow(12);
const MAX_GAS: NearGas = NearGas::new(300 * ONE_TERA_GAS);

pub(crate) const DEFAULT_CALL_FN_GAS: NearGas = NearGas::new(10 * ONE_TERA_GAS);
pub(crate) const DEFAULT_CALL_DEPOSIT: NearToken = NearToken::from_near(0);
pub(crate) const DEFAULT_PRIORITY_FEE: u64 = 0;

use super::client::Client;
use super::Result;

pub struct Function {
    pub(crate) name: String,
    pub(crate) args: Result<Vec<u8>>,
    pub(crate) deposit: NearToken,
    pub(crate) gas: NearGas,
}

impl Function {
    /// Initialize a new instance of [`Function`], tied to a specific function on a
    /// contract that lives directly on a contract we've specified in [`Transaction`].
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            args: Ok(vec![]),
            deposit: DEFAULT_CALL_DEPOSIT,
            gas: DEFAULT_CALL_FN_GAS,
        }
    }

    /// Provide the arguments for the call. These args are serialized bytes from either
    /// a JSON or Borsh serializable set of arguments. To use the more specific versions
    /// with better quality of life, use `args_json` or `args_borsh`.
    pub fn args(mut self, args: Vec<u8>) -> Self {
        if self.args.is_err() {
            return self;
        }
        self.args = Ok(args);
        self
    }

    /// Similar to `args`, specify an argument that is JSON serializable and can be
    /// accepted by the equivalent contract. Recommend to use something like
    /// `serde_json::json!` macro to easily serialize the arguments.
    pub fn args_json<U: serde::Serialize>(mut self, args: U) -> Self {
        self.args = serde_json::to_vec(&args).map_err(Into::into);
        self
    }

    /// Similar to `args`, specify an argument that is borsh serializable and can be
    /// accepted by the equivalent contract.
    pub fn args_borsh<U: borsh::BorshSerialize>(mut self, args: U) -> Self {
        self.args = borsh::to_vec(&args).map_err(Into::into);
        self
    }

    /// Specify the amount of tokens to be deposited where `deposit` is the amount of
    /// tokens in yocto near.
    pub fn deposit(mut self, deposit: NearToken) -> Self {
        self.deposit = deposit;
        self
    }

    /// Specify the amount of gas to be used.
    pub fn gas(mut self, gas: NearGas) -> Self {
        self.gas = gas;
        self
    }

    /// Use the maximum amount of gas possible to perform this function call into the contract.
    pub fn max_gas(self) -> Self {
        self.gas(MAX_GAS)
    }
}

pub struct Transaction<'a> {
    client: &'a Client,
    signer: near_crypto::InMemorySigner,
    receiver_id: AccountId,
    actions: Result<Vec<Action>>,
}

impl<'a> Transaction<'a> {
    pub(crate) fn new(
        client: &'a Client,
        signer: near_crypto::InMemorySigner,
        receiver_id: AccountId,
    ) -> Self {
        Self {
            client,
            signer,
            receiver_id,
            actions: Ok(vec![]),
        }
    }

    /// Adds a key to the `receiver_id`'s account, where the public key can be used
    /// later to delete the same key.
    pub fn add_key(mut self, pk: PublicKey, ak: AccessKey) -> Self {
        if let Ok(actions) = &mut self.actions {
            actions.push(
                AddKeyAction {
                    public_key: pk,
                    access_key: ak,
                }
                .into(),
            );
        }

        self
    }

    /// Call into the `receiver_id`'s contract with the specific function arguments.
    pub fn call(mut self, function: Function) -> Self {
        let args = match function.args {
            Ok(args) => args,
            Err(err) => {
                self.actions = Err(err);
                return self;
            }
        };

        if let Ok(actions) = &mut self.actions {
            actions.push(Action::FunctionCall(Box::new(FunctionCallAction {
                method_name: function.name.to_string(),
                args,
                deposit: function.deposit.as_yoctonear(),
                gas: function.gas.as_u64(),
            })));
        }

        self
    }

    /// Create a new account with the account id being `receiver_id`.
    pub fn create_account(mut self) -> Self {
        if let Ok(actions) = &mut self.actions {
            actions.push(CreateAccountAction {}.into());
        }
        self
    }

    /// Deletes the `receiver_id`'s account. The beneficiary specified by
    /// `beneficiary_id` will receive the funds of the account deleted.
    pub fn delete_account(mut self, beneficiary_id: &AccountId) -> Self {
        if let Ok(actions) = &mut self.actions {
            actions.push(
                DeleteAccountAction {
                    beneficiary_id: beneficiary_id.clone(),
                }
                .into(),
            );
        }
        self
    }

    /// Deletes a key from the `receiver_id`'s account, where the public key is
    /// associated with the access key to be deleted.
    pub fn delete_key(mut self, pk: PublicKey) -> Self {
        if let Ok(actions) = &mut self.actions {
            actions.push(DeleteKeyAction { public_key: pk }.into());
        }
        self
    }

    /// Deploy contract code or WASM bytes to the `receiver_id`'s account.
    pub fn deploy(mut self, code: &[u8]) -> Self {
        if let Ok(actions) = &mut self.actions {
            actions.push(DeployContractAction { code: code.into() }.into());
        }
        self
    }

    /// An action which stakes the signer's tokens and setups a validator public key.
    pub fn stake(mut self, stake: NearToken, pk: PublicKey) -> Self {
        if let Ok(actions) = &mut self.actions {
            actions.push(
                StakeAction {
                    stake: stake.as_yoctonear(),
                    public_key: pk,
                }
                .into(),
            );
        }
        self
    }

    /// Transfer `deposit` amount from `signer`'s account into `receiver_id`'s account.
    pub fn transfer(mut self, deposit: NearToken) -> Self {
        if let Ok(actions) = &mut self.actions {
            actions.push(
                TransferAction {
                    deposit: deposit.as_yoctonear(),
                }
                .into(),
            );
        }
        self
    }

    pub async fn transact(self) -> Result<FinalExecutionOutcomeView> {
        self.client
            .send_batch_tx(&self.signer, &self.receiver_id, self.actions?)
            .await
    }

    pub async fn transact_async(self) -> Result<CryptoHash> {
        self.client
            .send_batch_tx_async(&self.signer, &self.receiver_id, self.actions?)
            .await
    }
}

/// Similar to a [`Transaction`], but more specific to making a call into a contract.
/// Note, only one call can be made per `CallTransaction`.
pub struct CallTransaction<'a> {
    client: &'a Client,
    signer: near_crypto::InMemorySigner,
    contract_id: AccountId,
    function: Function,
}

impl<'a> CallTransaction<'a> {
    pub(crate) fn new(
        client: &'a Client,
        contract_id: AccountId,
        signer: near_crypto::InMemorySigner,
        function: &str,
    ) -> Self {
        Self {
            client,
            signer,
            contract_id,
            function: Function::new(function),
        }
    }

    /// Provide the arguments for the call. These args are serialized bytes from either
    /// a JSON or Borsh serializable set of arguments. To use the more specific versions
    /// with better quality of life, use `args_json` or `args_borsh`.
    pub fn args(mut self, args: Vec<u8>) -> Self {
        self.function = self.function.args(args);
        self
    }

    /// Similar to `args`, specify an argument that is JSON serializable and can be
    /// accepted by the equivalent contract. Recommend to use something like
    /// `serde_json::json!` macro to easily serialize the arguments.
    pub fn args_json<U: serde::Serialize>(mut self, args: U) -> Self {
        self.function = self.function.args_json(args);
        self
    }

    /// Similar to `args`, specify an argument that is borsh serializable and can be
    /// accepted by the equivalent contract.
    pub fn args_borsh<U: borsh::BorshSerialize>(mut self, args: U) -> Self {
        self.function = self.function.args_borsh(args);
        self
    }

    /// Specify the amount of tokens to be deposited where `deposit` is the amount of
    /// tokens in yocto near.
    pub fn deposit(mut self, deposit: NearToken) -> Self {
        self.function = self.function.deposit(deposit);
        self
    }

    /// Specify the amount of gas to be used where `gas` is the amount of gas in yocto near.
    pub fn gas(mut self, gas: NearGas) -> Self {
        self.function = self.function.gas(gas);
        self
    }

    /// Use the maximum amount of gas possible to perform this transaction.
    pub fn max_gas(self) -> Self {
        self.gas(MAX_GAS)
    }

    pub fn signer(mut self, signer: near_crypto::InMemorySigner) -> Self {
        self.signer = signer;
        self
    }

    pub async fn transact(self) -> Result<FinalExecutionOutcomeView> {
        self.client
            .call(
                &self.signer,
                &self.contract_id,
                self.function.name.to_string(),
                self.function.args?,
                self.function.gas,
                self.function.deposit,
            )
            .await
    }

    pub async fn transact_async(self) -> Result<CryptoHash> {
        self.client
            .send_batch_tx_async(
                &self.signer,
                &self.contract_id,
                vec![FunctionCallAction {
                    args: self.function.args?,
                    method_name: self.function.name,
                    gas: self.function.gas.as_u64(),
                    deposit: self.function.deposit.as_yoctonear(),
                }
                .into()],
            )
            .await
    }
}
