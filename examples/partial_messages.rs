#!/usr/bin/env cargo
//! Example of using the "include_partial_messages" option to stream partial messages
//! from Claude Code SDK.
//!
//! This feature allows you to receive stream events that contain incremental
//! updates as Claude generates responses. This is useful for:
//! - Building real-time UIs that show text as it's being generated
//! - Monitoring tool use progress
//! - Getting early results before the full response is complete
//!
//! Note: Partial message streaming requires the CLI to support it, and the
//! messages will include StreamEvent messages interspersed with regular messages.

use claude_agent_sdk::{ClaudeAgentOptions, ClaudeSDKClient, Message};
use futures::StreamExt;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    println!("Partial Message Streaming Example");
    println!("{}", "=".repeat(50));
    println!();

    // Enable partial message streaming with extended thinking
    let mut env = HashMap::new();
    env.insert("MAX_THINKING_TOKENS".to_string(), "8000".to_string());

    let options = ClaudeAgentOptions {
        include_partial_messages: true,
        model: Some("claude-sonnet-4-5".to_string()),
        max_turns: Some(2),
        env,
        ..Default::default()
    };

    let mut client = ClaudeSDKClient::new(options);

    match client.connect().await {
        Ok(_) => {
            // Send a prompt that will generate a streaming response
            let prompt = "Think of three jokes, then tell one";
            println!("Prompt: {}\n", prompt);
            println!("{}", "=".repeat(50));

            if let Err(e) = client.query(prompt).await {
                eprintln!("Query error: {}", e);
                return;
            }

            let mut message_count = 0;
            let mut stream_event_count = 0;
            let mut partial_text = String::new();

            let mut stream = client.receive_response();
            while let Some(result) = stream.next().await {
                match result {
                    Ok(message) => {
                        message_count += 1;

                        match &message {
                            Message::StreamEvent(event) => {
                                stream_event_count += 1;
                                println!("\n📡 StreamEvent #{}", stream_event_count);
                                println!("   Type: {}", event.event_type);

                                // Show partial content if available
                                if let Some(delta) = event.data.get("delta") {
                                    if let Some(text) = delta.get("text").and_then(|t| t.as_str()) {
                                        partial_text.push_str(text);
                                        print!("{}", text);
                                        std::io::Write::flush(&mut std::io::stdout()).ok();
                                    }
                                }

                                // Show thinking if available
                                if let Some(thinking) = event.data.get("thinking").and_then(|t| t.as_str()) {
                                    if !thinking.is_empty() {
                                        println!("\n💭 Thinking: {}", thinking);
                                    }
                                }
                            }
                            Message::Assistant(msg) => {
                                if !msg.content.is_empty() {
                                    println!("\n\n✅ Complete Assistant Message:");
                                    println!("   {}", msg.content);
                                }

                                // Show thinking blocks
                                for block in &msg.content_blocks {
                                    if block.block_type == "thinking" {
                                        if let Some(thinking) = block.text.as_ref() {
                                            println!("\n💭 Complete Thinking:");
                                            println!("   {}", thinking);
                                        }
                                    }
                                }
                            }
                            Message::Result(msg) => {
                                println!("\n\n🏁 Result: {:?}", msg.result_type);
                                break;
                            }
                            Message::User(msg) => {
                                println!("\n👤 User: {}", msg.content);
                            }
                            Message::System(msg) => {
                                if msg.subtype != "init" {
                                    println!("\n⚙️  System [{}]: {:?}", msg.subtype, msg.data);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("\nError: {}", e);
                        break;
                    }
                }
            }

            println!("\n\n{}", "=".repeat(50));
            println!("Statistics:");
            println!("  Total messages: {}", message_count);
            println!("  Stream events: {}", stream_event_count);
            println!("  Partial text length: {} chars", partial_text.len());

            if let Err(e) = client.disconnect().await {
                eprintln!("Disconnect error: {}", e);
            }
        }
        Err(e) => eprintln!("Connection error: {}", e),
    }

    println!("\n{}", "=".repeat(50));
    println!("\nKey takeaways:");
    println!("- include_partial_messages enables real-time streaming");
    println!("- StreamEvent messages show incremental updates");
    println!("- Useful for building responsive UIs");
    println!("- You still receive complete messages at the end");
    println!("- Can monitor thinking process in real-time");
}
