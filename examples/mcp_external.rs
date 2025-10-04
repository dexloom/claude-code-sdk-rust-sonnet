#!/usr/bin/env cargo
//! Example: Using external MCP servers with Claude CLI
//!
//! This example shows how to configure Claude to use external MCP servers
//! (like the filesystem server) via stdio transport.
//!
//! Prerequisites:
//! - Claude CLI installed: npm install -g @anthropic-ai/claude-code
//! - MCP server package: npm install -g @modelcontextprotocol/server-filesystem

use claude_agent_sdk::{ClaudeAgentOptions, ClaudeSDKClient, ContentBlock, McpServerConfig, Message};
use futures::StreamExt;
use std::collections::HashMap;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("External MCP Server Example");
    println!("{}", "=".repeat(60));
    println!();

    // Get current directory for filesystem server
    let current_dir = env::current_dir()?;
    let allowed_dir = current_dir.to_str().unwrap_or(".");

    println!("Configuring MCP filesystem server...");
    println!("Allowed directory: {}", allowed_dir);
    println!();

    // Configure external MCP filesystem server
    let mut mcp_servers = HashMap::new();
    mcp_servers.insert(
        "filesystem".to_string(),
        McpServerConfig::Stdio {
            command: "npx".to_string(),
            args: Some(vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-filesystem".to_string(),
                allowed_dir.to_string(),
            ]),
            env: None,
        },
    );

    let options = ClaudeAgentOptions {
        mcp_servers,
        max_turns: Some(3),
        ..Default::default()
    };

    let mut client = ClaudeSDKClient::new(options);

    println!("Connecting to Claude with MCP filesystem server...");
    match client.connect().await {
        Ok(_) => println!("✓ Connected!\n"),
        Err(e) => {
            eprintln!("✗ Connection error: {}", e);
            eprintln!("\nMake sure:");
            eprintln!("  1. Claude CLI is installed: npm install -g @anthropic-ai/claude-code");
            eprintln!("  2. MCP server is installed: npm install -g @modelcontextprotocol/server-filesystem");
            return Err(e.into());
        }
    }

    // Query using MCP tools
    let prompt = "List the files in the current directory using your filesystem tools";
    println!("Query: {}", prompt);
    println!("{}", "-".repeat(60));

    client.query(prompt.to_string()).await?;

    {
        let mut stream = client.receive_response();
        while let Some(result) = stream.next().await {
            match result {
                Ok(message) => {
                    match &message {
                        Message::Assistant { message, .. } => {
                            for block in &message.message.content {
                                match block {
                                    ContentBlock::Text { text } => {
                                        if !text.is_empty() {
                                            println!("\nClaude: {}", text);
                                        }
                                    }
                                    ContentBlock::ToolUse { name, .. } => {
                                        println!("\n[Using MCP tool: {}]", name);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Message::Result { duration_ms, total_cost_usd, .. } => {
                            println!("\n{}", "=".repeat(60));
                            println!("Session complete!");
                            println!("Duration: {}ms", duration_ms);
                            if let Some(cost) = total_cost_usd {
                                println!("Cost: ${:.4}", cost);
                            }
                            break;
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
            }
        }
    }

    client.disconnect().await?;

    println!("\n{}", "=".repeat(60));
    println!("Key Points:");
    println!("  ✓ External MCP servers run as separate processes");
    println!("  ✓ Communication via stdio (stdin/stdout)");
    println!("  ✓ Claude CLI handles server lifecycle");
    println!("  ✓ Tools are automatically available to Claude");

    Ok(())
}
