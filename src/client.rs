use reqwest::multipart::Form;
use reqwest::Method;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::audio::Audio;
use crate::chat::Chat;
use crate::embeddings::Embeddings;
use crate::error::{ApiErrorEnvelope, OpenAiError};
use crate::images::Images;
use crate::models::Models;
use crate::moderations::Moderations;
use crate::responses::Responses;

/// Default API endpoint.
pub const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";

/// Asynchronous OpenAI API client.
///
/// ```no_run
/// use openai_chatgpt_api::OpenAiClient;
/// use openai_chatgpt_api::responses::ResponseRequest;
///
/// # async fn run() -> Result<(), openai_chatgpt_api::OpenAiError> {
/// let client = OpenAiClient::from_env()?;
/// let request = ResponseRequest::new("gpt-5.6-terra", "Hello!");
/// let response = client.responses().create(&request).await?;
/// println!("{}", response.output_text());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct OpenAiClient {
    http: reqwest::Client,
    api_key: String,
    base_url: String,
    organization: Option<String>,
    project: Option<String>,
}

impl OpenAiClient {
    /// Creates a client with default settings from an API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::builder(api_key).build()
    }

    /// Builds a client from environment variables: `OPENAI_API_KEY` (required),
    /// `OPENAI_BASE_URL`, `OPENAI_ORG_ID`, and `OPENAI_PROJECT_ID` (optional).
    pub fn from_env() -> Result<Self, OpenAiError> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| OpenAiError::Config("OPENAI_API_KEY is not set".to_string()))?;
        let mut builder = Self::builder(api_key);
        if let Ok(base_url) = std::env::var("OPENAI_BASE_URL") {
            builder = builder.base_url(base_url);
        }
        if let Ok(organization) = std::env::var("OPENAI_ORG_ID") {
            builder = builder.organization(organization);
        }
        if let Ok(project) = std::env::var("OPENAI_PROJECT_ID") {
            builder = builder.project(project);
        }
        Ok(builder.build())
    }

    /// Starts a builder for customizing base URL, headers, or HTTP client.
    pub fn builder(api_key: impl Into<String>) -> OpenAiClientBuilder {
        OpenAiClientBuilder {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
            organization: None,
            project: None,
            http: None,
        }
    }

    /// Responses API (`/v1/responses`) — the primary generation API.
    pub fn responses(&self) -> Responses<'_> {
        Responses { client: self }
    }

    /// Chat Completions API (`/v1/chat/completions`).
    pub fn chat(&self) -> Chat<'_> {
        Chat { client: self }
    }

    /// Embeddings API (`/v1/embeddings`).
    pub fn embeddings(&self) -> Embeddings<'_> {
        Embeddings { client: self }
    }

    /// Image generation and editing (`/v1/images/*`).
    pub fn images(&self) -> Images<'_> {
        Images { client: self }
    }

    /// Speech synthesis, transcription, and translation (`/v1/audio/*`).
    pub fn audio(&self) -> Audio<'_> {
        Audio { client: self }
    }

    /// Model listing and management (`/v1/models`).
    pub fn models(&self) -> Models<'_> {
        Models { client: self }
    }

    /// Content moderation (`/v1/moderations`).
    pub fn moderations(&self) -> Moderations<'_> {
        Moderations { client: self }
    }

    fn request(&self, method: Method, path: &str) -> reqwest::RequestBuilder {
        let mut request = self
            .http
            .request(method, format!("{}{}", self.base_url, path))
            .bearer_auth(&self.api_key);
        if let Some(organization) = &self.organization {
            request = request.header("OpenAI-Organization", organization);
        }
        if let Some(project) = &self.project {
            request = request.header("OpenAI-Project", project);
        }
        request
    }

    pub(crate) async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, OpenAiError> {
        let response = self.request(Method::GET, path).send().await?;
        Self::parse_body(Self::check(response).await?).await
    }

    pub(crate) async fn post_json<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Result<T, OpenAiError> {
        let response = self.request(Method::POST, path).json(body).send().await?;
        Self::parse_body(Self::check(response).await?).await
    }

    pub(crate) async fn post_empty<T: DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, OpenAiError> {
        let response = self.request(Method::POST, path).send().await?;
        Self::parse_body(Self::check(response).await?).await
    }

    pub(crate) async fn delete_json<T: DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, OpenAiError> {
        let response = self.request(Method::DELETE, path).send().await?;
        Self::parse_body(Self::check(response).await?).await
    }

    /// POSTs a multipart form and returns the raw response body.
    pub(crate) async fn post_multipart_text(
        &self,
        path: &str,
        form: Form,
    ) -> Result<String, OpenAiError> {
        let response = self
            .request(Method::POST, path)
            .multipart(form)
            .send()
            .await?;
        Ok(Self::check(response).await?.text().await?)
    }

    pub(crate) async fn post_multipart_json<T: DeserializeOwned>(
        &self,
        path: &str,
        form: Form,
    ) -> Result<T, OpenAiError> {
        let text = self.post_multipart_text(path, form).await?;
        serde_json::from_str(&text).map_err(|e| OpenAiError::parse(e, &text))
    }

    /// POSTs JSON and returns the raw binary response body (audio synthesis).
    pub(crate) async fn post_json_bytes(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Result<Vec<u8>, OpenAiError> {
        let response = self.request(Method::POST, path).json(body).send().await?;
        Ok(Self::check(response).await?.bytes().await?.to_vec())
    }

    /// POSTs JSON and returns the response for server-sent-event consumption.
    pub(crate) async fn post_json_sse(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Result<reqwest::Response, OpenAiError> {
        let response = self.request(Method::POST, path).json(body).send().await?;
        Self::check(response).await
    }

    async fn check(response: reqwest::Response) -> Result<reqwest::Response, OpenAiError> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        let status = status.as_u16();
        let body = response.text().await.unwrap_or_default();
        match serde_json::from_str::<ApiErrorEnvelope>(&body) {
            Ok(envelope) => Err(OpenAiError::Api {
                status,
                message: envelope.error.message,
                error_type: envelope.error.error_type,
                code: envelope.error.code,
                param: envelope.error.param,
            }),
            Err(_) => Err(OpenAiError::Api {
                status,
                message: body,
                error_type: None,
                code: None,
                param: None,
            }),
        }
    }

    async fn parse_body<T: DeserializeOwned>(
        response: reqwest::Response,
    ) -> Result<T, OpenAiError> {
        let text = response.text().await?;
        serde_json::from_str(&text).map_err(|e| OpenAiError::parse(e, &text))
    }
}

/// Builder for [`OpenAiClient`].
#[derive(Debug)]
pub struct OpenAiClientBuilder {
    api_key: String,
    base_url: String,
    organization: Option<String>,
    project: Option<String>,
    http: Option<reqwest::Client>,
}

impl OpenAiClientBuilder {
    /// Overrides the API endpoint (proxies, Azure gateways, mock servers).
    /// A trailing slash is stripped.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        self.base_url = base_url
            .strip_suffix('/')
            .map(str::to_string)
            .unwrap_or(base_url);
        self
    }

    /// Sets the `OpenAI-Organization` header.
    pub fn organization(mut self, organization: impl Into<String>) -> Self {
        self.organization = Some(organization.into());
        self
    }

    /// Sets the `OpenAI-Project` header.
    pub fn project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// Supplies a preconfigured `reqwest::Client` (timeouts, proxies, ...).
    pub fn http_client(mut self, http: reqwest::Client) -> Self {
        self.http = Some(http);
        self
    }

    /// Finalizes the builder into a client.
    pub fn build(self) -> OpenAiClient {
        OpenAiClient {
            http: self.http.unwrap_or_default(),
            api_key: self.api_key,
            base_url: self.base_url,
            organization: self.organization,
            project: self.project,
        }
    }
}

/// Result of a DELETE call (responses, fine-tuned models).
#[derive(Debug, Clone, Deserialize)]
pub struct DeletedObject {
    /// Id of the deleted object.
    pub id: String,
    /// Object type of the deleted object.
    pub object: Option<String>,
    /// Whether the deletion succeeded.
    #[serde(default)]
    pub deleted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_strips_trailing_slash() {
        let client = OpenAiClient::builder("k")
            .base_url("https://example.com/v1/")
            .build();
        assert_eq!(client.base_url, "https://example.com/v1");
    }

    #[test]
    fn builder_defaults() {
        let client = OpenAiClient::new("k");
        assert_eq!(client.base_url, DEFAULT_BASE_URL);
        assert!(client.organization.is_none());
        assert!(client.project.is_none());
    }
}
