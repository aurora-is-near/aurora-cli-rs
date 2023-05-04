use crate::utils;
use clap::Subcommand;
use serde_json::Value;
use std::borrow::Cow;

#[derive(Subcommand)]
pub enum Solidity {
    UnaryCall {
        #[clap(short, long)]
        abi_path: String,
        #[clap(short, long)]
        method_name: String,
        #[clap(short, long)]
        arg: Option<String>,
        #[clap(short, long)]
        stdin_arg: Option<bool>,
    },
    /// Allows invoking a solidity functions by passing in a JSON object.
    /// The names of the fields are the argument names of the function, and
    /// the values are strings that can be parsed into the correct types.
    CallArgsByName {
        #[clap(short, long)]
        abi_path: String,
        #[clap(short, long)]
        method_name: String,
        #[clap(short, long)]
        arg: Option<String>,
        #[clap(short, long)]
        stdin_arg: Option<bool>,
    },
}

impl Solidity {
    pub fn abi_decode(&self, output: &[u8]) -> anyhow::Result<Vec<ethabi::Token>> {
        let (abi, method_name) = match self {
            Self::UnaryCall {
                abi_path,
                method_name,
                ..
            }
            | Self::CallArgsByName {
                abi_path,
                method_name,
                ..
            } => (utils::abi::read_contract(abi_path)?, method_name),
        };

        let function = abi.function(method_name)?;
        let tokens = function.decode_output(output)?;
        Ok(tokens)
    }

    pub fn abi_encode(&self) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::UnaryCall {
                abi_path,
                method_name,
                arg,
                stdin_arg,
            } => {
                let abi = utils::abi::read_contract(abi_path)?;
                let function = abi.function(method_name)?;
                if function.inputs.len() != 1 {
                    anyhow::bail!("Function must take only one argument");
                }
                let arg_type = &function.inputs.first().unwrap().kind;
                let arg = read_arg(arg.as_deref(), *stdin_arg);

                function
                    .encode_input(&[utils::abi::parse_arg(arg.trim(), arg_type)?])
                    .map_err(Into::into)
            }
            Self::CallArgsByName {
                abi_path,
                method_name,
                arg,
                stdin_arg,
            } => {
                let contract = utils::abi::read_contract(abi_path)?;
                let function = contract.function(method_name)?;
                let args: Value = serde_json::from_str(&read_arg(arg.as_deref(), *stdin_arg))?;
                let tokens = utils::abi::parse_args(&function.inputs, &args)?;

                function.encode_input(&tokens).map_err(Into::into)
            }
        }
    }
}

fn read_arg(arg: Option<&str>, stdin_arg: Option<bool>) -> Cow<str> {
    arg.map_or_else(
        || match stdin_arg {
            Some(true) => {
                let mut buf = String::new();
                std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf).unwrap();
                Cow::Owned(buf)
            }
            None | Some(false) => Cow::Owned(String::new()),
        },
        Cow::Borrowed,
    )
}
