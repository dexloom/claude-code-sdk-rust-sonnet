//! Tests for transport layer

use claude_agent_sdk::errors::ClaudeSDKError;
use claude_agent_sdk::transport::Transport;
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use serde_json::{json, Value};
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex};

/// Mock transport for testing
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

    pub fn get_written_data(&self) -> Vec<String> {
        self.written_data.lock().unwrap().clone()
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
        if !*self.ready.lock().unwrap() {
            return Err(ClaudeSDKError::transport("Not ready"));
        }
        self.written_data.lock().unwrap().push(data);
        Ok(())
    }

    fn read_messages(&mut self) -> Pin<Box<dyn Stream<Item = claude_agent_sdk::errors::Result<Value>> + Send + '_>> {
        let messages = self.messages.lock().unwrap().clone();
        Box::pin(futures::stream::iter(
            messages.into_iter().map(Ok)
        ))
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
async fn test_mock_transport_connect() {
    let mut transport = MockTransport::new(vec![]);
    assert!(!transport.is_ready());

    transport.connect().await.unwrap();
    assert!(transport.is_ready());
}

#[tokio::test]
async fn test_mock_transport_write() {
    let mut transport = MockTransport::new(vec![]);
    transport.connect().await.unwrap();

    transport.write("test message\n".to_string()).await.unwrap();

    let written = transport.get_written_data();
    assert_eq!(written.len(), 1);
    assert_eq!(written[0], "test message\n");
}

#[tokio::test]
async fn test_mock_transport_write_before_connect() {
    let mut transport = MockTransport::new(vec![]);

    let result = transport.write("test\n".to_string()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mock_transport_read_messages() {
    let messages = vec![
        json!({"type": "user", "message": {"role": "user", "content": "Hello"}}),
        json!({"type": "assistant", "message": {"content": [{"type": "text", "text": "Hi"}], "model": "claude-sonnet-4"}}),
    ];

    let mut transport = MockTransport::new(messages.clone());
    transport.connect().await.unwrap();

    let mut stream = transport.read_messages();

    let msg1 = stream.next().await.unwrap().unwrap();
    assert_eq!(msg1["type"], "user");

    let msg2 = stream.next().await.unwrap().unwrap();
    assert_eq!(msg2["type"], "assistant");

    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn test_mock_transport_close() {
    let mut transport = MockTransport::new(vec![]);
    transport.connect().await.unwrap();
    assert!(transport.is_ready());

    transport.close().await.unwrap();
    assert!(!transport.is_ready());
}

#[tokio::test]
async fn test_mock_transport_multiple_writes() {
    let mut transport = MockTransport::new(vec![]);
    transport.connect().await.unwrap();

    transport.write("message 1\n".to_string()).await.unwrap();
    transport.write("message 2\n".to_string()).await.unwrap();
    transport.write("message 3\n".to_string()).await.unwrap();

    let written = transport.get_written_data();
    assert_eq!(written.len(), 3);
    assert_eq!(written[0], "message 1\n");
    assert_eq!(written[1], "message 2\n");
    assert_eq!(written[2], "message 3\n");
}

#[tokio::test]
async fn test_mock_transport_end_input() {
    let mut transport = MockTransport::new(vec![]);
    transport.connect().await.unwrap();

    let result = transport.end_input().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_mock_transport_json_messages() {
    let messages = vec![
        json!({
            "type": "result",
            "subtype": "complete",
            "duration_ms": 1000,
            "duration_api_ms": 500,
            "is_error": false,
            "num_turns": 1,
            "session_id": "test"
        }),
    ];

    let mut transport = MockTransport::new(messages);
    transport.connect().await.unwrap();

    let mut stream = transport.read_messages();
    let msg = stream.next().await.unwrap().unwrap();

    assert_eq!(msg["type"], "result");
    assert_eq!(msg["subtype"], "complete");
    assert_eq!(msg["duration_ms"], 1000);
}

#[tokio::test]
async fn test_transport_trait_bounds() {
    fn assert_send<T: Send>() {}
    assert_send::<MockTransport>();
}

#[tokio::test]
async fn test_transport_lifecycle() {
    let mut transport = MockTransport::new(vec![
        json!({"type": "user", "message": {"role": "user", "content": "test"}}),
    ]);

    // Initial state
    assert!(!transport.is_ready());

    // Connect
    transport.connect().await.unwrap();
    assert!(transport.is_ready());

    // Write
    transport.write("data\n".to_string()).await.unwrap();

    // Read
    {
        let mut stream = transport.read_messages();
        assert!(stream.next().await.is_some());
    }

    // Close
    transport.close().await.unwrap();
    assert!(!transport.is_ready());
}
