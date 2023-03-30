use serde_derive::{Deserialize, Serialize};
use serde_json::{ json, Value};
use crate::error::ChatGptError;
use crate::v1::{ChatGptRequest, convert_from_value, trim_value};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatGptResponseEdits {
    value: Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatGptRequestEdits {
    model: String,
    input: String,
    instruction: String,
    n: Option<isize>,
    temperature: Option<f32>,
    top_p: Option<f32>,
}

impl ChatGptRequest for ChatGptRequestEdits {
    fn from_value(value: Value) -> Result<Self, ChatGptError> where Self: Sized {
        convert_from_value(value)
    }

    fn to_value(&self) -> Value {
        trim_value(json!(self)).unwrap()
    }
}

impl ChatGptRequestEdits {
    pub fn new(model: &str, instruction: &str,input:&str) -> Self {
        Self {
            model: model.to_string(),
            input: input.to_string(),
            instruction: instruction.to_string(),
            n: None,
            temperature: None,
            top_p: None,
        }
    }
}