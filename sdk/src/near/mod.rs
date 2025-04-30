pub mod client;
pub mod error;
pub mod operations;
pub mod query;
pub mod workspace;

/// A type alias for `anyhow::Result<T, Error>`.
pub type Result<T> = anyhow::Result<T, error::Error>;
