use near_primitives::errors::{ActionError, ActionErrorKind, FunctionCallError};

pub mod client;
pub mod contract;
pub mod error;

pub trait ContractMethod
where
    Self::Response: ContractMethodResponse,
{
    type Response;

    fn method_name(&self) -> &str;

    fn deposit(&self) -> u128 {
        0
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        Ok(Vec::new())
    }

    fn parse_response(response: Vec<u8>) -> Result<Self::Response, error::Error> {
        Self::Response::parse(response)
    }

    fn parse_error(error: ActionError) -> Result<error::SiloError, std::io::Error> {
        match error.kind {
            ActionErrorKind::FunctionCallError(FunctionCallError::ExecutionError(s)) => {
                catch_panic(&s)
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Unexpected action error: ".to_string() + &error.to_string(),
            )),
        }
    }
}

pub trait ContractMethodResponse: borsh::BorshDeserialize {
    fn parse(value: Vec<u8>) -> Result<Self, error::Error> {
        borsh::from_slice(&value).map_err(Into::into)
    }
}

fn catch_panic(error_msg: &str) -> Result<error::SiloError, std::io::Error> {
    const ERR_MSG_PREFIX: &str = "Smart contract panicked: ";

    if error_msg.starts_with(ERR_MSG_PREFIX) {
        let error_msg = error_msg[ERR_MSG_PREFIX.len()..].to_string();
        let error = serde_json::from_str::<error::SiloError>(&error_msg)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, error_msg))?;
        Ok(error)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unexpected error: ".to_string() + &error_msg,
        ))
    }
}

impl ContractMethodResponse for String {
    fn parse(rsp: Vec<u8>) -> Result<Self, error::Error> {
        String::from_utf8(rsp)
            .map(|s| s.trim_matches('\"').to_string())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e).into())
    }
}
