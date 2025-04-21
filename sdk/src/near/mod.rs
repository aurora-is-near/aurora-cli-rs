pub mod client;
mod error;
pub mod operations;

/// A type alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, error::Error>;
