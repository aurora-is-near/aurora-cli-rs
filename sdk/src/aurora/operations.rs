use aurora_engine_types::{
    account_id::AccountId,
    parameters::connector::{SetEthConnectorContractAccountArgs, WithdrawSerializeType},
};
use near_crypto::InMemorySigner;
use near_primitives::views::{CallResult, FinalExecutionOutcomeView};
use near_token::NearToken;

use crate::{near, utils::AsPrimitive};

pub trait CallOperation {
    type Output;

    fn into_call_transaction(
        self,
        signer: &near_crypto::InMemorySigner,
        engine: &AccountId,
    ) -> near::operations::CallTransaction;

    fn parse(result: FinalExecutionOutcomeView) -> anyhow::Result<Self::Output>;
}

pub trait ViewOperation {
    type Output;

    fn into_view_transaction(self, engine: &AccountId) -> near::operations::ViewTransaction;

    fn parse(result: CallResult) -> anyhow::Result<Self::Output>;
}

pub struct GetLatestReleaseHash {}

impl ViewOperation for GetLatestReleaseHash {
    type Output = String;

    fn into_view_transaction(self, engine: &AccountId) -> near::operations::ViewTransaction {
        near::operations::ViewTransaction::new(&engine.as_primitive(), "get_latest_release_hash")
    }

    fn parse(result: CallResult) -> anyhow::Result<Self::Output> {
        String::from_utf8(result.result).map_err(Into::into)
    }
}

pub struct SetEthConnectorContractAccount {
    pub deposit: NearToken,
    pub contract_account: AccountId,
}

impl CallOperation for SetEthConnectorContractAccount {
    type Output = ();

    fn into_call_transaction(
        self,
        signer: &InMemorySigner,
        engine: &AccountId,
    ) -> near::operations::CallTransaction {
        near::operations::CallTransaction::new(
            signer,
            &engine.as_primitive(),
            "set_eth_connector_contract_account",
        )
        .args_borsh(&SetEthConnectorContractAccountArgs {
            account: self.contract_account,
            withdraw_serialize_type: WithdrawSerializeType::Borsh,
        })
    }

    fn parse(_: FinalExecutionOutcomeView) -> anyhow::Result<Self::Output> {
        Ok(())
    }
}
