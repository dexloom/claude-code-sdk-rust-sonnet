//! # Claude Agent SDK for Rust
//!
//! Rust SDK for interacting with Claude Code. This SDK provides both high-level
//! and low-level APIs for building applications that leverage Claude Code's capabilities.
//!
//! ## Quick Start
//!
//! ```no_run
//! use claude_agent_sdk::{query, ClaudeAgentOptions};
//! use futures::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let options = ClaudeAgentOptions::default();
//!     let mut stream = query("What is 2 + 2?".to_string(), options).await?;
//!
//!     while let Some(message) = stream.next().await {
//!         println!("{:?}", message?);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! - **Simple Query API**: One-shot queries with the `query()` function
//! - **Interactive Client**: Bidirectional communication with `ClaudeSDKClient`
//! - **Tool Permissions**: Fine-grained control over tool execution
//! - **Hooks**: Intercept and modify behavior at key points
//! - **MCP Support**: Integration with Model Context Protocol servers
//! - **Type Safety**: Strong typing with serde serialization

pub mod client;
pub mod errors;
pub mod mcp;
pub mod message_parser;
pub mod query;
pub mod transport;
pub mod types;

// Re-export main types
pub use client::ClaudeSDKClient;
pub use errors::{ClaudeSDKError, Result};
pub use mcp::{create_mcp_server, McpTool, SdkMcpServer, ToolParameter};
pub use types::{
    AgentDefinition, ClaudeAgentOptions, ContentBlock, HookCallback, HookContext, HookJSONOutput, HookMatcher,
    McpServerConfig, Message, PermissionMode, PermissionResult, PermissionUpdate, SettingSource, SystemPrompt,
    ToolPermissionContext,
};

use futures::stream::{Stream, StreamExt};
use message_parser::parse_message;
use std::pin::Pin;
use transport::subprocess::SubprocessCLITransport;

/// Query Claude Code for one-shot or unidirectional streaming interactions.
///
/// This function is ideal for simple, stateless queries where you don't need
/// bidirectional communication or conversation management.
///
/// # Arguments
///
/// * `prompt` - The prompt to send to Claude
/// * `options` - Configuration options for the query
///
/// # Returns
///
/// A stream of `Message` objects representing the conversation
///
/// # Example
///
/// ```no_run
/// use claude_agent_sdk::{query, ClaudeAgentOptions};
/// use futures::StreamExt;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let options = ClaudeAgentOptions::default();
///     let mut stream = query("Hello, Claude!".to_string(), options).await?;
///
///     while let Some(message) = stream.next().await {
///         match message? {
///             claude_agent_sdk::Message::Assistant { message, .. } => {
///                 for block in message.message.content {
///                     if let claude_agent_sdk::ContentBlock::Text { text } = block {
///                         println!("Claude: {}", text);
///                     }
///                 }
///             }
///             _ => {}
///         }
///     }
///
///     Ok(())
/// }
/// ```
pub async fn query(
    prompt: String,
    options: ClaudeAgentOptions,
) -> Result<Pin<Box<dyn Stream<Item = Result<Message>> + Send>>> {
    std::env::set_var("CLAUDE_CODE_ENTRYPOINT", "sdk-rust");

    let transport = SubprocessCLITransport::new(options.clone(), false)?;
    let mut boxed_transport = Box::new(transport) as Box<dyn transport::Transport>;
    boxed_transport.connect().await?;

    // For string prompts, write the prompt and close stdin
    let prompt_msg = serde_json::json!({
        "type": "user",
        "message": {
            "role": "user",
            "content": prompt
        }
    });
    boxed_transport
        .write(format!("{}\n", serde_json::to_string(&prompt_msg)?))
        .await?;
    boxed_transport.end_input().await?;

    let can_use_tool = options.can_use_tool.clone();

    let mut q = query::Query::new(boxed_transport, false, can_use_tool, None);
    q.start().await?;

    // Create a channel to send messages through
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    // Spawn a task to read from query and send to channel
    tokio::spawn(async move {
        let mut stream = q.receive_messages();
        use futures::stream::StreamExt;
        while let Some(result) = stream.next().await {
            let parsed = match result {
                Ok(value) => parse_message(value),
                Err(e) => Err(e),
            };
            if tx.send(parsed).is_err() {
                break;
            }
        }
    });

    // Convert receiver to stream
    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    Ok(Box::pin(stream))
}

/// Version of the SDK
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
