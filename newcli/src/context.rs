use aurora_sdk_rs::aurora::client::Client;
use aurora_sdk_rs::aurora::error::Error;
use aurora_sdk_rs::aurora::{ContractMethod, ContractMethodResponse};
use aurora_sdk_rs::near;
use aurora_sdk_rs::near::jsonrpc::errors::{JsonRpcError, JsonRpcServerError};

use crate::cli::Cli;

pub struct Context {
    pub cli: Cli,
    pub client: Client,
}

impl Context {
    pub async fn view<M>(&self, method: M) -> Result<M::Response, Error>
    where
        M: ContractMethod,
        M::Response: ContractMethodResponse,
    {
        let method_name = method.method_name();
        let params = method.params()?;
        let view_query = self
            .client
            .near
            .view(&self.cli.engine, method_name)
            .args(params);
        let view_result = if let Some(height) = self.cli.block_height {
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
}
