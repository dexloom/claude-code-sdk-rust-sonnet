#!/usr/bin/env cargo
//! Example demonstrating setting sources control.
//!
//! This example shows how to use the setting_sources option to control which
//! settings are loaded, including custom slash commands, agents, and other
//! configurations.
//!
//! Setting sources determine where Claude Code loads configurations from:
//! - "user": Global user settings (~/.claude/)
//! - "project": Project-level settings (.claude/ in project)
//! - "local": Local gitignored settings (.claude-local/)
//!
//! IMPORTANT: When setting_sources is not provided (None), NO settings are loaded
//! by default. This creates an isolated environment. To load settings, explicitly
//! specify which sources to use.
//!
//! Usage:
//! cargo run --example setting_sources          # List examples
//! cargo run --example setting_sources all      # Run all examples
//! cargo run --example setting_sources default  # Run specific example

use claude_agent_sdk::{ClaudeAgentOptions, ClaudeSDKClient, Message};
use futures::StreamExt;
use std::env;

fn extract_slash_commands(msg: &Message) -> Vec<String> {
    if let Message::System(system_msg) = msg {
        if system_msg.subtype == "init" {
            if let Some(commands) = system_msg.data.get("slash_commands") {
                if let Some(arr) = commands.as_array() {
                    return arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                }
            }
        }
    }
    Vec::new()
}

async fn example_default() {
    println!("=== Default Behavior Example ===");
    println!("Setting sources: None (default)");
    println!("Expected: No custom slash commands will be available\n");

    let options = ClaudeAgentOptions {
        setting_sources: None, // Default: no settings loaded
        max_turns: Some(1),
        ..Default::default()
    };

    let mut client = ClaudeSDKClient::new(options);

    if let Err(e) = client.connect().await {
        eprintln!("Connection error: {}", e);
        return;
    }

    if let Err(e) = client.query("What is 2 + 2?").await {
        eprintln!("Query error: {}", e);
        return;
    }

    let mut stream = client.receive_response();
    while let Some(result) = stream.next().await {
        match result {
            Ok(message) => {
                if let Message::System(ref msg) = message {
                    if msg.subtype == "init" {
                        let commands = extract_slash_commands(&message);
                        println!("Available slash commands: {:?}", commands);
                        if commands.contains(&"commit".to_string()) {
                            println!("❌ /commit is available (unexpected)");
                        } else {
                            println!("✓ /commit is NOT available (expected - no settings loaded)");
                        }
                        break;
                    }
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    let _ = client.disconnect().await;
    println!();
}

async fn example_user_only() {
    println!("=== User Settings Only Example ===");
    println!("Setting sources: ['user']");
    println!("Expected: Project slash commands (like /commit) will NOT be available\n");

    let options = ClaudeAgentOptions {
        setting_sources: Some(vec!["user".to_string()]),
        max_turns: Some(1),
        ..Default::default()
    };

    let mut client = ClaudeSDKClient::new(options);

    if let Err(e) = client.connect().await {
        eprintln!("Connection error: {}", e);
        return;
    }

    if let Err(e) = client.query("What is 2 + 2?").await {
        eprintln!("Query error: {}", e);
        return;
    }

    let mut stream = client.receive_response();
    while let Some(result) = stream.next().await {
        match result {
            Ok(message) => {
                if let Message::System(ref msg) = message {
                    if msg.subtype == "init" {
                        let commands = extract_slash_commands(&message);
                        println!("Available slash commands: {:?}", commands);
                        if commands.contains(&"commit".to_string()) {
                            println!("❌ /commit is available (unexpected)");
                        } else {
                            println!("✓ /commit is NOT available (expected)");
                        }
                        break;
                    }
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    let _ = client.disconnect().await;
    println!();
}

async fn example_project_and_user() {
    println!("=== Project + User Settings Example ===");
    println!("Setting sources: ['user', 'project']");
    println!("Expected: Project slash commands (like /commit) WILL be available\n");

    let options = ClaudeAgentOptions {
        setting_sources: Some(vec!["user".to_string(), "project".to_string()]),
        max_turns: Some(1),
        ..Default::default()
    };

    let mut client = ClaudeSDKClient::new(options);

    if let Err(e) = client.connect().await {
        eprintln!("Connection error: {}", e);
        return;
    }

    if let Err(e) = client.query("What is 2 + 2?").await {
        eprintln!("Query error: {}", e);
        return;
    }

    let mut stream = client.receive_response();
    while let Some(result) = stream.next().await {
        match result {
            Ok(message) => {
                if let Message::System(ref msg) = message {
                    if msg.subtype == "init" {
                        let commands = extract_slash_commands(&message);
                        println!("Available slash commands: {:?}", commands);
                        if commands.contains(&"commit".to_string()) {
                            println!("✓ /commit is available (expected)");
                        } else {
                            println!("❌ /commit is NOT available (unexpected)");
                        }
                        break;
                    }
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    let _ = client.disconnect().await;
    println!();
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run --example setting_sources <example_name>");
        println!("\nAvailable examples:");
        println!("  all              - Run all examples");
        println!("  default          - Default behavior (no settings)");
        println!("  user_only        - User settings only");
        println!("  project_and_user - Project + user settings");
        return;
    }

    println!("Claude SDK Setting Sources Examples");
    println!("{}", "=".repeat(50));
    println!();

    match args[1].as_str() {
        "all" => {
            example_default().await;
            println!("{}", "-".repeat(50));
            println!();

            example_user_only().await;
            println!("{}", "-".repeat(50));
            println!();

            example_project_and_user().await;
        }
        "default" => example_default().await,
        "user_only" => example_user_only().await,
        "project_and_user" => example_project_and_user().await,
        _ => {
            println!("Error: Unknown example '{}'", args[1]);
            println!("\nAvailable examples:");
            println!("  all              - Run all examples");
            println!("  default          - Default behavior (no settings)");
            println!("  user_only        - User settings only");
            println!("  project_and_user - Project + user settings");
        }
    }

    println!("\n{}", "=".repeat(50));
    println!("\nKey takeaways:");
    println!("- Default (None): No settings loaded - isolated environment");
    println!("- user: Only global user settings");
    println!("- project: Only project-level settings");
    println!("- Combine sources to load multiple setting locations");
}
