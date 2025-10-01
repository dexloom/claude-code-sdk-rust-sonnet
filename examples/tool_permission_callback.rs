#!/usr/bin/env cargo
//! Example demonstrating tool permission callbacks.
//!
//! Tool permission callbacks allow you to control which tools can be used
//! and how they're used. You can:
//! - Allow tools to execute
//! - Deny tools from executing
//! - Modify tool inputs before execution
//!
//! This example shows a permission callback that:
//! 1. Blocks all Bash commands (deny)
//! 2. Allows Read tool usage
//! 3. Modifies Glob patterns to be more restrictive

use claude_agent_sdk::{ClaudeAgentOptions, ClaudeSDKClient, Message, PermissionResult};
use futures::StreamExt;
use serde_json::Value;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("Tool Permission Callback Example");
    println!("{}", "=".repeat(50));
    println!();

    // Create permission callback
    let permission_callback = Arc::new(
        move |tool_name: String, tool_input: Value| -> PermissionResult {
            println!("🔒 Permission check for tool: {}", tool_name);

            match tool_name.as_str() {
                "Bash" => {
                    println!("  ❌ DENIED: Bash commands are not allowed");
                    PermissionResult::Deny {
                        message: "Bash commands are disabled for security reasons".to_string(),
                        interrupt: false,
                    }
                }
                "Read" => {
                    println!("  ✓ ALLOWED: Read tool");
                    PermissionResult::Allow {
                        updated_input: None,
                        updated_permissions: None,
                    }
                }
                "Glob" => {
                    // Modify glob patterns to be more restrictive
                    if let Some(pattern) = tool_input.get("pattern").and_then(|p| p.as_str()) {
                        println!("  🔧 MODIFIED: Restricting glob pattern");
                        println!("     Original: {}", pattern);

                        // Only allow searching in specific directories
                        let restricted_pattern = if pattern.starts_with("examples/") {
                            pattern.to_string()
                        } else {
                            format!("examples/{}", pattern)
                        };

                        println!("     Modified: {}", restricted_pattern);

                        let mut modified_input = tool_input.clone();
                        if let Some(obj) = modified_input.as_object_mut() {
                            obj.insert("pattern".to_string(), restricted_pattern.into());
                        }

                        PermissionResult::Allow {
                            updated_input: Some(modified_input),
                            updated_permissions: None,
                        }
                    } else {
                        println!("  ✓ ALLOWED: Glob tool (no pattern to modify)");
                        PermissionResult::Allow {
                            updated_input: None,
                            updated_permissions: None,
                        }
                    }
                }
                _ => {
                    println!("  ✓ ALLOWED: {} tool", tool_name);
                    PermissionResult::Allow {
                        updated_input: None,
                        updated_permissions: None,
                    }
                }
            }
        },
    ) as Arc<dyn Fn(String, Value) -> PermissionResult + Send + Sync>;

    let options = ClaudeAgentOptions {
        can_use_tool: Some(permission_callback),
        max_turns: Some(3),
        ..Default::default()
    };

    let mut client = ClaudeSDKClient::new(options);

    match client.connect().await {
        Ok(_) => {
            // Test 1: Try to use bash (should be denied)
            println!("Test 1: Attempting to use bash command");
            println!("{}", "-".repeat(50));

            let prompt = "List files in the current directory using bash";
            println!("Prompt: {}\n", prompt);

            if let Err(e) = client.query(prompt).await {
                eprintln!("Query error: {}", e);
                return;
            }

            {
                let mut stream = client.receive_response();
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(message) => match message {
                            Message::Assistant(msg) => {
                                if !msg.content.is_empty() {
                                    println!("\nAssistant: {}", msg.content);
                                }
                            }
                            Message::Result(msg) => {
                                println!("\n✓ Conversation ended: {:?}", msg.result_type);
                                break;
                            }
                            _ => {}
                        },
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
            }

            println!("\n{}", "=".repeat(50));
            println!("\nTest 2: Use Glob tool (pattern will be modified)");
            println!("{}", "-".repeat(50));

            let prompt2 = "Find all .rs files in the project";
            println!("Prompt: {}\n", prompt2);

            if let Err(e) = client.query(prompt2).await {
                eprintln!("Query error: {}", e);
                return;
            }

            {
                let mut stream = client.receive_response();
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(message) => match message {
                            Message::Assistant(msg) => {
                                if !msg.content.is_empty() {
                                    println!("\nAssistant: {}", msg.content);
                                }
                            }
                            Message::Result(msg) => {
                                println!("\n✓ Conversation ended: {:?}", msg.result_type);
                                break;
                            }
                            _ => {}
                        },
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
            }

            if let Err(e) = client.disconnect().await {
                eprintln!("Disconnect error: {}", e);
            }
        }
        Err(e) => eprintln!("Connection error: {}", e),
    }

    println!("\n{}", "=".repeat(50));
    println!("\nKey takeaways:");
    println!("- Bash commands were blocked (deny)");
    println!("- Glob patterns were restricted to examples/ directory (modify)");
    println!("- Other tools were allowed normally (allow)");
}
