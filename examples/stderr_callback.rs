#!/usr/bin/env cargo
//! Example demonstrating stderr callback for capturing CLI debug output.
//!
//! This example shows how to use the stderr callback to capture and process
//! debug output from the Claude CLI. This is useful for:
//! - Debugging CLI issues
//! - Monitoring internal operations
//! - Filtering and logging specific events

use claude_agent_sdk::{query, ClaudeAgentOptions};
use futures::StreamExt;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    println!("Stderr Callback Example");
    println!("{}", "=".repeat(50));
    println!();

    // Collect stderr messages in a shared vector
    let stderr_messages: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let stderr_messages_clone = stderr_messages.clone();

    // Create stderr callback
    let stderr_callback = Arc::new(move |message: String| {
        // Store all messages
        if let Ok(mut messages) = stderr_messages_clone.lock() {
            messages.push(message.clone());
        }

        // Print errors immediately
        if message.contains("[ERROR]") {
            println!("🔴 Error detected: {}", message);
        }
        // Print warnings
        else if message.contains("[WARN]") {
            println!("⚠️  Warning: {}", message);
        }
        // Optionally print debug messages (commented out to reduce noise)
        // else if message.contains("[DEBUG]") {
        //     println!("🔍 Debug: {}", message);
        // }
    }) as Arc<dyn Fn(String) + Send + Sync>;

    // Create options with stderr callback and enable debug mode
    let mut extra_args = std::collections::HashMap::new();
    extra_args.insert("debug-to-stderr".to_string(), None);

    let options = ClaudeAgentOptions {
        stderr: Some(stderr_callback),
        extra_args,
        max_turns: Some(1),
        ..Default::default()
    };

    println!("Running query with stderr capture...");
    println!("{}", "-".repeat(50));

    // Run a simple query
    match query("What is 2+2?", options).await {
        Ok(mut stream) => {
            while let Some(result) = stream.next().await {
                match result {
                    Ok(message) => {
                        if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                            if !content.is_empty() {
                                println!("\nResponse: {}", content);
                            }
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        }
        Err(e) => eprintln!("Query error: {}", e),
    }

    // Show what we captured
    println!("\n{}", "=".repeat(50));
    if let Ok(messages) = stderr_messages.lock() {
        println!("\nCaptured {} stderr lines", messages.len());

        if !messages.is_empty() {
            println!("\nFirst few stderr lines:");
            for (i, msg) in messages.iter().take(5).enumerate() {
                let preview = if msg.len() > 100 {
                    format!("{}...", &msg[..100])
                } else {
                    msg.clone()
                };
                println!("  {}: {}", i + 1, preview);
            }

            // Count different message types
            let error_count = messages.iter().filter(|m| m.contains("[ERROR]")).count();
            let warn_count = messages.iter().filter(|m| m.contains("[WARN]")).count();
            let debug_count = messages.iter().filter(|m| m.contains("[DEBUG]")).count();

            println!("\nMessage type breakdown:");
            println!("  Errors:   {}", error_count);
            println!("  Warnings: {}", warn_count);
            println!("  Debug:    {}", debug_count);
            println!("  Other:    {}", messages.len() - error_count - warn_count - debug_count);
        }
    }

    println!("\n{}", "=".repeat(50));
    println!("\nKey takeaways:");
    println!("- stderr callback receives all CLI debug output");
    println!("- You can filter messages by content (errors, warnings, etc.)");
    println!("- Useful for debugging and monitoring CLI behavior");
    println!("- Enable with 'debug-to-stderr' extra arg for full output");
}
