#!/usr/bin/env cargo
//! Example demonstrating custom agent definitions.
//!
//! This example shows how to define custom agents with specific:
//! - Tool restrictions
//! - Custom prompts
//! - Models
//!
//! Three examples are provided:
//! 1. Code reviewer agent (only allowed Read and Glob tools)
//! 2. Documentation writer agent (custom prompt and model)
//! 3. Multiple agents working together

use claude_agent_sdk::{query, AgentDefinition, ClaudeAgentOptions, Message};
use futures::StreamExt;
use std::collections::HashMap;

async fn example_code_reviewer() {
    println!("=== Example 1: Code Reviewer Agent ===");
    println!("Agent with restricted tools (Read and Glob only)\n");

    let mut agents = HashMap::new();
    agents.insert(
        "code-reviewer".to_string(),
        AgentDefinition {
            description: "Reviews code for best practices and potential bugs".to_string(),
            prompt: "You are a code reviewer. Review code for best practices, potential bugs, \
                     and suggest improvements. You can only read files and search for patterns."
                .to_string(),
            tools: Some(vec!["Read".to_string(), "Glob".to_string()]),
            model: Some("claude-sonnet-4".to_string()),
        },
    );

    let options = ClaudeAgentOptions {
        agents,
        max_turns: Some(2),
        ..Default::default()
    };

    println!("Prompt: Review the code quality in examples/quick_start.rs");
    println!("{}", "-".repeat(50));

    match query("Review the code quality in examples/quick_start.rs".to_string(), options).await {
        Ok(mut stream) => {
            while let Some(result) = stream.next().await {
                match result {
                    Ok(Message::Assistant { message, .. }) => {
                        for block in &message.message.content {
                            if let claude_agent_sdk::ContentBlock::Text { text } = block {
                                if !text.is_empty() {
                                    println!("\nReviewer: {}", text);
                                }
                            }
                        }
                    }
                    Ok(Message::Result { .. }) => break,
                    Err(e) => eprintln!("Error: {}", e),
                    _ => {}
                }
            }
        }
        Err(e) => eprintln!("Query error: {}", e),
    }

    println!();
}

async fn example_doc_writer() {
    println!("=== Example 2: Documentation Writer Agent ===");
    println!("Agent with custom prompt and model\n");

    let mut agents = HashMap::new();
    agents.insert(
        "doc-writer".to_string(),
        AgentDefinition {
            description: "Writes clear and comprehensive technical documentation".to_string(),
            prompt: "You are a technical documentation writer. You write clear, concise, \
                     and comprehensive documentation. Focus on explaining complex concepts \
                     in simple terms."
                .to_string(),
            tools: Some(vec![
                "Read".to_string(),
                "Glob".to_string(),
                "Write".to_string(),
            ]),
            model: Some("claude-sonnet-4".to_string()),
        },
    );

    let options = ClaudeAgentOptions {
        agents,
        max_turns: Some(2),
        ..Default::default()
    };

    println!("Prompt: Explain what the query function does in lib.rs");
    println!("{}", "-".repeat(50));

    match query("Explain what the query function does in lib.rs".to_string(), options).await {
        Ok(mut stream) => {
            while let Some(result) = stream.next().await {
                match result {
                    Ok(Message::Assistant { message, .. }) => {
                        for block in &message.message.content {
                            if let claude_agent_sdk::ContentBlock::Text { text } = block {
                                if !text.is_empty() {
                                    println!("\nDoc Writer: {}", text);
                                }
                            }
                        }
                    }
                    Ok(Message::Result { .. }) => break,
                    Err(e) => eprintln!("Error: {}", e),
                    _ => {}
                }
            }
        }
        Err(e) => eprintln!("Query error: {}", e),
    }

    println!();
}

async fn example_multiple_agents() {
    println!("=== Example 3: Multiple Agents ===");
    println!("Defining multiple specialized agents\n");

    let mut agents = HashMap::new();

    // Security auditor
    agents.insert(
        "security-auditor".to_string(),
        AgentDefinition {
            description: "Security vulnerability analysis specialist".to_string(),
            prompt: "You are a security auditor. Look for security vulnerabilities, \
                     unsafe code patterns, and potential security issues."
                .to_string(),
            tools: Some(vec![
                "Read".to_string(),
                "Glob".to_string(),
                "Grep".to_string(),
            ]),
            model: Some("claude-sonnet-4".to_string()),
        },
    );

    // Performance optimizer
    agents.insert(
        "performance-optimizer".to_string(),
        AgentDefinition {
            description: "Performance optimization expert".to_string(),
            prompt: "You are a performance optimizer. Analyze code for performance bottlenecks, \
                     inefficient algorithms, and suggest optimizations."
                .to_string(),
            tools: Some(vec!["Read".to_string(), "Glob".to_string()]),
            model: Some("claude-sonnet-4".to_string()),
        },
    );

    // Test generator
    agents.insert(
        "test-generator".to_string(),
        AgentDefinition {
            description: "Automated test generation specialist".to_string(),
            prompt: "You are a test generator. Create comprehensive unit tests and integration tests \
                     for code. Follow best practices for testing."
                .to_string(),
            tools: Some(vec![
                "Read".to_string(),
                "Glob".to_string(),
                "Write".to_string(),
            ]),
            model: Some("claude-sonnet-4".to_string()),
        },
    );

    let options = ClaudeAgentOptions {
        agents,
        max_turns: Some(2),
        ..Default::default()
    };

    println!("Defined agents:");
    println!("  - security-auditor: Security vulnerability analysis");
    println!("  - performance-optimizer: Performance optimization suggestions");
    println!("  - test-generator: Test generation");
    println!();
    println!("Prompt: What is 2 + 2?");
    println!("{}", "-".repeat(50));

    match query("What is 2 + 2?".to_string(), options).await {
        Ok(mut stream) => {
            while let Some(result) = stream.next().await {
                match result {
                    Ok(Message::Assistant { message, .. }) => {
                        for block in &message.message.content {
                            if let claude_agent_sdk::ContentBlock::Text { text } = block {
                                if !text.is_empty() {
                                    println!("\nResponse: {}", text);
                                }
                            }
                        }
                    }
                    Ok(Message::Result { .. }) => break,
                    Err(e) => eprintln!("Error: {}", e),
                    _ => {}
                }
            }
        }
        Err(e) => eprintln!("Query error: {}", e),
    }

    println!();
}

#[tokio::main]
async fn main() {
    println!("Custom Agent Definitions Examples");
    println!("{}", "=".repeat(50));
    println!();

    // Run all examples
    example_code_reviewer().await;
    println!("{}", "=".repeat(50));
    println!();

    example_doc_writer().await;
    println!("{}", "=".repeat(50));
    println!();

    example_multiple_agents().await;
    println!("{}", "=".repeat(50));

    println!("\nKey takeaways:");
    println!("- Agents can have restricted tool access for safety");
    println!("- Each agent can have its own prompt and personality");
    println!("- You can define multiple specialized agents for different tasks");
    println!("- Agents can use different models based on their needs");
}
