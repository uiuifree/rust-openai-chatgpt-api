//! Embedding vectors with cosine similarity between two texts.
//!
//! Run: `OPENAI_API_KEY=sk-... cargo run --example embeddings`

use openai_chatgpt_api::embeddings::EmbeddingsRequest;
use openai_chatgpt_api::OpenAiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenAiClient::from_env()?;

    let request = EmbeddingsRequest::new(
        "text-embedding-3-small",
        vec!["The cat sits on the mat.", "A feline rests on a rug."],
    );
    let response = client.embeddings().create(&request).await?;

    let a = &response.data[0].embedding;
    let b = &response.data[1].embedding;
    println!("dimensions: {}", a.len());
    println!("cosine similarity: {:.4}", cosine_similarity(a, b));
    Ok(())
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / (norm_a * norm_b)
}
