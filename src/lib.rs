//! Typed async Rust client for the OpenAI API.
//!
//! Covers the current (2026) API surface:
//!
//! | Module | Endpoint |
//! |---|---|
//! | [`responses`] | `/v1/responses` — primary generation API (text, tools, reasoning, streaming) |
//! | [`chat`] | `/v1/chat/completions` — Chat Completions (incl. streaming) |
//! | [`embeddings`] | `/v1/embeddings` |
//! | [`images`] | `/v1/images/generations`, `/v1/images/edits` |
//! | [`audio`] | `/v1/audio/speech`, `/v1/audio/transcriptions`, `/v1/audio/translations` |
//! | [`moderations`] | `/v1/moderations` |
//! | [`models`] | `/v1/models` |
//!
//! # Quick start
//!
//! ```no_run
//! use openai_chatgpt_api::OpenAiClient;
//! use openai_chatgpt_api::responses::ResponseRequest;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), openai_chatgpt_api::OpenAiError> {
//!     let client = OpenAiClient::from_env()?; // OPENAI_API_KEY
//!     let request = ResponseRequest::new("gpt-5.6-terra", "Explain ownership in Rust.")
//!         .instructions("You are a concise assistant.")
//!         .max_output_tokens(500);
//!     let response = client.responses().create(&request).await?;
//!     println!("{}", response.output_text());
//!     Ok(())
//! }
//! ```
//!
//! # Streaming
//!
//! ```no_run
//! use futures_util::StreamExt;
//! use openai_chatgpt_api::OpenAiClient;
//! use openai_chatgpt_api::responses::{ResponseRequest, ResponseStreamEvent};
//!
//! # async fn run() -> Result<(), openai_chatgpt_api::OpenAiError> {
//! let client = OpenAiClient::from_env()?;
//! let request = ResponseRequest::new("gpt-5.6-luna", "Write a haiku.");
//! let mut stream = client.responses().stream(&request).await?;
//! while let Some(event) = stream.next().await {
//!     if let ResponseStreamEvent::OutputTextDelta { delta, .. } = event? {
//!         print!("{delta}");
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Model names are passed as plain strings: the API's model lineup changes
//! faster than any type system should chase. See
//! <https://platform.openai.com/docs/models> for the current list.

#![warn(missing_docs)]

mod client;
mod error;
mod file;
mod sse;

pub mod audio;
pub mod chat;
pub mod embeddings;
pub mod images;
pub mod models;
pub mod moderations;
pub mod responses;

pub use client::{DeletedObject, OpenAiClient, OpenAiClientBuilder, DEFAULT_BASE_URL};
pub use error::OpenAiError;
pub use file::FilePart;
pub use sse::EventStream;
