#!/usr/bin/env cargo
//! Example: Calculator MCP Server.
//!
//! This example demonstrates how to create an in-process MCP server with
//! calculator tools using the Claude Code Rust SDK.
//!
//! Unlike external MCP servers that require separate processes, this server
//! runs directly within your Rust application, providing better performance
//! and simpler deployment.

use claude_agent_sdk::{
    create_mcp_server, ClaudeAgentOptions, ClaudeSDKClient, ContentBlock, McpServerConfig,
    McpTool, Message, ToolParameter,
};
use futures::StreamExt;
use serde_json::{json, Value};
use std::collections::HashMap;

// Calculator tool implementations

fn create_add_tool() -> McpTool {
    let mut params = HashMap::new();
    params.insert(
        "a".to_string(),
        ToolParameter {
            param_type: "number".to_string(),
            description: Some("First number".to_string()),
        },
    );
    params.insert(
        "b".to_string(),
        ToolParameter {
            param_type: "number".to_string(),
            description: Some("Second number".to_string()),
        },
    );

    McpTool::new("add", "Add two numbers", params, |args: Value| async move {
        let a = args["a"]
            .as_f64()
            .ok_or_else(|| "Invalid parameter 'a'".to_string())?;
        let b = args["b"]
            .as_f64()
            .ok_or_else(|| "Invalid parameter 'b'".to_string())?;
        let result = a + b;

        Ok(json!({
            "content": [{"type": "text", "text": format!("{} + {} = {}", a, b, result)}]
        }))
    })
}

fn create_subtract_tool() -> McpTool {
    let mut params = HashMap::new();
    params.insert(
        "a".to_string(),
        ToolParameter {
            param_type: "number".to_string(),
            description: Some("First number".to_string()),
        },
    );
    params.insert(
        "b".to_string(),
        ToolParameter {
            param_type: "number".to_string(),
            description: Some("Second number".to_string()),
        },
    );

    McpTool::new(
        "subtract",
        "Subtract one number from another",
        params,
        |args: Value| async move {
            let a = args["a"]
                .as_f64()
                .ok_or_else(|| "Invalid parameter 'a'".to_string())?;
            let b = args["b"]
                .as_f64()
                .ok_or_else(|| "Invalid parameter 'b'".to_string())?;
            let result = a - b;

            Ok(json!({
                "content": [{"type": "text", "text": format!("{} - {} = {}", a, b, result)}]
            }))
        },
    )
}

fn create_multiply_tool() -> McpTool {
    let mut params = HashMap::new();
    params.insert(
        "a".to_string(),
        ToolParameter {
            param_type: "number".to_string(),
            description: Some("First number".to_string()),
        },
    );
    params.insert(
        "b".to_string(),
        ToolParameter {
            param_type: "number".to_string(),
            description: Some("Second number".to_string()),
        },
    );

    McpTool::new("multiply", "Multiply two numbers", params, |args: Value| async move {
        let a = args["a"]
            .as_f64()
            .ok_or_else(|| "Invalid parameter 'a'".to_string())?;
        let b = args["b"]
            .as_f64()
            .ok_or_else(|| "Invalid parameter 'b'".to_string())?;
        let result = a * b;

        Ok(json!({
            "content": [{"type": "text", "text": format!("{} × {} = {}", a, b, result)}]
        }))
    })
}

fn create_divide_tool() -> McpTool {
    let mut params = HashMap::new();
    params.insert(
        "a".to_string(),
        ToolParameter {
            param_type: "number".to_string(),
            description: Some("Dividend".to_string()),
        },
    );
    params.insert(
        "b".to_string(),
        ToolParameter {
            param_type: "number".to_string(),
            description: Some("Divisor".to_string()),
        },
    );

    McpTool::new(
        "divide",
        "Divide one number by another",
        params,
        |args: Value| async move {
            let a = args["a"]
                .as_f64()
                .ok_or_else(|| "Invalid parameter 'a'".to_string())?;
            let b = args["b"]
                .as_f64()
                .ok_or_else(|| "Invalid parameter 'b'".to_string())?;

            if b == 0.0 {
                return Ok(json!({
                    "content": [{"type": "text", "text": "Error: Division by zero is not allowed"}],
                    "is_error": true
                }));
            }

            let result = a / b;
            Ok(json!({
                "content": [{"type": "text", "text": format!("{} ÷ {} = {}", a, b, result)}]
            }))
        },
    )
}

fn create_sqrt_tool() -> McpTool {
    let mut params = HashMap::new();
    params.insert(
        "n".to_string(),
        ToolParameter {
            param_type: "number".to_string(),
            description: Some("Number to calculate square root of".to_string()),
        },
    );

    McpTool::new("sqrt", "Calculate square root", params, |args: Value| async move {
        let n = args["n"]
            .as_f64()
            .ok_or_else(|| "Invalid parameter 'n'".to_string())?;

        if n < 0.0 {
            return Ok(json!({
                "content": [{"type": "text", "text": format!("Error: Cannot calculate square root of negative number {}", n)}],
                "is_error": true
            }));
        }

        let result = n.sqrt();
        Ok(json!({
            "content": [{"type": "text", "text": format!("√{} = {}", n, result)}]
        }))
    })
}

fn create_power_tool() -> McpTool {
    let mut params = HashMap::new();
    params.insert(
        "base".to_string(),
        ToolParameter {
            param_type: "number".to_string(),
            description: Some("Base number".to_string()),
        },
    );
    params.insert(
        "exponent".to_string(),
        ToolParameter {
            param_type: "number".to_string(),
            description: Some("Exponent".to_string()),
        },
    );

    McpTool::new(
        "power",
        "Raise a number to a power",
        params,
        |args: Value| async move {
            let base = args["base"]
                .as_f64()
                .ok_or_else(|| "Invalid parameter 'base'".to_string())?;
            let exponent = args["exponent"]
                .as_f64()
                .ok_or_else(|| "Invalid parameter 'exponent'".to_string())?;

            let result = base.powf(exponent);
            Ok(json!({
                "content": [{"type": "text", "text": format!("{}^{} = {}", base, exponent, result)}]
            }))
        },
    )
}

fn display_message(msg: &Message) {
    match msg {
        Message::User { message, .. } => {
            if let Some(content) = &message.message.content {
                println!("User: {}", content);
            }
        }
        Message::Assistant { message, .. } => {
            // Show text content and tool usage from content blocks
            for block in &message.message.content {
                match block {
                    ContentBlock::Text { text } => {
                        if !text.is_empty() {
                            println!("Claude: {}", text);
                        }
                    }
                    ContentBlock::ToolUse { name, input, .. } => {
                        println!("Using tool: {}", name);
                        println!("  Input: {}", input);
                    }
                    _ => {}
                }
            }
        }
        Message::Result { total_cost_usd, .. } => {
            println!("Result ended");
            if let Some(cost) = total_cost_usd {
                println!("Cost: ${:.6}", cost);
            }
        }
        _ => {}
    }
}

#[tokio::main]
async fn main() {
    println!("Calculator MCP Server Example");
    println!("{}", "=".repeat(50));
    println!();
    println!("⚠️  WARNING: In-process MCP servers (SDK type) are not yet");
    println!("   fully integrated with Claude CLI subprocess transport.");
    println!();
    println!("   This example demonstrates the API but may not work as expected.");
    println!("   For working MCP integration, see:");
    println!("     - cargo run --example mcp_demo (direct tool calls)");
    println!("     - cargo run --example mcp_external (external MCP servers)");
    println!();
    println!("{}", "=".repeat(50));
    println!();

    // Create the calculator server with all tools
    let calculator = create_mcp_server(
        "calculator",
        "2.0.0",
        vec![
            create_add_tool(),
            create_subtract_tool(),
            create_multiply_tool(),
            create_divide_tool(),
            create_sqrt_tool(),
            create_power_tool(),
        ],
    );

    // Configure Claude to use the calculator server with allowed tools
    // Pre-approve all calculator MCP tools so they can be used without permission prompts
    let mut mcp_servers = HashMap::new();
    mcp_servers.insert(
        "calc".to_string(),
        McpServerConfig::SDK {
            name: "calculator".to_string(),
            instance: Some(()),  // Placeholder - actual server handling needs implementation
        },
    );

    let options = ClaudeAgentOptions {
        mcp_servers,
        allowed_tools: vec![
            "mcp__calc__add".to_string(),
            "mcp__calc__subtract".to_string(),
            "mcp__calc__multiply".to_string(),
            "mcp__calc__divide".to_string(),
            "mcp__calc__sqrt".to_string(),
            "mcp__calc__power".to_string(),
        ],
        max_turns: Some(3),
        ..Default::default()
    };

    // Example prompts to demonstrate calculator usage
    let prompts = vec![
        "List your tools",
        "Calculate 15 + 27",
        "What is 100 divided by 7?",
        "Calculate the square root of 144",
        "What is 2 raised to the power of 8?",
        "Calculate (12 + 8) * 3 - 10",
    ];

    for prompt in prompts {
        println!("{}", "=".repeat(50));
        println!("Prompt: {}", prompt);
        println!("{}", "=".repeat(50));

        let mut client = ClaudeSDKClient::new(options.clone());

        match client.connect().await {
            Ok(_) => {
                if let Err(e) = client.query(prompt.to_string()).await {
                    eprintln!("Query error: {}", e);
                    continue;
                }

                {
                    let mut stream = client.receive_response();
                    while let Some(result) = stream.next().await {
                        match result {
                            Ok(message) => {
                                let is_result = matches!(&message, Message::Result { .. });
                                display_message(&message);
                                if is_result {
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("Error: {}", e);
                                break;
                            }
                        }
                    }
                }

                if let Err(e) = client.disconnect().await {
                    eprintln!("Disconnect error: {}", e);
                }
            }
            Err(e) => eprintln!("Connection error: {}", e),
        }

        println!();
    }

    println!("{}", "=".repeat(50));
    println!("\nKey takeaways:");
    println!("- In-process MCP servers run directly in your application");
    println!("- Tools are defined with type-safe handlers");
    println!("- Pre-approved tools execute without permission prompts");
    println!("- Errors are handled gracefully (e.g., division by zero)");
}
