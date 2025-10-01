//! Tests for error handling

use claude_agent_sdk::errors::{ClaudeSDKError, Result};

#[test]
fn test_cli_not_found_error() {
    let error = ClaudeSDKError::cli_not_found("claude");
    assert!(matches!(error, ClaudeSDKError::CLINotFound(_)));
    assert!(error.to_string().contains("claude"));
}

#[test]
fn test_cli_connection_error() {
    let error = ClaudeSDKError::cli_connection("Connection failed");
    assert!(matches!(error, ClaudeSDKError::CLIConnection(_)));
    assert_eq!(error.to_string(), "CLI connection error: Connection failed");
}

#[test]
fn test_process_error() {
    let error = ClaudeSDKError::process("Command failed", Some(1), Some("stderr output".to_string()));
    match error {
        ClaudeSDKError::Process {
            message,
            exit_code,
            stderr,
        } => {
            assert_eq!(message, "Command failed");
            assert_eq!(exit_code, Some(1));
            assert_eq!(stderr, Some("stderr output".to_string()));
        }
        _ => panic!("Expected Process error"),
    }
}

#[test]
fn test_message_parse_error() {
    let data = serde_json::json!({"invalid": "data"});
    let error = ClaudeSDKError::message_parse("Failed to parse", Some(data.clone()));
    match error {
        ClaudeSDKError::MessageParse { message, data: d } => {
            assert_eq!(message, "Failed to parse");
            assert_eq!(d, Some(data));
        }
        _ => panic!("Expected MessageParse error"),
    }
}

#[test]
fn test_control_protocol_error() {
    let error = ClaudeSDKError::control_protocol("Protocol error");
    assert!(matches!(error, ClaudeSDKError::ControlProtocol(_)));
    assert_eq!(error.to_string(), "Control protocol error: Protocol error");
}

#[test]
fn test_transport_error() {
    let error = ClaudeSDKError::transport("Transport failed");
    assert!(matches!(error, ClaudeSDKError::Transport(_)));
    assert_eq!(error.to_string(), "Transport error: Transport failed");
}

#[test]
fn test_invalid_config_error() {
    let error = ClaudeSDKError::invalid_config("Invalid option");
    assert!(matches!(error, ClaudeSDKError::InvalidConfig(_)));
    assert_eq!(error.to_string(), "Invalid configuration: Invalid option");
}

#[test]
fn test_timeout_error() {
    let error = ClaudeSDKError::timeout("Request timed out");
    assert!(matches!(error, ClaudeSDKError::Timeout(_)));
    assert_eq!(error.to_string(), "Timeout: Request timed out");
}

#[test]
fn test_json_decode_error() {
    let json_error = serde_json::from_str::<serde_json::Value>("{invalid json").unwrap_err();
    let error: ClaudeSDKError = json_error.into();
    assert!(matches!(error, ClaudeSDKError::JSONDecode(_)));
}

#[test]
fn test_io_error() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let error: ClaudeSDKError = io_error.into();
    assert!(matches!(error, ClaudeSDKError::IO(_)));
}

#[test]
fn test_result_type_alias() {
    fn returns_result() -> Result<i32> {
        Ok(42)
    }

    assert_eq!(returns_result().unwrap(), 42);
}

#[test]
fn test_error_chain() {
    fn inner_error() -> Result<()> {
        Err(ClaudeSDKError::cli_not_found("claude"))
    }

    fn outer_error() -> Result<()> {
        inner_error().map_err(|e| ClaudeSDKError::transport(format!("Wrapper: {}", e)))
    }

    let result = outer_error();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, ClaudeSDKError::Transport(_)));
    assert!(error.to_string().contains("claude"));
}

#[test]
fn test_process_error_display() {
    let error = ClaudeSDKError::process("Failed", Some(127), Some("Command not found".to_string()));
    let display = error.to_string();
    assert!(display.contains("exit code"));
    assert!(display.contains("127"));
}

#[test]
fn test_error_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ClaudeSDKError>();
}
