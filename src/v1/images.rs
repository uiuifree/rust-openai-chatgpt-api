use reqwest::multipart::{Form};
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::error::ChatGptError;
use crate::v1::{ChatGptRequest, convert_form, convert_from_value, trim_value};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatGptResponseImagesGenerations {
    pub(crate) value: Value,
}

impl ChatGptResponseImagesGenerations {
    pub fn get_urls(&self) -> Vec<String> {
        let mut urls = vec![];
        if let Some(value) = self.value.get("data") {
            if let Some(rows) = value.as_array() {
                for row in rows {
                    if let Some(url) = row.get("url") {
                        urls.push(url.to_string());
                    }
                }
            }
        }
        urls
    }
    pub fn b64_jsons(&self) -> Vec<String> {
        let mut urls = vec![];
        if let Some(value) = self.value.get("data") {
            if let Some(rows) = value.as_array() {
                for row in rows {
                    if let Some(url) = row.get("b64_json") {
                        urls.push(url.to_string());
                    }
                }
            }
        }
        urls
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatGptRequestImagesGenerations {
    prompt: String,
    n: Option<isize>,
    size: Option<String>,
    response_format: String,
    user: Option<String>,
}

impl ChatGptRequest for ChatGptRequestImagesGenerations {
    fn from_value(value: Value) -> Result<Self, ChatGptError> where Self: Sized {
        convert_from_value(value)
    }

    fn to_value(&self) -> Value {
        trim_value(json!(self)).unwrap()
    }
}

impl ChatGptRequestImagesGenerations {
    pub fn new(prompt: &str, n: isize) -> Self {
        Self {
            prompt: prompt.to_string(),
            n: Some(n),
            size: None,
            response_format: "url".to_string(),
            user: None,
        }
    }
    pub fn set_response_format(&mut self, response_format: &str) -> &mut Self {
        self.response_format = response_format.to_string();
        self
    }
}


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatGptResponseImagesEdits {
    pub(crate) value: Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatGptRequestImagesEdits {
    image: String,
    mask: Option<String>,
    prompt: String,
    n: Option<f32>,
    size: Option<String>,
    response_format: Option<String>,
    user: Option<String>,
}

impl ChatGptRequestImagesEdits {
    pub fn new(image: &str, prompt: &str) -> ChatGptRequestImagesEdits {
        ChatGptRequestImagesEdits {
            image: image.to_string(),
            mask: None,
            prompt: prompt.to_string(),
            n: None,
            size: None,
            response_format: None,
            user: None,
        }
    }
}

impl ChatGptRequest for ChatGptRequestImagesEdits {
    fn from_value(value: Value) -> Result<Self, ChatGptError> where Self: Sized {
        convert_from_value(value)
    }

    fn to_value(&self) -> Value {
        trim_value(json!(self)).unwrap()
    }
}

impl Into<Form> for ChatGptRequestImagesEdits {
    fn into(self) -> Form {
        convert_form(self.to_value(), vec!["image".to_string()])
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatGptResponseImagesVariation {
    pub(crate) value: Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatGptRequestImagesVariation {
    image: String,
    n: Option<isize>,
    size: Option<String>,
    response_format: Option<String>,
    user: Option<String>,
}

impl ChatGptRequestImagesVariation {
    pub fn new(image: &str, n: isize) -> ChatGptRequestImagesVariation {
        ChatGptRequestImagesVariation {
            image: image.to_string(),
            n: Some(n),
            size: None,
            response_format: None,
            user: None,
        }
    }
}

impl ChatGptRequest for ChatGptRequestImagesVariation {
    fn from_value(value: Value) -> Result<Self, ChatGptError> where Self: Sized {
        convert_from_value(value)
    }

    fn to_value(&self) -> Value {
        trim_value(json!(self)).unwrap()
    }
}

impl Into<Form> for ChatGptRequestImagesVariation {
    fn into(self) -> Form {
        convert_form(self.to_value(), vec!["image".to_string()])
    }
}

