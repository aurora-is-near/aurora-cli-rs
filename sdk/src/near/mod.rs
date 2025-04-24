pub mod client;
pub mod error;
pub mod operations;
pub mod query;
pub mod workspace;

/// A type alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, error::Error>;
