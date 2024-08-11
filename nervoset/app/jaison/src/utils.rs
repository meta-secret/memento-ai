use anyhow::Result;
use async_openai::types::{
    ChatCompletionRequestSystemMessageArgs, CreateEmbeddingRequestArgs, CreateEmbeddingResponse,
    CreateSpeechRequestArgs, SpeechModel, Voice,
};
use async_openai::{
    types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
    Client,
};
use lazy_static::lazy_static;
use qdrant_client::qdrant::vectors_config::Config;
use qdrant_client::qdrant::{
    CreateCollection, Distance, PointStruct, SearchParamsBuilder, SearchPointsBuilder,
    UpsertPointsBuilder, VectorParams, VectorsConfig,
};
use qdrant_client::{Payload, Qdrant};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use teloxide::prelude::*;
use teloxide::types::{ChatAction, KeyboardButton, KeyboardMarkup};
use tiktoken_rs::cl100k_base;
use tracing::{error, info};

lazy_static! {
    static ref LOCALIZATIONS: HashMap<String, Value> = load_localizations();
}

pub(crate) async fn send_main_menu(
    bot: &Bot,
    chat_id: ChatId,
    language_code: String,
) -> Result<()> {
    // let lang = language_code.unwrap_or("ru".to_string());
    let loc = LOCALIZATIONS
        .get(&language_code)
        .unwrap_or(&LOCALIZATIONS["ru"]);

    let keyboard = KeyboardMarkup::new(vec![
        vec![KeyboardButton::new(
            loc["main_menu"]["education"].as_str().unwrap().to_string(),
        )],
        vec![KeyboardButton::new(
            loc["main_menu"]["ask_question"]
                .as_str()
                .unwrap()
                .to_string(),
        )],
        vec![KeyboardButton::new(
            loc["main_menu"]["agent_creation"]
                .as_str()
                .unwrap()
                .to_string(),
        )],
        vec![KeyboardButton::new(
            loc["main_menu"]["exit"].as_str().unwrap().to_string(),
        )],
    ]);

    bot.send_message(
        chat_id,
        loc["main_menu"]["menu_prompt"]
            .as_str()
            .unwrap()
            .to_string(),
    )
    .reply_markup(keyboard)
    .await?;

    Ok(())
}

pub(crate) async fn send_edu_menu(bot: &Bot, chat_id: ChatId, language_code: String) -> Result<()> {
    // let lang = language_code.unwrap_or("ru".to_string());
    let loc = LOCALIZATIONS
        .get(&language_code)
        .unwrap_or(&LOCALIZATIONS["ru"]);

    let keyboard = KeyboardMarkup::new(vec![
        vec![
            KeyboardButton::new(
                loc["edu_menu"]["previous_chapter"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            ),
            KeyboardButton::new(
                loc["edu_menu"]["next_chapter"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            ),
        ],
        vec![KeyboardButton::new(
            loc["edu_menu"]["ask_question"]
                .as_str()
                .unwrap()
                .to_string(),
        )],
        vec![KeyboardButton::new(
            loc["edu_menu"]["exit"].as_str().unwrap().to_string(),
        )],
    ])
    .resize_keyboard(true);

    bot.send_message(
        chat_id,
        loc["edu_menu"]["menu_prompt"].as_str().unwrap().to_string(),
    )
    .reply_markup(keyboard)
    .await?;

    Ok(())
}

pub(crate) async fn send_question_menu(
    bot: &Bot,
    chat_id: ChatId,
    language_code: String,
) -> Result<()> {
    // let lang = language_code.unwrap_or("ru".to_string());
    let loc = LOCALIZATIONS
        .get(&language_code)
        .unwrap_or(&LOCALIZATIONS["ru"]);

    let keyboard = KeyboardMarkup::new(vec![
        vec![KeyboardButton::new(
            loc["question_menu"]["back_to_edu"]
                .as_str()
                .unwrap()
                .to_string(),
        )],
        vec![KeyboardButton::new(
            loc["question_menu"]["voice_response"]
                .as_str()
                .unwrap()
                .to_string(),
        )],
        vec![KeyboardButton::new(
            loc["question_menu"]["exit"].as_str().unwrap().to_string(),
        )],
    ]);

    bot.send_message(
        chat_id,
        loc["question_menu"]["menu_prompt"]
            .as_str()
            .unwrap()
            .to_string(),
    )
    .reply_markup(keyboard)
    .await?;

    Ok(())
}

pub(crate) async fn send_creation_menu(
    bot: &Bot,
    chat_id: ChatId,
    language_code: String,
) -> Result<()> {
    // let lang = language_code.unwrap_or("ru".to_string());
    let loc = LOCALIZATIONS
        .get(&language_code)
        .unwrap_or(&LOCALIZATIONS["ru"]);

    let keyboard = KeyboardMarkup::new(vec![
        vec![
            KeyboardButton::new(
                loc["creation_menu"]["specify_sphere"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            ),
            KeyboardButton::new(
                loc["creation_menu"]["specify_behaviour_model"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            ),
        ],
        vec![KeyboardButton::new(
            loc["creation_menu"]["create_system_role"]
                .as_str()
                .unwrap()
                .to_string(),
        )],
        vec![KeyboardButton::new(
            loc["creation_menu"]["top_up_db"]
                .as_str()
                .unwrap()
                .to_string(),
        )],
        vec![KeyboardButton::new(
            loc["creation_menu"]["exit"].as_str().unwrap().to_string(),
        )],
    ]);

    bot.send_message(
        chat_id,
        loc["creation_menu"]["menu_prompt"]
            .as_str()
            .unwrap()
            .to_string(),
    )
    .reply_markup(keyboard)
    .await?;

    Ok(())
}

pub(crate) async fn send_system_role_creation_menu(
    bot: &Bot,
    chat_id: ChatId,
    language_code: String,
) -> Result<()> {
    // let lang = language_code.unwrap_or("ru".to_string());
    let loc = LOCALIZATIONS
        .get(&language_code)
        .unwrap_or(&LOCALIZATIONS["ru"]);

    let keyboard = KeyboardMarkup::new(vec![
        vec![KeyboardButton::new(
            loc["system_role_creation_menu"]["confirm"]
                .as_str()
                .unwrap()
                .to_string(),
        )],
        vec![KeyboardButton::new(
            loc["system_role_creation_menu"]["decline"]
                .as_str()
                .unwrap()
                .to_string(),
        )],
    ]);

    bot.send_message(
        chat_id,
        loc["system_role_creation_menu"]["menu_prompt"]
            .as_str()
            .unwrap()
            .to_string(),
    )
    .reply_markup(keyboard)
    .await?;

    Ok(())
}


pub(crate) async fn send_db_operation_menu(
    bot: &Bot,
    chat_id: ChatId,
    language_code: String,
) -> Result<()> {
    // let lang = language_code.unwrap_or("ru".to_string());
    let loc = LOCALIZATIONS
        .get(&language_code)
        .unwrap_or(&LOCALIZATIONS["ru"]);

    let keyboard = KeyboardMarkup::new(vec![
        vec![KeyboardButton::new(
            loc["db_operation_menu"]["top_up"]
                .as_str()
                .unwrap()
                .to_string(),
        )],
        vec![KeyboardButton::new(
            loc["db_operation_menu"]["edit"]
                .as_str()
                .unwrap()
                .to_string(),
        )],
        vec![KeyboardButton::new(
            loc["db_operation_menu"]["exit"]
                .as_str()
                .unwrap()
                .to_string(),
        )],
    ]);

    bot.send_message(
        chat_id,
        loc["db_operation_menu"]["menu_prompt"]
            .as_str()
            .unwrap()
            .to_string(),
    )
    .reply_markup(keyboard)
    .await?;

    Ok(())
}


pub(crate) async fn send_db_topping_up_menu(
    bot: &Bot,
    chat_id: ChatId,
    language_code: String,
) -> Result<()> {
    let loc = LOCALIZATIONS
        .get(&language_code)
        .unwrap_or(&LOCALIZATIONS["ru"]);

    let keyboard = KeyboardMarkup::new(vec![vec![KeyboardButton::new(
        loc["db_topping_up_menu"]["exit"]
            .as_str()
            .unwrap()
            .to_string(),
    )]]);

    bot.send_message(
        chat_id,
        loc["db_topping_up_menu"]["menu_prompt"]
            .as_str()
            .unwrap()
            .to_string(),
    )
    .reply_markup(keyboard)
    .await?;

    Ok(())
}

fn load_localizations() -> HashMap<String, Value> {
    let mut localizations = HashMap::new();

    let ru_content = fs::read_to_string("resources/localization/ru.json").unwrap();
    let en_content = fs::read_to_string("resources/localization/en.json").unwrap();

    let ru_json: Value = serde_json::from_str(&ru_content).unwrap();
    let en_json: Value = serde_json::from_str(&en_content).unwrap();

    localizations.insert("ru".to_string(), ru_json);
    localizations.insert("en".to_string(), en_json);

    localizations
}


pub async fn vectorize(data: String) -> Result<Vec<f32>> {
    let client = Client::new();

    let request = CreateEmbeddingRequestArgs::default()
        .model("text-embedding-3-large")
        .input(data)
        .build()?;

    let response: CreateEmbeddingResponse = client.embeddings().create(request).await?;
    let embedding = response.data.into_iter().next().unwrap().embedding;

    Ok(embedding)
}

pub async fn qdrant_search(msg: Message, query_vector: Vec<f32>) -> Result<String> {
    let client = Qdrant::from_url(
        "https://20a8f64b-9558-4081-b30a-7abc040f962b.us-east4-0.gcp.cloud.qdrant.io:6334",
    )
    .api_key(std::env::var("QDRANT_API_KEY").expect("QDRANT_API_KEY is not provided"))
    .build()?;

    let user_id = msg.from().map(|user| user.id.0).unwrap_or(0) as i64;

    let collection_names = vec![user_id.to_string()];

    // let collection_names = vec![
    //     "base_v2_1",
    //     "base_v2_2",
    //     "base_v2_3",
    //     "base_v2_4",
    //     "base_v2_5",
    //     "base_v2_6",
    // ];

    let mut all_results = Vec::new();

    for collection_name in collection_names {
        match client
            .search_points(
                SearchPointsBuilder::new(collection_name.clone(), query_vector.clone(), 5)
                    .with_payload(true)
                    .params(SearchParamsBuilder::default().exact(true)),
            )
            .await
        {
            Ok(result) => all_results.extend(result.result),
            Err(err) => {
                error!(
                    "Error during points search in collection {}: {:?}",
                    collection_name, err
                );
            }
        }
    }

    // info!("all_results: {:?}", all_results);

    // Filtering and sorting results
    let mut filtered_results: Vec<_> = all_results
        .into_iter()
        .filter(|point| point.score > 0.3)
        .collect();

    filtered_results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut combined_payloads = Vec::new();
    for found_point in filtered_results {
        if let Some(payload) = found_point.payload.get("text") {
            if let Some(text) = payload.as_str() {
                combined_payloads.push(text.to_string());
            }
        }
    }

    if combined_payloads.is_empty() {
        Ok("No data found.".to_string())
    } else {
        let combined_payload = combined_payloads.join("\n");
        // info!("combined payload: {:?}", combined_payload);
        let new_combined_payload = tokenize_and_truncate(combined_payload)?;
        info!("new_combined_payload: {:?}", new_combined_payload);
        Ok(new_combined_payload)
    }
}

pub async fn qdrant_upsert(msg: Message) -> Result<()> {
    let client = Qdrant::from_url(
        "https://20a8f64b-9558-4081-b30a-7abc040f962b.us-east4-0.gcp.cloud.qdrant.io:6334",
    )
    .api_key(std::env::var("QDRANT_API_KEY").expect("QDRANT_API_KEY is not provided"))
    .build()?;

    let user_id = msg.from().map(|user| user.id.0).unwrap_or(0) as i64;
    let collection_name = user_id.to_string();
    info!("collection_name: {}", collection_name);
    let text_to_save = msg.text().unwrap_or_default().to_string();
    info!("text_to_save: {}", text_to_save);
    let vector = vectorize(text_to_save.clone()).await?;

    let collection_exists = client.collection_exists(&collection_name).await?;
    info!("Exists? {}", collection_exists);

    if !collection_exists {
        let details = CreateCollection {
            collection_name: collection_name.clone(),
            vectors_config: Some(VectorsConfig {
                config: Some(Config::Params(VectorParams {
                    size: vector.len() as u64,
                    distance: Distance::Cosine.into(),
                    ..Default::default()
                })),
            }),
            ..Default::default()
        };

        client.create_collection(details).await?;
    }

    let current_point_count = client
        .collection_info(&collection_name)
        .await?
        .result
        .unwrap_or_default()
        .points_count
        .unwrap_or(0);

    let actual_id = current_point_count + 1;

    let payload: Payload = json!({"text": text_to_save}).try_into().unwrap();
    info!("Payload: {:?}", payload);
    let point = PointStruct::new(actual_id, vector, payload);
    let points = vec![point];

    client
        .upsert_points(UpsertPointsBuilder::new(collection_name, points))
        .await?;

    Ok(())
}

pub fn tokenize_and_truncate(data: String) -> Result<String> {
    let bpe = cl100k_base().unwrap();

    let tokens = bpe.encode_ordinary(&*data);
    info!("Input tokens quantity: {:?}", tokens.len());

    if tokens.len() > 10000 {
        let truncated_tokens = tokens[..10000].to_vec();

        let truncated_data = bpe.decode(truncated_tokens)?;

        // Checking truncated data token quantity
        let truncated_text_tokens = bpe.encode_ordinary(&*truncated_data);
        info!(
            "Truncated input tokens quantity: {:?}",
            truncated_text_tokens.len()
        );

        Ok(truncated_data)
    } else {
        info!("Input tokens quantity < 10000, no need to truncate");
        Ok(data.to_string())
    }
}

pub(crate) async fn text_to_speech(bot: &Bot, msg: &Message, text: String) -> Result<PathBuf> {
    let client = Client::new();
    let request = CreateSpeechRequestArgs::default()
        .input(&text)
        .voice(Voice::Onyx)
        .model(SpeechModel::Tts1)
        .speed(1.25)
        .build()?;
    bot.send_chat_action(msg.chat.id, ChatAction::RecordVoice)
        .await?;
    let response = client.audio().speech(request).await?;
    bot.send_chat_action(msg.chat.id, ChatAction::RecordVoice)
        .await?;
    let audio_file_path = "./operation_data/audio_response.mp3";
    response.save(audio_file_path).await?;
    Ok(PathBuf::from(audio_file_path))
}

pub(crate) async fn llm_processing_with_rag(
    msg: Message,
    system_role: String,
    request: String,
) -> Result<String> {
    let client = Client::new();

    let vectorized_request = vectorize(request.clone()).await?;

    let qdrant_data = qdrant_search(msg, vectorized_request).await?;

    let llm_user_message = format!(
        "Запрос пользователя: {}\nПолезная информация для формирования ответа: {}",
        request, qdrant_data
    );
    info!("llm_user_message: {:?}", llm_user_message);

    let llm_request = CreateChatCompletionRequestArgs::default()
        .max_tokens(2048u32)
        .model("gpt-3.5-turbo")
        .temperature(0.3)
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_role.as_str())
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(llm_user_message)
                .build()?
                .into(),
        ])
        .build()?;

    let response = client.chat().create(llm_request).await?;

    if let Some(choice) = response.choices.get(0) {
        let content = choice.message.content.clone().unwrap_or_else(|| {
            "Извини, я не смог понять твой вопрос. Пожалуйста, попробуй снова.".to_string()
        });
        Ok(content)
    } else {
        Ok("Извини, я не смог понять твой вопрос. Пожалуйста, попробуй снова.".to_string())
    }
}

pub(crate) async fn llm_processing_creation_utility(
    system_role: String,
    request: String,
) -> Result<String> {
    let client = Client::new();

    let llm_request = CreateChatCompletionRequestArgs::default()
        .max_tokens(2048u32)
        .model("gpt-4o")
        .temperature(0.4)
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_role.as_str())
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(request)
                .build()?
                .into(),
        ])
        .build()?;

    let response = client.chat().create(llm_request).await?;

    if let Some(choice) = response.choices.get(0) {
        let content = choice.message.content.clone().unwrap_or_else(|| {
            "Извини, я не смог понять твой вопрос. Пожалуйста, попробуй снова.".to_string()
        });
        Ok(content)
    } else {
        Ok("Извини, я не смог понять твой вопрос. Пожалуйста, попробуй снова.".to_string())
    }
}


pub async fn save_data_for_local_use (data: String, msg: Message) -> anyhow::Result<()> {
    let user_id = msg.from().map(|user| user.id.0).unwrap_or(0) as i64;
    let user_folder_name = user_id.to_string();

    let user_folder_path = format!("resources/education_resources/{}", user_folder_name);
    fs::create_dir_all(&user_folder_path)?;
    
    let system_role = fs::read_to_string("resources/creation_resources/preprocessing_data_for_local_db.txt")
        .map_err(|e| format!("Failed to read 'start_message': {}", e))
        .unwrap();
    let result = llm_processing_creation_utility(system_role, data).await?;
    
    info!("Saved to local db: {}", result);

    let file_count = fs::read_dir(&user_folder_path)?.count();
    let next_file_number = file_count + 1;
    let file_path = format!("{}/{}.txt", user_folder_path, next_file_number);

    let mut file = OpenOptions::new().create(true).write(true).open(&file_path)?;
    file.write_all(result.as_bytes())?;
    
    Ok (())
}