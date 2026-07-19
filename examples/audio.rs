//! Text-to-speech, then transcribe the generated audio back to text.
//!
//! Run: `OPENAI_API_KEY=sk-... cargo run --example audio`

use openai_chatgpt_api::audio::{SpeechRequest, TranscriptionRequest};
use openai_chatgpt_api::{FilePart, OpenAiClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenAiClient::from_env()?;

    // Text to speech
    let request = SpeechRequest::new(
        "gpt-4o-mini-tts",
        "Hello! This audio was generated from Rust.",
        "marin",
    );
    let mp3_bytes = client.audio().speech(&request).await?;
    std::fs::write("speech.mp3", &mp3_bytes)?;
    println!("saved speech.mp3 ({} bytes)", mp3_bytes.len());

    // Speech to text
    let request =
        TranscriptionRequest::new("gpt-4o-transcribe", FilePart::new("speech.mp3", mp3_bytes))
            .language("en");
    let transcript = client.audio().transcribe(request).await?;
    println!("transcript: {}", transcript.text);
    Ok(())
}
