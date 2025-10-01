//! Subprocess transport implementation using Claude Code CLI.

use crate::errors::{ClaudeSDKError, Result};
use crate::transport::Transport;
use crate::types::{ClaudeAgentOptions, McpServerConfig, SystemPrompt};
use async_trait::async_trait;
use bytes::BytesMut;
use futures::stream::Stream;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::pin::Pin;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::mpsc;
use tracing::{debug, error};

const DEFAULT_MAX_BUFFER_SIZE: usize = 1024 * 1024; // 1MB
const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct SubprocessCLITransport {
    cli_path: PathBuf,
    options: ClaudeAgentOptions,
    is_streaming: bool,
    process: Option<Child>,
    stdin: Option<ChildStdin>,
    ready: bool,
    max_buffer_size: usize,
    message_rx: Option<mpsc::UnboundedReceiver<Result<Value>>>,
}

impl SubprocessCLITransport {
    pub fn new(options: ClaudeAgentOptions, is_streaming: bool) -> Result<Self> {
        let cli_path = Self::find_cli()?;
        let max_buffer_size = options.max_buffer_size.unwrap_or(DEFAULT_MAX_BUFFER_SIZE);

        Ok(Self {
            cli_path,
            options,
            is_streaming,
            process: None,
            stdin: None,
            ready: false,
            max_buffer_size,
            message_rx: None,
        })
    }

    pub fn with_cli_path(mut self, path: PathBuf) -> Self {
        self.cli_path = path;
        self
    }

    fn find_cli() -> Result<PathBuf> {
        // Check if 'claude' is in PATH
        if let Ok(path) = which::which("claude") {
            return Ok(path);
        }

        // Check common installation locations
        let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/root"));
        let locations = vec![
            PathBuf::from(format!("{}/.npm-global/bin/claude", home)),
            PathBuf::from("/usr/local/bin/claude"),
            PathBuf::from(format!("{}/.local/bin/claude", home)),
            PathBuf::from(format!("{}/node_modules/.bin/claude", home)),
            PathBuf::from(format!("{}/.yarn/bin/claude", home)),
        ];

        for path in locations {
            if path.exists() && path.is_file() {
                return Ok(path);
            }
        }

        Err(ClaudeSDKError::cli_not_found(
            "Claude Code not found. Install with:\n  npm install -g @anthropic-ai/claude-code",
        ))
    }

    fn build_command(&self) -> Vec<String> {
        let mut cmd = vec![
            self.cli_path.to_string_lossy().to_string(),
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--verbose".to_string(),
        ];

        // System prompt
        if let Some(ref system_prompt) = self.options.system_prompt {
            match system_prompt {
                SystemPrompt::Text(text) => {
                    cmd.push("--system-prompt".to_string());
                    cmd.push(text.clone());
                }
                SystemPrompt::Preset { preset: _, append } => {
                    if let Some(append_text) = append {
                        cmd.push("--append-system-prompt".to_string());
                        cmd.push(append_text.clone());
                    }
                }
            }
        }

        // Allowed tools
        if !self.options.allowed_tools.is_empty() {
            cmd.push("--allowedTools".to_string());
            cmd.push(self.options.allowed_tools.join(","));
        }

        // Max turns
        if let Some(max_turns) = self.options.max_turns {
            cmd.push("--max-turns".to_string());
            cmd.push(max_turns.to_string());
        }

        // Disallowed tools
        if !self.options.disallowed_tools.is_empty() {
            cmd.push("--disallowedTools".to_string());
            cmd.push(self.options.disallowed_tools.join(","));
        }

        // Model
        if let Some(ref model) = self.options.model {
            cmd.push("--model".to_string());
            cmd.push(model.clone());
        }

        // Permission prompt tool
        if let Some(ref tool_name) = self.options.permission_prompt_tool_name {
            cmd.push("--permission-prompt-tool".to_string());
            cmd.push(tool_name.clone());
        }

        // Permission mode
        if let Some(ref mode) = self.options.permission_mode {
            cmd.push("--permission-mode".to_string());
            cmd.push(mode.clone());
        }

        // Continue conversation
        if self.options.continue_conversation {
            cmd.push("--continue".to_string());
        }

        // Resume
        if let Some(ref resume) = self.options.resume {
            cmd.push("--resume".to_string());
            cmd.push(resume.clone());
        }

        // Settings
        if let Some(ref settings) = self.options.settings {
            cmd.push("--settings".to_string());
            cmd.push(settings.clone());
        }

        // Add directories
        for dir in &self.options.add_dirs {
            cmd.push("--add-dir".to_string());
            cmd.push(dir.to_string_lossy().to_string());
        }

        // MCP servers
        if !self.options.mcp_servers.is_empty() {
            let servers_for_cli = self.build_mcp_config();
            if !servers_for_cli.is_empty() {
                cmd.push("--mcp-config".to_string());
                let config_json = serde_json::json!({ "mcpServers": servers_for_cli });
                cmd.push(serde_json::to_string(&config_json).unwrap());
            }
        }

        // Include partial messages
        if self.options.include_partial_messages {
            cmd.push("--include-partial-messages".to_string());
        }

        // Fork session
        if self.options.fork_session {
            cmd.push("--fork-session".to_string());
        }

        // Agents
        if !self.options.agents.is_empty() {
            cmd.push("--agents".to_string());
            cmd.push(serde_json::to_string(&self.options.agents).unwrap());
        }

        // Setting sources
        if let Some(ref sources) = self.options.setting_sources {
            cmd.push("--setting-sources".to_string());
            let sources_str: Vec<String> = sources
                .iter()
                .map(|s| serde_json::to_string(s).unwrap().trim_matches('"').to_string())
                .collect();
            cmd.push(sources_str.join(","));
        }

        // Extra args
        for (flag, value) in &self.options.extra_args {
            cmd.push(format!("--{}", flag));
            if let Some(val) = value {
                cmd.push(val.clone());
            }
        }

        // Input format
        if self.is_streaming {
            cmd.push("--input-format".to_string());
            cmd.push("stream-json".to_string());
        } else {
            // For non-streaming, prompt would be added here
            // This is handled differently in Rust - we'll pass it via stdin
            cmd.push("--print".to_string());
            cmd.push("--".to_string());
            cmd.push(String::new()); // Placeholder, actual prompt via stdin
        }

        cmd
    }

    fn build_mcp_config(&self) -> HashMap<String, Value> {
        let mut servers_for_cli = HashMap::new();

        for (name, config) in &self.options.mcp_servers {
            match config {
                McpServerConfig::SDK { name: _, instance: _ } => {
                    // For SDK servers, only pass type and name, not the instance
                    servers_for_cli.insert(
                        name.clone(),
                        serde_json::json!({
                            "type": "sdk",
                            "name": name
                        }),
                    );
                }
                _ => {
                    // For external servers, serialize as-is
                    if let Ok(value) = serde_json::to_value(config) {
                        servers_for_cli.insert(name.clone(), value);
                    }
                }
            }
        }

        servers_for_cli
    }

    fn spawn_stderr_handler(
        stderr: Option<tokio::process::ChildStderr>,
        callback: Option<Arc<dyn Fn(String) + Send + Sync>>,
    ) {
        if let Some(stderr) = stderr {
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if let Some(ref cb) = callback {
                        cb(line);
                    }
                }
            });
        }
    }
}

#[async_trait]
impl Transport for SubprocessCLITransport {
    async fn connect(&mut self) -> Result<()> {
        if self.process.is_some() {
            return Ok(());
        }

        let cmd_args = self.build_command();
        debug!("Starting Claude CLI: {:?}", cmd_args);

        let mut command = Command::new(&cmd_args[0]);
        command.args(&cmd_args[1..]);
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());

        // Configure stderr
        if self.options.stderr_callback.is_some() {
            command.stderr(Stdio::piped());
        } else {
            command.stderr(Stdio::null());
        }

        // Set working directory
        if let Some(ref cwd) = self.options.cwd {
            command.current_dir(cwd);
        }

        // Set environment variables
        let mut env_vars = self.options.env.clone();
        env_vars.insert("CLAUDE_CODE_ENTRYPOINT".to_string(), "sdk-rust".to_string());
        env_vars.insert("CLAUDE_AGENT_SDK_VERSION".to_string(), SDK_VERSION.to_string());

        for (key, value) in env_vars {
            command.env(key, value);
        }

        // Set user if specified
        #[cfg(unix)]
        if let Some(ref user) = self.options.user {
            use users::get_user_by_name;
            if let Some(user_info) = get_user_by_name(user) {
                use std::os::unix::process::CommandExt;
                command.uid(user_info.uid());
            }
        }

        // Spawn process
        let mut child = command.spawn().map_err(|e| {
            if let Some(ref cwd) = self.options.cwd {
                if !cwd.exists() {
                    return ClaudeSDKError::cli_connection(format!("Working directory does not exist: {:?}", cwd));
                }
            }
            ClaudeSDKError::cli_connection(format!("Failed to start Claude Code: {}", e))
        })?;

        // Take stdin
        self.stdin = child.stdin.take();

        // Take stdout and spawn reader task
        if let Some(stdout) = child.stdout.take() {
            let (tx, rx) = mpsc::unbounded_channel();
            let max_buffer_size = self.max_buffer_size;

            tokio::spawn(async move {
                if let Err(e) = Self::read_stdout(stdout, tx, max_buffer_size).await {
                    error!("Error reading stdout: {}", e);
                }
            });

            self.message_rx = Some(rx);
        }

        // Spawn stderr handler if callback is provided
        let stderr = child.stderr.take();
        if let Some(callback) = self.options.stderr_callback.as_ref() {
            let callback_clone = callback.clone();
            Self::spawn_stderr_handler(stderr, Some(callback_clone));
        }

        self.process = Some(child);
        self.ready = true;

        Ok(())
    }

    async fn write(&mut self, data: String) -> Result<()> {
        if !self.ready {
            return Err(ClaudeSDKError::transport("Transport is not ready for writing"));
        }

        if let Some(ref mut stdin) = self.stdin {
            stdin
                .write_all(data.as_bytes())
                .await
                .map_err(|e| ClaudeSDKError::transport(format!("Failed to write to stdin: {}", e)))?;
            stdin.flush().await.map_err(|e| ClaudeSDKError::transport(format!("Failed to flush stdin: {}", e)))?;
            Ok(())
        } else {
            Err(ClaudeSDKError::transport("Stdin not available"))
        }
    }

    fn read_messages(&mut self) -> Pin<Box<dyn Stream<Item = Result<Value>> + Send + '_>> {
        if let Some(rx) = self.message_rx.take() {
            Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx))
        } else {
            Box::pin(futures::stream::empty())
        }
    }

    async fn close(&mut self) -> Result<()> {
        self.ready = false;

        // Close stdin
        if let Some(mut stdin) = self.stdin.take() {
            let _ = stdin.shutdown().await;
        }

        // Terminate process
        if let Some(mut process) = self.process.take() {
            let _ = process.kill().await;
            let _ = process.wait().await;
        }

        Ok(())
    }

    fn is_ready(&self) -> bool {
        self.ready
    }

    async fn end_input(&mut self) -> Result<()> {
        if let Some(mut stdin) = self.stdin.take() {
            stdin.shutdown().await.map_err(|e| ClaudeSDKError::transport(format!("Failed to close stdin: {}", e)))?;
        }
        Ok(())
    }
}

impl SubprocessCLITransport {
    async fn read_stdout(stdout: ChildStdout, tx: mpsc::UnboundedSender<Result<Value>>, max_buffer_size: usize) -> Result<()> {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        let mut json_buffer = BytesMut::new();

        while let Ok(Some(line)) = lines.next_line().await {
            let line_trimmed = line.trim();
            if line_trimmed.is_empty() {
                continue;
            }

            // Accumulate into buffer
            json_buffer.extend_from_slice(line_trimmed.as_bytes());

            if json_buffer.len() > max_buffer_size {
                let err = ClaudeSDKError::JSONDecode(serde_json::Error::io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("JSON buffer exceeded {} bytes", max_buffer_size),
                )));
                let _ = tx.send(Err(err));
                json_buffer.clear();
                continue;
            }

            // Try to parse
            match serde_json::from_slice::<Value>(&json_buffer) {
                Ok(value) => {
                    if tx.send(Ok(value)).is_err() {
                        break; // Receiver dropped
                    }
                    json_buffer.clear();
                }
                Err(_) => {
                    // Keep accumulating - might be partial JSON
                    continue;
                }
            }
        }

        Ok(())
    }
}

impl Drop for SubprocessCLITransport {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            let _ = process.start_kill();
        }
    }
}
