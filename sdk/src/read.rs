use primitive_types::U256;

use crate::aurora::Aurora;
use crate::near::broadcast::Broadcast;
use crate::near::response::Response;
use crate::utils::hex_to_vec;

#[async_trait::async_trait(?Send)]
pub trait ReadClient {
    async fn get_chain_id(&self) -> anyhow::Result<Response<U256>>;
    async fn get_nonce(&self, address: &str) -> anyhow::Result<Response<U256>>;
    async fn get_owner(&self) -> anyhow::Result<Response<String>>;
    async fn get_version(&self) -> anyhow::Result<Response<String>>;
}

#[async_trait::async_trait(?Send)]
impl<B: Broadcast> ReadClient for Aurora<B> {
    async fn get_chain_id(&self) -> anyhow::Result<Response<U256>> {
        self.view::<U256>("get_chain_id", None).await
    }

    async fn get_nonce(&self, address: &str) -> anyhow::Result<Response<U256>> {
        let args = hex_to_vec(address)?;
        self.view::<U256>("get_nonce", Some(args)).await
    }

    async fn get_owner(&self) -> anyhow::Result<Response<String>> {
        self.view::<String>("get_owner", None).await
    }

    async fn get_version(&self) -> anyhow::Result<Response<String>> {
        self.view::<String>("get_version", None).await
    }
}
