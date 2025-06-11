#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc)]

pub mod aurora;
pub mod near;

// Re-export the procedural macros
pub use aurora_sdk_macros::ContractMethod;
