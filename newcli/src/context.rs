use aurora_sdk_rs::aurora::client::Client;

use crate::cli::Cli;

pub struct Context {
    pub cli: Cli,
    pub client: Client,
}
