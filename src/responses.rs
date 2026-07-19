//! Responses API (`/v1/responses`) — OpenAI's primary generation API.
//!
//! Supersedes Chat Completions for new projects: single `input` (text or item
//! list), built-in tools, reasoning control, and server-side conversation
//! state via `previous_response_id`.
//!
//! ```no_run
//! use openai_chatgpt_api::responses::ResponseRequest;
//! use openai_chatgpt_api::OpenAiClient;
//!
//! # async fn run() -> Result<(), openai_chatgpt_api::OpenAiError> {
//! let client = OpenAiClient::from_env()?;
//! let request = ResponseRequest::new("gpt-5.6-terra", "Hello!")
//!     .instructions("You are a concise assistant.");
//! let response = client.responses().create(&request).await?;
//! println!("{}", response.output_text());
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::client::{DeletedObject, OpenAiClient};
use crate::error::OpenAiError;
use crate::sse::{sse_events, EventStream};

/// Accessor returned by [`OpenAiClient::responses`].
pub struct Responses<'a> {
    pub(crate) client: &'a OpenAiClient,
}

impl Responses<'_> {
    /// Creates a model response (`POST /v1/responses`).
    pub async fn create(&self, request: &ResponseRequest) -> Result<Response, OpenAiError> {
        self.client.post_json("/responses", request).await
    }

    /// Creates a response and streams typed server-sent events.
    pub async fn stream(
        &self,
        request: &ResponseRequest,
    ) -> Result<EventStream<ResponseStreamEvent>, OpenAiError> {
        let mut request = request.clone();
        request.stream = Some(true);
        let response = self.client.post_json_sse("/responses", &request).await?;
        Ok(sse_events(response))
    }

    /// Fetches a stored response by id (`GET /v1/responses/{id}`).
    pub async fn retrieve(&self, response_id: &str) -> Result<Response, OpenAiError> {
        self.client
            .get_json(&format!("/responses/{response_id}"))
            .await
    }

    /// Deletes a stored response (`DELETE /v1/responses/{id}`).
    pub async fn delete(&self, response_id: &str) -> Result<DeletedObject, OpenAiError> {
        self.client
            .delete_json(&format!("/responses/{response_id}"))
            .await
    }

    /// Cancels a background response (created with `background(true)`).
    pub async fn cancel(&self, response_id: &str) -> Result<Response, OpenAiError> {
        self.client
            .post_empty(&format!("/responses/{response_id}/cancel"))
            .await
    }
}

/// Request body for `POST /v1/responses`.
#[derive(Debug, Clone, Serialize)]
pub struct ResponseRequest {
    model: String,
    input: ResponseInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning: Option<ReasoningConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<TextConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parallel_tool_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    store: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    previous_response_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    truncation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    service_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    background: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

impl ResponseRequest {
    /// Creates a request. `input` accepts plain text or a list of [`InputItem`]s.
    pub fn new(model: impl Into<String>, input: impl Into<ResponseInput>) -> Self {
        Self {
            model: model.into(),
            input: input.into(),
            instructions: None,
            max_output_tokens: None,
            temperature: None,
            top_p: None,
            reasoning: None,
            text: None,
            tools: None,
            tool_choice: None,
            parallel_tool_calls: None,
            store: None,
            previous_response_id: None,
            metadata: None,
            truncation: None,
            service_tier: None,
            background: None,
            stream: None,
        }
    }

    /// System-level guidance (replaces the system message of Chat Completions).
    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    /// Upper bound on generated tokens, including invisible reasoning tokens.
    pub fn max_output_tokens(mut self, max_output_tokens: u32) -> Self {
        self.max_output_tokens = Some(max_output_tokens);
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

    /// Full reasoning configuration; see [`ResponseRequest::reasoning_effort`]
    /// for the common case.
    pub fn reasoning(mut self, reasoning: ReasoningConfig) -> Self {
        self.reasoning = Some(reasoning);
        self
    }

    /// Shorthand for setting `reasoning.effort`.
    /// Known values: `"none"`, `"minimal"`, `"low"`, `"medium"`, `"high"`.
    pub fn reasoning_effort(mut self, effort: impl Into<String>) -> Self {
        self.reasoning.get_or_insert_with(Default::default).effort = Some(effort.into());
        self
    }

    /// Full text output configuration; see [`ResponseRequest::json_schema`]
    /// and [`ResponseRequest::verbosity`] for the common cases.
    pub fn text(mut self, text: TextConfig) -> Self {
        self.text = Some(text);
        self
    }

    /// Shorthand for Structured Outputs: sets `text.format` to a strict JSON schema.
    pub fn json_schema(mut self, name: impl Into<String>, schema: Value) -> Self {
        self.text.get_or_insert_with(Default::default).format = Some(TextFormat::JsonSchema {
            name: name.into(),
            schema,
            description: None,
            strict: Some(true),
        });
        self
    }

    /// Shorthand for setting `text.verbosity`. Known values: `"low"`, `"medium"`, `"high"`.
    pub fn verbosity(mut self, verbosity: impl Into<String>) -> Self {
        self.text.get_or_insert_with(Default::default).verbosity = Some(verbosity.into());
        self
    }

    /// Tools the model may call: custom functions and OpenAI built-ins.
    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
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

    /// Whether the response is persisted server-side (API default: `true`).
    pub fn store(mut self, store: bool) -> Self {
        self.store = Some(store);
        self
    }

    /// Chains onto a stored response to continue a conversation server-side.
    pub fn previous_response_id(mut self, previous_response_id: impl Into<String>) -> Self {
        self.previous_response_id = Some(previous_response_id.into());
        self
    }

    /// Up to 16 key-value pairs attached to the stored response.
    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Context overflow handling: `"auto"` or `"disabled"`.
    pub fn truncation(mut self, truncation: impl Into<String>) -> Self {
        self.truncation = Some(truncation.into());
        self
    }

    /// Processing tier: `"auto"`, `"default"`, `"flex"`, `"scale"`, or `"priority"`.
    pub fn service_tier(mut self, service_tier: impl Into<String>) -> Self {
        self.service_tier = Some(service_tier.into());
        self
    }

    /// Runs the response in the background; poll with [`Responses::retrieve`].
    pub fn background(mut self, background: bool) -> Self {
        self.background = Some(background);
        self
    }
}

/// `input` parameter: plain text or a list of items.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ResponseInput {
    /// A single user message given as plain text.
    Text(String),
    /// A structured list of messages and tool results.
    Items(Vec<InputItem>),
}

impl From<&str> for ResponseInput {
    fn from(text: &str) -> Self {
        ResponseInput::Text(text.to_string())
    }
}

impl From<String> for ResponseInput {
    fn from(text: String) -> Self {
        ResponseInput::Text(text)
    }
}

impl From<Vec<InputItem>> for ResponseInput {
    fn from(items: Vec<InputItem>) -> Self {
        ResponseInput::Items(items)
    }
}

/// One item of a structured `input` list.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum InputItem {
    /// A conversation message with a role.
    Message(InputMessage),
    /// The result of a function call, fed back to the model.
    FunctionCallOutput(FunctionCallOutput),
    /// Escape hatch for item types without a dedicated variant.
    Other(Value),
}

impl InputItem {
    /// Builds a message with an arbitrary role.
    pub fn message(role: impl Into<String>, content: impl Into<InputContent>) -> Self {
        InputItem::Message(InputMessage {
            role: role.into(),
            content: content.into(),
        })
    }

    /// Builds a `system` message.
    pub fn system(content: impl Into<InputContent>) -> Self {
        Self::message("system", content)
    }

    /// Builds a `developer` message.
    pub fn developer(content: impl Into<InputContent>) -> Self {
        Self::message("developer", content)
    }

    /// Builds a `user` message.
    pub fn user(content: impl Into<InputContent>) -> Self {
        Self::message("user", content)
    }

    /// Builds an `assistant` message (e.g. replaying conversation history).
    pub fn assistant(content: impl Into<InputContent>) -> Self {
        Self::message("assistant", content)
    }

    /// Returns a function-call result to the model, continuing a tool loop.
    pub fn function_call_output(call_id: impl Into<String>, output: impl Into<String>) -> Self {
        InputItem::FunctionCallOutput(FunctionCallOutput {
            item_type: "function_call_output",
            call_id: call_id.into(),
            output: output.into(),
        })
    }
}

/// A role-tagged input message.
#[derive(Debug, Clone, Serialize)]
pub struct InputMessage {
    /// `"user"`, `"assistant"`, `"system"`, or `"developer"`.
    pub role: String,
    /// Plain text or multimodal parts.
    pub content: InputContent,
}

/// Function-call result item (`type: "function_call_output"`).
#[derive(Debug, Clone, Serialize)]
pub struct FunctionCallOutput {
    #[serde(rename = "type")]
    item_type: &'static str,
    /// The `call_id` of the [`FunctionCall`] this answers.
    pub call_id: String,
    /// Result payload, typically JSON-encoded.
    pub output: String,
}

/// Message content: plain text or multimodal parts.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum InputContent {
    /// Plain text content.
    Text(String),
    /// Multimodal content parts (text, images, files).
    Parts(Vec<InputContentPart>),
}

impl From<&str> for InputContent {
    fn from(text: &str) -> Self {
        InputContent::Text(text.to_string())
    }
}

impl From<String> for InputContent {
    fn from(text: String) -> Self {
        InputContent::Text(text)
    }
}

impl From<Vec<InputContentPart>> for InputContent {
    fn from(parts: Vec<InputContentPart>) -> Self {
        InputContent::Parts(parts)
    }
}

/// One multimodal content part of an input message.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum InputContentPart {
    /// Text part (`type: "input_text"`).
    #[serde(rename = "input_text")]
    Text {
        /// The text content.
        text: String,
    },
    /// Image part (`type: "input_image"`).
    #[serde(rename = "input_image")]
    Image {
        /// An https URL or a `data:` URI.
        image_url: String,
        /// Analysis detail: `"low"`, `"high"`, or `"auto"`.
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
    },
    /// File part (`type: "input_file"`); set exactly one source field.
    #[serde(rename = "input_file")]
    File {
        /// Id of a file uploaded via the Files API.
        #[serde(skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
        /// Publicly reachable file URL.
        #[serde(skip_serializing_if = "Option::is_none")]
        file_url: Option<String>,
        /// Filename accompanying inline `file_data`.
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
        /// Inline base64-encoded file content.
        #[serde(skip_serializing_if = "Option::is_none")]
        file_data: Option<String>,
    },
}

impl InputContentPart {
    /// Builds a text part.
    pub fn text(text: impl Into<String>) -> Self {
        InputContentPart::Text { text: text.into() }
    }

    /// Builds an image part. `image_url` accepts an https URL or a `data:` URI.
    pub fn image_url(image_url: impl Into<String>) -> Self {
        InputContentPart::Image {
            image_url: image_url.into(),
            detail: None,
        }
    }
}

/// `reasoning` parameter (reasoning-capable models).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReasoningConfig {
    /// `"none"`, `"minimal"`, `"low"`, `"medium"`, or `"high"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    /// Reasoning summary detail: `"auto"`, `"concise"`, or `"detailed"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

/// `text` parameter: output format and verbosity.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextConfig {
    /// Output format; defaults to plain text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<TextFormat>,
    /// Output length preference: `"low"`, `"medium"`, or `"high"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<String>,
}

/// Output format for [`TextConfig::format`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TextFormat {
    /// Plain text (default).
    #[serde(rename = "text")]
    Text,
    /// Any syntactically valid JSON object (legacy JSON mode).
    #[serde(rename = "json_object")]
    JsonObject,
    /// Structured Outputs: JSON constrained by a schema.
    #[serde(rename = "json_schema")]
    JsonSchema {
        /// Schema name reported back by the API.
        name: String,
        /// The JSON Schema describing the output.
        schema: Value,
        /// Human-readable purpose of the schema.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        /// Enforce exact schema adherence (recommended: `true`).
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },
}

/// Tool the model may call: a custom function or an OpenAI built-in.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Tool {
    /// A custom function the model can request to call.
    #[serde(rename = "function")]
    Function {
        /// Function name reported in [`FunctionCall::name`].
        name: String,
        /// What the function does — used by the model to decide when to call it.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        /// JSON Schema of the function arguments.
        parameters: Value,
        /// Enforce exact schema adherence (recommended: `true`).
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },
    /// OpenAI-hosted web search.
    #[serde(rename = "web_search")]
    WebSearch,
    /// OpenAI-hosted search over uploaded vector stores.
    #[serde(rename = "file_search")]
    FileSearch {
        /// Vector stores to search.
        vector_store_ids: Vec<String>,
    },
    /// OpenAI-hosted Python sandbox.
    #[serde(rename = "code_interpreter")]
    CodeInterpreter {
        /// Container config, e.g. `json!({"type": "auto"})`.
        container: Value,
    },
    /// OpenAI-hosted image generation as a tool.
    #[serde(rename = "image_generation")]
    ImageGeneration,
    /// Escape hatch for tool types without a dedicated variant.
    #[serde(untagged)]
    Other(Value),
}

impl Tool {
    /// Builds a strict function tool. `parameters` is a JSON Schema.
    pub fn function(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: Value,
    ) -> Self {
        Tool::Function {
            name: name.into(),
            description: Some(description.into()),
            parameters,
            strict: Some(true),
        }
    }
}

/// Response object returned by the Responses API.
#[derive(Debug, Clone, Deserialize)]
pub struct Response {
    /// Unique identifier (`resp_...`).
    pub id: String,
    /// Always `"response"`.
    pub object: Option<String>,
    /// Unix timestamp of creation.
    pub created_at: Option<f64>,
    /// `"completed"`, `"failed"`, `"in_progress"`, `"incomplete"`, ...
    pub status: Option<String>,
    /// Model that produced the response.
    pub model: Option<String>,
    /// Generated items: messages, function calls, reasoning, tool activity.
    #[serde(default)]
    pub output: Vec<OutputItem>,
    /// Populated when `status` is `"failed"`.
    pub error: Option<ResponseErrorDetail>,
    /// Why the response is `"incomplete"` (e.g. `max_output_tokens` reached).
    pub incomplete_details: Option<Value>,
    /// The response this one continued from, if any.
    pub previous_response_id: Option<String>,
    /// Token accounting.
    pub usage: Option<ResponseUsage>,
    /// Echo of the request's reasoning configuration.
    pub reasoning: Option<ReasoningConfig>,
    /// Echo of the request's metadata.
    #[serde(default)]
    pub metadata: Option<HashMap<String, String>>,
}

impl Response {
    /// Concatenation of every `output_text` part across output messages.
    pub fn output_text(&self) -> String {
        let mut text = String::new();
        for item in &self.output {
            let OutputItem::Message(message) = item else {
                continue;
            };
            for content in &message.content {
                if let OutputContent::OutputText { text: part, .. } = content {
                    text.push_str(part);
                }
            }
        }
        text
    }

    /// Function calls the model requested this turn.
    pub fn function_calls(&self) -> Vec<&FunctionCall> {
        self.output
            .iter()
            .filter_map(|item| match item {
                OutputItem::FunctionCall(call) => Some(call),
                _ => None,
            })
            .collect()
    }
}

/// Failure details of a `"failed"` response.
#[derive(Debug, Clone, Deserialize)]
pub struct ResponseErrorDetail {
    /// Machine-readable error code.
    pub code: Option<String>,
    /// Human-readable description.
    pub message: String,
}

/// One item of the `output` array.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum OutputItem {
    /// An assistant message with text content.
    #[serde(rename = "message")]
    Message(OutputMessage),
    /// A request to call one of the supplied function tools.
    #[serde(rename = "function_call")]
    FunctionCall(FunctionCall),
    /// Reasoning trace of a reasoning-capable model.
    #[serde(rename = "reasoning")]
    Reasoning(ReasoningItem),
    /// Item types without a dedicated variant (built-in tool calls etc.).
    #[serde(untagged)]
    Other(Value),
}

/// Assistant message inside [`OutputItem::Message`].
#[derive(Debug, Clone, Deserialize)]
pub struct OutputMessage {
    /// Item identifier (`msg_...`).
    pub id: Option<String>,
    /// Always `"assistant"`.
    pub role: String,
    /// Item status, e.g. `"completed"`.
    pub status: Option<String>,
    /// Text and refusal parts.
    #[serde(default)]
    pub content: Vec<OutputContent>,
}

/// Function call requested by the model.
#[derive(Debug, Clone, Deserialize)]
pub struct FunctionCall {
    /// Item identifier (`fc_...`).
    pub id: Option<String>,
    /// Correlate results via [`InputItem::function_call_output`].
    pub call_id: String,
    /// Name of the function to invoke.
    pub name: String,
    /// JSON-encoded arguments.
    pub arguments: String,
    /// Item status, e.g. `"completed"`.
    pub status: Option<String>,
}

/// Reasoning trace inside [`OutputItem::Reasoning`].
#[derive(Debug, Clone, Deserialize)]
pub struct ReasoningItem {
    /// Item identifier (`rs_...`).
    pub id: Option<String>,
    /// Summary parts (present when `reasoning.summary` was requested).
    #[serde(default)]
    pub summary: Vec<Value>,
}

/// One content part of an output message.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum OutputContent {
    /// Generated text.
    #[serde(rename = "output_text")]
    OutputText {
        /// The text content.
        text: String,
        /// Citations and other annotations.
        #[serde(default)]
        annotations: Vec<Value>,
    },
    /// The model declined to comply.
    #[serde(rename = "refusal")]
    Refusal {
        /// Explanation of the refusal.
        refusal: String,
    },
    /// Content types without a dedicated variant.
    #[serde(untagged)]
    Other(Value),
}

/// Token accounting for a response.
#[derive(Debug, Clone, Deserialize)]
pub struct ResponseUsage {
    /// Tokens in the input (prompt).
    #[serde(default)]
    pub input_tokens: u64,
    /// Tokens generated, including reasoning tokens.
    #[serde(default)]
    pub output_tokens: u64,
    /// `input_tokens + output_tokens`.
    #[serde(default)]
    pub total_tokens: u64,
    /// Input token breakdown.
    pub input_tokens_details: Option<InputTokensDetails>,
    /// Output token breakdown.
    pub output_tokens_details: Option<OutputTokensDetails>,
}

/// Breakdown of [`ResponseUsage::input_tokens`].
#[derive(Debug, Clone, Deserialize)]
pub struct InputTokensDetails {
    /// Tokens served from the prompt cache (billed at a discount).
    #[serde(default)]
    pub cached_tokens: u64,
}

/// Breakdown of [`ResponseUsage::output_tokens`].
#[derive(Debug, Clone, Deserialize)]
pub struct OutputTokensDetails {
    /// Invisible reasoning tokens (billed as output).
    #[serde(default)]
    pub reasoning_tokens: u64,
}

/// Server-sent event emitted by [`Responses::stream`].
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseStreamEvent {
    /// The response was accepted and streaming begins.
    #[serde(rename = "response.created")]
    Created {
        /// Snapshot of the in-progress response.
        response: Response,
    },
    /// Generation is underway.
    #[serde(rename = "response.in_progress")]
    InProgress {
        /// Snapshot of the in-progress response.
        response: Response,
    },
    /// A new output item started.
    #[serde(rename = "response.output_item.added")]
    OutputItemAdded {
        /// The item in its initial state.
        item: OutputItem,
        /// Position within `output`.
        output_index: Option<u32>,
    },
    /// An output item finished.
    #[serde(rename = "response.output_item.done")]
    OutputItemDone {
        /// The finalized item.
        item: OutputItem,
        /// Position within `output`.
        output_index: Option<u32>,
    },
    /// Incremental text — the variant to watch for displaying output live.
    #[serde(rename = "response.output_text.delta")]
    OutputTextDelta {
        /// The text fragment to append.
        delta: String,
        /// Id of the message item being written.
        item_id: Option<String>,
        /// Position within `output`.
        output_index: Option<u32>,
        /// Position within the message's `content`.
        content_index: Option<u32>,
    },
    /// A text part finished.
    #[serde(rename = "response.output_text.done")]
    OutputTextDone {
        /// The complete text of the part.
        text: String,
    },
    /// Incremental function-call arguments.
    #[serde(rename = "response.function_call_arguments.delta")]
    FunctionCallArgumentsDelta {
        /// The arguments fragment to append.
        delta: String,
        /// Id of the function-call item being written.
        item_id: Option<String>,
    },
    /// Terminal event: generation finished successfully.
    #[serde(rename = "response.completed")]
    Completed {
        /// The final response, including usage.
        response: Response,
    },
    /// Terminal event: generation failed.
    #[serde(rename = "response.failed")]
    Failed {
        /// The failed response; see [`Response::error`].
        response: Response,
    },
    /// Terminal event: generation stopped early (e.g. token limit).
    #[serde(rename = "response.incomplete")]
    Incomplete {
        /// The partial response; see [`Response::incomplete_details`].
        response: Response,
    },
    /// The stream itself errored.
    #[serde(rename = "error")]
    Error {
        /// Machine-readable error code.
        code: Option<String>,
        /// Human-readable description.
        message: String,
    },
    /// Event types without a dedicated variant.
    #[serde(untagged)]
    Other(Value),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_minimal_request() {
        let request = ResponseRequest::new("gpt-5.6-terra", "Hello");
        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(value, json!({"model": "gpt-5.6-terra", "input": "Hello"}));
    }

    #[test]
    fn serializes_full_request() {
        let request = ResponseRequest::new(
            "gpt-5.6-sol",
            vec![
                InputItem::user("What is in this image?"),
                InputItem::user(vec![InputContentPart::image_url(
                    "https://example.com/a.png",
                )]),
                InputItem::function_call_output("call_1", "42"),
            ],
        )
        .instructions("Be brief.")
        .max_output_tokens(500)
        .reasoning_effort("high")
        .verbosity("low")
        .tools(vec![
            Tool::function("get_weather", "Get weather", json!({"type": "object"})),
            Tool::WebSearch,
        ])
        .store(false)
        .previous_response_id("resp_123");
        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(value["instructions"], "Be brief.");
        assert_eq!(value["max_output_tokens"], 500);
        assert_eq!(value["reasoning"]["effort"], "high");
        assert_eq!(value["text"]["verbosity"], "low");
        assert_eq!(value["input"][0]["role"], "user");
        assert_eq!(value["input"][0]["content"], "What is in this image?");
        assert_eq!(value["input"][1]["content"][0]["type"], "input_image");
        assert_eq!(value["input"][2]["type"], "function_call_output");
        assert_eq!(value["tools"][0]["type"], "function");
        assert_eq!(value["tools"][0]["name"], "get_weather");
        assert_eq!(value["tools"][0]["strict"], true);
        assert_eq!(value["tools"][1], json!({"type": "web_search"}));
        assert_eq!(value["store"], false);
        assert_eq!(value["previous_response_id"], "resp_123");
        assert!(value.get("stream").is_none());
    }

    #[test]
    fn serializes_json_schema_format() {
        let request = ResponseRequest::new("gpt-5.6-luna", "hi")
            .json_schema("answer", json!({"type": "object", "properties": {}}));
        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(value["text"]["format"]["type"], "json_schema");
        assert_eq!(value["text"]["format"]["name"], "answer");
        assert_eq!(value["text"]["format"]["strict"], true);
    }

    #[test]
    fn deserializes_response_and_extracts_text() {
        let body = json!({
            "id": "resp_1",
            "object": "response",
            "created_at": 1_750_000_000,
            "status": "completed",
            "model": "gpt-5.6-terra",
            "output": [
                {"type": "reasoning", "id": "rs_1", "summary": []},
                {
                    "type": "message",
                    "id": "msg_1",
                    "role": "assistant",
                    "status": "completed",
                    "content": [
                        {"type": "output_text", "text": "Hello ", "annotations": []},
                        {"type": "output_text", "text": "world"}
                    ]
                },
                {"type": "web_search_call", "id": "ws_1", "status": "completed"}
            ],
            "usage": {
                "input_tokens": 10,
                "output_tokens": 5,
                "total_tokens": 15,
                "output_tokens_details": {"reasoning_tokens": 2}
            }
        });
        let response: Response = serde_json::from_value(body).unwrap();
        assert_eq!(response.output_text(), "Hello world");
        assert_eq!(response.status.as_deref(), Some("completed"));
        let usage = response.usage.unwrap();
        assert_eq!(usage.total_tokens, 15);
        assert_eq!(usage.output_tokens_details.unwrap().reasoning_tokens, 2);
        assert!(matches!(response.output[2], OutputItem::Other(_)));
    }

    #[test]
    fn deserializes_function_call_output_item() {
        let body = json!({
            "id": "resp_2",
            "output": [{
                "type": "function_call",
                "id": "fc_1",
                "call_id": "call_9",
                "name": "get_weather",
                "arguments": "{\"city\":\"Tokyo\"}",
                "status": "completed"
            }]
        });
        let response: Response = serde_json::from_value(body).unwrap();
        let calls = response.function_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "get_weather");
        assert_eq!(calls[0].call_id, "call_9");
    }

    #[test]
    fn deserializes_stream_events() {
        let delta: ResponseStreamEvent = serde_json::from_value(json!({
            "type": "response.output_text.delta",
            "item_id": "msg_1",
            "output_index": 0,
            "content_index": 0,
            "delta": "Hel"
        }))
        .unwrap();
        assert!(
            matches!(delta, ResponseStreamEvent::OutputTextDelta { delta, .. } if delta == "Hel")
        );

        let unknown: ResponseStreamEvent = serde_json::from_value(json!({
            "type": "response.audio.delta",
            "delta": "..."
        }))
        .unwrap();
        assert!(matches!(unknown, ResponseStreamEvent::Other(_)));
    }
}
