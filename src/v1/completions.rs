use std::collections::HashMap;
use serde_derive::{Deserialize, Serialize};
use serde_json::{ json, Value};
use crate::error::ChatGptError;
use crate::v1::{ChatGptRequest, convert_from_value};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatGptResponseCompletions {
    value: Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatGptRequestCompletionsCreate {
    model: String,
    // string or array
    prompt: Option<Vec<String>>,
    suffix: Option<String>,
    max_tokens: Option<isize>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    n: Option<isize>,
    stream: Option<bool>,
    logprobs: Option<isize>,
    echo: Option<bool>,
    // string or array
    stop: Option<Vec<String>>,
    presence_penalty: Option<f32>,
    frequency_penalty: Option<f32>,
    best_of: Option<f32>,
    logit_bias: Option<HashMap<String, isize>>,
}

impl ChatGptRequest for ChatGptRequestCompletionsCreate {
    fn from_value(value: Value) -> Result<Self, ChatGptError> where Self: Sized {
        convert_from_value(value)
    }

    fn to_value(&self) -> Value {
        json!(self)
    }
}

impl ChatGptRequestCompletionsCreate {
    pub fn new(model: &str, max_tokens: isize, prompt: &str) -> Self {
        Self {
            model: model.to_string(),
            max_tokens: Some(max_tokens),
            prompt: Some(vec![prompt.to_string()]),
            ..Default::default()
        }
    }
}

impl Default for ChatGptRequestCompletionsCreate {
    fn default() -> Self {
        ChatGptRequestCompletionsCreate {
            model: "".to_string(),
            prompt: None,
            suffix: None,
            max_tokens: None,
            temperature: None,
            top_p: None,
            n: None,
            stream: None,
            logprobs: None,
            echo: None,
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            best_of: None,
            logit_bias: None,
        }
    }
}

pub enum ChatGptPrompt {}