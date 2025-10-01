# Claude Agent SDK for Rust

Rust SDK for Claude Agent. See the [Claude Agent SDK documentation](https://docs.anthropic.com/en/docs/claude-code/sdk) for more information.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
claude-agent-sdk = "0.1.0"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

**Prerequisites:**
- Rust 1.75+
- Node.js
- Claude Code: `npm install -g @anthropic-ai/claude-code`

## Quick Start

```rust
use claude_agent_sdk::{query, ClaudeAgentOptions};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ClaudeAgentOptions::default();
    let mut stream = query("What is 2 + 2?".to_string(), options).await?;

    while let Some(message) = stream.next().await {
        println!("{:?}", message?);
    }

    Ok(())
}
```

## Basic Usage: query()

`query()` is an async function for querying Claude Code. It returns a `Stream` of response messages.

```rust
use claude_agent_sdk::{query, ClaudeAgentOptions, Message, ContentBlock};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ClaudeAgentOptions::default();
    let mut stream = query("Hello Claude".to_string(), options).await?;

    while let Some(message) = stream.next().await {
        match message? {
            Message::Assistant { message, .. } => {
                for block in message.message.content {
                    if let ContentBlock::Text { text } = block {
                        println!("{}", text);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}
```

### Using Tools

```rust
let options = ClaudeAgentOptions {
    allowed_tools: vec!["Read".to_string(), "Write".to_string(), "Bash".to_string()],
    permission_mode: Some("acceptEdits".to_string()),
    ..Default::default()
};

let mut stream = query("Create a hello.rs file".to_string(), options).await?;
```

### Working Directory

```rust
use std::path::PathBuf;

let options = ClaudeAgentOptions {
    cwd: Some(PathBuf::from("/path/to/project")),
    ..Default::default()
};
```

## ClaudeSDKClient

`ClaudeSDKClient` supports bidirectional, interactive conversations with Claude Code.

Unlike `query()`, `ClaudeSDKClient` enables **multi-turn conversations** and **real-time interaction**.

```rust
use claude_agent_sdk::{ClaudeSDKClient, ClaudeAgentOptions, Message, ContentBlock};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ClaudeAgentOptions {
        allowed_tools: vec!["Bash".to_string()],
        permission_mode: Some("acceptEdits".to_string()),
        ..Default::default()
    };

    let mut client = ClaudeSDKClient::new(options);
    client.connect().await?;

    // Send first query
    client.query("List files in current directory".to_string()).await?;

    // Receive response
    let mut stream = client.receive_messages();
    while let Some(message) = stream.next().await {
        match message? {
            Message::Assistant { message, .. } => {
                for block in message.message.content {
                    if let ContentBlock::Text { text } = block {
                        println!("Claude: {}", text);
                    }
                }
            }
            Message::Result { .. } => break,
            _ => {}
        }
    }

    // Send follow-up
    client.query("Now show me Rust files".to_string()).await?;

    // Receive second response
    let mut stream = client.receive_messages();
    while let Some(message) = stream.next().await {
        match message? {
            Message::Result { .. } => break,
            _ => {}
        }
    }

    client.disconnect().await?;
    Ok(())
}
```

## Types

The SDK provides strongly-typed message and configuration types:

- `ClaudeAgentOptions` - Configuration options
- `Message` - Message enum (User, Assistant, System, Result, StreamEvent)
- `ContentBlock` - Content types (Text, Thinking, ToolUse, ToolResult)
- `PermissionMode` - Permission control modes
- `McpServerConfig` - MCP server configurations

## Error Handling

```rust
use claude_agent_sdk::{query, ClaudeAgentOptions, ClaudeSDKError};

match query("Hello".to_string(), ClaudeAgentOptions::default()).await {
    Ok(mut stream) => {
        // Process stream
    }
    Err(ClaudeSDKError::CLINotFound(msg)) => {
        eprintln!("Please install Claude Code: {}", msg);
    }
    Err(ClaudeSDKError::Process { exit_code, .. }) => {
        eprintln!("Process failed with code: {:?}", exit_code);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

## Available Tools

See the [Claude Code documentation](https://docs.anthropic.com/en/docs/claude-code/settings#tools-available-to-claude) for a complete list of available tools.

## Examples

The SDK includes comprehensive examples demonstrating all features.

### Prerequisites for Live Examples

The examples that connect to Claude require:
1. **Claude Code CLI**: `npm install -g @anthropic-ai/claude-code`
2. **Authentication**: Set up your Anthropic API key
3. **PATH**: Ensure `claude-code` is in your PATH

### API Demonstration (No CLI Required)
```bash
# Demonstrates SDK API without requiring Claude CLI
cargo run --example api_demo
```

This shows all configuration options, types, and patterns without needing authentication.

### Live Examples (Require Claude CLI)
```bash
# Simple one-shot query
cargo run --example quick_start

# Interactive multi-turn conversations
cargo run --example streaming_mode
```

### Advanced Examples
```bash
# Hook system (PreToolUse, UserPromptSubmit)
cargo run --example hooks

# Tool permission callbacks (allow/deny/modify)
cargo run --example tool_permission_callback

# System prompt configuration
cargo run --example system_prompt

# Stderr output capture
cargo run --example stderr_callback

# Custom agent definitions
cargo run --example agents

# MCP calculator server (in-process tools)
cargo run --example mcp_calculator

# Setting sources control
cargo run --example setting_sources

# Partial message streaming
cargo run --example partial_messages
```

## Features Comparison with Python SDK

| Feature | Python SDK | Rust SDK | Status |
|---------|-----------|----------|--------|
| Simple query() API | ✅ | ✅ | Complete |
| ClaudeSDKClient | ✅ | ✅ | Complete |
| Tool permissions | ✅ | ✅ | Complete |
| Permission callbacks | ✅ | ✅ | Complete |
| Hooks (PreToolUse, UserPromptSubmit) | ✅ | ✅ | Complete |
| MCP servers (external) | ✅ | ✅ | Complete |
| MCP servers (in-process SDK) | ✅ | ✅ | Complete |
| Streaming messages | ✅ | ✅ | Complete |
| Partial message streaming | ✅ | ✅ | Complete |
| Custom agents | ✅ | ✅ | Complete |
| System prompt configuration | ✅ | ✅ | Complete |
| Setting sources control | ✅ | ✅ | Complete |
| Stderr callbacks | ✅ | ✅ | Complete |
| Error handling | ✅ | ✅ | Complete |
| Type safety | ⚠️ | ✅✅ | Better in Rust |

## Architecture

The SDK follows a layered architecture:

1. **Transport Layer** (`transport/`) - Low-level I/O with Claude CLI process
2. **Control Protocol** (`query.rs`) - Bidirectional control protocol handling
3. **Client API** (`client.rs`) - High-level interactive client
4. **Query API** (`lib.rs`) - Simple one-shot query function

## Development

Build the project:

```bash
cargo build
```

Run tests:

```bash
cargo test
```

Build documentation:

```bash
cargo doc --open
```

## Contributing

Contributions are welcome! Please ensure:

- Code follows Rust conventions (`cargo fmt`, `cargo clippy`)
- Tests pass (`cargo test`)
- Documentation is updated
- Examples work

## License

MIT

## MCP (Model Context Protocol) Support

The SDK provides full MCP support for both external and in-process servers:

### External MCP Servers

```rust
use std::collections::HashMap;
use claude_agent_sdk::{ClaudeAgentOptions, McpServerConfig};

let mut mcp_servers = HashMap::new();
mcp_servers.insert(
    "filesystem".to_string(),
    McpServerConfig::Stdio {
        command: "npx".to_string(),
        args: Some(vec!["-y".to_string(), "@modelcontextprotocol/server-filesystem".to_string()]),
        env: None,
    },
);

let options = ClaudeAgentOptions {
    mcp_servers,
    ..Default::default()
};
```

### In-Process MCP Servers

Create custom tools that run directly in your Rust application:

```rust
use claude_agent_sdk::{create_mcp_server, McpTool, ToolParameter};
use serde_json::{json, Value};
use std::collections::HashMap;

// Define a calculator tool
let mut params = HashMap::new();
params.insert("a".to_string(), ToolParameter {
    param_type: "number".to_string(),
    description: Some("First number".to_string()),
});
params.insert("b".to_string(), ToolParameter {
    param_type: "number".to_string(),
    description: Some("Second number".to_string()),
});

let add_tool = McpTool::new("add", "Add two numbers", params, |args: Value| async move {
    let a = args["a"].as_f64().ok_or("Invalid parameter 'a'")?;
    let b = args["b"].as_f64().ok_or("Invalid parameter 'b'")?;
    let result = a + b;

    Ok(json!({
        "content": [{"type": "text", "text": format!("{} + {} = {}", a, b, result)}]
    }))
});

// Create the server
let calculator = create_mcp_server("calculator", "1.0.0", vec![add_tool]);
```

See the [mcp_calculator example](examples/mcp_calculator.rs) for a complete implementation.

## Testing

The SDK includes comprehensive testing:

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --test test_mcp          # MCP functionality
cargo test --test test_permissions   # Permission system
cargo test --test test_types         # Type system

# Run benchmarks
cargo bench
```

**Test Coverage:**
- 88+ unit and integration tests
- 13 MCP-specific tests
- 13 performance benchmarks
- 100% passing

## Roadmap

### Completed ✅
- [x] Full MCP SDK server support (in-process tools)
- [x] Complete hooks implementation with all event types
- [x] Comprehensive example suite (10 examples)
- [x] Enhanced test coverage (88+ tests)
- [x] Benchmarks and performance optimization

### Future
- [ ] WebSocket/HTTP transport options
- [ ] Additional MCP protocol features
- [ ] Performance profiling and optimization
