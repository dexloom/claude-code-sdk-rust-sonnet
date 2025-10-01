//! Transport implementations for Claude SDK.

use crate::errors::Result;
use async_trait::async_trait;
use futures::stream::Stream;
use serde_json::Value;
use std::pin::Pin;

pub mod subprocess;

/// Abstract transport for Claude communication.
///
/// This is a low-level transport interface that handles raw I/O with the Claude
/// process or service. The Query layer builds on top of this to implement the
/// control protocol and message routing.
#[async_trait]
pub trait Transport: Send {
    /// Connect the transport and prepare for communication.
    async fn connect(&mut self) -> Result<()>;

    /// Write raw data to the transport.
    async fn write(&mut self, data: String) -> Result<()>;

    /// Read and parse messages from the transport.
    fn read_messages(&mut self) -> Pin<Box<dyn Stream<Item = Result<Value>> + Send + '_>>;

    /// Close the transport connection and clean up resources.
    async fn close(&mut self) -> Result<()>;

    /// Check if transport is ready for communication.
    fn is_ready(&self) -> bool;

    /// End the input stream (close stdin for process transports).
    async fn end_input(&mut self) -> Result<()>;
}
