#!/usr/bin/env cargo
//! Example demonstrating different system prompt configurations.
//!
//! This example shows four ways to configure the system prompt:
//! 1. No system prompt (default)
//! 2. Custom string system prompt
//! 3. Preset system prompt (uses Claude Code's built-in preset)
//! 4. Preset + append (adds to the built-in preset)

use claude_agent_sdk::{ClaudeAgentOptions, ClaudeSDKClient, Message, SystemPromptConfig};
use futures::StreamExt;
use std::collections::HashMap;

async fn example_no_system_prompt() {
    println!("=== Example 1: No System Prompt ===");
    println!("Using default behavior (no custom system prompt)\n");

    let options = ClaudeAgentOptions {
        max_turns: Some(1),
        ..Default::default()
    };

    let mut client = ClaudeSDKClient::new(options);

    if let Err(e) = run_query(&mut client, "What is 2 + 2?").await {
        eprintln!("Error: {}", e);
    }

    println!();
}

async fn example_string_system_prompt() {
    println!("=== Example 2: String System Prompt ===");
    println!("Custom system prompt as a string\n");

    let options = ClaudeAgentOptions {
        system_prompt: Some(SystemPromptConfig::String(
            "You are a helpful assistant that always responds in a very concise manner. \
             Keep all responses to one sentence or less."
                .to_string(),
        )),
        max_turns: Some(1),
        ..Default::default()
    };

    let mut client = ClaudeSDKClient::new(options);

    if let Err(e) = run_query(&mut client, "What is the capital of France?").await {
        eprintln!("Error: {}", e);
    }

    println!();
}

async fn example_preset_system_prompt() {
    println!("=== Example 3: Preset System Prompt ===");
    println!("Using Claude Code's built-in preset system prompt\n");

    let mut config = HashMap::new();
    config.insert("type".to_string(), "preset".to_string());

    let options = ClaudeAgentOptions {
        system_prompt: Some(SystemPromptConfig::Preset(config)),
        max_turns: Some(1),
        ..Default::default()
    };

    let mut client = ClaudeSDKClient::new(options);

    if let Err(e) = run_query(&mut client, "What is 2 + 2?").await {
        eprintln!("Error: {}", e);
    }

    println!();
}

async fn example_preset_with_append() {
    println!("=== Example 4: Preset + Append System Prompt ===");
    println!("Using preset and appending additional instructions\n");

    let mut config = HashMap::new();
    config.insert("type".to_string(), "preset".to_string());
    config.insert(
        "append".to_string(),
        "Always end your responses with 'Hope this helps!'".to_string(),
    );

    let options = ClaudeAgentOptions {
        system_prompt: Some(SystemPromptConfig::Preset(config)),
        max_turns: Some(1),
        ..Default::default()
    };

    let mut client = ClaudeSDKClient::new(options);

    if let Err(e) = run_query(&mut client, "What is 5 + 3?").await {
        eprintln!("Error: {}", e);
    }

    println!();
}

async fn run_query(client: &mut ClaudeSDKClient, prompt: &str) -> Result<(), Box<dyn std::error::Error>> {
    client.connect().await?;

    println!("Prompt: {}", prompt);
    println!("{}", "-".repeat(50));

    client.query(prompt).await?;

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
                    break;
                }
                _ => {}
            },
            Err(e) => return Err(e.into()),
        }
    }

    client.disconnect().await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    println!("System Prompt Configuration Examples");
    println!("{}", "=".repeat(50));
    println!();

    // Run all examples
    example_no_system_prompt().await;
    println!("{}", "=".repeat(50));
    println!();

    example_string_system_prompt().await;
    println!("{}", "=".repeat(50));
    println!();

    example_preset_system_prompt().await;
    println!("{}", "=".repeat(50));
    println!();

    example_preset_with_append().await;
    println!("{}", "=".repeat(50));

    println!("\nKey takeaways:");
    println!("- No system prompt: Uses Claude's default behavior");
    println!("- String: Complete custom system prompt");
    println!("- Preset: Uses Claude Code's built-in optimized prompt");
    println!("- Preset + Append: Extends the built-in prompt with custom instructions");
}
