pub mod client;
pub mod error;
pub mod operations;
pub mod query;
pub(crate) mod rpc_client;
pub mod types;
pub use near_crypto as crypto;
pub use near_jsonrpc_client as jsonrpc;
pub use near_primitives as primitives;
pub use near_sdk::json_types;
pub use near_token as token;

/// A type alias for `anyhow::Result<T, Error>`.
pub type Result<T> = anyhow::Result<T, error::Error>;
