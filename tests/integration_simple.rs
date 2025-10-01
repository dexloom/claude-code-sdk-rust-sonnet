//! Simple integration tests without full Query lifecycle
//! These tests verify the components work together without spawning background tasks

use claude_agent_sdk::errors::Result;
use claude_agent_sdk::message_parser::parse_message;
use claude_agent_sdk::types::{ClaudeAgentOptions, ContentBlock, Message};
use serde_json::json;

#[test]
fn test_parse_complete_workflow() {
    let messages = vec![
        json!({
            "type": "assistant",
            "message": {
                "content": [
                    {"type": "text", "text": "Let me read that file."},
                    {
                        "type": "tool_use",
                        "id": "tool_123",
                        "name": "Read",
                        "input": {"file_path": "/test.txt"}
                    }
                ],
                "model": "claude-sonnet-4"
            }
        }),
        json!({
            "type": "result",
            "subtype": "complete",
            "duration_ms": 1000,
            "duration_api_ms": 500,
            "is_error": false,
            "num_turns": 2,
            "session_id": "test_session"
        }),
    ];

    // Parse all messages
    let parsed: Vec<Message> = messages
        .into_iter()
        .map(|m| parse_message(m).unwrap())
        .collect();

    assert_eq!(parsed.len(), 2);

    // Verify first message is assistant with tool use
    match &parsed[0] {
        Message::Assistant { message, .. } => {
            assert_eq!(message.message.content.len(), 2);
            assert!(matches!(message.message.content[1], ContentBlock::ToolUse { .. }));
        }
        _ => panic!("Expected Assistant message"),
    }

    // Verify second message is result
    match &parsed[1] {
        Message::Result { session_id, is_error, .. } => {
            assert_eq!(session_id, "test_session");
            assert!(!is_error);
        }
        _ => panic!("Expected Result message"),
    }
}

#[test]
fn test_options_with_tools() {
    let options = ClaudeAgentOptions {
        allowed_tools: vec!["Read".to_string(), "Write".to_string(), "Bash".to_string()],
        permission_mode: Some("acceptEdits".to_string()),
        max_turns: Some(10),
        ..Default::default()
    };

    assert_eq!(options.allowed_tools.len(), 3);
    assert!(options.allowed_tools.contains(&"Read".to_string()));
    assert!(options.allowed_tools.contains(&"Write".to_string()));
    assert!(options.allowed_tools.contains(&"Bash".to_string()));
    assert_eq!(options.permission_mode.as_ref().unwrap(), "acceptEdits");
}

#[test]
fn test_multi_turn_conversation() {
    let turns = vec![
        json!({
            "type": "user",
            "message": {"role": "user", "content": "Hello"},
            "parent_tool_use_id": null
        }),
        json!({
            "type": "assistant",
            "message": {
                "content": [{"type": "text", "text": "Hi! How can I help?"}],
                "model": "claude-sonnet-4"
            }
        }),
        json!({
            "type": "user",
            "message": {"role": "user", "content": "What's 2+2?"},
            "parent_tool_use_id": null
        }),
        json!({
            "type": "assistant",
            "message": {
                "content": [{"type": "text", "text": "2+2 equals 4."}],
                "model": "claude-sonnet-4"
            }
        }),
    ];

    let messages: Vec<Message> = turns
        .into_iter()
        .map(|t| parse_message(t).unwrap())
        .collect();

    assert_eq!(messages.len(), 4);

    let mut user_count = 0;
    let mut assistant_count = 0;

    for msg in messages {
        match msg {
            Message::User { .. } => user_count += 1,
            Message::Assistant { .. } => assistant_count += 1,
            _ => {}
        }
    }

    assert_eq!(user_count, 2);
    assert_eq!(assistant_count, 2);
}

#[test]
fn test_error_handling_workflow() {
    let error_result = json!({
        "type": "result",
        "subtype": "error",
        "duration_ms": 500,
        "duration_api_ms": 200,
        "is_error": true,
        "num_turns": 0,
        "session_id": "error_session",
        "result": "Command failed"
    });

    let message = parse_message(error_result).unwrap();

    match message {
        Message::Result { is_error, result, session_id, .. } => {
            assert!(is_error);
            assert_eq!(result.as_ref().unwrap(), "Command failed");
            assert_eq!(session_id, "error_session");
        }
        _ => panic!("Expected Result message"),
    }
}

#[test]
fn test_thinking_and_response_workflow() {
    let messages = vec![
        json!({
            "type": "assistant",
            "message": {
                "content": [
                    {
                        "type": "thinking",
                        "thinking": "I need to analyze this carefully...",
                        "signature": "sig_abc"
                    },
                    {
                        "type": "text",
                        "text": "Here's my analysis:"
                    }
                ],
                "model": "claude-sonnet-4"
            }
        }),
    ];

    let parsed: Vec<Message> = messages
        .into_iter()
        .map(|m| parse_message(m).unwrap())
        .collect();

    match &parsed[0] {
        Message::Assistant { message, .. } => {
            assert_eq!(message.message.content.len(), 2);
            assert!(matches!(message.message.content[0], ContentBlock::Thinking { .. }));
            assert!(matches!(message.message.content[1], ContentBlock::Text { .. }));
        }
        _ => panic!("Expected Assistant message"),
    }
}
