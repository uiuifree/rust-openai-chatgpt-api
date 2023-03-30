use serde_json::json;
use openai_chatgpt_api::{ChatGpt, ChatGptChatFormat, ChatGptRequest,  ChatGptRequestAudioTranslations, ChatGptRequestChatCompletions, ChatGptRequestCompletionsCreate, ChatGptRequestEdits, ChatGptRequestEmbeddingsGenerations, ChatGptRequestImagesEdits, ChatGptRequestImagesGenerations, ChatGptRequestImagesVariation, ChatGptResponse};

fn get_client() -> ChatGpt {
    let key = std::env::var("CHATGPT_KEY").unwrap();
    ChatGpt::new(key.as_str())
}

#[tokio::test]
async fn test_model() {
    let gpt = get_client();
    let a = gpt.models_list().await;
    // let a = gpt.completions_create("text-davinci-003").await;
    a.clone().unwrap().to_value();
    dbg!(a);
    // let a = gpt.chat_completions("Hello, my name is", 10, "").await;
    // println!("{:?}",a);
    assert!(true);
}


#[tokio::test]
async fn test_completions() {
    let gpt = get_client();
    let request = ChatGptRequestCompletionsCreate::new(
        "text-davinci-003",
        100,
        "Say this is a test",
    );
    let res = gpt.completions_create(&request).await;
    assert!(res.is_ok());
    let request = ChatGptRequestCompletionsCreate::from_value(json!({
                "model": "text-davinci-003",
            "prompt": ["Say this is a test"],
            "max_tokens": 7,
            "temperature": 0.0

        })).unwrap();
    let res = gpt.completions_create(&request).await;
    assert!(res.is_ok());
    dbg!(res);
}

#[tokio::test]
async fn chat() {
    let gpt = get_client();
    let request = ChatGptRequestChatCompletions::new(
        "gpt-3.5-turbo",
        vec![
            ChatGptChatFormat::new_system("Rust OSS開発者"),
            ChatGptChatFormat::new_user("ChatGPT API のRustライブラリを作ったのでエンジニアが好みそうなReadmeを作って欲しい。言語は英語で"),
        ],
    );
    dbg!(request.clone().to_value());

    let res = gpt.chat_completions(&request).await;
    dbg!(res.unwrap().to_value());
}

#[tokio::test]
async fn edit() {
    let gpt = get_client();
    let request = ChatGptRequestEdits::new(
        "text-davinci-edit-001",
        "Fix the spelling mistakes",
        " day of the wek is it?",
    );
    dbg!(request.clone().to_value());

    let res = gpt.edits(&request).await;
    dbg!(res);
}


#[tokio::test]
async fn image() {
    let gpt = get_client();
    // let mut request = ChatGptRequestImagesGenerations::new("Japan Home. Wood Picture.", 1);
    // dbg!(request.clone().to_value());
    // let res = gpt.images_generations(&request).await;
    // assert!(res.is_ok(), "error: {:?}", res);
    // let res = res.unwrap();
    // println!("{:?}", res);
    // let urls = res.get_urls();
    // for url in urls {
    //     println!("{}", url);
    // }
    // let mut request = ChatGptRequestImagesGenerations::new("Japan Home. Wood Picture.", 1);
    // request.set_response_format("b64_json");
    // let res = gpt.images_generations(&request).await;
    // assert!(res.is_ok(), "error: {:?}", res);
    // let res = res.unwrap();
    // let rows = res.b64_jsons();
    // assert_ne!(rows.len(), 0);

    let request = ChatGptRequestImagesVariation::new(
        test_png().as_str(),
        1,
    );
    let res = gpt.images_variations(&request).await;
    assert!(res.is_ok(), "error: {:?}", res);
    println!("{:?}", res);
}


#[tokio::test]
async fn embeddings() {
    let gpt = get_client();
    let request = ChatGptRequestEmbeddingsGenerations::new("text-embedding-ada-002", "The food was delicious and the waiter...");
    dbg!(request.clone().to_value());

    let res = gpt.embeddings(&request).await;
    dbg!(res);
}


#[tokio::test]
async fn audio() {
    let gpt = get_client();
    // let request = ChatGptRequestAudioTranscriptions::new(
    //     "whisper-1",
    //     test_mp3().as_str());
    // let res = gpt.audio_transcriptions(&request).await;
    let request = ChatGptRequestAudioTranslations::new(
        "whisper-1",
        test_mp3().as_str());
    let res = gpt.audio_translations(&request).await;
    dbg!(res);
}


fn test_mp3() -> String {
    "./001-sibutomo.mp3".to_string()
}

fn test_png() -> String {
    "./img-aefyVvkbJ2nVn7V9nl5OVm4m.png".to_string()
}