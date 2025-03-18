use near_crypto::InMemorySigner;
use near_primitives::types::AccountId;

use crate::near::{
    broadcast,
    operations::ViewTransaction,
    response::{FromBytes, Response},
    Client,
};

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

    pub async fn view<T: FromBytes>(
        &self,
        method: &str,
        args: Option<Vec<u8>>,
    ) -> anyhow::Result<Response<T>> {
        self.client
            .view(ViewTransaction::new(&self.engine, method).args(args.unwrap_or_default()))
            .await
    }
}
