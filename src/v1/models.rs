use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use crate::v1::{ChatGptResponse};


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatGptResponseModelList {
    pub(crate) value: Value,
}

impl ChatGptResponse for ChatGptResponseModelList {
    fn to_value(&self) -> &Value {
        &self.value
    }
}


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatGptResponseModelRetrieve {
    pub(crate) value: Value,
}

impl ChatGptResponse for ChatGptResponseModelRetrieve {
    fn to_value(&self) -> &Value {
        &self.value
    }
}

