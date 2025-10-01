//! ClaudeSDKClient for bidirectional conversations with Claude Code.

use crate::errors::Result;
use crate::message_parser::parse_message;
use crate::query::Query;
use crate::transport::subprocess::SubprocessCLITransport;
use crate::types::{ClaudeAgentOptions, Message};
use futures::stream::{Stream, StreamExt};
use std::pin::Pin;

/// Client for bidirectional, interactive conversations with Claude Code.
pub struct ClaudeSDKClient {
    options: ClaudeAgentOptions,
    query: Option<Query>,
}

impl ClaudeSDKClient {
    /// Create a new ClaudeSDKClient with the given options.
    pub fn new(options: ClaudeAgentOptions) -> Self {
        Self { options, query: None }
    }

    /// Connect to Claude Code and start the session.
    pub async fn connect(&mut self) -> Result<()> {
        let transport = SubprocessCLITransport::new(self.options.clone(), true)?;
        let mut boxed_transport = Box::new(transport) as Box<dyn crate::transport::Transport>;
        boxed_transport.connect().await?;

        let can_use_tool = self.options.can_use_tool.clone();

        let mut query = Query::new(boxed_transport, true, can_use_tool, None);
        query.start().await?;
        query.initialize().await?;

        self.query = Some(query);
        Ok(())
    }

    /// Send a query/prompt to Claude.
    pub async fn query(&self, prompt: String) -> Result<()> {
        if let Some(ref query) = self.query {
            let message = serde_json::json!({
                "type": "user",
                "message": {
                    "role": "user",
                    "content": prompt
                },
                "parent_tool_use_id": null,
                "session_id": "default"
            });

            let mut transport = query.transport.lock().await;
            transport.write(format!("{}\n", serde_json::to_string(&message)?)).await?;
            Ok(())
        } else {
            Err(crate::errors::ClaudeSDKError::cli_connection(
                "Not connected. Call connect() first.",
            ))
        }
    }

    /// Receive messages from Claude.
    pub fn receive_messages(&mut self) -> Pin<Box<dyn Stream<Item = Result<Message>> + Send + '_>> {
        if let Some(ref mut query) = self.query {
            Box::pin(query.receive_messages().filter_map(|result| async move {
                match result {
                    Ok(value) => Some(parse_message(value)),
                    Err(e) => Some(Err(e)),
                }
            }))
        } else {
            Box::pin(futures::stream::empty())
        }
    }

    /// Receive messages until a Result message is received.
    pub fn receive_response(&mut self) -> Pin<Box<dyn Stream<Item = Result<Message>> + Send + '_>> {
        Box::pin(self.receive_messages().take_while(|msg| {
            let is_result = matches!(msg, Ok(Message::Result { .. }));
            futures::future::ready(!is_result)
        }))
    }

    /// Send an interrupt signal.
    pub async fn interrupt(&self) -> Result<()> {
        if let Some(ref query) = self.query {
            query.interrupt().await
        } else {
            Err(crate::errors::ClaudeSDKError::cli_connection(
                "Not connected. Call connect() first.",
            ))
        }
    }

    /// Change permission mode during conversation.
    pub async fn set_permission_mode(&self, mode: String) -> Result<()> {
        if let Some(ref query) = self.query {
            query.set_permission_mode(mode).await
        } else {
            Err(crate::errors::ClaudeSDKError::cli_connection(
                "Not connected. Call connect() first.",
            ))
        }
    }

    /// Disconnect from Claude.
    pub async fn disconnect(&self) -> Result<()> {
        if let Some(ref query) = self.query {
            query.close().await
        } else {
            Ok(())
        }
    }
}

impl Drop for ClaudeSDKClient {
    fn drop(&mut self) {
        // Cleanup will happen through Drop impls of child structs
    }
}
