use std::io;

use near_jsonrpc_client::methods::query::RpcQueryError;
use near_primitives::errors::{ActionError, ActionErrorKind, FunctionCallError};
use near_primitives::hash::CryptoHash;

use crate::aurora::error::EngineError;
use crate::aurora::{
    convert_call_msg_to_error, convert_view_msg_to_error, parse_action_error, parse_query_error,
};

#[test]
fn test_parse_action_error_with_function_call_error() {
    // Arrange
    let error_msg = "Smart contract panicked: ERR_PARSE_ARGS".to_string();
    let action_error = ActionError {
        kind: ActionErrorKind::FunctionCallError(FunctionCallError::ExecutionError(error_msg)),
        index: Some(0),
    };

    // Act
    let result = parse_action_error(action_error);

    assert!(
        matches!(result, Ok(EngineError::ParseArgs)),
        "Expected EngineError::ParseArgs, got {result:?}",
    );
}

#[test]
fn test_parse_action_error_with_unexpected_error() {
    // Arrange
    let action_error = ActionError {
        kind: ActionErrorKind::AccountDoesNotExist {
            account_id: "test.near".parse().unwrap(),
        },
        index: Some(0),
    };

    // Act
    let result = parse_action_error(action_error);

    // Assert
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Unexpected action error"));
    }
}

#[test]
fn test_convert_call_msg_to_error_with_valid_json() {
    // Arrange
    let error_msg = "Smart contract panicked: ERR_PARSE_ARGS";

    // Act
    let result = convert_call_msg_to_error(error_msg);

    // Assert
    match result {
        Ok(engine_error) => {
            assert!(matches!(engine_error, EngineError::ParseArgs));
        }
        Err(e) => panic!("Expected EngineError, got error: {e}"),
    }
}

#[test]
fn test_convert_call_msg_to_error_with_invalid_prefix() {
    // Arrange
    let error_msg = "Invalid prefix: {\"OutOfGas\":null}";

    // Act
    let result = convert_call_msg_to_error(error_msg);

    // Assert
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Unexpected error"));
    }
}

#[test]
fn test_convert_call_msg_to_error_with_invalid_json() {
    const ERR_MSG: &str = "some_unknown_error";
    // Arrange
    let error_msg = format!("Smart contract panicked: {ERR_MSG}");

    // Act
    let result = convert_call_msg_to_error(&error_msg);
    // Assert

    let expected = ERR_MSG.to_string();
    match result {
        Ok(EngineError::Unknown(msg)) => {
            assert_eq!(msg, expected);
        }
        Err(e) => panic!("Expected EngineError::Unknown, got {e}"),
        _ => panic!("Expected EngineError::Unknown, got {result:?}"),
    }
}

#[test]
fn test_parse_query_error_with_contract_execution_error() {
    // Arrange
    let vm_error = "wasm execution failed with error: FunctionCallError(HostError(GuestPanic { panic_msg: \"ERR_OUT_OF_GAS\" }))".to_string();
    let query_error = RpcQueryError::ContractExecutionError {
        vm_error,
        block_height: 0,
        block_hash: CryptoHash::default(),
    };

    // Act
    let result = parse_query_error(query_error);

    // Assert
    assert!(
        matches!(result, Ok(EngineError::OutOfGas)),
        "Expected EngineError::OutOfGas, got {result:?}",
    );
}

#[test]
fn test_parse_query_error_with_unexpected_error() {
    // Arrange
    let query_error = RpcQueryError::InvalidAccount {
        requested_account_id: "test.near".parse().unwrap(),
        block_height: 0,
        block_hash: CryptoHash::default(),
    };

    // Act
    let result = parse_query_error(query_error);

    // Assert
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Unexpected query error"));
    }
}

#[test]
fn test_convert_view_msg_to_error_with_valid_message() {
    // Arrange
    let input = "wasm execution failed with error: FunctionCallError(HostError(GuestPanic { panic_msg: \"ERR_OUT_OF_GAS\" }))";

    // Act
    let result = convert_view_msg_to_error(input);

    // Assert
    assert!(
        matches!(result, Ok(EngineError::OutOfGas)),
        "Expected EngineError::OutOfGas, got {result:?}",
    );
}

#[test]
fn test_convert_view_msg_to_error_with_invalid_message() {
    // Arrange
    let input = "wasm execution failed with no panic message";

    // Act
    let result = convert_view_msg_to_error(input);

    // Assert
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), io::ErrorKind::NotFound);
    }
}
