//! Stream a response token by token via server-sent events.
//!
//! Run: `OPENAI_API_KEY=sk-... cargo run --example streaming`

use std::io::Write;

use futures_util::StreamExt;
use openai_chatgpt_api::responses::{ResponseRequest, ResponseStreamEvent};
use openai_chatgpt_api::OpenAiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenAiClient::from_env()?;

    let request = ResponseRequest::new(
        "gpt-5.6-luna",
        "Write a haiku about the Rust borrow checker.",
    );
    let mut stream = client.responses().stream(&request).await?;

    while let Some(event) = stream.next().await {
        match event? {
            ResponseStreamEvent::OutputTextDelta { delta, .. } => {
                print!("{delta}");
                std::io::stdout().flush()?;
            }
            ResponseStreamEvent::Completed { response } => {
                println!();
                if let Some(usage) = &response.usage {
                    println!("--- {} tokens total", usage.total_tokens);
                }
            }
            _ => {}
        }
    }
    Ok(())
}
