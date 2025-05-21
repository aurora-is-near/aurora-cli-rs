use borsh::BorshDeserialize;
use near_crypto::PublicKey;
use near_jsonrpc_client::{
    errors::JsonRpcError, methods, methods::RpcMethod, methods::query::RpcQueryRequest,
};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::{
    account::AccessKey,
    hash::CryptoHash,
    types::{AccountId, BlockHeight, BlockId, BlockReference},
    views::{BlockView, QueryRequest},
};
use serde::de::DeserializeOwned;
use std::fmt::{Debug, Display};

use super::Result;
use super::error::Error;
use super::operations::Function;
use super::rpc_client::RpcClient;

pub struct Query<'a, M> {
    method: M,
    client: &'a RpcClient,
    block_ref: BlockReference,
}

impl<'a, M> Query<'a, M> {
    pub(crate) fn new(client: &'a RpcClient, method: M) -> Self {
        Self {
            method,
            client,
            block_ref: BlockReference::latest(),
        }
    }

    #[must_use]
    pub fn block_height(mut self, height: BlockHeight) -> Self {
        self.block_ref = BlockId::Height(height).into();
        self
    }

    #[must_use]
    pub fn block_hash(mut self, hash: CryptoHash) -> Self {
        self.block_ref = BlockId::Hash(CryptoHash(hash.0)).into();
        self
    }
}

impl<'a, T, R> IntoFuture for Query<'a, T>
where
    T: ProcessQuery<Output = R> + Send + Sync + 'static,
    <T as ProcessQuery>::Method: RpcMethod + Debug + Send + Sync,
    <<T as ProcessQuery>::Method as RpcMethod>::Response: Debug + Send + Sync,
    <<T as ProcessQuery>::Method as RpcMethod>::Error: Debug + Display + Send + Sync,
{
    type Output = Result<R>;

    type IntoFuture = std::pin::Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let response = self
                .client
                .query(&self.method.into_request(self.block_ref)?)
                .await
                .map_err(|e| T::from_error(e))?;

            T::from_response(response)
        })
    }
}

pub trait ProcessQuery {
    type Method: RpcMethod;

    type Output;

    fn into_request(self, block_ref: BlockReference) -> Result<Self::Method>;

    fn from_response(resp: <Self::Method as RpcMethod>::Response) -> Result<Self::Output>;

    fn from_error(err: JsonRpcError<<Self::Method as RpcMethod>::Error>) -> Error;
}

pub struct ViewFunction {
    pub(crate) account_id: AccountId,
    pub(crate) function: Function,
}

pub struct ViewCode {
    pub(crate) account_id: AccountId,
}

pub struct ViewAccount {
    pub(crate) account_id: AccountId,
}

pub struct ViewBlock;

pub struct ViewAccessKey {
    pub(crate) account_id: AccountId,
    pub(crate) public_key: PublicKey,
}

pub struct ViewAccessKeyList {
    pub(crate) account_id: AccountId,
}

impl ProcessQuery for ViewFunction {
    type Method = RpcQueryRequest;
    type Output = near_primitives::views::CallResult;

    fn into_request(self, block_ref: BlockReference) -> Result<Self::Method> {
        let request = Self::Method {
            block_reference: block_ref,
            request: QueryRequest::CallFunction {
                account_id: self.account_id,
                method_name: self.function.name,
                args: self.function.args.into(),
            },
        };

        Ok(request)
    }

    fn from_response(resp: <Self::Method as RpcMethod>::Response) -> Result<Self::Output> {
        match resp.kind {
            QueryResponseKind::CallResult(result) => Ok(result),
            _ => Err(Error::UnexpectedQueryResponseKind(Box::new(resp.kind))),
        }
    }

    fn from_error(err: JsonRpcError<<Self::Method as RpcMethod>::Error>) -> Error {
        err.into()
    }
}

pub trait JsonIntoResult<T> {
    fn into_result(self) -> Result<T>;
}

impl<T: DeserializeOwned> JsonIntoResult<T> for near_primitives::views::CallResult {
    fn into_result(self) -> Result<T> {
        serde_json::from_slice(&self.result).map_err(Into::into)
    }
}

pub trait BorshIntoResult<T> {
    fn into_result(self) -> Result<T>;
}

impl<T: BorshDeserialize> BorshIntoResult<T> for near_primitives::views::CallResult {
    fn into_result(self) -> Result<T> {
        borsh::from_slice(&self.result).map_err(Into::into)
    }
}

// Specific builder methods attached to a ViewFunction.
impl Query<'_, ViewFunction> {
    #[must_use]
    pub fn args(mut self, args: Vec<u8>) -> Self {
        self.method.function = self.method.function.args(args);
        self
    }

    /// Sets the arguments for the view function call using JSON serialization.
    ///
    /// # Errors
    ///
    /// This method will return an error if the serialization of the arguments fails.
    pub fn args_json<U: serde::Serialize>(mut self, args: U) -> Result<Self> {
        self.method.function = self.method.function.args_json(args)?;
        Ok(self)
    }

    /// Sets the arguments for the view function call using Borsh serialization.
    ///
    /// # Errors
    ///
    /// This method will return an error if the serialization of the arguments fails.
    pub fn args_borsh<U: near_primitives::borsh::BorshSerialize>(
        mut self,
        args: U,
    ) -> Result<Self> {
        self.method.function = self.method.function.args_borsh(args)?;
        Ok(self)
    }
}

impl ProcessQuery for ViewCode {
    type Method = RpcQueryRequest;
    type Output = Vec<u8>;

    fn into_request(self, block_reference: BlockReference) -> Result<Self::Method> {
        Ok(Self::Method {
            block_reference,
            request: QueryRequest::ViewCode {
                account_id: self.account_id,
            },
        })
    }

    fn from_response(resp: <Self::Method as RpcMethod>::Response) -> Result<Self::Output> {
        match resp.kind {
            QueryResponseKind::ViewCode(code) => Ok(code.code),
            _ => Err(Error::UnexpectedQueryResponseKind(Box::new(resp.kind))),
        }
    }

    fn from_error(err: JsonRpcError<<Self::Method as RpcMethod>::Error>) -> Error {
        err.into()
    }
}

impl ProcessQuery for ViewAccount {
    type Method = RpcQueryRequest;
    type Output = near_primitives::views::AccountView;

    fn into_request(self, block_reference: BlockReference) -> Result<Self::Method> {
        Ok(Self::Method {
            block_reference,
            request: QueryRequest::ViewAccount {
                account_id: self.account_id,
            },
        })
    }

    fn from_response(resp: <Self::Method as RpcMethod>::Response) -> Result<Self::Output> {
        match resp.kind {
            QueryResponseKind::ViewAccount(account) => Ok(account),
            _ => Err(Error::UnexpectedQueryResponseKind(Box::new(resp.kind))),
        }
    }

    fn from_error(err: JsonRpcError<<Self::Method as RpcMethod>::Error>) -> Error {
        err.into()
    }
}

impl ProcessQuery for ViewBlock {
    type Method = methods::block::RpcBlockRequest;
    type Output = BlockView;

    fn into_request(self, block_reference: BlockReference) -> Result<Self::Method> {
        Ok(Self::Method { block_reference })
    }

    fn from_response(view: BlockView) -> Result<Self::Output> {
        Ok(view)
    }

    fn from_error(err: JsonRpcError<<Self::Method as RpcMethod>::Error>) -> Error {
        err.into()
    }
}

impl ProcessQuery for ViewAccessKey {
    type Method = RpcQueryRequest;
    type Output = AccessKey;

    fn into_request(self, block_reference: BlockReference) -> Result<Self::Method> {
        Ok(Self::Method {
            block_reference,
            request: QueryRequest::ViewAccessKey {
                account_id: self.account_id,
                public_key: self.public_key,
            },
        })
    }

    fn from_response(resp: <Self::Method as RpcMethod>::Response) -> Result<Self::Output> {
        match resp.kind {
            QueryResponseKind::AccessKey(key) => Ok(key.into()),
            _ => Err(Error::UnexpectedQueryResponseKind(Box::new(resp.kind))),
        }
    }

    fn from_error(err: JsonRpcError<<Self::Method as RpcMethod>::Error>) -> Error {
        err.into()
    }
}

impl ProcessQuery for ViewAccessKeyList {
    type Method = RpcQueryRequest;
    type Output = near_primitives::views::AccessKeyList;

    fn into_request(self, block_reference: BlockReference) -> Result<Self::Method> {
        Ok(Self::Method {
            block_reference,
            request: QueryRequest::ViewAccessKeyList {
                account_id: self.account_id,
            },
        })
    }

    fn from_response(resp: <Self::Method as RpcMethod>::Response) -> Result<Self::Output> {
        match resp.kind {
            QueryResponseKind::AccessKeyList(list) => Ok(list),
            _ => Err(Error::UnexpectedQueryResponseKind(Box::new(resp.kind))),
        }
    }

    fn from_error(err: JsonRpcError<<Self::Method as RpcMethod>::Error>) -> Error {
        err.into()
    }
}
