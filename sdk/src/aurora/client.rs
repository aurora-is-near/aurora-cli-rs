use near_primitives::{errors::TxExecutionError, hash::CryptoHash};
use near_token::NearToken;

use crate::near;

use super::{ContractMethod, ContractMethodResponse};

pub struct Client {
    pub(crate) near: near::client::Client,
}

impl Client {
    pub const fn new(near: near::client::Client) -> Self {
        Self { near }
    }

    pub async fn call<M>(
        &self,
        account_id: &near_primitives::types::AccountId,
        method: M,
    ) -> Result<M::Response, super::error::Error>
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
            ) => Err(M::parse_error(action_error)?.into()), // catching silo errors
            _ => Err(super::error::Error::ExecutionNotStarted),
        }
    }

    pub async fn call_async<M>(
        &self,
        account_id: &near_primitives::types::AccountId,
        method: M,
    ) -> Result<CryptoHash, super::error::Error>
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
}
