pub(crate) mod http;
pub(crate) mod error;
mod v1;

use serde_json::{Value};
use crate::error::ChatGptError;
use crate::http::HttpClient;

pub use crate::v1::models::*;
pub use crate::v1::completions::*;
pub use crate::v1::chat::*;
pub use crate::v1::edits::*;
pub use crate::v1::images::*;
pub use crate::v1::embeddings::*;
pub use crate::v1::audio::*;
pub use crate::v1::{ChatGptResponse, ChatGptRequest};

pub struct ChatGpt {
    oepenai_api_key: String,
    org_id: String,
}


impl ChatGpt {
    pub fn new(oepenai_api_key: &str) -> Self {
        Self {
            oepenai_api_key: oepenai_api_key.to_string(),
            org_id: "".to_string(),
        }
    }
    pub fn new_org(oepenai_api_key: String, org_id: String) -> Self {
        Self {
            oepenai_api_key,
            org_id,
        }
    }


    pub async fn models_list(&self) -> Result<ChatGptResponseModelList, ChatGptError> {
        let url = "https://api.openai.com/v1/models";
        let value = HttpClient::get(self.oepenai_api_key.as_str(), self.org_id.as_str(), url, &Value::default()).await?;
        Ok(ChatGptResponseModelList { value })
    }
    pub async fn models_retrieve(&self, model: &str) -> Result<ChatGptResponseModelRetrieve, ChatGptError> {
        let url = format!("https://api.openai.com/v1/models/{}", model);
        let value = HttpClient::get(self.oepenai_api_key.as_str(), self.org_id.as_str(), url.as_str(), &Value::default()).await?;
        Ok(ChatGptResponseModelRetrieve { value })
    }
    pub async fn completions_create(&self, request: &ChatGptRequestCompletionsCreate) -> Result<ChatGptResponseModelRetrieve, ChatGptError> {
        let url = "https://api.openai.com/v1/completions";

        let value = HttpClient::post(self.oepenai_api_key.as_str(), self.org_id.as_str(), url, &request.to_value()).await?;
        Ok(ChatGptResponseModelRetrieve { value })
    }
    pub async fn chat_completions(&self, request: &ChatGptRequestChatCompletions) -> Result<ChatGptResponseChatCompletions, ChatGptError> {
        let url = "https://api.openai.com/v1/chat/completions";
        let data = request.to_value();
        let value = HttpClient::post(self.oepenai_api_key.as_str(), self.org_id.as_str(), url, &data).await?;
        Ok(ChatGptResponseChatCompletions { value })
    }
    pub async fn edits(&self, request: &ChatGptRequestEdits) -> Result<Value, ChatGptError> {
        let url = "https://api.openai.com/v1/edits";
        let data = request.to_value();
        HttpClient::post(self.oepenai_api_key.as_str(), self.org_id.as_str(), url, &data).await
    }
    pub async fn images_generations(&self, request: &ChatGptRequestImagesGenerations) -> Result<ChatGptResponseImagesGenerations, ChatGptError> {
        let url = "https://api.openai.com/v1/images/generations";
        let data = request.to_value();
        let value = HttpClient::post(self.oepenai_api_key.as_str(), self.org_id.as_str(), url, &data).await?;
        Ok(ChatGptResponseImagesGenerations { value })
    }
    pub async fn images_edits(&self, request: &ChatGptRequestImagesEdits) -> Result<ChatGptResponseImagesEdits, ChatGptError> {
        let url = "https://api.openai.com/v1/images/edits";
        let value = HttpClient::post_data(self.oepenai_api_key.as_str(), self.org_id.as_str(), url, request.clone().into()).await?;
        Ok(ChatGptResponseImagesEdits { value })
    }
    pub async fn images_variations(&self, request: &ChatGptRequestImagesVariation) -> Result<ChatGptResponseImagesVariation, ChatGptError> {
        let url = "https://api.openai.com/v1/images/variations";
        let value = HttpClient::post_data(self.oepenai_api_key.as_str(), self.org_id.as_str(), url, request.clone().into()).await?;
        Ok(ChatGptResponseImagesVariation { value })
    }
    pub async fn embeddings(&self, request: &ChatGptRequestEmbeddingsGenerations) -> Result<Value, ChatGptError> {
        let url = "https://api.openai.com/v1/embeddings";
        let data = request.to_value();
        HttpClient::post(self.oepenai_api_key.as_str(), self.org_id.as_str(), url, &data).await
    }
    pub async fn audio_transcriptions(&self, request: &ChatGptRequestAudioTranscriptions) -> Result<ChatGptResponseAudioTranscriptions, ChatGptError> {
        let url = "https://api.openai.com/v1/audio/transcriptions";
        let value = HttpClient::post_data(
            self.oepenai_api_key.as_str(),
            self.org_id.as_str(),
            url,
            request.clone().into(),
        ).await?;
        Ok(ChatGptResponseAudioTranscriptions { value })
    }
    pub async fn audio_translations(&self, request: &ChatGptRequestAudioTranslations) -> Result<ChatGptResponseAudioTranslations, ChatGptError> {
        let url = "https://api.openai.com/v1/audio/translations";
        let value = HttpClient::post_data(
            self.oepenai_api_key.as_str(),
            self.org_id.as_str(),
            url,
            request.clone().into(),
        ).await?;
        Ok(ChatGptResponseAudioTranslations { value })
    }
}


