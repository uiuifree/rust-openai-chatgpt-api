//! Models API (`/v1/models`).

use serde::Deserialize;

use crate::client::{DeletedObject, OpenAiClient};
use crate::error::OpenAiError;

/// Accessor returned by [`OpenAiClient::models`].
pub struct Models<'a> {
    pub(crate) client: &'a OpenAiClient,
}

impl Models<'_> {
    /// Lists the models available to your account (`GET /v1/models`).
    pub async fn list(&self) -> Result<ModelList, OpenAiError> {
        self.client.get_json("/models").await
    }

    /// Fetches one model's metadata (`GET /v1/models/{id}`).
    pub async fn retrieve(&self, model_id: &str) -> Result<Model, OpenAiError> {
        self.client.get_json(&format!("/models/{model_id}")).await
    }

    /// Deletes a fine-tuned model owned by your organization.
    pub async fn delete(&self, model_id: &str) -> Result<DeletedObject, OpenAiError> {
        self.client
            .delete_json(&format!("/models/{model_id}"))
            .await
    }
}

/// Response body of the model list endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelList {
    /// Always `"list"`.
    pub object: Option<String>,
    /// The available models.
    #[serde(default)]
    pub data: Vec<Model>,
}

/// Metadata of one model.
#[derive(Debug, Clone, Deserialize)]
pub struct Model {
    /// Model identifier usable in requests (e.g. `"gpt-5.6-terra"`).
    pub id: String,
    /// Always `"model"`.
    pub object: Option<String>,
    /// Unix timestamp of model creation.
    pub created: Option<u64>,
    /// Owning organization (e.g. `"openai"` or your org for fine-tunes).
    pub owned_by: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserializes_model_list() {
        let body = json!({
            "object": "list",
            "data": [
                {"id": "gpt-5.6-terra", "object": "model", "created": 1_780_000_000u64, "owned_by": "openai"},
                {"id": "text-embedding-3-small", "object": "model"}
            ]
        });
        let list: ModelList = serde_json::from_value(body).unwrap();
        assert_eq!(list.data.len(), 2);
        assert_eq!(list.data[0].id, "gpt-5.6-terra");
        assert_eq!(list.data[1].created, None);
    }
}
