//! Structured Outputs: force the model to answer with JSON matching a schema.
//!
//! Run: `OPENAI_API_KEY=sk-... cargo run --example structured_output`

use openai_chatgpt_api::responses::ResponseRequest;
use openai_chatgpt_api::OpenAiClient;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
struct Person {
    name: String,
    age: u32,
    city: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenAiClient::from_env()?;

    let request = ResponseRequest::new(
        "gpt-5.6-terra",
        "Extract the person from: 'Alice, 30 years old, lives in Tokyo.'",
    )
    .json_schema(
        "person",
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer" },
                "city": { "type": "string" }
            },
            "required": ["name", "age", "city"],
            "additionalProperties": false
        }),
    );

    let response = client.responses().create(&request).await?;
    let person: Person = serde_json::from_str(&response.output_text())?;
    println!("{} ({}) lives in {}", person.name, person.age, person.city);
    Ok(())
}
