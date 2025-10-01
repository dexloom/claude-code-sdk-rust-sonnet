#!/usr/bin/env cargo
//! Example demonstrating hook functionality.
//!
//! Hooks allow you to intercept and modify the agent's behavior at key points:
//! - PreToolUse: Intercept before a tool is used (can block/modify)
//! - UserPromptSubmit: Intercept user prompts before submission
//!
//! This example shows:
//! 1. Blocking bash commands using PreToolUse hook
//! 2. Adding context to user prompts using UserPromptSubmit hook

use claude_agent_sdk::{ClaudeAgentOptions, ClaudeSDKClient, HookCallback, HookMatcher, Message};
use futures::StreamExt;
use serde_json::{json, Value};
use std::sync::Arc;

async fn example_pre_tool_use() {
    println!("=== PreToolUse Hook Example ===");
    println!("This example blocks bash commands using a PreToolUse hook\n");

    // Create a PreToolUse hook that blocks bash commands
    let pre_tool_use_hook = Arc::new(move |payload: Value| -> Option<Value> {
        println!("🪝 PreToolUse hook triggered");

        // Extract tool information
        if let Some(input) = payload.get("input").and_then(|i| i.as_object()) {
            if let Some(tool_name) = input.get("name").and_then(|n| n.as_str()) {
                println!("  Tool: {}", tool_name);

                if tool_name == "Bash" {
                    println!("  ❌ Blocking bash command");
                    // Return error to block the tool
                    return Some(json!({
                        "error": "Bash commands are not allowed in this session"
                    }));
                } else {
                    println!("  ✓ Allowing tool");
                }
            }
        }

        // Return None to allow the tool to proceed
        None
    }) as Arc<dyn Fn(Value) -> Option<Value> + Send + Sync>;

    let hook_callback = HookCallback {
        name: "pre_tool_use".to_string(),
        callback: pre_tool_use_hook,
    };

    let options = ClaudeAgentOptions {
        hooks: Some(vec![(
            HookMatcher {
                name: "pre_tool_use".to_string(),
                matchers: vec![],
            },
            vec![hook_callback],
        )]),
        max_turns: Some(3),
        ..Default::default()
    };

    let mut client = ClaudeSDKClient::new(options);

    match client.connect().await {
        Ok(_) => {
            // Send a query that tries to use bash
            let prompt = "What is the current date? Use a bash command to find out.";
            println!("Prompt: {}\n", prompt);
            println!("{}", "=".repeat(50));

            if let Err(e) = client.query(prompt).await {
                eprintln!("Query error: {}", e);
                return;
            }

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
                            println!("\n✓ Result: {:?}", msg.result_type);
                        }
                        _ => {}
                    },
                    Err(e) => eprintln!("Error: {}", e),
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

async fn example_user_prompt_submit() {
    println!("=== UserPromptSubmit Hook Example ===");
    println!("This example adds context to user prompts before submission\n");

    // Create a UserPromptSubmit hook that adds context
    let user_prompt_submit_hook = Arc::new(move |payload: Value| -> Option<Value> {
        println!("🪝 UserPromptSubmit hook triggered");

        if let Some(prompt) = payload.get("prompt").and_then(|p| p.as_str()) {
            println!("  Original prompt: {}", prompt);

            // Add additional context
            let enhanced_prompt = format!(
                "{}\n\nAdditional context: Please keep your response concise and to the point.",
                prompt
            );

            println!("  Enhanced prompt: {}", enhanced_prompt);

            // Return modified prompt
            return Some(json!({
                "prompt": enhanced_prompt
            }));
        }

        None
    }) as Arc<dyn Fn(Value) -> Option<Value> + Send + Sync>;

    let hook_callback = HookCallback {
        name: "user_prompt_submit".to_string(),
        callback: user_prompt_submit_hook,
    };

    let options = ClaudeAgentOptions {
        hooks: Some(vec![(
            HookMatcher {
                name: "user_prompt_submit".to_string(),
                matchers: vec![],
            },
            vec![hook_callback],
        )]),
        max_turns: Some(2),
        ..Default::default()
    };

    let mut client = ClaudeSDKClient::new(options);

    match client.connect().await {
        Ok(_) => {
            let prompt = "What is 2 + 2?";
            println!("Prompt: {}\n", prompt);
            println!("{}", "=".repeat(50));

            if let Err(e) = client.query(prompt).await {
                eprintln!("Query error: {}", e);
                return;
            }

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
                            println!("\n✓ Result: {:?}", msg.result_type);
                        }
                        _ => {}
                    },
                    Err(e) => eprintln!("Error: {}", e),
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

#[tokio::main]
async fn main() {
    println!("Claude SDK Hooks Examples");
    println!("{}", "=".repeat(50));
    println!();

    // Run both examples
    example_pre_tool_use().await;
    println!("{}", "-".repeat(50));
    println!();
    example_user_prompt_submit().await;
}
