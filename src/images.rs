//! Images API (`/v1/images/generations`, `/v1/images/edits`).
//!
//! Current models: `gpt-image-2`, `gpt-image-1`, `gpt-image-1-mini`.
//! DALL·E models and the variations endpoint were removed from the API in May 2026.
//! `gpt-image-*` models always return base64-encoded images (`b64_json`).

use reqwest::multipart::Form;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::client::OpenAiClient;
use crate::error::OpenAiError;
use crate::file::FilePart;

/// Accessor returned by [`OpenAiClient::images`].
pub struct Images<'a> {
    pub(crate) client: &'a OpenAiClient,
}

impl Images<'_> {
    /// Generates images from a text prompt (`POST /v1/images/generations`).
    pub async fn generate(
        &self,
        request: &ImageGenerationRequest,
    ) -> Result<ImagesResponse, OpenAiError> {
        self.client.post_json("/images/generations", request).await
    }

    /// Edits or extends images. Consumes the request because it uploads the
    /// image bytes as a multipart form.
    pub async fn edit(&self, request: ImageEditRequest) -> Result<ImagesResponse, OpenAiError> {
        let form = request.into_form()?;
        self.client.post_multipart_json("/images/edits", form).await
    }
}

/// Request body for `POST /v1/images/generations`.
#[derive(Debug, Clone, Serialize)]
pub struct ImageGenerationRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_compression: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    moderation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
}

impl ImageGenerationRequest {
    /// Creates a request from a model id (e.g. `"gpt-image-2"`) and a prompt.
    pub fn new(model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            prompt: prompt.into(),
            n: None,
            size: None,
            quality: None,
            output_format: None,
            output_compression: None,
            background: None,
            moderation: None,
            user: None,
        }
    }

    /// Number of images to generate (1-10).
    pub fn n(mut self, n: u8) -> Self {
        self.n = Some(n);
        self
    }

    /// `"1024x1024"`, `"1536x1024"`, `"1024x1536"`, or `"auto"`.
    pub fn size(mut self, size: impl Into<String>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// `"low"`, `"medium"`, `"high"`, or `"auto"`.
    pub fn quality(mut self, quality: impl Into<String>) -> Self {
        self.quality = Some(quality.into());
        self
    }

    /// `"png"`, `"jpeg"`, or `"webp"`.
    pub fn output_format(mut self, output_format: impl Into<String>) -> Self {
        self.output_format = Some(output_format.into());
        self
    }

    /// Compression level 0-100 for `jpeg`/`webp` output.
    pub fn output_compression(mut self, output_compression: u8) -> Self {
        self.output_compression = Some(output_compression);
        self
    }

    /// `"transparent"`, `"opaque"`, or `"auto"`.
    pub fn background(mut self, background: impl Into<String>) -> Self {
        self.background = Some(background.into());
        self
    }

    /// `"auto"` (default) or `"low"`.
    pub fn moderation(mut self, moderation: impl Into<String>) -> Self {
        self.moderation = Some(moderation.into());
        self
    }

    /// Stable end-user identifier for abuse monitoring.
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }
}

/// Request for `POST /v1/images/edits` (multipart).
#[derive(Debug, Clone)]
pub struct ImageEditRequest {
    model: String,
    prompt: String,
    images: Vec<FilePart>,
    mask: Option<FilePart>,
    n: Option<u8>,
    size: Option<String>,
    quality: Option<String>,
    output_format: Option<String>,
    background: Option<String>,
    user: Option<String>,
}

impl ImageEditRequest {
    /// Creates a request from a model id, the base image, and an edit prompt.
    pub fn new(model: impl Into<String>, image: FilePart, prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            prompt: prompt.into(),
            images: vec![image],
            mask: None,
            n: None,
            size: None,
            quality: None,
            output_format: None,
            background: None,
            user: None,
        }
    }

    /// Adds a reference image (`gpt-image-*` models accept several).
    pub fn add_image(mut self, image: FilePart) -> Self {
        self.images.push(image);
        self
    }

    /// Transparent areas of the mask indicate where to edit.
    pub fn mask(mut self, mask: FilePart) -> Self {
        self.mask = Some(mask);
        self
    }

    /// Number of images to generate (1-10).
    pub fn n(mut self, n: u8) -> Self {
        self.n = Some(n);
        self
    }

    /// `"1024x1024"`, `"1536x1024"`, `"1024x1536"`, or `"auto"`.
    pub fn size(mut self, size: impl Into<String>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// `"low"`, `"medium"`, `"high"`, or `"auto"`.
    pub fn quality(mut self, quality: impl Into<String>) -> Self {
        self.quality = Some(quality.into());
        self
    }

    /// `"png"`, `"jpeg"`, or `"webp"`.
    pub fn output_format(mut self, output_format: impl Into<String>) -> Self {
        self.output_format = Some(output_format.into());
        self
    }

    /// `"transparent"`, `"opaque"`, or `"auto"`.
    pub fn background(mut self, background: impl Into<String>) -> Self {
        self.background = Some(background.into());
        self
    }

    /// Stable end-user identifier for abuse monitoring.
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    fn into_form(self) -> Result<Form, OpenAiError> {
        let mut form = Form::new()
            .text("model", self.model)
            .text("prompt", self.prompt);
        let image_field = if self.images.len() > 1 {
            "image[]"
        } else {
            "image"
        };
        for image in self.images {
            form = form.part(image_field, image.into_part()?);
        }
        if let Some(mask) = self.mask {
            form = form.part("mask", mask.into_part()?);
        }
        if let Some(n) = self.n {
            form = form.text("n", n.to_string());
        }
        if let Some(size) = self.size {
            form = form.text("size", size);
        }
        if let Some(quality) = self.quality {
            form = form.text("quality", quality);
        }
        if let Some(output_format) = self.output_format {
            form = form.text("output_format", output_format);
        }
        if let Some(background) = self.background {
            form = form.text("background", background);
        }
        if let Some(user) = self.user {
            form = form.text("user", user);
        }
        Ok(form)
    }
}

/// Response body of the image endpoints.
#[derive(Debug, Clone, Deserialize)]
pub struct ImagesResponse {
    /// Unix timestamp of creation.
    pub created: Option<u64>,
    /// The generated images.
    #[serde(default)]
    pub data: Vec<ImageData>,
    /// Token accounting (`gpt-image-*` models).
    pub usage: Option<Value>,
}

impl ImagesResponse {
    /// Base64 payloads of all returned images.
    pub fn b64_images(&self) -> Vec<&str> {
        self.data
            .iter()
            .filter_map(|image| image.b64_json.as_deref())
            .collect()
    }
}

/// One generated image.
#[derive(Debug, Clone, Deserialize)]
pub struct ImageData {
    /// Base64-encoded image (always set by `gpt-image-*` models).
    pub b64_json: Option<String>,
    /// Hosted image URL (legacy response format).
    pub url: Option<String>,
    /// The prompt after model-side revision, if revised.
    pub revised_prompt: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_generation_request() {
        let request = ImageGenerationRequest::new("gpt-image-2", "A wooden Japanese house")
            .size("1536x1024")
            .quality("high")
            .output_format("webp")
            .output_compression(80)
            .background("transparent");
        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(value["model"], "gpt-image-2");
        assert_eq!(value["size"], "1536x1024");
        assert_eq!(value["output_compression"], 80);
        assert!(value.get("n").is_none());
    }

    #[test]
    fn deserializes_response_and_collects_b64() {
        let body = json!({
            "created": 1_750_000_000u64,
            "data": [{"b64_json": "abc"}, {"b64_json": "def"}, {"url": "https://x"}]
        });
        let response: ImagesResponse = serde_json::from_value(body).unwrap();
        assert_eq!(response.b64_images(), vec!["abc", "def"]);
        assert_eq!(response.data[2].url.as_deref(), Some("https://x"));
    }

    #[test]
    fn edit_request_builds_form() {
        let request = ImageEditRequest::new(
            "gpt-image-2",
            FilePart::new("base.png", vec![1, 2, 3]),
            "Add a cat",
        )
        .add_image(FilePart::new("ref.png", vec![4]))
        .mask(FilePart::new("mask.png", vec![5]))
        .quality("high");
        assert!(request.into_form().is_ok());
    }
}
