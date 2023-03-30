use std::fmt::{Debug};
use reqwest::{Error, RequestBuilder, Response};
use reqwest::multipart::Form;
use serde_json::{Value};
use crate::error::ChatGptError;


#[derive(Default, Debug)]
pub struct HttpClient {}

impl HttpClient {
    pub async fn get<T>(openai_key: &str, org_key: &str, url: &str, data: &Value) -> Result<T, ChatGptError>
        where
            T: for<'de> serde::Deserialize<'de>,
    {
        let mut response = reqwest::Client::new().get(url);
        response = set_auth(response, openai_key, org_key);
        if !data.is_null() {
            response = response.json(&data);
        }
        response_parse(response.send().await).await
    }
    pub async fn post<T, U>(openai_key: &str, org_key: &str, url: &str, params: U) -> Result<T, ChatGptError>
        where
            T: for<'de> serde::Deserialize<'de>,
            U: serde::Serialize + std::fmt::Debug
    {
        let mut response = reqwest::Client::new()
            .post(url);
        response = set_auth(response, openai_key, org_key);

        let response = response.json(&params)
            .send()
            .await;
        response_parse(response).await
    }

    pub async fn post_data<T>(openai_key: &str, org_key: &str, url: &str, form: Form) -> Result<T, ChatGptError>
        where
            T: for<'de> serde::Deserialize<'de>,

    {
        println!("{:?}", form);
        let mut response = reqwest::Client::new()
            .post(url)
            .multipart(form);
        response = set_auth(response, openai_key, org_key);
        let response = response
            .send()
            .await;
        // println!("{:?}",response.unwrap().text().await);
        response_parse(response).await

        // Err(ChatGptError::Connection("".to_string()))
    }
}


fn set_auth(mut request: RequestBuilder, openai_key: &str, org_key: &str) -> RequestBuilder {
    if !openai_key.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", openai_key));
    }
    if !org_key.is_empty() {
        request = request.header("OpenAI-Organization", format!("{}", org_key))
    }
    request
}

async fn response_parse<T>(response: Result<Response, Error>) -> Result<T, ChatGptError>
    where
        T: for<'de> serde::Deserialize<'de>,
{
    let response = match response {
        Ok(response) => { response }
        Err(e) => {
            return Err(ChatGptError::Connection(e.to_string()));
        }
    };
    let status = response.status().as_u16();
    let res = response.text().await;
    let mut text = "".to_string();
    let mut json = Value::default();
    if res.is_ok() {
        text = res.unwrap();
        json = match serde_json::from_str(text.as_str()) {
            Ok(e) => e,
            _ => Value::default()
        };
    }

    if !(200 <= status && status < 300) {
        return Err(ChatGptError::Status(status, text));
    }
    let parse = serde_json::from_str(json.to_string().as_str());
    if parse.is_err() {
        return Err(ChatGptError::JsonParse(json));
    }

    Ok(parse.unwrap())
}