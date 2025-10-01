//! Quick start example for Claude Agent SDK

use claude_agent_sdk::{query, ClaudeAgentOptions, ContentBlock, Message};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple query with default options
    let options = ClaudeAgentOptions::default();
    let mut stream = query("What is 2 + 2?".to_string(), options).await?;

    println!("Querying Claude Code...\n");

    while let Some(message) = stream.next().await {
        match message? {
            Message::Assistant { message, .. } => {
                for block in message.message.content {
                    match block {
                        ContentBlock::Text { text } => {
                            println!("Claude: {}", text);
                        }
                        ContentBlock::Thinking { thinking, .. } => {
                            println!("[Thinking: {}]", thinking);
                        }
                        ContentBlock::ToolUse { name, .. } => {
                            println!("[Using tool: {}]", name);
                        }
                        ContentBlock::ToolResult { tool_use_id, .. } => {
                            println!("[Tool result for: {}]", tool_use_id);
                        }
                    }
                }
            }
            Message::Result {
                duration_ms,
                num_turns,
                total_cost_usd,
                ..
            } => {
                println!("\n--- Session Complete ---");
                println!("Duration: {}ms", duration_ms);
                println!("Turns: {}", num_turns);
                if let Some(cost) = total_cost_usd {
                    println!("Cost: ${:.4}", cost);
                }
            }
            _ => {}
        }
    }

    Ok(())
}
