use near_primitives::views::CallResult;
use primitive_types::U256;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<T> {
    pub result: T,
    pub logs: Vec<String>,
}

impl<T> TryFrom<CallResult> for Response<T>
where
    T: FromBytes,
{
    type Error = anyhow::Error;

    fn try_from(call_result: CallResult) -> anyhow::Result<Self> {
        Ok(Self {
            result: T::from_bytes(call_result.result)?,
            logs: call_result.logs,
        })
    }
}

pub trait FromBytes: Sized {
    fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<Self>;
}

impl FromBytes for U256 {
    fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<Self> {
        Ok(Self::from_big_endian(&bytes))
    }
}

impl FromBytes for String {
    fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<Self> {
        Self::from_utf8(bytes)
            .map(|r| r.trim().to_string())
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitive_types::U256;

    #[test]
    fn test_from_call_result_u256() {
        let call_result = CallResult {
            result: U256::from(42).to_big_endian().to_vec(),
            logs: vec![],
        };

        let response: Response<U256> = call_result.try_into().unwrap();
        assert_eq!(response.result, U256::from(42));
    }
}
