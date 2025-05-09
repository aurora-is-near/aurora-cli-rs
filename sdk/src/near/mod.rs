pub mod client;
pub mod error;
pub mod operations;
pub(crate) mod query;
pub(crate) mod rpc_client;
pub mod types;

/// A type alias for `anyhow::Result<T, Error>`.
pub type Result<T> = anyhow::Result<T, error::Error>;
