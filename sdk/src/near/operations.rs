use near_crypto::{InMemorySigner, PublicKey};
use near_gas::NearGas;
use near_primitives::{
    account::AccessKey,
    action::{
        Action, AddKeyAction, CreateAccountAction, DeleteAccountAction, DeleteKeyAction,
        DeployContractAction, FunctionCallAction,
    },
    types::AccountId,
};
use near_token::NearToken;

const DEFAULT_CALL_DEPOSIT: NearToken = NearToken::from_near(0);
const DEFAULT_CALL_FN_GAS: NearGas = NearGas::from_tgas(300);

pub struct Function {
    pub(crate) method: String,
    pub(crate) args: anyhow::Result<Vec<u8>>,
    pub(crate) deposit: NearToken,
    pub(crate) gas: NearGas,
}

impl Function {
    pub fn new(method: &str) -> Self {
        Self {
            method: method.to_string(),
            args: Ok(vec![]),
            deposit: DEFAULT_CALL_DEPOSIT,
            gas: DEFAULT_CALL_FN_GAS,
        }
    }

    pub fn args(mut self, args: Vec<u8>) -> Self {
        self.args = Ok(args);
        self
    }

    pub fn args_json(mut self, args: &impl serde::Serialize) -> Self {
        self.args = serde_json::to_vec(args).map_err(anyhow::Error::from);
        self
    }

    pub fn args_borsh(mut self, args: &impl borsh::ser::BorshSerialize) -> Self {
        self.args = borsh::to_vec(&args).map_err(anyhow::Error::from);
        self
    }

    pub fn deposit(mut self, deposit: NearToken) -> Self {
        self.deposit = deposit;
        self
    }

    pub fn gas(mut self, gas: NearGas) -> Self {
        self.gas = gas;
        self
    }
}

pub struct Transaction {
    pub(crate) signer: InMemorySigner,
    pub(crate) receiver_id: AccountId,
    pub(crate) actions: anyhow::Result<Vec<Action>>,
}

impl Transaction {
    pub(crate) fn new(signer: InMemorySigner, receiver_id: AccountId) -> Self {
        Self {
            signer,
            receiver_id,
            actions: Ok(vec![]),
        }
    }

    pub fn action(mut self, action: Action) -> Self {
        self.actions = self.actions.and_then(|mut actions| {
            actions.push(action);
            Ok(actions)
        });
        self
    }

    pub fn call(mut self, method: String, args: Function) -> Self {
        self.actions = self.actions.and_then(|mut actions| {
            actions.push(
                FunctionCallAction {
                    method_name: method,
                    args: args.args?,
                    gas: args.gas.as_gas(),
                    deposit: args.deposit.as_yoctonear(),
                }
                .into(),
            );
            Ok(actions)
        });
        self
    }

    pub fn add_key(mut self, pk: PublicKey, ak: AccessKey) -> Self {
        self.actions = self.actions.and_then(|mut actions| {
            actions.push(
                AddKeyAction {
                    public_key: pk,
                    access_key: ak,
                }
                .into(),
            );
            Ok(actions)
        });
        self
    }

    pub fn create_account(mut self) -> Self {
        self.actions = self.actions.and_then(|mut actions| {
            actions.push(CreateAccountAction {}.into());
            Ok(actions)
        });
        self
    }

    pub fn delete_account(mut self, beneficiary_id: &AccountId) -> Self {
        self.actions = self.actions.and_then(|mut actions| {
            actions.push(
                DeleteAccountAction {
                    beneficiary_id: beneficiary_id.clone(),
                }
                .into(),
            );
            Ok(actions)
        });
        self
    }

    pub fn delete_key(mut self, public_key: PublicKey) -> Self {
        self.actions = self.actions.and_then(|mut actions| {
            actions.push(DeleteKeyAction { public_key }.into());
            Ok(actions)
        });
        self
    }

    pub fn deploy(mut self, code: Vec<u8>) -> Self {
        self.actions = self.actions.and_then(|mut actions| {
            actions.push(DeployContractAction { code }.into());
            Ok(actions)
        });
        self
    }
}

impl From<CallTransaction> for Transaction {
    fn from(call_tx: CallTransaction) -> Self {
        Self::new(call_tx.signer, call_tx.contract_id)
            .call(call_tx.function.method.clone(), call_tx.function)
    }
}

pub struct ViewTransaction {
    pub(crate) contract_id: AccountId,
    pub(crate) function: Function,
}

impl ViewTransaction {
    pub fn new(contract_id: &AccountId, method: &str) -> Self {
        Self {
            contract_id: contract_id.clone(),
            function: Function::new(method),
        }
    }

    pub fn args(mut self, args: Vec<u8>) -> Self {
        self.function = self.function.args(args);
        self
    }

    pub fn args_json(mut self, args: &impl serde::Serialize) -> Self {
        self.function = self.function.args_json(args);
        self
    }

    pub fn args_borsh(mut self, args: &impl borsh::ser::BorshSerialize) -> Self {
        self.function = self.function.args_borsh(args);
        self
    }
}

pub struct CallTransaction {
    pub(crate) signer: InMemorySigner,
    pub(crate) contract_id: AccountId,
    pub(crate) function: Function,
}

impl CallTransaction {
    pub fn new(signer: &InMemorySigner, contract_id: &AccountId, method: &str) -> Self {
        Self {
            signer: signer.clone(),
            contract_id: contract_id.clone(),
            function: Function::new(method),
        }
    }

    pub fn args(mut self, args: Vec<u8>) -> Self {
        self.function = self.function.args(args);
        self
    }

    pub fn args_json(mut self, args: &impl serde::Serialize) -> Self {
        self.function = self.function.args_json(args);
        self
    }

    pub fn args_borsh(mut self, args: &impl borsh::ser::BorshSerialize) -> Self {
        self.function = self.function.args_borsh(args);
        self
    }

    pub fn deposit(mut self, deposit: NearToken) -> Self {
        self.function = self.function.deposit(deposit);
        self
    }

    pub fn gas(mut self, gas: NearGas) -> Self {
        self.function = self.function.gas(gas);
        self
    }
}
