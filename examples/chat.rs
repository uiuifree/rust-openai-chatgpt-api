//! Chat Completions API (fully supported; prefer the Responses API for new code).
//!
//! Run: `OPENAI_API_KEY=sk-... cargo run --example chat`

use openai_chatgpt_api::chat::{ChatCompletionRequest, ChatMessage};
use openai_chatgpt_api::OpenAiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenAiClient::from_env()?;

    let request = ChatCompletionRequest::new(
        "gpt-5.6-terra",
        vec![
            ChatMessage::system("You are a helpful assistant."),
            ChatMessage::user("Name three uses for the newtype pattern in Rust."),
        ],
    )
    .max_completion_tokens(400);

    let completion = client.chat().create(&request).await?;
    println!("{}", completion.content().unwrap_or_default());
    Ok(())
}
