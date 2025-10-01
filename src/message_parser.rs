//! Message parser for Claude Code SDK responses.

use crate::errors::{ClaudeSDKError, Result};
use crate::types::{AssistantMessageContent, AssistantMessageInner, ContentBlock, Message, UserMessageContent, UserMessageInner};
use serde_json::Value;

/// Parse a message from CLI output into a typed Message object.
pub fn parse_message(data: Value) -> Result<Message> {
    let obj = data
        .as_object()
        .ok_or_else(|| ClaudeSDKError::message_parse("Expected message to be an object", Some(data.clone())))?;

    let message_type = obj
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ClaudeSDKError::message_parse("Message missing 'type' field", Some(data.clone())))?;

    match message_type {
        "user" => parse_user_message(&data),
        "assistant" => parse_assistant_message(&data),
        "system" => parse_system_message(&data),
        "result" => parse_result_message(&data),
        "stream_event" => parse_stream_event(&data),
        _ => Err(ClaudeSDKError::message_parse(
            format!("Unknown message type: {}", message_type),
            Some(data),
        )),
    }
}

fn parse_user_message(data: &Value) -> Result<Message> {
    let obj = data.as_object().ok_or_else(|| {
        ClaudeSDKError::message_parse("User message must be an object", Some(data.clone()))
    })?;

    let parent_tool_use_id = obj.get("parent_tool_use_id").and_then(|v| v.as_str()).map(String::from);

    let message_obj = obj
        .get("message")
        .and_then(|v| v.as_object())
        .ok_or_else(|| ClaudeSDKError::message_parse("User message missing 'message' field", Some(data.clone())))?;

    let role = message_obj
        .get("role")
        .and_then(|v| v.as_str())
        .unwrap_or("user")
        .to_string();

    let content = message_obj.get("content").cloned();

    Ok(Message::User {
        message: UserMessageContent {
            message: UserMessageInner { role, content },
        },
        parent_tool_use_id,
    })
}

fn parse_assistant_message(data: &Value) -> Result<Message> {
    let obj = data.as_object().ok_or_else(|| {
        ClaudeSDKError::message_parse("Assistant message must be an object", Some(data.clone()))
    })?;

    let parent_tool_use_id = obj.get("parent_tool_use_id").and_then(|v| v.as_str()).map(String::from);

    let message_obj = obj
        .get("message")
        .and_then(|v| v.as_object())
        .ok_or_else(|| ClaudeSDKError::message_parse("Assistant message missing 'message' field", Some(data.clone())))?;

    let model = message_obj
        .get("model")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ClaudeSDKError::message_parse("Assistant message missing 'model' field", Some(data.clone())))?
        .to_string();

    let content_array = message_obj
        .get("content")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ClaudeSDKError::message_parse("Assistant message missing 'content' array", Some(data.clone())))?;

    let mut content_blocks = Vec::new();
    for block in content_array {
        content_blocks.push(parse_content_block(block)?);
    }

    Ok(Message::Assistant {
        message: AssistantMessageContent {
            message: AssistantMessageInner { content: content_blocks, model },
        },
        parent_tool_use_id,
    })
}

fn parse_content_block(block: &Value) -> Result<ContentBlock> {
    let obj = block
        .as_object()
        .ok_or_else(|| ClaudeSDKError::message_parse("Content block must be an object", Some(block.clone())))?;

    let block_type = obj
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ClaudeSDKError::message_parse("Content block missing 'type' field", Some(block.clone())))?;

    match block_type {
        "text" => {
            let text = obj
                .get("text")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ClaudeSDKError::message_parse("Text block missing 'text' field", Some(block.clone())))?
                .to_string();
            Ok(ContentBlock::Text { text })
        }
        "thinking" => {
            let thinking = obj
                .get("thinking")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ClaudeSDKError::message_parse("Thinking block missing 'thinking' field", Some(block.clone())))?
                .to_string();
            let signature = obj
                .get("signature")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ClaudeSDKError::message_parse("Thinking block missing 'signature' field", Some(block.clone())))?
                .to_string();
            Ok(ContentBlock::Thinking { thinking, signature })
        }
        "tool_use" => {
            let id = obj
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ClaudeSDKError::message_parse("Tool use block missing 'id' field", Some(block.clone())))?
                .to_string();
            let name = obj
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ClaudeSDKError::message_parse("Tool use block missing 'name' field", Some(block.clone())))?
                .to_string();
            let input = obj
                .get("input")
                .ok_or_else(|| ClaudeSDKError::message_parse("Tool use block missing 'input' field", Some(block.clone())))?
                .clone();
            Ok(ContentBlock::ToolUse { id, name, input })
        }
        "tool_result" => {
            let tool_use_id = obj
                .get("tool_use_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ClaudeSDKError::message_parse("Tool result block missing 'tool_use_id' field", Some(block.clone())))?
                .to_string();
            let content = obj.get("content").cloned();
            let is_error = obj.get("is_error").and_then(|v| v.as_bool());
            Ok(ContentBlock::ToolResult {
                tool_use_id,
                content,
                is_error,
            })
        }
        _ => Err(ClaudeSDKError::message_parse(
            format!("Unknown content block type: {}", block_type),
            Some(block.clone()),
        )),
    }
}

fn parse_system_message(data: &Value) -> Result<Message> {
    let obj = data.as_object().ok_or_else(|| {
        ClaudeSDKError::message_parse("System message must be an object", Some(data.clone()))
    })?;

    let subtype = obj
        .get("subtype")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ClaudeSDKError::message_parse("System message missing 'subtype' field", Some(data.clone())))?
        .to_string();

    Ok(Message::System {
        subtype,
        data: data.clone(),
    })
}

fn parse_result_message(data: &Value) -> Result<Message> {
    let obj = data.as_object().ok_or_else(|| {
        ClaudeSDKError::message_parse("Result message must be an object", Some(data.clone()))
    })?;

    let subtype = obj
        .get("subtype")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ClaudeSDKError::message_parse("Result message missing 'subtype' field", Some(data.clone())))?
        .to_string();

    let duration_ms = obj
        .get("duration_ms")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| ClaudeSDKError::message_parse("Result message missing 'duration_ms' field", Some(data.clone())))?;

    let duration_api_ms = obj
        .get("duration_api_ms")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| ClaudeSDKError::message_parse("Result message missing 'duration_api_ms' field", Some(data.clone())))?;

    let is_error = obj
        .get("is_error")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| ClaudeSDKError::message_parse("Result message missing 'is_error' field", Some(data.clone())))?;

    let num_turns = obj
        .get("num_turns")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| ClaudeSDKError::message_parse("Result message missing 'num_turns' field", Some(data.clone())))?
        as i32;

    let session_id = obj
        .get("session_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ClaudeSDKError::message_parse("Result message missing 'session_id' field", Some(data.clone())))?
        .to_string();

    let total_cost_usd = obj.get("total_cost_usd").and_then(|v| v.as_f64());
    let usage = obj.get("usage").cloned();
    let result = obj.get("result").and_then(|v| v.as_str()).map(String::from);

    Ok(Message::Result {
        subtype,
        duration_ms,
        duration_api_ms,
        is_error,
        num_turns,
        session_id,
        total_cost_usd,
        usage,
        result,
    })
}

fn parse_stream_event(data: &Value) -> Result<Message> {
    let obj = data.as_object().ok_or_else(|| {
        ClaudeSDKError::message_parse("Stream event must be an object", Some(data.clone()))
    })?;

    let uuid = obj
        .get("uuid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ClaudeSDKError::message_parse("Stream event missing 'uuid' field", Some(data.clone())))?
        .to_string();

    let session_id = obj
        .get("session_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ClaudeSDKError::message_parse("Stream event missing 'session_id' field", Some(data.clone())))?
        .to_string();

    let event = obj
        .get("event")
        .ok_or_else(|| ClaudeSDKError::message_parse("Stream event missing 'event' field", Some(data.clone())))?
        .clone();

    let parent_tool_use_id = obj.get("parent_tool_use_id").and_then(|v| v.as_str()).map(String::from);

    Ok(Message::StreamEvent {
        uuid,
        session_id,
        event,
        parent_tool_use_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_text_block() {
        let block = json!({
            "type": "text",
            "text": "Hello, world!"
        });
        let result = parse_content_block(&block).unwrap();
        match result {
            ContentBlock::Text { text } => assert_eq!(text, "Hello, world!"),
            _ => panic!("Expected TextBlock"),
        }
    }

    #[test]
    fn test_parse_result_message() {
        let data = json!({
            "type": "result",
            "subtype": "complete",
            "duration_ms": 1000,
            "duration_api_ms": 500,
            "is_error": false,
            "num_turns": 2,
            "session_id": "test-session",
            "total_cost_usd": 0.001
        });
        let result = parse_message(data).unwrap();
        match result {
            Message::Result {
                subtype,
                duration_ms,
                session_id,
                ..
            } => {
                assert_eq!(subtype, "complete");
                assert_eq!(duration_ms, 1000);
                assert_eq!(session_id, "test-session");
            }
            _ => panic!("Expected Result message"),
        }
    }
}
