//! Integration tests for the Claude Agent SDK
//!
//! Note: These tests use mock transports and don't require the Claude CLI to be installed.
//! For end-to-end tests with the actual CLI, see the examples directory.

use claude_agent_sdk::errors::Result;
use claude_agent_sdk::message_parser::parse_message;
use claude_agent_sdk::query::Query;
use claude_agent_sdk::types::{ClaudeAgentOptions, ContentBlock, Message};
use claude_agent_sdk::transport::Transport;
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use serde_json::{json, Value};
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex};

// Include mock transport from test_transport
pub struct MockTransport {
    messages: Arc<StdMutex<Vec<Value>>>,
    written_data: Arc<StdMutex<Vec<String>>>,
    connected: Arc<StdMutex<bool>>,
    ready: Arc<StdMutex<bool>>,
}

impl MockTransport {
    pub fn new(messages: Vec<Value>) -> Self {
        Self {
            messages: Arc::new(StdMutex::new(messages)),
            written_data: Arc::new(StdMutex::new(Vec::new())),
            connected: Arc::new(StdMutex::new(false)),
            ready: Arc::new(StdMutex::new(false)),
        }
    }
}

#[async_trait]
impl Transport for MockTransport {
    async fn connect(&mut self) -> claude_agent_sdk::errors::Result<()> {
        *self.connected.lock().unwrap() = true;
        *self.ready.lock().unwrap() = true;
        Ok(())
    }

    async fn write(&mut self, data: String) -> claude_agent_sdk::errors::Result<()> {
        self.written_data.lock().unwrap().push(data);
        Ok(())
    }

    fn read_messages(&mut self) -> Pin<Box<dyn Stream<Item = claude_agent_sdk::errors::Result<Value>> + Send + '_>> {
        let messages = self.messages.lock().unwrap().clone();
        Box::pin(futures::stream::iter(messages.into_iter().map(Ok)))
    }

    async fn close(&mut self) -> claude_agent_sdk::errors::Result<()> {
        *self.connected.lock().unwrap() = false;
        *self.ready.lock().unwrap() = false;
        Ok(())
    }

    fn is_ready(&self) -> bool {
        *self.ready.lock().unwrap()
    }

    async fn end_input(&mut self) -> claude_agent_sdk::errors::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_query_with_mock_transport() {
    let messages = vec![
        json!({
            "type": "assistant",
            "message": {
                "content": [
                    {"type": "text", "text": "Hello!"}
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
            "num_turns": 1,
            "session_id": "test_session"
        }),
    ];

    let transport = MockTransport::new(messages);
    let mut boxed_transport = Box::new(transport) as Box<dyn claude_agent_sdk::transport::Transport>;
    boxed_transport.connect().await.unwrap();

    let mut query = Query::new(boxed_transport, false, None, None);
    query.start().await.unwrap();

    let mut stream = query.receive_messages();
    let mut message_count = 0;

    while let Some(result) = stream.next().await {
        let value = result.unwrap();
        let message = parse_message(value).unwrap();

        match message {
            Message::Assistant { message, .. } => {
                assert!(!message.message.content.is_empty());
                message_count += 1;
            }
            Message::Result { session_id, .. } => {
                assert_eq!(session_id, "test_session");
                message_count += 1;
            }
            _ => {}
        }
    }

    assert_eq!(message_count, 2);
}

#[tokio::test]
async fn test_streaming_conversation() {
    let messages = vec![
        json!({
            "type": "assistant",
            "message": {
                "content": [
                    {"type": "text", "text": "First response"}
                ],
                "model": "claude-sonnet-4"
            }
        }),
        json!({
            "type": "result",
            "subtype": "complete",
            "duration_ms": 500,
            "duration_api_ms": 250,
            "is_error": false,
            "num_turns": 1,
            "session_id": "session_1"
        }),
        json!({
            "type": "assistant",
            "message": {
                "content": [
                    {"type": "text", "text": "Second response"}
                ],
                "model": "claude-sonnet-4"
            }
        }),
        json!({
            "type": "result",
            "subtype": "complete",
            "duration_ms": 600,
            "duration_api_ms": 300,
            "is_error": false,
            "num_turns": 2,
            "session_id": "session_1"
        }),
    ];

    let transport = MockTransport::new(messages);
    let mut boxed_transport = Box::new(transport) as Box<dyn claude_agent_sdk::transport::Transport>;
    boxed_transport.connect().await.unwrap();

    let mut query = Query::new(boxed_transport, true, None, None);
    query.start().await.unwrap();
    query.initialize().await.unwrap();

    let mut stream = query.receive_messages();
    let mut responses = Vec::new();

    while let Some(result) = stream.next().await {
        let value = result.unwrap();
        let message = parse_message(value).unwrap();
        responses.push(message);
    }

    assert_eq!(responses.len(), 4);
}

#[tokio::test]
async fn test_tool_use_workflow() {
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
        }),
        json!({
            "type": "assistant",
            "message": {
                "content": [
                    {"type": "text", "text": "Here's what the file contains."}
                ],
                "model": "claude-sonnet-4"
            }
        }),
        json!({
            "type": "result",
            "subtype": "complete",
            "duration_ms": 2000,
            "duration_api_ms": 1500,
            "is_error": false,
            "num_turns": 2,
            "session_id": "tool_session"
        }),
    ];

    let transport = MockTransport::new(messages);
    let mut boxed_transport = Box::new(transport) as Box<dyn claude_agent_sdk::transport::Transport>;
    boxed_transport.connect().await.unwrap();

    let mut query = Query::new(boxed_transport, false, None, None);
    query.start().await.unwrap();

    let mut stream = query.receive_messages();
    let mut has_tool_use = false;
    let mut has_tool_result = false;

    while let Some(result) = stream.next().await {
        let value = result.unwrap();
        let message = parse_message(value).unwrap();

        if let Message::Assistant { message, .. } = message {
            for block in message.message.content {
                match block {
                    ContentBlock::ToolUse { .. } => has_tool_use = true,
                    ContentBlock::ToolResult { .. } => has_tool_result = true,
                    _ => {}
                }
            }
        }
    }

    assert!(has_tool_use);
    assert!(has_tool_result);
}

#[tokio::test]
async fn test_error_message_handling() {
    let messages = vec![
        json!({
            "type": "result",
            "subtype": "error",
            "duration_ms": 100,
            "duration_api_ms": 50,
            "is_error": true,
            "num_turns": 0,
            "session_id": "error_session",
            "result": "An error occurred"
        }),
    ];

    let transport = MockTransport::new(messages);
    let mut boxed_transport = Box::new(transport) as Box<dyn claude_agent_sdk::transport::Transport>;
    boxed_transport.connect().await.unwrap();

    let mut query = Query::new(boxed_transport, false, None, None);
    query.start().await.unwrap();

    let mut stream = query.receive_messages();
    let result_msg = stream.next().await.unwrap().unwrap();
    let message = parse_message(result_msg).unwrap();

    match message {
        Message::Result { is_error, result, .. } => {
            assert!(is_error);
            assert_eq!(result, Some("An error occurred".to_string()));
        }
        _ => panic!("Expected Result message"),
    }
}

#[tokio::test]
async fn test_system_message_handling() {
    let messages = vec![
        json!({
            "type": "system",
            "subtype": "info",
            "message": "System information"
        }),
        json!({
            "type": "assistant",
            "message": {
                "content": [
                    {"type": "text", "text": "Response"}
                ],
                "model": "claude-sonnet-4"
            }
        }),
        json!({
            "type": "result",
            "subtype": "complete",
            "duration_ms": 500,
            "duration_api_ms": 250,
            "is_error": false,
            "num_turns": 1,
            "session_id": "sys_session"
        }),
    ];

    let transport = MockTransport::new(messages);
    let mut boxed_transport = Box::new(transport) as Box<dyn claude_agent_sdk::transport::Transport>;
    boxed_transport.connect().await.unwrap();

    let mut query = Query::new(boxed_transport, false, None, None);
    query.start().await.unwrap();

    let mut stream = query.receive_messages();
    let mut has_system_message = false;

    while let Some(result) = stream.next().await {
        let value = result.unwrap();
        let message = parse_message(value).unwrap();

        if matches!(message, Message::System { .. }) {
            has_system_message = true;
        }
    }

    assert!(has_system_message);
}

#[tokio::test]
async fn test_options_configuration() {
    let options = ClaudeAgentOptions {
        allowed_tools: vec!["Read".to_string(), "Write".to_string()],
        permission_mode: Some("acceptEdits".to_string()),
        max_turns: Some(5),
        ..Default::default()
    };

    assert_eq!(options.allowed_tools.len(), 2);
    assert_eq!(options.permission_mode.as_ref().unwrap(), "acceptEdits");
    assert_eq!(options.max_turns, Some(5));
}

#[tokio::test]
async fn test_concurrent_message_processing() {
    let messages = vec![
        json!({"type": "assistant", "message": {"content": [{"type": "text", "text": "1"}], "model": "claude-sonnet-4"}}),
        json!({"type": "assistant", "message": {"content": [{"type": "text", "text": "2"}], "model": "claude-sonnet-4"}}),
        json!({"type": "assistant", "message": {"content": [{"type": "text", "text": "3"}], "model": "claude-sonnet-4"}}),
        json!({"type": "result", "subtype": "complete", "duration_ms": 1000, "duration_api_ms": 500, "is_error": false, "num_turns": 3, "session_id": "concurrent"}),
    ];

    let transport = MockTransport::new(messages);
    let mut boxed_transport = Box::new(transport) as Box<dyn claude_agent_sdk::transport::Transport>;
    boxed_transport.connect().await.unwrap();

    let mut query = Query::new(boxed_transport, false, None, None);
    query.start().await.unwrap();

    let stream = query.receive_messages();
    let results: Vec<_> = stream.collect().await;

    assert_eq!(results.len(), 4);
    assert!(results.iter().all(|r| r.is_ok()));
}
