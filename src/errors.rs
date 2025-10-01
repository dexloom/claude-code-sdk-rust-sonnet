//! Error types for Claude Agent SDK.

use thiserror::Error;

/// Base error type for all Claude SDK errors.
#[derive(Error, Debug)]
pub enum ClaudeSDKError {
    #[error("CLI connection error: {0}")]
    CLIConnection(String),

    #[error("Claude Code not found: {0}")]
    CLINotFound(String),

    #[error("Process error (exit code: {exit_code:?}): {message}")]
    Process {
        message: String,
        exit_code: Option<i32>,
        stderr: Option<String>,
    },

    #[error("JSON decode error: {0}")]
    JSONDecode(#[from] serde_json::Error),

    #[error("Message parse error: {message}")]
    MessageParse {
        message: String,
        data: Option<serde_json::Value>,
    },

    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Control protocol error: {0}")]
    ControlProtocol(String),

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Timeout: {0}")]
    Timeout(String),
}

pub type Result<T> = std::result::Result<T, ClaudeSDKError>;

impl ClaudeSDKError {
    /// Create a CLI not found error.
    pub fn cli_not_found(path: impl Into<String>) -> Self {
        Self::CLINotFound(path.into())
    }

    /// Create a CLI connection error.
    pub fn cli_connection(msg: impl Into<String>) -> Self {
        Self::CLIConnection(msg.into())
    }

    /// Create a process error.
    pub fn process(message: impl Into<String>, exit_code: Option<i32>, stderr: Option<String>) -> Self {
        Self::Process {
            message: message.into(),
            exit_code,
            stderr,
        }
    }

    /// Create a message parse error.
    pub fn message_parse(message: impl Into<String>, data: Option<serde_json::Value>) -> Self {
        Self::MessageParse {
            message: message.into(),
            data,
        }
    }

    /// Create a control protocol error.
    pub fn control_protocol(msg: impl Into<String>) -> Self {
        Self::ControlProtocol(msg.into())
    }

    /// Create a transport error.
    pub fn transport(msg: impl Into<String>) -> Self {
        Self::Transport(msg.into())
    }

    /// Create an invalid configuration error.
    pub fn invalid_config(msg: impl Into<String>) -> Self {
        Self::InvalidConfig(msg.into())
    }

    /// Create a timeout error.
    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::Timeout(msg.into())
    }
}
