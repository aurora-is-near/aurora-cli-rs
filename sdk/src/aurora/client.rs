use near_jsonrpc_client::errors::{JsonRpcError, JsonRpcServerError};
use near_primitives::{errors::TxExecutionError, hash::CryptoHash, types::AccountId};
use near_token::NearToken;

use crate::near;

use super::{ContractMethod, ContractMethodResponse, error::Error};

#[derive(Clone)]
pub struct Client {
    pub near: near::client::Client,
}

impl Client {
    pub const fn new(near: near::client::Client) -> Self {
        Self { near }
    }

    pub async fn call<M>(&self, account_id: &AccountId, method: M) -> Result<M::Response, Error>
    where
        M: ContractMethod,
        M::Response: ContractMethodResponse,
    {
        let method_name = method.method_name();
        let params = method.params()?;

        let outcome = self
            .near
            .call(account_id, method_name)
            .args(params)
            .deposit(NearToken::from_yoctonear(method.deposit()))
            .transact()
            .await?;

        match outcome.status {
            near_primitives::views::FinalExecutionStatus::SuccessValue(value) => {
                M::parse_response(value)
            }

            near_primitives::views::FinalExecutionStatus::Failure(
                TxExecutionError::ActionError(action_error),
            ) => Err(M::parse_error(action_error.into())?), // catching silo errors
            _ => Err(Error::ExecutionNotStarted),
        }
    }

    pub async fn call_async<M>(
        &self,
        account_id: &AccountId,
        method: M,
    ) -> Result<CryptoHash, Error>
    where
        M: ContractMethod,
        M::Response: ContractMethodResponse,
    {
        let method_name = method.method_name();
        let params = method.params()?;

        self.near
            .call(account_id, method_name)
            .args(params)
            .deposit(NearToken::from_yoctonear(method.deposit()))
            .transact_async()
            .await
            .map_err(Into::into)
    }

    pub async fn view<M>(&self, account_id: &AccountId, method: M) -> Result<M::Response, Error>
    where
        M: ContractMethod,
        M::Response: ContractMethodResponse,
    {
        self.view_in_block(account_id, method, None).await
    }

    pub async fn view_in_block<M>(
        &self,
        account_id: &AccountId,
        method: M,
        block_height: Option<u64>,
    ) -> Result<M::Response, Error>
    where
        M: ContractMethod,
        M::Response: ContractMethodResponse,
    {
        let method_name = method.method_name();
        let params = method.params()?;

        let view_query = self.near.view(account_id, method_name).args(params);
        let view_result = if let Some(height) = block_height {
            view_query.block_height(height)
        } else {
            view_query
        }
        .await;

        match view_result {
            Ok(call_result) => Ok(M::parse_response(call_result.result)?),

            Err(near::error::Error::RpcQueryError(JsonRpcError::ServerError(
                JsonRpcServerError::HandlerError(query_error),
            ))) => Err(M::parse_error(query_error.into())?),
            Err(e) => Err(e.into()),
        }
    }

    pub const fn near(&self) -> &near::client::Client {
        &self.near
    }
}
