use std::fmt::{Debug, Formatter};
use serde_json::{Value};

#[derive(Clone)]
pub enum ChatGptError {
    Connection(String),
    Status(u16, String),
    JsonParse(Value),
}

impl ChatGptError {
    pub fn to_string(&self) -> String {
        match self {
            ChatGptError::Connection(e) => { e.to_string() }
            ChatGptError::JsonParse(e) => { e.to_string() }
            ChatGptError::Status(status, value) => {
                format!("status: {} ,message:{}", status, value)
            }
        }
    }
}

impl Debug for ChatGptError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_string().as_str())
    }
}
