//! Basic text generation with the Responses API (the current primary API).
//!
//! Run: `OPENAI_API_KEY=sk-... cargo run --example responses`

use openai_chatgpt_api::responses::ResponseRequest;
use openai_chatgpt_api::OpenAiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenAiClient::from_env()?;

    let request = ResponseRequest::new(
        "gpt-5.6-terra",
        "Explain Rust's ownership model in three sentences.",
    )
    .instructions("You are a concise assistant.")
    .max_output_tokens(300);

    let response = client.responses().create(&request).await?;

    println!("{}", response.output_text());
    if let Some(usage) = &response.usage {
        println!("--- {} tokens total", usage.total_tokens);
    }
    Ok(())
}
