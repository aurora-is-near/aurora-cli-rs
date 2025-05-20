use std::{io, sync::OnceLock};

use error::EngineError;
use near_jsonrpc_client::methods::query::RpcQueryError;
use near_primitives::errors::{ActionError, ActionErrorKind, FunctionCallError};
use regex::Regex;

pub mod client;
pub mod contract;
pub mod error;

pub enum MethodType {
    View,
    Call,
}

#[derive(Debug, thiserror::Error)]
pub enum MethodExecutionError {
    #[error("ActionError: {0}")]
    ActionError(#[from] ActionError),
    #[error("QueryError: {0}")]
    QueryError(#[from] RpcQueryError),
}

pub trait ContractMethod
where
    Self::Response: ContractMethodResponse,
{
    type Response;

    fn method_name(&self) -> &'static str;

    #[must_use]
    fn method_type() -> MethodType {
        MethodType::Call
    }

    fn deposit(&self) -> u128 {
        0
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        Ok(Vec::new())
    }

    fn parse_response(response: Vec<u8>) -> Result<Self::Response, error::Error> {
        Self::Response::parse(response)
    }

    fn parse_error(error: MethodExecutionError) -> Result<error::EngineError, std::io::Error> {
        match error {
            MethodExecutionError::ActionError(action_error) => parse_action_error(action_error),
            MethodExecutionError::QueryError(query_error) => parse_query_error(query_error),
        }
    }
}

pub trait ContractMethodResponse: borsh::BorshDeserialize {
    fn parse(value: Vec<u8>) -> Result<Self, error::Error> {
        borsh::from_slice(&value).map_err(Into::into)
    }
}

fn parse_action_error(action_error: ActionError) -> Result<error::EngineError, io::Error> {
    match action_error.kind {
        ActionErrorKind::FunctionCallError(FunctionCallError::ExecutionError(error_msg)) => {
            catch_panic(&error_msg)
        }
        _ => Err(io::Error::new(
            io::ErrorKind::Other,
            "Unexpected action error: ".to_string() + &action_error.to_string(),
        )),
    }
}

fn catch_panic(error_msg: &str) -> Result<error::EngineError, io::Error> {
    const ERR_MSG_PREFIX: &str = "Smart contract panicked: ";

    if let Some(msg) = error_msg.strip_prefix(ERR_MSG_PREFIX) {
        let error = serde_json::from_str::<error::EngineError>(msg)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, msg))?;
        Ok(error)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unexpected error: ".to_string() + error_msg,
        ))
    }
}

fn parse_query_error(query_error: RpcQueryError) -> Result<error::EngineError, std::io::Error> {
    match query_error {
        RpcQueryError::ContractExecutionError {
            vm_error,
            block_height: _,
            block_hash: _,
        } => catch_view_panic(&vm_error),
        _ => Err(io::Error::new(
            io::ErrorKind::Other,
            "Unexpected query error: ".to_string() + &query_error.to_string(),
        )),
    }
}

static PANIC_REGEX: OnceLock<Regex> = OnceLock::new();

fn catch_view_panic(input: &str) -> Result<error::EngineError, std::io::Error> {
    let re = PANIC_REGEX.get_or_init(|| Regex::new(r#"panic_msg: "([^"]+)""#).unwrap());

    re.captures(input)
        .and_then(|caps| caps.get(1))
        .map(|msg| EngineError::from(msg.as_str().to_string()))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "panic message not found"))
}

impl ContractMethodResponse for String {
    fn parse(rsp: Vec<u8>) -> Result<Self, error::Error> {
        Self::from_utf8(rsp)
            .map(|s| s.trim_matches('\"').to_string())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e).into())
    }
}
