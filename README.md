# openai_chatgpt_api — OpenAI API client for Rust

[![crates.io](https://img.shields.io/crates/v/openai_chatgpt_api?style=flat-square)](https://crates.io/crates/openai_chatgpt_api)
[![docs.rs](https://img.shields.io/docsrs/openai_chatgpt_api?style=flat-square)](https://docs.rs/openai_chatgpt_api)
[![license](https://img.shields.io/crates/l/openai_chatgpt_api?style=flat-square)](LICENSE)

**Typed async Rust client for the OpenAI API** — Responses API, Chat Completions,
Embeddings, Images (gpt-image), Audio (text-to-speech / transcription / translation),
Moderations, and Models. Server-sent-event streaming, Structured Outputs (JSON Schema),
function calling, and multimodal input included.

- **Current API surface (2026)**: built around the Responses API (`/v1/responses`),
  OpenAI's primary generation API. Retired endpoints (edits, legacy completions,
  DALL·E variations) are intentionally absent.
- **Typed end to end**: builder-style requests, typed responses and errors — no
  `serde_json::Value` juggling for normal use.
- **Machine-readable summary**: see [`llms.txt`](llms.txt) if you are an AI
  assistant (or feeding one).

| API | Endpoint | Support |
|---|---|---|
| Responses (primary) | `/v1/responses` | create / stream / retrieve / delete / cancel |
| Chat Completions | `/v1/chat/completions` | create / stream |
| Embeddings | `/v1/embeddings` | create |
| Images | `/v1/images/generations`, `/v1/images/edits` | generate / edit |
| Audio | `/v1/audio/speech`, `/v1/audio/transcriptions`, `/v1/audio/translations` | speech / transcribe / translate |
| Moderations | `/v1/moderations` | create |
| Models | `/v1/models` | list / retrieve / delete |

## Installation

```sh
cargo add openai_chatgpt_api
cargo add tokio --features full
cargo add futures-util   # only needed for streaming
```

## Quick start

```rust
use openai_chatgpt_api::responses::ResponseRequest;
use openai_chatgpt_api::OpenAiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // reads OPENAI_API_KEY (and optionally OPENAI_BASE_URL / OPENAI_ORG_ID / OPENAI_PROJECT_ID)
    let client = OpenAiClient::from_env()?;

    let request = ResponseRequest::new("gpt-5.6-terra", "Explain Rust ownership in three sentences.")
        .instructions("You are a concise assistant.")
        .max_output_tokens(500);

    let response = client.responses().create(&request).await?;
    println!("{}", response.output_text());
    Ok(())
}
```

Every example below is self-contained inside that `main` body. Runnable versions
live in [`examples/`](examples/) — try `cargo run --example responses`.

## Which API should I use?

Use the **Responses API** (`client.responses()`) for new projects — it is
OpenAI's recommended primary API, with built-in tools and server-side
conversation state. **Chat Completions** (`client.chat()`) remains fully
supported for existing message-array codebases.

## How to stream output token by token

```rust
use futures_util::StreamExt;
use openai_chatgpt_api::responses::{ResponseRequest, ResponseStreamEvent};

let request = ResponseRequest::new("gpt-5.6-luna", "Write a haiku about Rust.");

let mut stream = client.responses().stream(&request).await?;
while let Some(event) = stream.next().await {
    match event? {
        ResponseStreamEvent::OutputTextDelta { delta, .. } => print!("{delta}"),
        ResponseStreamEvent::Completed { .. } => println!(),
        _ => {}
    }
}
```

`client.chat().stream(&request)` works the same way and yields
`ChatCompletionChunk`s (use `chunk.delta_content()`).

## How to call functions (tools)

```rust
use openai_chatgpt_api::responses::{InputItem, ResponseRequest, Tool};
use serde_json::json;

let tools = vec![Tool::function(
    "get_weather",
    "Get the current weather for a city",
    json!({
        "type": "object",
        "properties": { "city": { "type": "string" } },
        "required": ["city"],
        "additionalProperties": false
    }),
)];

// 1. The model decides to call your function
let request = ResponseRequest::new("gpt-5.6-terra", "What's the weather in Tokyo?")
    .tools(tools.clone());
let response = client.responses().create(&request).await?;

// 2. Execute the calls and send the results back
let mut items = Vec::new();
for call in response.function_calls() {
    let result = json!({ "temperature_c": 22 }); // your real implementation here
    items.push(InputItem::function_call_output(&call.call_id, result.to_string()));
}
let follow_up = ResponseRequest::new("gpt-5.6-terra", items)
    .previous_response_id(&response.id)
    .tools(tools);

// 3. The model answers using the results
let final_response = client.responses().create(&follow_up).await?;
println!("{}", final_response.output_text());
```

Built-in OpenAI tools work too: `Tool::WebSearch`, `Tool::FileSearch { .. }`,
`Tool::CodeInterpreter { .. }`, `Tool::ImageGeneration`.

## How to get structured JSON output (Structured Outputs)

```rust
use openai_chatgpt_api::responses::ResponseRequest;
use serde_json::json;

let request = ResponseRequest::new("gpt-5.6-terra", "Extract: 'Alice, 30, Tokyo'")
    .json_schema("person", json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age":  { "type": "integer" },
            "city": { "type": "string" }
        },
        "required": ["name", "age", "city"],
        "additionalProperties": false
    }));

let response = client.responses().create(&request).await?;
let person: serde_json::Value = serde_json::from_str(&response.output_text())?;
```

The same `.json_schema(name, schema)` shorthand exists on `ChatCompletionRequest`.

## How to send images to the model (vision / multimodal input)

```rust
use openai_chatgpt_api::responses::{InputContentPart, InputItem, ResponseRequest};

let request = ResponseRequest::new("gpt-5.6-terra", vec![InputItem::user(vec![
    InputContentPart::text("What is in this image?"),
    InputContentPart::image_url("https://example.com/photo.png"), // or a data: URI
])]);
let response = client.responses().create(&request).await?;
```

## How to continue a conversation

```rust
let first = client.responses()
    .create(&ResponseRequest::new("gpt-5.6-terra", "My name is Alice."))
    .await?;

let second = client.responses()
    .create(&ResponseRequest::new("gpt-5.6-terra", "What's my name?")
        .previous_response_id(&first.id))
    .await?; // "Alice" — history is kept server-side
```

## How to use Chat Completions

```rust
use openai_chatgpt_api::chat::{ChatCompletionRequest, ChatMessage};

let request = ChatCompletionRequest::new(
    "gpt-5.6-terra",
    vec![
        ChatMessage::system("You are a helpful assistant."),
        ChatMessage::user("Hello!"),
    ],
)
.max_completion_tokens(500); // note: max_tokens is deprecated upstream

let completion = client.chat().create(&request).await?;
println!("{}", completion.content().unwrap_or_default());
```

## How to create embeddings

```rust
use openai_chatgpt_api::embeddings::EmbeddingsRequest;

let request = EmbeddingsRequest::new("text-embedding-3-small", "The food was delicious.")
    .dimensions(256); // optional truncation
let response = client.embeddings().create(&request).await?;
let vector: &Vec<f32> = &response.data[0].embedding;
```

## How to generate and edit images

```rust
use openai_chatgpt_api::images::{ImageEditRequest, ImageGenerationRequest};
use openai_chatgpt_api::FilePart;

// Generation — gpt-image models return base64, not URLs
let request = ImageGenerationRequest::new("gpt-image-2", "A watercolor Japanese house")
    .size("1536x1024")
    .quality("high");
let images = client.images().generate(&request).await?;
for b64 in images.b64_images() { /* base64-decode and save (see examples/images.rs) */ }

// Editing
let request = ImageEditRequest::new(
    "gpt-image-2",
    FilePart::from_path("input.png")?,
    "Add a cat sitting on the porch",
);
let edited = client.images().edit(request).await?;
```

## How to synthesize speech and transcribe audio

```rust
use openai_chatgpt_api::audio::{SpeechRequest, TranscriptionRequest, TranslationRequest};
use openai_chatgpt_api::FilePart;

// Text to speech (TTS)
let request = SpeechRequest::new("gpt-4o-mini-tts", "こんにちは!", "marin");
let mp3_bytes = client.audio().speech(&request).await?;
std::fs::write("voice.mp3", mp3_bytes)?;

// Speech to text (transcription)
let request = TranscriptionRequest::new("gpt-4o-transcribe", FilePart::from_path("voice.mp3")?)
    .language("ja");
let transcript = client.audio().transcribe(request).await?;
println!("{}", transcript.text);

// Translate any language to English
let request = TranslationRequest::new("whisper-1", FilePart::from_path("voice.mp3")?);
let translated = client.audio().translate(request).await?;
```

## How to moderate content

```rust
use openai_chatgpt_api::moderations::ModerationRequest;

let request = ModerationRequest::new("Some user input").model("omni-moderation-latest");
let moderation = client.moderations().create(&request).await?;
if moderation.results[0].flagged { /* reject */ }
```

## How to handle errors

```rust
use openai_chatgpt_api::OpenAiError;

match client.responses().create(&request).await {
    Ok(response) => println!("{}", response.output_text()),
    Err(OpenAiError::Api { status, message, code, .. }) => {
        // e.g. status=429 (rate limit), 401 (bad key), 400 (bad request)
        eprintln!("API error {status}: {message} (code: {code:?})");
    }
    Err(other) => eprintln!("{other}"), // Http / Parse / Config / Io
}
```

## Client configuration

```rust
let client = OpenAiClient::builder("sk-...")
    .organization("org-...")                          // OpenAI-Organization header
    .project("proj_...")                              // OpenAI-Project header
    .base_url("https://my-proxy.example.com/v1")      // proxies / gateways / mocks
    .http_client(reqwest::Client::builder().build()?) // timeouts, proxies, ...
    .build();
```

## Model quick reference (July 2026)

Model ids are plain strings — new models work without a crate update.
Check the [OpenAI model list](https://platform.openai.com/docs/models) for the latest.

| Task | Current models |
|---|---|
| Text / reasoning | `gpt-5.6-sol` (flagship), `gpt-5.6-terra` (balanced), `gpt-5.6-luna` (budget) |
| Image generation | `gpt-image-2`, `gpt-image-1-mini` |
| Embeddings | `text-embedding-3-large`, `text-embedding-3-small` |
| Speech synthesis | `gpt-4o-mini-tts`, `tts-1-hd` |
| Transcription | `gpt-4o-transcribe`, `gpt-4o-mini-transcribe`, `whisper-1` |
| Moderation | `omni-moderation-latest` |

## Testing

```sh
cargo test                       # offline: unit + doc tests
OPENAI_API_KEY=sk-... cargo test # additionally runs live API tests
```

## Migrating from v0.1

v0.2 is a full rewrite for the current OpenAI API. The v0.1 types (`ChatGpt`,
`ChatGptRequestChatCompletions`, `ChatGptChatFormat`, `to_value()`, ...) no
longer exist — do not use them in new code.

| v0.1 | v0.2 |
|---|---|
| `ChatGpt::new(key)` | `OpenAiClient::new(key)` |
| `gpt.chat_completions(&req)` | `client.chat().create(&req)` or `client.responses().create(&req)` |
| `ChatGptChatFormat::new_user(text)` | `ChatMessage::user(text)` / `InputItem::user(text)` |
| `res.to_value()` | typed fields: `response.output_text()`, `completion.content()` |
| `gpt.edits(..)`, `gpt.completions_create(..)`, `gpt.images_variations(..)` | removed — endpoints retired by OpenAI |

## License

MIT — see [LICENSE](LICENSE). Contributions welcome via
[GitHub](https://github.com/uiuifree/rust-openai-chatgpt-api).
