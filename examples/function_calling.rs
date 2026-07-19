//! Complete function-calling (tool) loop with the Responses API:
//! model requests a call -> you execute it -> model uses the result.
//!
//! Run: `OPENAI_API_KEY=sk-... cargo run --example function_calling`

use openai_chatgpt_api::responses::{InputItem, ResponseRequest, Tool};
use openai_chatgpt_api::OpenAiClient;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenAiClient::from_env()?;

    let tools = vec![Tool::function(
        "get_weather",
        "Get the current weather for a city",
        json!({
            "type": "object",
            "properties": { "city": { "type": "string" } },
            "required": ["city"],
            "additionalProperties": false
        }),
    )];

    let request = ResponseRequest::new("gpt-5.6-terra", "What's the weather in Tokyo right now?")
        .tools(tools.clone());
    let response = client.responses().create(&request).await?;

    let mut items = Vec::new();
    for call in response.function_calls() {
        println!("model called {}({})", call.name, call.arguments);
        let result = json!({ "temperature_c": 22, "condition": "sunny" });
        items.push(InputItem::function_call_output(
            &call.call_id,
            result.to_string(),
        ));
    }

    let follow_up = ResponseRequest::new("gpt-5.6-terra", items)
        .previous_response_id(&response.id)
        .tools(tools);
    let final_response = client.responses().create(&follow_up).await?;

    println!("{}", final_response.output_text());
    Ok(())
}
