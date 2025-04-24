use near_crypto::{InMemorySigner, PublicKey};
use near_primitives::types::AccountId;

use super::client::Client;
use super::operations::{CallTransaction, Function, Transaction};
use super::query::{Query, ViewAccessKey, ViewAccessKeyList, ViewAccount, ViewFunction};
use super::Result;

pub struct Workspace {
    client: Client,
    signer: InMemorySigner,
}

impl Workspace {
    pub fn new(addr: &str, api_key: Option<String>, signer: InMemorySigner) -> Result<Self> {
        let client = Client::new(addr, api_key)?;
        Ok(Self { client, signer })
    }

    pub fn call(&self, contract_id: &AccountId, method: &str) -> CallTransaction {
        CallTransaction::new(
            &self.client,
            contract_id.to_owned(),
            self.signer.clone(),
            method,
        )
    }

    pub fn batch(&self, contract_id: &AccountId) -> Transaction {
        Transaction::new(&self.client, self.signer.clone(), contract_id.to_owned())
    }

    pub fn view_access_keys(&self, account_id: &AccountId) -> Query<'_, ViewAccessKeyList> {
        Query::new(
            &self.client,
            ViewAccessKeyList {
                account_id: account_id.to_owned(),
            },
        )
    }

    pub fn view_account(&self, account_id: &AccountId) -> Query<'_, ViewAccount> {
        Query::new(
            &self.client,
            ViewAccount {
                account_id: account_id.to_owned(),
            },
        )
    }

    pub fn view_access_key(&self, id: &AccountId, pk: &PublicKey) -> Query<'_, ViewAccessKey> {
        Query::new(
            &self.client,
            ViewAccessKey {
                account_id: id.clone(),
                public_key: pk.clone(),
            },
        )
    }

    pub fn view(&self, contract_id: &AccountId, method: &str) -> Query<'_, ViewFunction> {
        Query::new(
            &self.client,
            ViewFunction {
                account_id: contract_id.to_owned(),
                function: Function::new(method),
            },
        )
    }
}
