//! Chat Completions API (`/v1/chat/completions`).
//!
//! Still fully supported; prefer [`crate::responses`] for new projects.
//!
//! ```no_run
//! use openai_chatgpt_api::chat::{ChatCompletionRequest, ChatMessage};
//! use openai_chatgpt_api::OpenAiClient;
//!
//! # async fn run() -> Result<(), openai_chatgpt_api::OpenAiError> {
//! let client = OpenAiClient::from_env()?;
//! let request = ChatCompletionRequest::new(
//!     "gpt-5.6-terra",
//!     vec![
//!         ChatMessage::system("You are a helpful assistant."),
//!         ChatMessage::user("Hello!"),
//!     ],
//! );
//! let completion = client.chat().create(&request).await?;
//! println!("{}", completion.content().unwrap_or_default());
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::client::OpenAiClient;
use crate::error::OpenAiError;
use crate::sse::{sse_events, EventStream};

/// Accessor returned by [`OpenAiClient::chat`].
pub struct Chat<'a> {
    pub(crate) client: &'a OpenAiClient,
}

impl Chat<'_> {
    /// Creates a chat completion (`POST /v1/chat/completions`).
    pub async fn create(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletion, OpenAiError> {
        self.client.post_json("/chat/completions", request).await
    }

    /// Creates a chat completion and streams the chunks.
    ///
    /// The final chunk carries [`ChatCompletionChunk::usage`].
    pub async fn stream(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<EventStream<ChatCompletionChunk>, OpenAiError> {
        let mut request = request.clone();
        request.stream = Some(true);
        if request.stream_options.is_none() {
            request.stream_options = Some(StreamOptions {
                include_usage: true,
            });
        }
        let response = self
            .client
            .post_json_sse("/chat/completions", &request)
            .await?;
        Ok(sse_events(response))
    }
}

/// Request body for `POST /v1/chat/completions`.
#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_completion_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    logit_bias: Option<HashMap<String, i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    logprobs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_logprobs: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ChatResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ChatTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parallel_tool_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verbosity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    store: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    service_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_options: Option<StreamOptions>,
}

impl ChatCompletionRequest {
    /// Creates a request from a model id and the conversation so far.
    pub fn new(model: impl Into<String>, messages: Vec<ChatMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            max_completion_tokens: None,
            temperature: None,
            top_p: None,
            n: None,
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            logprobs: None,
            top_logprobs: None,
            seed: None,
            response_format: None,
            tools: None,
            tool_choice: None,
            parallel_tool_calls: None,
            reasoning_effort: None,
            verbosity: None,
            store: None,
            metadata: None,
            service_tier: None,
            user: None,
            stream: None,
            stream_options: None,
        }
    }

    /// Upper bound on generated tokens (replaces the deprecated `max_tokens`).
    pub fn max_completion_tokens(mut self, max_completion_tokens: u32) -> Self {
        self.max_completion_tokens = Some(max_completion_tokens);
        self
    }

    /// Sampling temperature (0.0-2.0). Not supported by reasoning models.
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Nucleus sampling mass (0.0-1.0). Alternative to `temperature`.
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Number of choices to generate (billed per choice).
    pub fn n(mut self, n: u32) -> Self {
        self.n = Some(n);
        self
    }

    /// Up to 4 sequences that stop generation when produced.
    pub fn stop(mut self, stop: Vec<String>) -> Self {
        self.stop = Some(stop);
        self
    }

    /// Penalizes tokens already present (-2.0 to 2.0).
    pub fn presence_penalty(mut self, presence_penalty: f32) -> Self {
        self.presence_penalty = Some(presence_penalty);
        self
    }

    /// Penalizes tokens by frequency so far (-2.0 to 2.0).
    pub fn frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.frequency_penalty = Some(frequency_penalty);
        self
    }

    /// Per-token-id bias (-100 to 100), keyed by token id.
    pub fn logit_bias(mut self, logit_bias: HashMap<String, i32>) -> Self {
        self.logit_bias = Some(logit_bias);
        self
    }

    /// Whether to return log probabilities of output tokens.
    pub fn logprobs(mut self, logprobs: bool) -> Self {
        self.logprobs = Some(logprobs);
        self
    }

    /// Number of most-likely alternatives per position (0-20); requires `logprobs`.
    pub fn top_logprobs(mut self, top_logprobs: u8) -> Self {
        self.top_logprobs = Some(top_logprobs);
        self
    }

    /// Best-effort deterministic sampling seed.
    pub fn seed(mut self, seed: i64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Output format; see [`ChatCompletionRequest::json_schema`] for the
    /// common case.
    pub fn response_format(mut self, response_format: ChatResponseFormat) -> Self {
        self.response_format = Some(response_format);
        self
    }

    /// Shorthand for Structured Outputs with a strict JSON schema.
    pub fn json_schema(mut self, name: impl Into<String>, schema: Value) -> Self {
        self.response_format = Some(ChatResponseFormat::JsonSchema {
            json_schema: JsonSchemaSpec {
                name: name.into(),
                schema,
                description: None,
                strict: Some(true),
            },
        });
        self
    }

    /// Function tools the model may call.
    pub fn tools(mut self, tools: Vec<ChatTool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Constrains tool selection: `"none"`, `"auto"`, `"required"`, or a
    /// specific-tool object (passed through as raw JSON).
    pub fn tool_choice(mut self, tool_choice: Value) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Whether the model may issue several tool calls in one turn.
    pub fn parallel_tool_calls(mut self, parallel_tool_calls: bool) -> Self {
        self.parallel_tool_calls = Some(parallel_tool_calls);
        self
    }

    /// Reasoning depth for reasoning-capable models.
    /// Known values: `"none"`, `"minimal"`, `"low"`, `"medium"`, `"high"`.
    pub fn reasoning_effort(mut self, reasoning_effort: impl Into<String>) -> Self {
        self.reasoning_effort = Some(reasoning_effort.into());
        self
    }

    /// Output length preference. Known values: `"low"`, `"medium"`, `"high"`.
    pub fn verbosity(mut self, verbosity: impl Into<String>) -> Self {
        self.verbosity = Some(verbosity.into());
        self
    }

    /// Whether to persist the completion server-side (e.g. for evals).
    pub fn store(mut self, store: bool) -> Self {
        self.store = Some(store);
        self
    }

    /// Up to 16 key-value pairs attached to the stored completion.
    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Processing tier: `"auto"`, `"default"`, `"flex"`, `"scale"`, or `"priority"`.
    pub fn service_tier(mut self, service_tier: impl Into<String>) -> Self {
        self.service_tier = Some(service_tier.into());
        self
    }

    /// Stable end-user identifier for abuse monitoring.
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct StreamOptions {
    pub(crate) include_usage: bool,
}

/// One conversation message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// `"system"`, `"developer"`, `"user"`, `"assistant"`, or `"tool"`.
    pub role: String,
    /// Message content; `None` for assistant messages that only carry tool calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<ChatContent>,
    /// Optional participant name to disambiguate same-role messages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Tool calls issued by an assistant message being replayed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    /// For `role: "tool"` — the id of the call this message answers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    /// Builds a message with an arbitrary role.
    pub fn new(role: impl Into<String>, content: impl Into<ChatContent>) -> Self {
        Self {
            role: role.into(),
            content: Some(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Builds a `system` message.
    pub fn system(content: impl Into<ChatContent>) -> Self {
        Self::new("system", content)
    }

    /// Builds a `developer` message.
    pub fn developer(content: impl Into<ChatContent>) -> Self {
        Self::new("developer", content)
    }

    /// Builds a `user` message.
    pub fn user(content: impl Into<ChatContent>) -> Self {
        Self::new("user", content)
    }

    /// Builds an `assistant` message (e.g. replaying conversation history).
    pub fn assistant(content: impl Into<ChatContent>) -> Self {
        Self::new("assistant", content)
    }

    /// Result of a tool call, echoed back with the `tool_call_id` it answers.
    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<ChatContent>) -> Self {
        Self {
            role: "tool".to_string(),
            content: Some(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}

/// Message content: plain text or multimodal parts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatContent {
    /// Plain text content.
    Text(String),
    /// Multimodal content parts (text, images).
    Parts(Vec<ChatContentPart>),
}

impl From<&str> for ChatContent {
    fn from(text: &str) -> Self {
        ChatContent::Text(text.to_string())
    }
}

impl From<String> for ChatContent {
    fn from(text: String) -> Self {
        ChatContent::Text(text)
    }
}

impl From<Vec<ChatContentPart>> for ChatContent {
    fn from(parts: Vec<ChatContentPart>) -> Self {
        ChatContent::Parts(parts)
    }
}

/// One multimodal content part of a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ChatContentPart {
    /// Text part.
    #[serde(rename = "text")]
    Text {
        /// The text content.
        text: String,
    },
    /// Image part.
    #[serde(rename = "image_url")]
    ImageUrl {
        /// The image location and detail level.
        image_url: ImageUrl,
    },
    /// Part types without a dedicated variant (e.g. `input_audio`).
    #[serde(untagged)]
    Other(Value),
}

impl ChatContentPart {
    /// Builds a text part.
    pub fn text(text: impl Into<String>) -> Self {
        ChatContentPart::Text { text: text.into() }
    }

    /// Builds an image part. `url` accepts an https URL or a `data:` URI.
    pub fn image_url(url: impl Into<String>) -> Self {
        ChatContentPart::ImageUrl {
            image_url: ImageUrl {
                url: url.into(),
                detail: None,
            },
        }
    }
}

/// Image reference inside [`ChatContentPart::ImageUrl`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    /// An https URL or a `data:` URI.
    pub url: String,
    /// Analysis detail: `"low"`, `"high"`, or `"auto"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// `response_format` parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ChatResponseFormat {
    /// Plain text (default).
    #[serde(rename = "text")]
    Text,
    /// Any syntactically valid JSON object (legacy JSON mode).
    #[serde(rename = "json_object")]
    JsonObject,
    /// Structured Outputs: JSON constrained by a schema.
    #[serde(rename = "json_schema")]
    JsonSchema {
        /// The schema specification.
        json_schema: JsonSchemaSpec,
    },
}

/// Schema specification for [`ChatResponseFormat::JsonSchema`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchemaSpec {
    /// Schema name reported back by the API.
    pub name: String,
    /// The JSON Schema describing the output.
    pub schema: Value,
    /// Human-readable purpose of the schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Enforce exact schema adherence (recommended: `true`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

/// Function tool definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatTool {
    #[serde(rename = "type")]
    tool_type: String,
    /// The function the model may call.
    pub function: FunctionDef,
}

impl ChatTool {
    /// Builds a strict function tool. `parameters` is a JSON Schema.
    pub fn function(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: Value,
    ) -> Self {
        Self {
            tool_type: "function".to_string(),
            function: FunctionDef {
                name: name.into(),
                description: Some(description.into()),
                parameters,
                strict: Some(true),
            },
        }
    }
}

/// Function signature inside [`ChatTool`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDef {
    /// Function name reported in tool calls.
    pub name: String,
    /// What the function does — used by the model to decide when to call it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema of the function arguments.
    pub parameters: Value,
    /// Enforce exact schema adherence (recommended: `true`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

/// Response body of a chat completion.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletion {
    /// Unique identifier (`chatcmpl-...`).
    pub id: Option<String>,
    /// Always `"chat.completion"`.
    pub object: Option<String>,
    /// Unix timestamp of creation.
    pub created: Option<u64>,
    /// Model that produced the completion.
    pub model: Option<String>,
    /// Generated choices (one unless `n` was set).
    #[serde(default)]
    pub choices: Vec<ChatChoice>,
    /// Token accounting.
    pub usage: Option<ChatUsage>,
    /// Backend configuration fingerprint (for `seed` reproducibility).
    pub system_fingerprint: Option<String>,
    /// Processing tier that actually served the request.
    pub service_tier: Option<String>,
}

impl ChatCompletion {
    /// Text content of the first choice.
    pub fn content(&self) -> Option<&str> {
        self.choices.first()?.message.content.as_deref()
    }
}

/// One generated choice.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatChoice {
    /// Position in `choices`.
    pub index: Option<u32>,
    /// The generated assistant message.
    pub message: AssistantMessage,
    /// `"stop"`, `"length"`, `"tool_calls"`, `"content_filter"`, ...
    pub finish_reason: Option<String>,
}

/// Assistant message inside a [`ChatChoice`].
#[derive(Debug, Clone, Deserialize)]
pub struct AssistantMessage {
    /// Always `"assistant"`.
    pub role: Option<String>,
    /// Generated text; `None` when the model only issued tool calls.
    pub content: Option<String>,
    /// Tool calls the model requested.
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Present when the model declined to comply.
    pub refusal: Option<String>,
}

/// Tool call issued by the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Call identifier to echo back via [`ChatMessage::tool`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Always `"function"`.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub call_type: Option<String>,
    /// The function name and arguments.
    pub function: FunctionCallData,
}

/// Function name and arguments of a [`ToolCall`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallData {
    /// Name of the function to invoke.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// JSON-encoded arguments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

/// Token accounting for a chat completion.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatUsage {
    /// Tokens in the prompt.
    #[serde(default)]
    pub prompt_tokens: u64,
    /// Tokens generated, including reasoning tokens.
    #[serde(default)]
    pub completion_tokens: u64,
    /// `prompt_tokens + completion_tokens`.
    #[serde(default)]
    pub total_tokens: u64,
    /// Prompt token breakdown (e.g. `cached_tokens`).
    pub prompt_tokens_details: Option<Value>,
    /// Completion token breakdown (e.g. `reasoning_tokens`).
    pub completion_tokens_details: Option<Value>,
}

/// One chunk of a streamed chat completion.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionChunk {
    /// Unique identifier, shared by all chunks of one completion.
    pub id: Option<String>,
    /// Always `"chat.completion.chunk"`.
    pub object: Option<String>,
    /// Unix timestamp of creation.
    pub created: Option<u64>,
    /// Model that produced the completion.
    pub model: Option<String>,
    /// Incremental choice updates.
    #[serde(default)]
    pub choices: Vec<ChunkChoice>,
    /// Present on the final chunk ([`Chat::stream`] requests usage by default).
    pub usage: Option<ChatUsage>,
}

impl ChatCompletionChunk {
    /// Incremental text of the first choice, if any.
    pub fn delta_content(&self) -> Option<&str> {
        self.choices.first()?.delta.content.as_deref()
    }
}

/// Incremental update to one choice.
#[derive(Debug, Clone, Deserialize)]
pub struct ChunkChoice {
    /// Position in `choices`.
    pub index: Option<u32>,
    /// The fields that changed in this chunk.
    #[serde(default)]
    pub delta: ChatDelta,
    /// Set on the last chunk of the choice.
    pub finish_reason: Option<String>,
}

/// Changed fields of a streamed choice.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ChatDelta {
    /// Sent once on the first chunk (`"assistant"`).
    pub role: Option<String>,
    /// Text fragment to append.
    pub content: Option<String>,
    /// Tool-call fragments to accumulate by `index`.
    pub tool_calls: Option<Vec<ToolCallDelta>>,
}

/// Fragment of a tool call within a stream.
#[derive(Debug, Clone, Deserialize)]
pub struct ToolCallDelta {
    /// Which tool call this fragment belongs to.
    pub index: Option<u32>,
    /// Call identifier (sent once per call).
    pub id: Option<String>,
    /// Name and/or arguments fragment.
    pub function: Option<FunctionCallData>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_minimal_request() {
        let request = ChatCompletionRequest::new(
            "gpt-5.6-terra",
            vec![ChatMessage::system("Be brief."), ChatMessage::user("Hi")],
        );
        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(
            value,
            json!({
                "model": "gpt-5.6-terra",
                "messages": [
                    {"role": "system", "content": "Be brief."},
                    {"role": "user", "content": "Hi"}
                ]
            })
        );
    }

    #[test]
    fn serializes_multimodal_and_tools() {
        let request = ChatCompletionRequest::new(
            "gpt-5.6-sol",
            vec![ChatMessage::user(vec![
                ChatContentPart::text("What is this?"),
                ChatContentPart::image_url("https://example.com/a.png"),
            ])],
        )
        .max_completion_tokens(100)
        .tools(vec![ChatTool::function(
            "lookup",
            "Look something up",
            json!({"type": "object"}),
        )])
        .json_schema("result", json!({"type": "object"}));
        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(value["messages"][0]["content"][1]["type"], "image_url");
        assert_eq!(value["max_completion_tokens"], 100);
        assert_eq!(value["tools"][0]["type"], "function");
        assert_eq!(value["tools"][0]["function"]["name"], "lookup");
        assert_eq!(value["response_format"]["type"], "json_schema");
        assert_eq!(value["response_format"]["json_schema"]["strict"], true);
    }

    #[test]
    fn deserializes_completion_with_tool_calls() {
        let body = json!({
            "id": "chatcmpl-1",
            "object": "chat.completion",
            "created": 1_750_000_000u64,
            "model": "gpt-5.6-terra",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [{
                        "id": "call_1",
                        "type": "function",
                        "function": {"name": "lookup", "arguments": "{}"}
                    }]
                },
                "finish_reason": "tool_calls"
            }],
            "usage": {"prompt_tokens": 20, "completion_tokens": 10, "total_tokens": 30}
        });
        let completion: ChatCompletion = serde_json::from_value(body).unwrap();
        assert_eq!(completion.content(), None);
        let calls = completion.choices[0].message.tool_calls.as_ref().unwrap();
        assert_eq!(calls[0].function.name.as_deref(), Some("lookup"));
        assert_eq!(completion.usage.unwrap().total_tokens, 30);
    }

    #[test]
    fn deserializes_stream_chunk() {
        let chunk: ChatCompletionChunk = serde_json::from_value(json!({
            "id": "chatcmpl-1",
            "object": "chat.completion.chunk",
            "choices": [{"index": 0, "delta": {"content": "Hel"}, "finish_reason": null}]
        }))
        .unwrap();
        assert_eq!(chunk.delta_content(), Some("Hel"));

        let last: ChatCompletionChunk = serde_json::from_value(json!({
            "id": "chatcmpl-1",
            "choices": [],
            "usage": {"prompt_tokens": 1, "completion_tokens": 2, "total_tokens": 3}
        }))
        .unwrap();
        assert_eq!(last.delta_content(), None);
        assert_eq!(last.usage.unwrap().total_tokens, 3);
    }
}
