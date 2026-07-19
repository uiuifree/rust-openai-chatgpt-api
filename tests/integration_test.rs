//! Live API tests. Each test is skipped unless `OPENAI_API_KEY` is set, so
//! `cargo test` stays green offline. Run with a key to exercise the real API:
//!
//! ```sh
//! OPENAI_API_KEY=sk-... cargo test --test integration_test -- --nocapture
//! ```

use futures_util::StreamExt;
use openai_chatgpt_api::chat::{ChatCompletionRequest, ChatMessage};
use openai_chatgpt_api::embeddings::EmbeddingsRequest;
use openai_chatgpt_api::moderations::ModerationRequest;
use openai_chatgpt_api::responses::{ResponseRequest, ResponseStreamEvent};
use openai_chatgpt_api::OpenAiClient;

const TEXT_MODEL: &str = "gpt-5.6-luna";

fn client() -> Option<OpenAiClient> {
    match OpenAiClient::from_env() {
        Ok(client) => Some(client),
        Err(_) => {
            eprintln!("OPENAI_API_KEY not set; skipping live API test");
            None
        }
    }
}

#[tokio::test]
async fn models_list_and_retrieve() {
    let Some(client) = client() else { return };
    let list = client.models().list().await.unwrap();
    assert!(!list.data.is_empty());
    let first = &list.data[0];
    let model = client.models().retrieve(&first.id).await.unwrap();
    assert_eq!(model.id, first.id);
}

#[tokio::test]
async fn responses_create() {
    let Some(client) = client() else { return };
    let request =
        ResponseRequest::new(TEXT_MODEL, "Reply with exactly: pong").max_output_tokens(2000);
    let response = client.responses().create(&request).await.unwrap();
    assert_eq!(response.status.as_deref(), Some("completed"));
    assert!(response.output_text().to_lowercase().contains("pong"));
    assert!(response.usage.unwrap().total_tokens > 0);
}

#[tokio::test]
async fn responses_stream() {
    let Some(client) = client() else { return };
    let request = ResponseRequest::new(TEXT_MODEL, "Count from 1 to 5.").max_output_tokens(2000);
    let mut stream = client.responses().stream(&request).await.unwrap();
    let mut text = String::new();
    let mut completed = false;
    while let Some(event) = stream.next().await {
        match event.unwrap() {
            ResponseStreamEvent::OutputTextDelta { delta, .. } => text.push_str(&delta),
            ResponseStreamEvent::Completed { .. } => completed = true,
            _ => {}
        }
    }
    assert!(completed);
    assert!(!text.is_empty());
}

#[tokio::test]
async fn chat_completions_create() {
    let Some(client) = client() else { return };
    let request = ChatCompletionRequest::new(
        TEXT_MODEL,
        vec![
            ChatMessage::system("Answer in one word."),
            ChatMessage::user("What color is the sky on a clear day?"),
        ],
    )
    .max_completion_tokens(2000);
    let completion = client.chat().create(&request).await.unwrap();
    assert!(completion.content().is_some());
}

#[tokio::test]
async fn embeddings_create() {
    let Some(client) = client() else { return };
    let request =
        EmbeddingsRequest::new("text-embedding-3-small", "The food was delicious.").dimensions(256);
    let response = client.embeddings().create(&request).await.unwrap();
    assert_eq!(response.data.len(), 1);
    assert_eq!(response.data[0].embedding.len(), 256);
}

#[tokio::test]
async fn moderations_create() {
    let Some(client) = client() else { return };
    let request = ModerationRequest::new("I want to hug a puppy.").model("omni-moderation-latest");
    let moderation = client.moderations().create(&request).await.unwrap();
    assert_eq!(moderation.results.len(), 1);
    assert!(!moderation.results[0].flagged);
}

#[tokio::test]
async fn api_error_is_typed() {
    let Some(client) = client() else { return };
    let request = ResponseRequest::new("no-such-model-xyz", "hi");
    let error = client.responses().create(&request).await.unwrap_err();
    match error {
        openai_chatgpt_api::OpenAiError::Api {
            status, message, ..
        } => {
            assert_eq!(status, 400);
            assert!(!message.is_empty());
        }
        other => panic!("expected Api error, got: {other}"),
    }
}
