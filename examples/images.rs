//! Generate an image with gpt-image-2 and save the base64 result to disk.
//!
//! Run: `OPENAI_API_KEY=sk-... cargo run --example images`

use base64::Engine;
use openai_chatgpt_api::images::ImageGenerationRequest;
use openai_chatgpt_api::OpenAiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenAiClient::from_env()?;

    let request = ImageGenerationRequest::new(
        "gpt-image-2",
        "A watercolor painting of a wooden Japanese house in the rain",
    )
    .size("1024x1024")
    .quality("medium");

    let images = client.images().generate(&request).await?;

    for (i, b64) in images.b64_images().iter().enumerate() {
        let bytes = base64::engine::general_purpose::STANDARD.decode(b64)?;
        let path = format!("image_{i}.png");
        std::fs::write(&path, bytes)?;
        println!("saved {path}");
    }
    Ok(())
}
