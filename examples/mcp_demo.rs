#!/usr/bin/env cargo
//! MCP Server Demo - Works without Claude CLI
//!
//! This example demonstrates the MCP server functionality
//! by directly calling the tools, showing how they work
//! without requiring the Claude CLI integration.

use claude_agent_sdk::{create_mcp_server, McpTool, ToolParameter};
use serde_json::{json, Value};
use std::collections::HashMap;

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

#[tokio::main]
async fn main() {
    println!("MCP Calculator Server - Direct Demo");
    println!("{}", "=".repeat(60));
    println!();

    // Create the calculator server
    let calculator = create_mcp_server(
        "calculator",
        "1.0.0",
        vec![
            create_add_tool(),
            create_multiply_tool(),
            create_sqrt_tool(),
        ],
    );

    println!("✓ MCP Server created: {}", calculator.name);
    println!("  Version: {}", calculator.version);
    println!("  Tools: {}", calculator.tools.len());
    println!();

    // List all available tools
    println!("Available Tools:");
    println!("{}", "-".repeat(60));
    let tools = calculator.list_tools();
    for (i, tool) in tools.iter().enumerate() {
        println!("{}. {} - {}",
            i + 1,
            tool["name"].as_str().unwrap(),
            tool["description"].as_str().unwrap()
        );
    }
    println!();

    // Test cases
    let test_cases = vec![
        ("add", json!({"a": 15, "b": 27}), "15 + 27"),
        ("multiply", json!({"a": 12, "b": 8}), "12 × 8"),
        ("sqrt", json!({"n": 144}), "√144"),
        ("sqrt", json!({"n": -1}), "√-1 (error case)"),
    ];

    println!("Running Test Cases:");
    println!("{}", "=".repeat(60));

    for (tool_name, args, description) in test_cases {
        println!("\n▶ Testing: {}", description);
        println!("  Tool: {}", tool_name);
        println!("  Args: {}", args);

        match calculator.call_tool(tool_name, args).await {
            Ok(result) => {
                if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
                    if let Some(first) = content.first() {
                        if let Some(text) = first.get("text").and_then(|t| t.as_str()) {
                            if result.get("is_error").and_then(|e| e.as_bool()).unwrap_or(false) {
                                println!("  ❌ {}", text);
                            } else {
                                println!("  ✓ Result: {}", text);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("  ❌ Error: {}", e);
            }
        }
    }

    println!();
    println!("{}", "=".repeat(60));
    println!("MCP Server Demo Complete!");
    println!();
    println!("Key Features Demonstrated:");
    println!("  ✓ Server creation with multiple tools");
    println!("  ✓ Tool listing with schemas");
    println!("  ✓ Direct tool execution (async)");
    println!("  ✓ Error handling (negative sqrt)");
    println!("  ✓ Type-safe parameter validation");
    println!();
    println!("Note: This demo calls the MCP tools directly.");
    println!("For Claude CLI integration, use external MCP servers (stdio/SSE).");
}
