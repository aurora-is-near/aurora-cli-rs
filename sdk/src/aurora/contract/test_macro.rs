use crate::ContractMethod;
use serde::{Deserialize, Serialize};

// Test with Vec<u8> field - should be returned as-is
#[derive(ContractMethod)]
#[contract_method(method = "raw_data_method", response = ())]
pub struct RawDataMethod {
    #[contract_param]
    pub raw_bytes: Vec<u8>, // Should be returned as-is
    #[allow(unused)]
    pub metadata: String, // Ignored
}

// Test with regular field - should be serialized
#[derive(ContractMethod, Serialize, Deserialize)]
#[contract_method(method = "text_method", response = String, deserialize_as = "json")]
pub struct TextMethod {
    #[contract_param(serialize_as = "json")]
    pub message: String, // Should be serialized to JSON
    pub timestamp: u64, // Ignored
}

// Test with Vec<u8> and unit response (no deserialize_as needed)
#[derive(ContractMethod)]
#[contract_method(method = "raw_borsh_method", response = ())]
pub struct RawBorshMethod {
    #[contract_param]
    pub data: Vec<u8>, // Should be returned as-is regardless of serialize_as
    #[allow(unused)]
    pub extra: String, // Ignored
}

// Test with regular field and Borsh serialization
#[derive(ContractMethod, borsh::BorshSerialize, borsh::BorshDeserialize)]
#[contract_method(method = "struct_borsh_method", response = ())]
pub struct StructBorshMethod {
    #[contract_param(serialize_as = "borsh")]
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
#[contract_method(method = "ping", response = ())]
pub struct Ping;

// Test String as UTF-8 bytes (without serialize_as)
#[derive(ContractMethod)]
#[contract_method(method = "string_utf8_method", response = ())]
pub struct StringUtf8Method {
    #[contract_param]
    pub text: String, // Should convert to UTF-8 bytes directly
}

// Test String with JSON serialization (explicit)
#[derive(ContractMethod)]
#[contract_method(method = "string_json_method", response = ())]
pub struct StringJsonMethod {
    #[contract_param(serialize_as = "json")]
    pub text: String, // Should use JSON serialization
}

// Test with JSON response deserialization
#[derive(ContractMethod, Serialize, Deserialize)]
#[contract_method(method = "json_response_method", response = String, deserialize_as = "json")]
pub struct JsonResponseMethod {
    #[contract_param(serialize_as = "borsh")]
    pub query: u64,
}

// Test with raw response deserialization for Vec<u8>
#[derive(ContractMethod)]
#[contract_method(method = "raw_response_method", response = Vec<u8>, deserialize_as = "raw")]
pub struct RawResponseMethod {
    #[contract_param(serialize_as = "json")]
    pub query: String,
}

// Additional test for demonstration
#[cfg(test)]
mod string_serialization_demo {
    use crate::aurora::contract::test_macro::StringJsonMethod;

    #[test]
    fn demonstrate_string_serialization_difference() {
        let text = "Hello World";

        // 1. Direct UTF-8 conversion (new default behavior for String without serialize_as)
        let utf8_bytes = text.as_bytes().to_vec();

        // 2. JSON serialization (when serialize_as = "json")
        let json_bytes = serde_json::to_vec(&text).unwrap();

        // 3. Borsh serialization (when serialize_as = "borsh")
        let borsh_bytes = borsh::to_vec(&text).unwrap();

        println!("\nDemonstration of different string serialization methods for '{text}':");
        println!(
            "UTF-8:  {:?} -> {:?}",
            utf8_bytes,
            String::from_utf8(utf8_bytes.clone()).unwrap()
        );
        println!(
            "JSON:   {:?} -> {:?}",
            json_bytes,
            String::from_utf8(json_bytes.clone()).unwrap()
        );
        println!("Borsh:  {:?} -> length {}", borsh_bytes, borsh_bytes.len());

        // JSON adds quotes, which might be undesirable
        assert_eq!(
            utf8_bytes,
            [72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100]
        ); // UTF-8
        assert_eq!(
            json_bytes,
            [34, 72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 34]
        ); // JSON with quotes

        // Show that UTF-8 is shorter and without extra characters
        assert!(utf8_bytes.len() < json_bytes.len());
        assert_eq!(utf8_bytes.len(), 11); // "Hello World" = 11 characters
        assert_eq!(json_bytes.len(), 13); // '"Hello World"' = 13 characters (with quotes)
    }

    #[test]
    fn test_serde_json_error_conversion() {
        // Demonstrate that serde_json::Error now converts to io::Error
        // instead of being added as a separate type in enum Error
        use crate::aurora::ContractMethod;

        // Create a method with JSON serialization
        let method = StringJsonMethod {
            text: "test".to_string(),
        };

        // Check that this works
        assert_eq!(method.method_name(), "string_json_method");

        // Simulate JSON deserialization error with a method that deserializes JSON
        // (we need a method with deserialize_as = "json")
        let invalid_json = b"invalid json";
        let result = super::JsonResponseMethod::parse_response(invalid_json.to_vec());

        // Ensure this is io::Error, not serde_json::Error
        assert!(result.is_err());
        let error = result.unwrap_err();

        // Check that error can be converted to io::Error
        match error {
            crate::aurora::error::Error::Io(_io_err) => {
                // Expected case - serde_json::Error converted to io::Error
                println!("✅ serde_json::Error correctly converted to io::Error");
            }
            other => {
                panic!("Expected io::Error, got: {other:?}");
            }
        }
    }
}

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

    #[test]
    fn test_string_json_method() {
        let method = StringJsonMethod {
            text: "test string".to_string(),
        };

        assert_eq!(method.method_name(), "string_json_method");

        let params = method.params().unwrap();
        let expected = serde_json::to_vec("test string").unwrap();
        assert_eq!(params, expected);
    }

    #[test]
    fn test_json_response_method() {
        let method = JsonResponseMethod { query: 12345 };

        assert_eq!(method.method_name(), "json_response_method");

        let params = method.params().unwrap();
        let expected = borsh::to_vec(&12345u64).unwrap();
        assert_eq!(params, expected);
    }

    #[test]
    fn test_raw_response_method() {
        let method = RawResponseMethod {
            query: "test query".to_string(),
        };

        assert_eq!(method.method_name(), "raw_response_method");

        let params = method.params().unwrap();
        let expected = serde_json::to_vec("test query").unwrap();
        assert_eq!(params, expected);
    }

    #[test]
    fn test_string_utf8_conversion() {
        let method = StringUtf8Method {
            text: "Hello World".to_string(),
        };

        assert_eq!(method.method_name(), "string_utf8_method");

        let params = method.params().unwrap();
        // String should convert to UTF-8 bytes WITHOUT JSON quotes
        let expected = b"Hello World".to_vec();
        assert_eq!(params, expected);

        // Ensure this is not JSON serialization (which would add quotes)
        let json_serialized = serde_json::to_vec("Hello World").unwrap();
        assert_ne!(
            params, json_serialized,
            "String should not serialize as JSON by default"
        );
    }

    #[test]
    fn test_unit_type_auto_detection() {
        // Demonstrate that for unit type () deserialize_as is not needed
        let method = Ping;
        assert_eq!(method.method_name(), "ping");

        // Unit type should always return Ok(())
        let any_response = b"any data here doesn't matter";
        let result = Ping::parse_response(any_response.to_vec());

        match result {
            Ok(()) => {
                println!(
                    "✅ Unit type () correctly ignores deserialize_as and always returns Ok(())"
                );
            }
            Err(e) => {
                panic!("Unit type should always return Ok(()), got error: {e:?}");
            }
        }
    }
}
