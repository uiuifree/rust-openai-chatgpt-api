//! Audio APIs: speech synthesis (`/v1/audio/speech`), transcription
//! (`/v1/audio/transcriptions`), and translation (`/v1/audio/translations`).
//!
//! Current models: `gpt-4o-mini-tts` / `tts-1` / `tts-1-hd` for synthesis,
//! `gpt-4o-transcribe` / `gpt-4o-mini-transcribe` / `whisper-1` for
//! transcription, and `whisper-1` for translation.

use reqwest::multipart::Form;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::client::OpenAiClient;
use crate::error::OpenAiError;
use crate::file::FilePart;

/// Accessor returned by [`OpenAiClient::audio`].
pub struct Audio<'a> {
    pub(crate) client: &'a OpenAiClient,
}

impl Audio<'_> {
    /// Synthesizes speech and returns the raw audio bytes.
    pub async fn speech(&self, request: &SpeechRequest) -> Result<Vec<u8>, OpenAiError> {
        self.client.post_json_bytes("/audio/speech", request).await
    }

    /// Transcribes audio in its source language. Consumes the request because
    /// it uploads the audio bytes as a multipart form.
    pub async fn transcribe(
        &self,
        request: TranscriptionRequest,
    ) -> Result<Transcript, OpenAiError> {
        let json_response = request.expects_json();
        let text = self
            .client
            .post_multipart_text("/audio/transcriptions", request.into_form()?)
            .await?;
        parse_transcript(json_response, text)
    }

    /// Translates audio into English. Consumes the request because it uploads
    /// the audio bytes as a multipart form.
    pub async fn translate(&self, request: TranslationRequest) -> Result<Transcript, OpenAiError> {
        let json_response = request.expects_json();
        let text = self
            .client
            .post_multipart_text("/audio/translations", request.into_form()?)
            .await?;
        parse_transcript(json_response, text)
    }
}

fn parse_transcript(json_response: bool, text: String) -> Result<Transcript, OpenAiError> {
    if json_response {
        return serde_json::from_str(&text).map_err(|e| OpenAiError::parse(e, &text));
    }
    Ok(Transcript {
        text,
        ..Default::default()
    })
}

/// Request body for `POST /v1/audio/speech`.
#[derive(Debug, Clone, Serialize)]
pub struct SpeechRequest {
    model: String,
    input: String,
    voice: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    speed: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    instructions: Option<String>,
}

impl SpeechRequest {
    /// Voices include `"alloy"`, `"echo"`, `"fable"`, `"onyx"`, `"nova"`,
    /// `"shimmer"`, `"coral"`, `"sage"`, `"marin"`, and `"cedar"`.
    pub fn new(
        model: impl Into<String>,
        input: impl Into<String>,
        voice: impl Into<String>,
    ) -> Self {
        Self {
            model: model.into(),
            input: input.into(),
            voice: voice.into(),
            response_format: None,
            speed: None,
            instructions: None,
        }
    }

    /// `"mp3"` (default), `"opus"`, `"aac"`, `"flac"`, `"wav"`, or `"pcm"`.
    pub fn response_format(mut self, response_format: impl Into<String>) -> Self {
        self.response_format = Some(response_format.into());
        self
    }

    /// Playback speed 0.25-4.0.
    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = Some(speed);
        self
    }

    /// Voice direction, e.g. "Speak cheerfully" (`gpt-4o-mini-tts`).
    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }
}

/// Request for `POST /v1/audio/transcriptions` (multipart).
#[derive(Debug, Clone)]
pub struct TranscriptionRequest {
    model: String,
    file: FilePart,
    language: Option<String>,
    prompt: Option<String>,
    response_format: Option<String>,
    temperature: Option<f32>,
}

impl TranscriptionRequest {
    /// Creates a request from a model id (e.g. `"gpt-4o-transcribe"`) and an
    /// audio file.
    pub fn new(model: impl Into<String>, file: FilePart) -> Self {
        Self {
            model: model.into(),
            file,
            language: None,
            prompt: None,
            response_format: None,
            temperature: None,
        }
    }

    /// ISO-639-1 code of the spoken language (improves accuracy).
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Text to guide style or continue a previous segment.
    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    /// `"json"` (default), `"verbose_json"`, `"text"`, `"srt"`, or `"vtt"`.
    /// Non-JSON formats are returned in [`Transcript::text`] as-is.
    pub fn response_format(mut self, response_format: impl Into<String>) -> Self {
        self.response_format = Some(response_format.into());
        self
    }

    /// Sampling temperature (0.0-1.0); 0 lets the model auto-tune.
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    fn expects_json(&self) -> bool {
        matches!(
            self.response_format.as_deref(),
            None | Some("json") | Some("verbose_json")
        )
    }

    fn into_form(self) -> Result<Form, OpenAiError> {
        let mut form = Form::new()
            .text("model", self.model)
            .part("file", self.file.into_part()?);
        if let Some(language) = self.language {
            form = form.text("language", language);
        }
        if let Some(prompt) = self.prompt {
            form = form.text("prompt", prompt);
        }
        if let Some(response_format) = self.response_format {
            form = form.text("response_format", response_format);
        }
        if let Some(temperature) = self.temperature {
            form = form.text("temperature", temperature.to_string());
        }
        Ok(form)
    }
}

/// Request for `POST /v1/audio/translations` (multipart).
#[derive(Debug, Clone)]
pub struct TranslationRequest {
    model: String,
    file: FilePart,
    prompt: Option<String>,
    response_format: Option<String>,
    temperature: Option<f32>,
}

impl TranslationRequest {
    /// Creates a request from a model id (`"whisper-1"`) and an audio file.
    pub fn new(model: impl Into<String>, file: FilePart) -> Self {
        Self {
            model: model.into(),
            file,
            prompt: None,
            response_format: None,
            temperature: None,
        }
    }

    /// Text to guide style; should be in English for translations.
    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    /// `"json"` (default), `"verbose_json"`, `"text"`, `"srt"`, or `"vtt"`.
    pub fn response_format(mut self, response_format: impl Into<String>) -> Self {
        self.response_format = Some(response_format.into());
        self
    }

    /// Sampling temperature (0.0-1.0); 0 lets the model auto-tune.
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    fn expects_json(&self) -> bool {
        matches!(
            self.response_format.as_deref(),
            None | Some("json") | Some("verbose_json")
        )
    }

    fn into_form(self) -> Result<Form, OpenAiError> {
        let mut form = Form::new()
            .text("model", self.model)
            .part("file", self.file.into_part()?);
        if let Some(prompt) = self.prompt {
            form = form.text("prompt", prompt);
        }
        if let Some(response_format) = self.response_format {
            form = form.text("response_format", response_format);
        }
        if let Some(temperature) = self.temperature {
            form = form.text("temperature", temperature.to_string());
        }
        Ok(form)
    }
}

/// Transcription / translation result.
///
/// For `"text"`, `"srt"`, and `"vtt"` response formats only [`Transcript::text`]
/// is populated (with the raw payload). `"verbose_json"` fills the optional fields.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Transcript {
    /// The transcribed or translated text (or raw `srt`/`vtt` payload).
    #[serde(default)]
    pub text: String,
    /// Detected language (`verbose_json`).
    pub language: Option<String>,
    /// Audio duration in seconds (`verbose_json`).
    pub duration: Option<f64>,
    /// Segment-level detail (`verbose_json`).
    pub segments: Option<Value>,
    /// Word-level timestamps (`verbose_json` with word granularity).
    pub words: Option<Value>,
    /// Token accounting (`gpt-4o-*` transcription models).
    pub usage: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_speech_request() {
        let request = SpeechRequest::new("gpt-4o-mini-tts", "こんにちは", "marin")
            .response_format("wav")
            .instructions("Speak slowly");
        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(value["voice"], "marin");
        assert_eq!(value["response_format"], "wav");
        assert!(value.get("speed").is_none());
    }

    #[test]
    fn parses_json_transcript() {
        let transcript =
            parse_transcript(true, json!({"text": "hello", "language": "en"}).to_string()).unwrap();
        assert_eq!(transcript.text, "hello");
        assert_eq!(transcript.language.as_deref(), Some("en"));
    }

    #[test]
    fn wraps_plain_text_transcript() {
        let transcript = parse_transcript(false, "1\n00:00:00,000 --> ...".to_string()).unwrap();
        assert!(transcript.text.starts_with("1\n"));
        assert!(transcript.language.is_none());
    }

    #[test]
    fn detects_expected_response_kind() {
        let file = FilePart::new("a.mp3", vec![0]);
        assert!(TranscriptionRequest::new("whisper-1", file.clone()).expects_json());
        assert!(TranscriptionRequest::new("whisper-1", file.clone())
            .response_format("verbose_json")
            .expects_json());
        assert!(!TranscriptionRequest::new("whisper-1", file)
            .response_format("srt")
            .expects_json());
    }
}
