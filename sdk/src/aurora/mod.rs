use aurora_engine_types::account_id::AccountId;
use near_crypto::InMemorySigner;
use near_primitives::views::CallResult;
use operations::{CallOperation, ViewOperation};

use crate::near::{broadcast, response::Response, Client};

pub mod operations;

pub struct Aurora<B: broadcast::Broadcast> {
    signer: InMemorySigner,
    engine: AccountId,
    client: Client<B>,
}

impl<B: broadcast::Broadcast> Aurora<B> {
    pub fn new(client: Client<B>, signer: InMemorySigner, engine: AccountId) -> Self {
        //todo?: builder
        Self {
            signer,
            engine,
            client,
        }
    }

    pub async fn call(&self, op: impl CallOperation) -> anyhow::Result<B::Output> {
        self.client
            .call(op.into_call_transaction(&self.signer, &self.engine))
            .await
    }

    pub async fn view(&self, op: impl ViewOperation) -> anyhow::Result<CallResult> {
        self.client
            .view(op.into_view_transaction(&self.engine))
            .await
    }

    pub fn into_sync(self) -> Aurora<broadcast::Sync> {
        Aurora {
            signer: self.signer,
            engine: self.engine,
            client: self.client.into_sync(),
        }
    }

    pub fn into_async(self) -> Aurora<broadcast::Async> {
        Aurora {
            signer: self.signer,
            engine: self.engine,
            client: self.client.into_async(),
        }
    }
}
