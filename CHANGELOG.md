# Changelog

## 0.2.0 (2026-07-20)

Full rewrite targeting the current OpenAI API. **Breaking: no v0.1 type survives.**

### Added

- Responses API (`/v1/responses`) — the current primary generation API:
  create / stream / retrieve / delete / cancel, function calling,
  Structured Outputs, reasoning control (`reasoning_effort`), multimodal
  input, server-side conversation state via `previous_response_id`
- SSE streaming for Responses and Chat Completions
  (`futures_util::Stream` of typed events)
- Moderations API (`/v1/moderations`)
- Text-to-speech (`/v1/audio/speech`)
- Typed errors (`OpenAiError::Api { status, message, error_type, code, param }`)
- Client builder: custom base URL, `OpenAI-Organization` / `OpenAI-Project`
  headers, custom `reqwest::Client`; `OpenAiClient::from_env()`
- Typed request builders and response structs for every endpoint
- Runnable examples under `examples/`, offline unit tests, env-gated live tests

### Changed

- Entry point renamed: `ChatGpt` → `OpenAiClient`; endpoint methods moved to
  accessors (`client.responses()`, `client.chat()`, ...)
- Chat Completions modernized: `max_completion_tokens` (replaces `max_tokens`),
  `tools`, `response_format: json_schema`, multimodal content parts
- Images modernized for `gpt-image-*` models (`quality`, `output_format`,
  `background`, base64 results via `ImagesResponse::b64_images()`)
- `reqwest` 0.11 → 0.13 (TLS defaults to rustls — no system OpenSSL needed);
  `serde_derive` dropped in favor of `serde` derive; MSRV 1.85

### Removed

- `/v1/edits` (retired by OpenAI in 2024)
- Legacy `/v1/completions` (GPT-3 era)
- Image variations (DALL·E 2 only; retired with DALL·E in May 2026)
- `Value`-based `ChatGptRequest*` / `ChatGptResponse*` wrapper types

## 0.1.2 (2023)

- Initial release: Models, Completions, Chat, Edits, Images, Embeddings, Audio
  over `serde_json::Value` wrappers
