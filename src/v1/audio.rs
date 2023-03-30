use reqwest::multipart::{Form};
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::error::ChatGptError;
use crate::v1::{ChatGptRequest, convert_form, convert_from_value, trim_value};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatGptResponseAudioTranscriptions {
    pub(crate) value: Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatGptRequestAudioTranscriptions {
    file: String,
    model: String,
    prompt: Option<String>,
    response_format: Option<String>,
    temperature: Option<f32>,
    language: Option<String>,
}

impl Into<Form> for ChatGptRequestAudioTranscriptions {
    fn into(self) -> Form {
        convert_form(self.to_value(), vec!["file".to_string()])
    }
}


impl ChatGptRequest for ChatGptRequestAudioTranscriptions {
    fn from_value(value: Value) -> Result<Self, ChatGptError> where Self: Sized {
        convert_from_value(value)
    }

    fn to_value(&self) -> Value {
        trim_value(json!(self)).unwrap()
    }
}

impl ChatGptRequestAudioTranscriptions {
    pub fn new(model: &str, file: &str) -> Self {
        Self {
            model: model.to_string(),
            prompt: None,
            response_format: None,
            temperature: None,
            file: file.to_string(),
            language: None,
        }
    }
}


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatGptResponseAudioTranslations {
    pub(crate) value: Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatGptRequestAudioTranslations {
    file: String,
    model: String,
    prompt: Option<String>,
    response_format: Option<String>,
    temperature: Option<f32>,
}

impl Into<Form> for ChatGptRequestAudioTranslations {
    fn into(self) -> Form {
        convert_form(self.to_value(), vec!["file".to_string()])
    }
}


impl ChatGptRequest for ChatGptRequestAudioTranslations {
    fn from_value(value: Value) -> Result<Self, ChatGptError> where Self: Sized {
        convert_from_value(value)
    }

    fn to_value(&self) -> Value {
        trim_value(json!(self)).unwrap()
    }
}

impl ChatGptRequestAudioTranslations {
    pub fn new(model: &str, file: &str) -> Self {
        Self {
            model: model.to_string(),
            prompt: None,
            response_format: None,
            temperature: None,
            file: file.to_string(),
        }
    }
}