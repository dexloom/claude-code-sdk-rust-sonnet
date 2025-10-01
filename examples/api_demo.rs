//! API demonstration without requiring Claude CLI
//!
//! This example demonstrates the SDK's API structure without actually
//! connecting to the Claude CLI. It shows how the types and builders work.

use claude_agent_sdk::{
    AgentDefinition, ClaudeAgentOptions, McpServerConfig, PermissionResult, SystemPrompt,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

fn main() {
    println!("Claude Agent SDK - API Demonstration");
    println!("{}", "=".repeat(60));
    println!();

    // 1. Basic Options
    println!("1. Basic ClaudeAgentOptions:");
    println!("{}", "-".repeat(60));
    let basic_options = ClaudeAgentOptions::default();
    println!("  ✓ Default options created");
    println!("  - Allowed tools: {:?}", basic_options.allowed_tools);
    println!("  - Max turns: {:?}", basic_options.max_turns);
    println!();

    // 2. Options with Tools
    println!("2. Options with Specific Tools:");
    println!("{}", "-".repeat(60));
    let tool_options = ClaudeAgentOptions {
        allowed_tools: vec!["Read".to_string(), "Write".to_string(), "Bash".to_string()],
        disallowed_tools: vec!["Delete".to_string()],
        max_turns: Some(10),
        ..Default::default()
    };
    println!("  ✓ Options with tool restrictions");
    println!("  - Allowed: {:?}", tool_options.allowed_tools);
    println!("  - Disallowed: {:?}", tool_options.disallowed_tools);
    println!();

    // 3. System Prompt Configuration
    println!("3. System Prompt Options:");
    println!("{}", "-".repeat(60));

    let _string_prompt = ClaudeAgentOptions {
        system_prompt: Some(SystemPrompt::Text(
            "You are a helpful coding assistant.".to_string(),
        )),
        ..Default::default()
    };
    println!("  ✓ String system prompt configured");

    let _preset_prompt = ClaudeAgentOptions {
        system_prompt: Some(SystemPrompt::Preset {
            preset: "default".to_string(),
            append: Some("Always be concise.".to_string()),
        }),
        ..Default::default()
    };
    println!("  ✓ Preset system prompt with append configured");
    println!();

    // 4. MCP Server Configuration
    println!("4. MCP Server Configuration:");
    println!("{}", "-".repeat(60));

    let mut mcp_servers = HashMap::new();
    mcp_servers.insert(
        "filesystem".to_string(),
        McpServerConfig::Stdio {
            command: "npx".to_string(),
            args: Some(vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-filesystem".to_string(),
            ]),
            env: None,
        },
    );

    let mcp_options = ClaudeAgentOptions {
        mcp_servers,
        ..Default::default()
    };
    println!("  ✓ MCP stdio server configured");
    println!("  - Server: filesystem");
    println!();

    // 5. Permission Callback
    println!("5. Permission Callback:");
    println!("{}", "-".repeat(60));

    let permission_callback = Arc::new(|tool_name: String, _input: serde_json::Value| {
        if tool_name == "Bash" {
            PermissionResult::Deny {
                message: "Bash not allowed".to_string(),
                interrupt: false,
            }
        } else {
            PermissionResult::Allow {
                updated_input: None,
                updated_permissions: None,
            }
        }
    });

    println!("  ✓ Permission callback defined");

    // Test the callback
    let result = (permission_callback)("Bash".to_string(), json!({}));
    match result {
        PermissionResult::Deny { message, .. } => {
            println!("  - Bash denied: {}", message);
        }
        _ => {}
    }

    let result = (permission_callback)("Read".to_string(), json!({}));
    match result {
        PermissionResult::Allow { .. } => {
            println!("  - Read allowed");
        }
        _ => {}
    }
    println!();

    // 6. Message Types
    println!("6. Message Type Examples:");
    println!("{}", "-".repeat(60));

    println!("  ✓ ContentBlock::Text - For text responses");
    println!("  ✓ ContentBlock::Thinking - For extended thinking");
    println!("  ✓ ContentBlock::ToolUse - For tool invocations");
    println!("  ✓ ContentBlock::ToolResult - For tool results");
    println!();

    // 7. Agent Definitions
    println!("7. Custom Agent Definition:");
    println!("{}", "-".repeat(60));

    let mut agents = HashMap::new();
    agents.insert(
        "code-reviewer".to_string(),
        AgentDefinition {
            description: "Reviews code for best practices".to_string(),
            prompt: "You are a code reviewer.".to_string(),
            tools: Some(vec!["Read".to_string(), "Glob".to_string()]),
            model: Some("claude-sonnet-4".to_string()),
        },
    );

    println!("  ✓ Code reviewer agent defined");
    println!("  - Tools: Read, Glob");
    println!("  - Model: claude-sonnet-4");
    println!();

    // 8. Complete Configuration
    println!("8. Complete Configuration Example:");
    println!("{}", "-".repeat(60));

    let complete_options = ClaudeAgentOptions {
        allowed_tools: vec!["Read".to_string(), "Write".to_string()],
        system_prompt: Some(SystemPrompt::Text("Be helpful.".to_string())),
        max_turns: Some(5),
        model: Some("claude-sonnet-4".to_string()),
        include_partial_messages: true,
        agents,
        ..Default::default()
    };

    println!("  ✓ Complete configuration created with:");
    println!("    - Tool restrictions");
    println!("    - System prompt");
    println!("    - Max turns: {:?}", complete_options.max_turns);
    println!("    - Model: {:?}", complete_options.model);
    println!("    - Partial messages: {}", complete_options.include_partial_messages);
    println!();

    println!("{}", "=".repeat(60));
    println!("API demonstration complete!");
    println!();
    println!("NOTE: To run actual examples that connect to Claude CLI:");
    println!("  1. Install: npm install -g @anthropic-ai/claude-code");
    println!("  2. Authenticate with your API key");
    println!("  3. Run: cargo run --example quick_start");
}
