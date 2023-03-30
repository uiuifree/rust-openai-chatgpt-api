pub(crate) mod models;
pub(crate) mod completions;
pub(crate) mod chat;
pub(crate) mod edits;
pub(crate) mod images;
pub(crate) mod embeddings;
pub(crate) mod audio;

use std::io::Read;
use std::path::Path;
use reqwest::multipart::{Form, Part};
use serde::{Deserialize};
use serde_json::{from_value, json, Map, Value};
use crate::error::ChatGptError;

pub trait ChatGptRequest {
    fn from_value(value: Value) -> Result<Self, ChatGptError> where Self: Sized;
    fn to_value(&self) -> Value;
}


pub trait ChatGptResponse {
    fn to_value(&self) -> &Value;
}


pub(crate) fn convert_from_value<T>(value: Value) -> Result<T, ChatGptError>
    where
        T: for<'de> Deserialize<'de>, {
    let request = from_value::<T>(value.clone());
    match request {
        Ok(request) => Ok(request),
        Err(e) => Err(ChatGptError::JsonParse(json!({
                "value": value,
                "error": e.to_string()
            }))),
    }
}

pub(crate) fn convert_form(value: Value, file_keys: Vec<String>) -> Form {
    let mut form = Form::new();
    if !value.is_object() {
        return form;
    }
    let data = value.as_object().unwrap().clone();
    for (key, value) in data {
        if !(value.is_string() || value.is_number()) {
            continue;
        }

        let mut new_value = "".to_string();

        if value.as_str().is_some() {
            new_value = value.as_str().unwrap().to_string();
        }
        if value.as_u64().is_some() {
            new_value = value.as_u64().unwrap().to_string();
        }
        if value.as_i64().is_some() {
            new_value = value.as_i64().unwrap().to_string();
        }
        if value.as_f64().is_some() {
            new_value = value.as_f64().unwrap().to_string();
        }

        if file_keys.contains(&key) {
            let file_path = new_value.to_string();
            let path = Path::new(&file_path);
            if !path.exists() {
                continue;
            }
            let mut file = std::fs::File::open(file_path.as_str()).unwrap();
            let mut byte = vec![];
            file.read_to_end(&mut byte).unwrap();
            let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
            let mut mine = "".to_string();
            if file_name.ends_with(".mp3") {
                mine = "audio/mpeg".to_string();
            } else if file_name.ends_with(".png") {
                mine = "image/png".to_string();
            }
            let part = Part::bytes(byte).file_name(file_name).mime_str(mine.as_str()).unwrap();
            // let part = Part::bytes(byte).file_name(file_name).mime_str("audio/mpeg").unwrap();
            form = form.part("file", part);
        } else {
            form = form.text(key.clone(), new_value);
        }
    }
    form
}


pub(crate) fn trim_value(value: Value) -> Option<Value> {
    match value {
        Value::Null => { None }
        Value::Bool(v) => { Some(Value::Bool(v)) }
        Value::Number(v) => { Some(Value::Number(v)) }
        Value::String(v) => { Some(Value::String(v)) }
        Value::Array(v) => {
            let mut rows = vec![];
            for row in v {
                if let Some(row) = trim_value(row) {
                    rows.push(row);
                }
            }
            Some(Value::Array(rows))
        }
        Value::Object(v) => {
            let mut hash = Map::new();
            for (key, value) in v {
                if let Some(value) = trim_value(value) {
                    hash.insert(key, value);
                }
            }
            if hash.is_empty() {
                None
            } else {
                Some(Value::Object(hash))
            }
        }
    }
}