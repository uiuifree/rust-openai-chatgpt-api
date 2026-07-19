//! Embeddings API (`/v1/embeddings`).
//!
//! Current models: `text-embedding-3-large`, `text-embedding-3-small`.

use serde::{Deserialize, Serialize};

use crate::client::OpenAiClient;
use crate::error::OpenAiError;

/// Accessor returned by [`OpenAiClient::embeddings`].
pub struct Embeddings<'a> {
    pub(crate) client: &'a OpenAiClient,
}

impl Embeddings<'_> {
    /// Creates embedding vectors (`POST /v1/embeddings`).
    pub async fn create(
        &self,
        request: &EmbeddingsRequest,
    ) -> Result<EmbeddingsResponse, OpenAiError> {
        self.client.post_json("/embeddings", request).await
    }
}

/// Request body for `POST /v1/embeddings`.
#[derive(Debug, Clone, Serialize)]
pub struct EmbeddingsRequest {
    model: String,
    input: EmbeddingInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    dimensions: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    encoding_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
}

impl EmbeddingsRequest {
    /// Creates a request. `input` accepts a single text or a batch of texts.
    pub fn new(model: impl Into<String>, input: impl Into<EmbeddingInput>) -> Self {
        Self {
            model: model.into(),
            input: input.into(),
            dimensions: None,
            encoding_format: None,
            user: None,
        }
    }

    /// Truncates `text-embedding-3-*` vectors to the given dimension count.
    pub fn dimensions(mut self, dimensions: u32) -> Self {
        self.dimensions = Some(dimensions);
        self
    }

    /// `"float"` (default) or `"base64"`.
    pub fn encoding_format(mut self, encoding_format: impl Into<String>) -> Self {
        self.encoding_format = Some(encoding_format.into());
        self
    }

    /// Stable end-user identifier for abuse monitoring.
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }
}

/// `input` parameter: one text or a batch.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum EmbeddingInput {
    /// A single text.
    Text(String),
    /// A batch of texts, embedded in one request.
    Texts(Vec<String>),
}

impl From<&str> for EmbeddingInput {
    fn from(text: &str) -> Self {
        EmbeddingInput::Text(text.to_string())
    }
}

impl From<String> for EmbeddingInput {
    fn from(text: String) -> Self {
        EmbeddingInput::Text(text)
    }
}

impl From<Vec<String>> for EmbeddingInput {
    fn from(texts: Vec<String>) -> Self {
        EmbeddingInput::Texts(texts)
    }
}

impl From<Vec<&str>> for EmbeddingInput {
    fn from(texts: Vec<&str>) -> Self {
        EmbeddingInput::Texts(texts.into_iter().map(str::to_string).collect())
    }
}

/// Response body of the embeddings endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingsResponse {
    /// Always `"list"`.
    pub object: Option<String>,
    /// One embedding per input, in input order.
    #[serde(default)]
    pub data: Vec<Embedding>,
    /// Model that produced the embeddings.
    pub model: Option<String>,
    /// Token accounting.
    pub usage: Option<EmbeddingsUsage>,
}

/// A single embedding vector.
#[derive(Debug, Clone, Deserialize)]
pub struct Embedding {
    /// Always `"embedding"`.
    pub object: Option<String>,
    /// Position of the corresponding input.
    #[serde(default)]
    pub index: u32,
    /// The embedding vector.
    pub embedding: Vec<f32>,
}

/// Token accounting for an embeddings request.
#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingsUsage {
    /// Tokens in the input texts.
    #[serde(default)]
    pub prompt_tokens: u64,
    /// Same as `prompt_tokens` (embeddings produce no output tokens).
    #[serde(default)]
    pub total_tokens: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_single_and_batch_input() {
        let single = EmbeddingsRequest::new("text-embedding-3-small", "hello").dimensions(256);
        let value = serde_json::to_value(&single).unwrap();
        assert_eq!(value["input"], "hello");
        assert_eq!(value["dimensions"], 256);

        let batch = EmbeddingsRequest::new("text-embedding-3-large", vec!["a", "b"]);
        let value = serde_json::to_value(&batch).unwrap();
        assert_eq!(value["input"], json!(["a", "b"]));
    }

    #[test]
    fn deserializes_response() {
        let body = json!({
            "object": "list",
            "data": [{"object": "embedding", "index": 0, "embedding": [0.1, -0.2]}],
            "model": "text-embedding-3-small",
            "usage": {"prompt_tokens": 2, "total_tokens": 2}
        });
        let response: EmbeddingsResponse = serde_json::from_value(body).unwrap();
        assert_eq!(response.data[0].embedding, vec![0.1, -0.2]);
        assert_eq!(response.usage.unwrap().total_tokens, 2);
    }
}
