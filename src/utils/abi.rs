use aurora_engine_types::U256;
use ethabi::{Function, Token};
use serde_json::Value;
use std::path::Path;

pub fn read_contract<P: AsRef<Path>>(abi_path: P) -> anyhow::Result<ethabi::Contract> {
    std::fs::File::open(abi_path.as_ref())
        .map_err(Into::into)
        .and_then(|reader| ethabi::Contract::load(reader).map_err(Into::into))
}

pub fn parse_args(function: &Function, args: &Value) -> anyhow::Result<Vec<Token>> {
    if matches!(args, Value::Null) {
        return Ok(vec![]);
    }

    let vars_map = args
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Expected JSON object"))?;
    let mut tokens = Vec::with_capacity(function.inputs.len());

    for input in &function.inputs {
        let arg = vars_map
            .get(&input.name)
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Missing variable"))?;
        let token = parse_arg(arg, &input.kind)?;
        tokens.push(token);
    }

    Ok(tokens)
}

pub fn parse_arg(arg: &str, kind: &ethabi::ParamType) -> anyhow::Result<Token> {
    match kind {
        ethabi::ParamType::Address => {
            let addr = super::hex_to_address(arg)?;
            Ok(Token::Address(addr.raw()))
        }
        ethabi::ParamType::Bytes => {
            let bytes = super::hex_to_vec(arg)?;
            Ok(Token::Bytes(bytes))
        }
        ethabi::ParamType::Int(_) => {
            let value = U256::from_dec_str(arg)?;
            Ok(Token::Int(value))
        }
        ethabi::ParamType::Uint(_) => {
            let value = U256::from_dec_str(arg)?;
            Ok(Token::Uint(value))
        }
        ethabi::ParamType::Bool => match arg.to_lowercase().as_str() {
            "true" => Ok(Token::Bool(true)),
            "false" => Ok(Token::Bool(false)),
            _ => anyhow::bail!("Expected true or false"),
        },
        ethabi::ParamType::String => Ok(Token::String(arg.into())),
        ethabi::ParamType::Array(arr_kind) => {
            let value: Value = serde_json::from_str(arg)?;
            parse_array(value, arr_kind).map(Token::Array)
        }
        ethabi::ParamType::FixedBytes(size) => {
            let bytes = super::hex_to_vec(arg)?;
            if &bytes.len() != size {
                anyhow::bail!("Incorrect FixedBytes length")
            }
            Ok(Token::FixedBytes(bytes))
        }
        ethabi::ParamType::FixedArray(arr_kind, size) => {
            let value: Value = serde_json::from_str(arg)?;
            let tokens = parse_array(value, arr_kind)?;
            if &tokens.len() != size {
                anyhow::bail!("Incorrect FixedArray length")
            }
            Ok(Token::FixedArray(tokens))
        }
        ethabi::ParamType::Tuple(tuple_kinds) => {
            let value: Value = serde_json::from_str(arg)?;
            let Value::Array(values) = value else { anyhow::bail!("Expected Array"); };
            if values.len() != tuple_kinds.len() {
                anyhow::bail!("Incorrect number of args for tuple size");
            }
            let mut tokens = Vec::with_capacity(values.len());

            for (v, kind) in values.iter().zip(tuple_kinds.iter()) {
                let token = parse_arg(&serde_json::to_string(v).unwrap(), kind)?;
                tokens.push(token);
            }

            Ok(Token::Tuple(tokens))
        }
    }
}

fn parse_array(value: Value, arr_kind: &ethabi::ParamType) -> anyhow::Result<Vec<Token>> {
    match value {
        Value::Array(values) => {
            let mut tokens = Vec::with_capacity(values.len());

            for v in values {
                let token = parse_arg(&serde_json::to_string(&v).unwrap(), arr_kind)?;
                tokens.push(token);
            }

            Ok(tokens)
        }
        _ => anyhow::bail!("Expected Array"),
    }
}
