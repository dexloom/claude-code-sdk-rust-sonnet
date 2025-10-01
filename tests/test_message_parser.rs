//! Tests for message parser

use claude_agent_sdk::message_parser::parse_message;
use claude_agent_sdk::types::{ContentBlock, Message};
use serde_json::json;

#[test]
fn test_parse_text_content_block() {
    let data = json!({
        "type": "assistant",
        "message": {
            "content": [
                {
                    "type": "text",
                    "text": "Hello, world!"
                }
            ],
            "model": "claude-sonnet-4"
        }
    });

    let result = parse_message(data).unwrap();
    match result {
        Message::Assistant { message, .. } => {
            assert_eq!(message.message.content.len(), 1);
            match &message.message.content[0] {
                ContentBlock::Text { text } => assert_eq!(text, "Hello, world!"),
                _ => panic!("Expected TextBlock"),
            }
        }
        _ => panic!("Expected Assistant message"),
    }
}

#[test]
fn test_parse_thinking_block() {
    let data = json!({
        "type": "assistant",
        "message": {
            "content": [
                {
                    "type": "thinking",
                    "thinking": "Let me think about this...",
                    "signature": "sig123"
                }
            ],
            "model": "claude-sonnet-4"
        }
    });

    let result = parse_message(data).unwrap();
    match result {
        Message::Assistant { message, .. } => {
            match &message.message.content[0] {
                ContentBlock::Thinking { thinking, signature } => {
                    assert_eq!(thinking, "Let me think about this...");
                    assert_eq!(signature, "sig123");
                }
                _ => panic!("Expected ThinkingBlock"),
            }
        }
        _ => panic!("Expected Assistant message"),
    }
}

#[test]
fn test_parse_tool_use_block() {
    let data = json!({
        "type": "assistant",
        "message": {
            "content": [
                {
                    "type": "tool_use",
                    "id": "tool_123",
                    "name": "Read",
                    "input": {"file_path": "/test/file.txt"}
                }
            ],
            "model": "claude-sonnet-4"
        }
    });

    let result = parse_message(data).unwrap();
    match result {
        Message::Assistant { message, .. } => {
            match &message.message.content[0] {
                ContentBlock::ToolUse { id, name, input } => {
                    assert_eq!(id, "tool_123");
                    assert_eq!(name, "Read");
                    assert_eq!(input["file_path"], "/test/file.txt");
                }
                _ => panic!("Expected ToolUseBlock"),
            }
        }
        _ => panic!("Expected Assistant message"),
    }
}

#[test]
fn test_parse_tool_result_block() {
    let data = json!({
        "type": "assistant",
        "message": {
            "content": [
                {
                    "type": "tool_result",
                    "tool_use_id": "tool_123",
                    "content": "File contents here",
                    "is_error": false
                }
            ],
            "model": "claude-sonnet-4"
        }
    });

    let result = parse_message(data).unwrap();
    match result {
        Message::Assistant { message, .. } => {
            match &message.message.content[0] {
                ContentBlock::ToolResult {
                    tool_use_id,
                    content,
                    is_error,
                } => {
                    assert_eq!(tool_use_id, "tool_123");
                    assert_eq!(content.as_ref().unwrap(), "File contents here");
                    assert_eq!(is_error, &Some(false));
                }
                _ => panic!("Expected ToolResultBlock"),
            }
        }
        _ => panic!("Expected Assistant message"),
    }
}

#[test]
fn test_parse_user_message() {
    let data = json!({
        "type": "user",
        "message": {
            "role": "user",
            "content": "Hello Claude!"
        },
        "parent_tool_use_id": null
    });

    let result = parse_message(data).unwrap();
    match result {
        Message::User {
            message,
            parent_tool_use_id,
        } => {
            assert_eq!(message.message.role, "user");
            assert_eq!(
                message.message.content.as_ref().unwrap(),
                "Hello Claude!"
            );
            assert_eq!(parent_tool_use_id, None);
        }
        _ => panic!("Expected User message"),
    }
}

#[test]
fn test_parse_system_message() {
    let data = json!({
        "type": "system",
        "subtype": "info",
        "message": "System notification"
    });

    let result = parse_message(data).unwrap();
    match result {
        Message::System { subtype, data } => {
            assert_eq!(subtype, "info");
            assert_eq!(data["message"], "System notification");
        }
        _ => panic!("Expected System message"),
    }
}

#[test]
fn test_parse_result_message() {
    let data = json!({
        "type": "result",
        "subtype": "complete",
        "duration_ms": 1500,
        "duration_api_ms": 1000,
        "is_error": false,
        "num_turns": 3,
        "session_id": "session_123",
        "total_cost_usd": 0.002,
        "usage": {
            "input_tokens": 100,
            "output_tokens": 50
        }
    });

    let result = parse_message(data).unwrap();
    match result {
        Message::Result {
            subtype,
            duration_ms,
            duration_api_ms,
            is_error,
            num_turns,
            session_id,
            total_cost_usd,
            usage,
            ..
        } => {
            assert_eq!(subtype, "complete");
            assert_eq!(duration_ms, 1500);
            assert_eq!(duration_api_ms, 1000);
            assert!(!is_error);
            assert_eq!(num_turns, 3);
            assert_eq!(session_id, "session_123");
            assert_eq!(total_cost_usd, Some(0.002));
            assert!(usage.is_some());
        }
        _ => panic!("Expected Result message"),
    }
}

#[test]
fn test_parse_stream_event() {
    let data = json!({
        "type": "stream_event",
        "uuid": "event_123",
        "session_id": "session_456",
        "event": {
            "type": "content_block_delta",
            "index": 0,
            "delta": {"type": "text_delta", "text": "Hello"}
        },
        "parent_tool_use_id": null
    });

    let result = parse_message(data).unwrap();
    match result {
        Message::StreamEvent {
            uuid,
            session_id,
            event,
            parent_tool_use_id,
        } => {
            assert_eq!(uuid, "event_123");
            assert_eq!(session_id, "session_456");
            assert_eq!(event["type"], "content_block_delta");
            assert_eq!(parent_tool_use_id, None);
        }
        _ => panic!("Expected StreamEvent message"),
    }
}

#[test]
fn test_parse_multiple_content_blocks() {
    let data = json!({
        "type": "assistant",
        "message": {
            "content": [
                {
                    "type": "text",
                    "text": "Let me read that file."
                },
                {
                    "type": "tool_use",
                    "id": "tool_456",
                    "name": "Read",
                    "input": {"file_path": "/test.txt"}
                },
                {
                    "type": "text",
                    "text": "Here's what I found."
                }
            ],
            "model": "claude-sonnet-4"
        }
    });

    let result = parse_message(data).unwrap();
    match result {
        Message::Assistant { message, .. } => {
            assert_eq!(message.message.content.len(), 3);
            assert!(matches!(
                message.message.content[0],
                ContentBlock::Text { .. }
            ));
            assert!(matches!(
                message.message.content[1],
                ContentBlock::ToolUse { .. }
            ));
            assert!(matches!(
                message.message.content[2],
                ContentBlock::Text { .. }
            ));
        }
        _ => panic!("Expected Assistant message"),
    }
}

#[test]
fn test_parse_invalid_message_type() {
    let data = json!({
        "type": "unknown_type",
        "data": "test"
    });

    let result = parse_message(data);
    assert!(result.is_err());
}

#[test]
fn test_parse_missing_required_field() {
    let data = json!({
        "type": "assistant",
        "message": {
            "content": []
            // Missing "model" field
        }
    });

    let result = parse_message(data);
    assert!(result.is_err());
}

#[test]
fn test_parse_invalid_content_block_type() {
    let data = json!({
        "type": "assistant",
        "message": {
            "content": [
                {
                    "type": "invalid_block",
                    "data": "test"
                }
            ],
            "model": "claude-sonnet-4"
        }
    });

    let result = parse_message(data);
    assert!(result.is_err());
}
