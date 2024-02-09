#[cfg(feature = "advanced")]
mod advanced;
#[cfg(feature = "simple")]
pub mod simple;

#[cfg(feature = "advanced")]
pub use advanced::{aurora, near, process_tx_data, run, Cli, Command};

#[cfg(feature = "simple")]
pub use simple::{command, run, Cli};

/// NEAR Endpoints.
const NEAR_MAINNET_ENDPOINT: &str = "https://archival-rpc.mainnet.near.org/";
const NEAR_TESTNET_ENDPOINT: &str = "https://archival-rpc.testnet.near.org/";
#[cfg(feature = "simple")]
const NEAR_LOCAL_ENDPOINT: &str = "http://127.0.0.1:3030/";
/// Aurora Endpoints.
#[cfg(feature = "advanced")]
const AURORA_MAINNET_ENDPOINT: &str = "https://mainnet.aurora.dev/";
#[cfg(feature = "advanced")]
const AURORA_TESTNET_ENDPOINT: &str = "https://testnet.aurora.dev/";
