//! Moderations API (`/v1/moderations`).
//!
//! Current model: `omni-moderation-latest` (text and images).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::client::OpenAiClient;
use crate::error::OpenAiError;

/// Accessor returned by [`OpenAiClient::moderations`].
pub struct Moderations<'a> {
    pub(crate) client: &'a OpenAiClient,
}

impl Moderations<'_> {
    /// Classifies content for policy violations (`POST /v1/moderations`).
    pub async fn create(&self, request: &ModerationRequest) -> Result<Moderation, OpenAiError> {
        self.client.post_json("/moderations", request).await
    }
}

/// Request body for `POST /v1/moderations`.
#[derive(Debug, Clone, Serialize)]
pub struct ModerationRequest {
    input: ModerationInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
}

impl ModerationRequest {
    /// Creates a request. `input` accepts a single text or a batch of texts.
    pub fn new(input: impl Into<ModerationInput>) -> Self {
        Self {
            input: input.into(),
            model: None,
        }
    }

    /// Defaults to the API's default moderation model; pass
    /// `"omni-moderation-latest"` explicitly for multimodal input.
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
}

/// `input` parameter: text, a batch of texts, or multimodal items.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ModerationInput {
    /// A single text.
    Text(String),
    /// A batch of texts, each classified separately.
    Texts(Vec<String>),
    /// Multimodal input items (e.g. `{"type": "image_url", ...}`).
    Items(Value),
}

impl From<&str> for ModerationInput {
    fn from(text: &str) -> Self {
        ModerationInput::Text(text.to_string())
    }
}

impl From<String> for ModerationInput {
    fn from(text: String) -> Self {
        ModerationInput::Text(text)
    }
}

impl From<Vec<String>> for ModerationInput {
    fn from(texts: Vec<String>) -> Self {
        ModerationInput::Texts(texts)
    }
}

impl From<Vec<&str>> for ModerationInput {
    fn from(texts: Vec<&str>) -> Self {
        ModerationInput::Texts(texts.into_iter().map(str::to_string).collect())
    }
}

/// Response body of the moderation endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct Moderation {
    /// Unique identifier (`modr-...`).
    pub id: Option<String>,
    /// Moderation model that produced the verdicts.
    pub model: Option<String>,
    /// One verdict per input, in input order.
    #[serde(default)]
    pub results: Vec<ModerationResult>,
}

/// Per-input moderation verdict. Category keys use the API's names
/// (e.g. `"harassment/threatening"`).
#[derive(Debug, Clone, Deserialize)]
pub struct ModerationResult {
    /// Whether any category was flagged.
    #[serde(default)]
    pub flagged: bool,
    /// Per-category boolean verdicts.
    #[serde(default)]
    pub categories: HashMap<String, bool>,
    /// Per-category confidence scores (0.0-1.0).
    #[serde(default)]
    pub category_scores: HashMap<String, f64>,
    /// Which input modalities triggered each category (multimodal models).
    pub category_applied_input_types: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_request() {
        let request = ModerationRequest::new("some text").model("omni-moderation-latest");
        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(
            value,
            json!({"input": "some text", "model": "omni-moderation-latest"})
        );
    }

    #[test]
    fn deserializes_result() {
        let body = json!({
            "id": "modr-1",
            "model": "omni-moderation-latest",
            "results": [{
                "flagged": true,
                "categories": {"violence": true, "harassment/threatening": false},
                "category_scores": {"violence": 0.98, "harassment/threatening": 0.01}
            }]
        });
        let moderation: Moderation = serde_json::from_value(body).unwrap();
        let result = &moderation.results[0];
        assert!(result.flagged);
        assert!(result.categories["violence"]);
        assert!(result.category_scores["harassment/threatening"] < 0.5);
    }
}
