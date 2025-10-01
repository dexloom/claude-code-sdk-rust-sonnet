#!/usr/bin/env cargo
//! Example demonstrating custom agent definitions.
//!
//! This example shows how to define custom agents with specific:
//! - Tool allowlists/denylists
//! - System prompts
//! - Models
//!
//! Three examples are provided:
//! 1. Code reviewer agent (only allowed Read and Glob tools)
//! 2. Documentation writer agent (custom system prompt and model)
//! 3. Multiple agents working together

use claude_agent_sdk::{query, AgentDefinition, ClaudeAgentOptions};
use futures::StreamExt;
use std::collections::HashMap;

async fn example_code_reviewer() {
    println!("=== Example 1: Code Reviewer Agent ===");
    println!("Agent with restricted tools (Read and Glob only)\n");

    let mut agents = HashMap::new();
    agents.insert(
        "code-reviewer".to_string(),
        AgentDefinition {
            allowed_tools: Some(vec!["Read".to_string(), "Glob".to_string()]),
            system_prompt: Some(
                "You are a code reviewer. Review code for best practices, potential bugs, \
                 and suggest improvements. You can only read files and search for patterns."
                    .to_string(),
            ),
            model: Some("claude-sonnet-4".to_string()),
            disallowed_tools: None,
        },
    );

    let options = ClaudeAgentOptions {
        agents: Some(agents),
        max_turns: Some(2),
        ..Default::default()
    };

    println!("Prompt: Review the code quality in examples/quick_start.rs");
    println!("{}", "-".repeat(50));

    match query("Review the code quality in examples/quick_start.rs", options).await {
        Ok(mut stream) => {
            while let Some(result) = stream.next().await {
                match result {
                    Ok(message) => {
                        if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                            if !content.is_empty() {
                                println!("\nReviewer: {}", content);
                            }
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        }
        Err(e) => eprintln!("Query error: {}", e),
    }

    println!();
}

async fn example_doc_writer() {
    println!("=== Example 2: Documentation Writer Agent ===");
    println!("Agent with custom system prompt and model\n");

    let mut agents = HashMap::new();
    agents.insert(
        "doc-writer".to_string(),
        AgentDefinition {
            system_prompt: Some(
                "You are a technical documentation writer. You write clear, concise, \
                 and comprehensive documentation. Focus on explaining complex concepts \
                 in simple terms."
                    .to_string(),
            ),
            model: Some("claude-sonnet-4".to_string()),
            allowed_tools: Some(vec!["Read".to_string(), "Glob".to_string(), "Write".to_string()]),
            disallowed_tools: None,
        },
    );

    let options = ClaudeAgentOptions {
        agents: Some(agents),
        max_turns: Some(2),
        ..Default::default()
    };

    println!("Prompt: Explain what the query function does in lib.rs");
    println!("{}", "-".repeat(50));

    match query("Explain what the query function does in lib.rs", options).await {
        Ok(mut stream) => {
            while let Some(result) = stream.next().await {
                match result {
                    Ok(message) => {
                        if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                            if !content.is_empty() {
                                println!("\nDoc Writer: {}", content);
                            }
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
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
            allowed_tools: Some(vec!["Read".to_string(), "Glob".to_string(), "Grep".to_string()]),
            system_prompt: Some(
                "You are a security auditor. Look for security vulnerabilities, \
                 unsafe code patterns, and potential security issues."
                    .to_string(),
            ),
            model: Some("claude-sonnet-4".to_string()),
            disallowed_tools: None,
        },
    );

    // Performance optimizer
    agents.insert(
        "performance-optimizer".to_string(),
        AgentDefinition {
            allowed_tools: Some(vec!["Read".to_string(), "Glob".to_string()]),
            system_prompt: Some(
                "You are a performance optimizer. Analyze code for performance bottlenecks, \
                 inefficient algorithms, and suggest optimizations."
                    .to_string(),
            ),
            model: Some("claude-sonnet-4".to_string()),
            disallowed_tools: None,
        },
    );

    // Test generator
    agents.insert(
        "test-generator".to_string(),
        AgentDefinition {
            allowed_tools: Some(vec![
                "Read".to_string(),
                "Glob".to_string(),
                "Write".to_string(),
            ]),
            system_prompt: Some(
                "You are a test generator. Create comprehensive unit tests and integration tests \
                 for code. Follow best practices for testing."
                    .to_string(),
            ),
            model: Some("claude-sonnet-4".to_string()),
            disallowed_tools: None,
        },
    );

    let options = ClaudeAgentOptions {
        agents: Some(agents),
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

    match query("What is 2 + 2?", options).await {
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
    println!("- Each agent can have its own system prompt and personality");
    println!("- You can define multiple specialized agents for different tasks");
    println!("- Agents can use different models based on their needs");
}
