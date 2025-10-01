//! Streaming mode example with ClaudeSDKClient

use claude_agent_sdk::{ClaudeAgentOptions, ClaudeSDKClient, ContentBlock, Message};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create options
    let options = ClaudeAgentOptions {
        allowed_tools: vec!["Read".to_string(), "Bash".to_string()],
        permission_mode: Some("acceptEdits".to_string()),
        ..Default::default()
    };

    // Create and connect client
    let mut client = ClaudeSDKClient::new(options);
    client.connect().await?;

    println!("Connected to Claude Code!\n");

    // Send first query
    client.query("What files are in the current directory?".to_string()).await?;

    // Receive response
    {
        let mut stream = client.receive_messages();
        while let Some(message) = stream.next().await {
            match message? {
                Message::Assistant { message, .. } => {
                    for block in message.message.content {
                        if let ContentBlock::Text { text } = block {
                            println!("Claude: {}", text);
                        }
                    }
                }
                Message::Result { .. } => {
                    println!("\n--- First query complete ---\n");
                    break;
                }
                _ => {}
            }
        }
    }

    // Send follow-up query
    client
        .query("Now list all Rust files in the src directory".to_string())
        .await?;

    // Receive second response
    {
        let mut stream = client.receive_messages();
        while let Some(message) = stream.next().await {
            match message? {
                Message::Assistant { message, .. } => {
                    for block in message.message.content {
                        if let ContentBlock::Text { text } = block {
                            println!("Claude: {}", text);
                        }
                    }
                }
                Message::Result { .. } => {
                    println!("\n--- Second query complete ---");
                    break;
                }
                _ => {}
            }
        }
    }

    // Disconnect
    client.disconnect().await?;
    println!("Disconnected.");

    Ok(())
}
