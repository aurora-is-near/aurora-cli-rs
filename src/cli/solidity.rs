use crate::{
    cli::erc20::{wrap_error, ParseError},
    utils,
};
use aurora_engine_types::U256;
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
    pub fn abi_decode(&self, output: &[u8]) -> Result<Vec<ethabi::Token>, ParseError> {
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
            } => (read_abi(abi_path)?, method_name),
        };

        let function = abi.function(method_name).map_err(wrap_error)?;
        let tokens = function.decode_output(output).map_err(wrap_error)?;
        Ok(tokens)
    }

    pub fn abi_encode(&self) -> Result<Vec<u8>, ParseError> {
        match self {
            Self::UnaryCall {
                abi_path,
                method_name,
                arg,
                stdin_arg,
            } => {
                let abi = read_abi(abi_path)?;
                let function = abi.function(method_name).map_err(wrap_error)?;
                if function.inputs.len() != 1 {
                    return Err(wrap_error("Function must take only one argument"));
                }
                let arg_type = &function.inputs.first().unwrap().kind;
                let arg = read_arg(arg.as_deref(), *stdin_arg);

                function
                    .encode_input(&[parse_arg(arg.trim(), arg_type)?])
                    .map_err(wrap_error)
            }
            Self::CallArgsByName {
                abi_path,
                method_name,
                arg,
                stdin_arg,
            } => {
                let abi = read_abi(abi_path)?;
                let function = abi.function(method_name).map_err(wrap_error)?;
                let arg: serde_json::Value =
                    serde_json::from_str(read_arg(arg.as_deref(), *stdin_arg).as_ref())
                        .map_err(wrap_error)?;
                let vars_map = arg
                    .as_object()
                    .ok_or_else(|| wrap_error("Expected JSON object"))?;
                let mut tokens = Vec::with_capacity(function.inputs.len());
                for input in &function.inputs {
                    let arg = vars_map
                        .get(&input.name)
                        .and_then(Value::as_str)
                        .ok_or_else(|| wrap_error("Missing variable"))?;
                    let token = parse_arg(arg, &input.kind)?;
                    tokens.push(token);
                }

                function.encode_input(&tokens).map_err(wrap_error)
            }
        }
    }
}

fn read_abi(abi_path: &str) -> Result<ethabi::Contract, ParseError> {
    let reader = std::fs::File::open(abi_path).map_err(wrap_error)?;
    ethabi::Contract::load(reader).map_err(wrap_error)
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

fn parse_arg(arg: &str, kind: &ethabi::ParamType) -> Result<ethabi::Token, ParseError> {
    match kind {
        ethabi::ParamType::Address => {
            let addr = utils::hex_to_address(arg).map_err(wrap_error)?;
            Ok(ethabi::Token::Address(addr.raw()))
        }
        ethabi::ParamType::Bytes => {
            let bytes = utils::hex_to_vec(arg).map_err(wrap_error)?;
            Ok(ethabi::Token::Bytes(bytes))
        }
        ethabi::ParamType::Int(_) => {
            let value = U256::from_dec_str(arg).map_err(wrap_error)?;
            Ok(ethabi::Token::Int(value))
        }
        ethabi::ParamType::Uint(_) => {
            let value = U256::from_dec_str(arg).map_err(wrap_error)?;
            Ok(ethabi::Token::Uint(value))
        }
        ethabi::ParamType::Bool => match arg.to_lowercase().as_str() {
            "true" => Ok(ethabi::Token::Bool(true)),
            "false" => Ok(ethabi::Token::Bool(false)),
            _ => Err(wrap_error("Expected true or false")),
        },
        ethabi::ParamType::String => Ok(ethabi::Token::String(arg.into())),
        ethabi::ParamType::Array(arr_kind) => {
            let value: serde_json::Value = serde_json::from_str(arg).map_err(wrap_error)?;
            parse_array(value, arr_kind).map(ethabi::Token::Array)
        }
        ethabi::ParamType::FixedBytes(size) => {
            let bytes = utils::hex_to_vec(arg).map_err(wrap_error)?;
            if &bytes.len() != size {
                return Err(wrap_error("Incorrect FixedBytes length"));
            }
            Ok(ethabi::Token::FixedBytes(bytes))
        }
        ethabi::ParamType::FixedArray(arr_kind, size) => {
            let value: serde_json::Value = serde_json::from_str(arg).map_err(wrap_error)?;
            let tokens = parse_array(value, arr_kind)?;
            if &tokens.len() != size {
                return Err(wrap_error("Incorrect FixedArray length"));
            }
            Ok(ethabi::Token::FixedArray(tokens))
        }
        ethabi::ParamType::Tuple(tuple_kinds) => {
            let value: serde_json::Value = serde_json::from_str(arg).map_err(wrap_error)?;
            let values = match value {
                serde_json::Value::Array(values) => values,
                _ => {
                    return Err(wrap_error("Expected Array"));
                }
            };
            if values.len() != tuple_kinds.len() {
                return Err(wrap_error("Incorrect number of args for tuple size"));
            }
            let mut tokens = Vec::with_capacity(values.len());
            for (v, kind) in values.iter().zip(tuple_kinds.iter()) {
                let token = parse_arg(&serde_json::to_string(v).unwrap(), kind)?;
                tokens.push(token);
            }
            Ok(ethabi::Token::Tuple(tokens))
        }
    }
}

fn parse_array(
    value: serde_json::Value,
    arr_kind: &ethabi::ParamType,
) -> Result<Vec<ethabi::Token>, ParseError> {
    match value {
        serde_json::Value::Array(values) => {
            let mut tokens = Vec::with_capacity(values.len());
            for v in values {
                let token = parse_arg(&serde_json::to_string(&v).unwrap(), arr_kind)?;
                tokens.push(token);
            }
            Ok(tokens)
        }
        _ => Err(wrap_error("Expected Array")),
    }
}
