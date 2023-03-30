# ChatGPT API Rust Library

## Overview

This Rust library provides a simple and efficient way to interact with the ChatGPT API, which is a state-of-the-art NLP
platform that can generate human-like responses to text queries. The library provides a convenient interface for sending
requests and receiving responses from the ChatGPT API, allowing developers to easily integrate the API into their
Rust-based projects.

## Features

- Easy-to-use API for sending requests and receiving responses.
- Provides responses in multiple formats, including text and JSON.
- Supports multiple endpoints and response languages.

## Installation

To use this library, add the following to your `Cargo.toml` file:

```toml
[dependencies]
openai_chatgpt_api = "0.1.0"
```

Then, add the following to your Rust code:

```rust
use openai_chatgpt_api::ChatGPT;
let chatgpt = ChatGpt::new("YOUR_API_KEY_HERE");
let request = ChatGptRequestChatCompletions::new(
    "gpt-3.5-turbo",
    vec![
        ChatGptChatFormat::new_system("Rust OSS開発者"),
        ChatGptChatFormat::new_user("ChatGPT API のRustライブラリを作ったのでエンジニアが好みそうなReadmeを作って欲しい。"),
    ]
);

let res = chatgpt.chat_completions(&request).await.unwrap();
println!("{:?}", response);
```

You can replace `"YOUR_API_KEY_HERE"` with your actual API key, which can be obtained from the ChatGPT API website.

## Usage

### Creating a New ChatGPT Object

To use the ChatGPT API Rust library, you first need to create a new `ChatGPT` object. You can do this using the
following code:

```rust
use openai_chatgpt_api::ChatGPT;
let chatgpt = ChatGpt::new("YOUR_API_KEY_HERE");
```

Replace `"YOUR_API_KEY_HERE"` with your actual API key.

### Chatting
Here is an example of how to use the library to chat with the ChatGPT API:

```rust
use openai_chatgpt_api::ChatGPT;
let request = ChatGptRequestChatCompletions::new(
    "gpt-3.5-turbo",
    vec![
        ChatGptChatFormat::new_system("Rust OSS開発者"),
        ChatGptChatFormat::new_user("ChatGPT API のRustライブラリを作ったのでエンジニアが好みそうなReadmeを作って欲しい。"),
    ]
);

let res = chatgpt.chat_completions(&request).await.unwrap();
println!("{:?}", response.to_value());
```
