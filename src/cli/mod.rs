#[cfg(feature = "advanced")]
mod advanced;
#[cfg(feature = "simple")]
pub mod simple;

#[cfg(feature = "advanced")]
pub use advanced::{aurora, near, process_tx_data, Cli, Command};

#[cfg(feature = "simple")]
pub use simple::{command, Cli, Command};
