use std::collections::HashMap;
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::ChatGptResponse;
use crate::error::ChatGptError;
use crate::v1::{ChatGptRequest, convert_from_value, trim_value};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatGptResponseChatCompletions {
    pub(crate) value: Value,
}

impl ChatGptResponse for ChatGptResponseChatCompletions {
    fn to_value(&self) -> &Value {
        &self.value
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatGptRequestChatCompletions {
    model: String,
    messages: Vec<ChatGptChatFormat>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    n: Option<isize>,
    stream: Option<bool>,
    stop: Option<Vec<String>>,
    max_tokens: Option<isize>,
    presence_penalty: Option<f32>,
    frequency_penalty: Option<f32>,
    logit_bias: Option<HashMap<String, isize>>,
    user: Option<String>,
}


impl ChatGptRequestChatCompletions {
    pub fn new(model: &str, messages: Vec<ChatGptChatFormat>) -> Self {
        Self {
            model: model.to_string(),
            messages,
            temperature: None,
            top_p: None,
            n: None,
            stream: None,
            stop: None,
            max_tokens: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatGptChatFormat {
    role: String,
    content: String,
}

impl ChatGptChatFormat {
    pub fn new(role: &str, content: &str) -> Self {
        Self {
            role: role.to_string(),
            content: content.to_string(),
        }
    }
    pub fn new_system(content: &str) -> Self {
        ChatGptChatFormat::new("system", content)
    }
    pub fn new_user(content: &str) -> Self {
        Self::new("user", content)
    }
    pub fn new_assistant(content: &str) -> Self {
        Self::new("assistant", content)
    }
}

impl ChatGptRequest for ChatGptRequestChatCompletions {
    fn from_value(value: Value) -> Result<Self, ChatGptError> where Self: Sized {
        convert_from_value(value)
    }

    fn to_value(&self) -> Value {
        trim_value(json!(self)).unwrap()
    }
}

