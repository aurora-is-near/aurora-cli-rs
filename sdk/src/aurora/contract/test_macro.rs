use crate::ContractMethod;
use serde::{Deserialize, Serialize};

// Test with Vec<u8> field - should be returned as-is
#[derive(ContractMethod)]
#[contract_method(method = "raw_data_method", response = (), serialize_as = "json")]
pub struct RawDataMethod {
    #[contract_param]
    pub raw_bytes: Vec<u8>, // Should be returned as-is
    #[allow(unused)]
    pub metadata: String, // Ignored
}

// Test with regular field - should be serialized
#[derive(ContractMethod, Serialize, Deserialize)]
#[contract_method(method = "text_method", response = String, serialize_as = "json")]
pub struct TextMethod {
    #[contract_param]
    pub message: String, // Should be serialized to JSON
    pub timestamp: u64, // Ignored
}

// Test with Vec<u8> and Borsh serialization
#[derive(ContractMethod)]
#[contract_method(method = "raw_borsh_method", response = (), serialize_as = "borsh")]
pub struct RawBorshMethod {
    #[contract_param]
    pub data: Vec<u8>, // Should be returned as-is regardless of serialize_as
    #[allow(unused)]
    pub extra: String, // Ignored
}

// Test with regular field and Borsh serialization
#[derive(ContractMethod, borsh::BorshSerialize, borsh::BorshDeserialize)]
#[contract_method(method = "struct_borsh_method", response = (), serialize_as = "borsh")]
pub struct StructBorshMethod {
    #[contract_param]
    pub user_data: UserData, // Should be serialized to Borsh
    pub cache: String, // Ignored
}

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize)]
pub struct UserData {
    pub id: u64,
    pub name: String,
}

// Unit structure - empty parameters
#[derive(ContractMethod)]
#[contract_method(method = "ping", response = (), serialize_as = "json")]
pub struct Ping;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aurora::ContractMethod;

    #[test]
    fn test_vec_u8_field_returned_as_is() {
        let method = RawDataMethod {
            raw_bytes: vec![1, 2, 3, 4, 5],
            metadata: "ignored".to_string(),
        };

        assert_eq!(method.method_name(), "raw_data_method");

        let params = method.params().unwrap();
        // Vec<u8> field should be returned as-is
        assert_eq!(params, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_vec_u8_field_with_borsh() {
        let method = RawBorshMethod {
            data: vec![10, 20, 30],
            extra: "ignored".to_string(),
        };

        assert_eq!(method.method_name(), "raw_borsh_method");

        let params = method.params().unwrap();
        // Vec<u8> field should be returned as-is, even with serialize_as = "borsh"
        assert_eq!(params, vec![10, 20, 30]);
    }

    #[test]
    fn test_string_field_json_serialization() {
        let method = TextMethod {
            message: "hello world".to_string(),
            timestamp: 1_234_567_890,
        };

        assert_eq!(method.method_name(), "text_method");

        let params = method.params().unwrap();
        let expected = serde_json::to_vec("hello world").unwrap();
        assert_eq!(params, expected);
    }

    #[test]
    fn test_struct_field_borsh_serialization() {
        let method = StructBorshMethod {
            user_data: UserData {
                id: 42,
                name: "alice".to_string(),
            },
            cache: "ignored".to_string(),
        };

        assert_eq!(method.method_name(), "struct_borsh_method");

        let params = method.params().unwrap();
        let expected = borsh::to_vec(&UserData {
            id: 42,
            name: "alice".to_string(),
        })
        .unwrap();
        assert_eq!(params, expected);
    }

    #[test]
    fn test_unit_struct() {
        let method = Ping;

        assert_eq!(method.method_name(), "ping");

        let params = method.params().unwrap();
        assert_eq!(params, Vec::<u8>::new());
    }

    #[test]
    fn test_empty_vec_u8() {
        let method = RawDataMethod {
            raw_bytes: Vec::new(),
            metadata: "ignored".to_string(),
        };

        assert_eq!(method.method_name(), "raw_data_method");

        let params = method.params().unwrap();
        assert_eq!(params, Vec::<u8>::new());
    }
}
