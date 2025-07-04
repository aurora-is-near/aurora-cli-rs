use serde_json::Value;
use std::collections::HashMap;

use crate::cli::OutputFormat;
pub enum CommandResult {
    Object(HashMap<String, Value>),
    Success(String),
}

impl CommandResult {
    pub fn success(msg: impl Into<String>) -> Self {
        Self::Success(msg.into())
    }
}

pub fn format_output(result: CommandResult, format: &OutputFormat) -> String {
    match format {
        OutputFormat::Plain => format_plain(result),
        OutputFormat::Json => format_json(result),
    }
}

fn format_plain(result: CommandResult) -> String {
    match result {
        CommandResult::Object(obj) => {
            let mut output = String::new();
            for (key, value) in obj {
                output.push_str(&format!("{}: {}\n", key, format_value_plain(&value)));
            }
            output.trim_end().to_string()
        }
        CommandResult::Success(msg) => msg,
    }
}

fn format_json(result: CommandResult) -> String {
    let json_value = match result {
        CommandResult::Object(obj) => Value::Object(obj.into_iter().collect()),
        CommandResult::Success(msg) => {
            let mut map = serde_json::Map::new();
            map.insert("success".to_string(), Value::Bool(true));
            map.insert("message".to_string(), Value::String(msg));
            Value::Object(map)
        }
    };

    serde_json::to_string_pretty(&json_value).unwrap_or_else(|_| "{}".to_string())
}

fn format_value_plain(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(arr) => format!(
            "[{}]",
            arr.iter()
                .map(format_value_plain)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Value::Object(_) => serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string()),
    }
}

#[macro_export]
macro_rules! output {
    ($format:expr, $result:expr) => {
        println!("{}", crate::common::output::format_output($result, $format));
    };
}

#[macro_export]
macro_rules! result_object {
    ($($key:expr => $value:expr),*) => {
        {
            let mut map = std::collections::HashMap::new();
            $(
                map.insert($key.to_string(), serde_json::json!($value));
            )*
            crate::common::output::CommandResult::Object(map)
        }
    };
}

#[macro_export]
macro_rules! result_array {
    ($($value:expr),*) => {
        {
            let mut arr = Vec::new();
            $(
                arr.push(serde_json::json!($value));
            )*
            crate::common::output::CommandResult::Array(arr)
        }
    };
}
