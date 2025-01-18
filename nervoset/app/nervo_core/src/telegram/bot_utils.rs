use std::collections::HashSet;
use crate::ai::nervo_llm::NervoLlm;
use crate::config::jarvis::JarvisAppState;
use crate::models::message_transcription_type::MessageTranscriptionType;
use crate::models::nervo_message_model::TelegramMessage;
use crate::models::qdrant_search_layers::QdrantSearchLayer;
use crate::models::system_messages::SystemMessage;
use crate::models::typing_action_model::TypingActionType;
use crate::models::user_model::TelegramUser;
use crate::telegram::message_parser::MessageParser;
use crate::utils::ai_utils::{
    filter_search_result, formation_system_role_llm_message, llm_conversation, update_search_content
};
use anyhow::bail;
use chrono::Utc;
use nervo_sdk::agent_type::{AgentType};
use nervo_sdk::api::spec::{LlmChat, LlmMessageContent, SendMessageRequest, UserLlmMessage};
use openai_dive::v1::api::Client;
use openai_dive::v1::resources::audio::{
    AudioSpeechParameters, AudioSpeechResponseFormat, AudioVoice,
};
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use async_openai::types::Embedding;
use qdrant_client::qdrant::{ScoredPoint};
use qdrant_client::qdrant::vectors::VectorsOptions;
use teloxide::prelude::ChatId;
use teloxide::prelude::*;
use teloxide::types::{
    ChatAction, ChatKind, InlineKeyboardButton, InlineKeyboardMarkup, InputFile, MediaKind,
    MessageId, MessageKind, ParseMode, ReplyParameters,
};
use teloxide::Bot;
use tokio::sync::{Mutex, RwLock};
use tokio::time::sleep;
use tracing::info;
use crate::utils::ai_utils_data::{TruncatingType};
use crate::utils::ai_utils_data::SortingType::{Ascending};

static LAST_MESSAGE_ID: LazyLock<Arc<Mutex<Option<MessageId>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(None)));
static SHOULD_STOP: LazyLock<Arc<RwLock<Option<TypingActionType>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

pub async fn start_conversation<'a>(
    app_state: Arc<JarvisAppState>,
    bot: &Bot,
    user_id: u64,
    msg: &Message,
    bot_name: String,
    agent_type: AgentType,
    mut parser: MessageParser<'a>,
) -> anyhow::Result<()> {
    info!("Start conversation");
    // We need it for future. Just to send spam etc.
    save_user_id(app_state.clone(), user_id.to_string()).await?;

    let message_text = parser.parse_tg_message_content().await?;

    let should_answer_as_reply =
        should_answer_as_reply(&msg, bot_name, message_text.clone()).await?;
    
    // Answer formation
    if should_answer_as_reply {
        {
            if let Some(feature_toggle) = &app_state.feature_toggle {
                if feature_toggle.localization {
                    {
                        let mut loc_manager = app_state.localisation_manager.write().await;
                        loc_manager.detect_language(message_text.as_str()).await?;
                    }
                }
            }
        }

        reply_to_user_message(
                app_state,
                &bot,
                &msg,
                user_id,
                message_text,
                agent_type,
                parser,
            )
                .await?;
    }
    Ok(())
}

async fn reply_to_user_message<'a>(
    app_state: Arc<JarvisAppState>,
    bot: &Bot,
    msg: &Message,
    user_id: u64,
    message_text: String,
    agent_type: AgentType,
    parser: MessageParser<'a>,
) -> anyhow::Result<()> {
    if !parser.is_tg_message_text().await? {
        system_message(
            app_state.clone(),
            &bot,
            &msg,
            SystemMessage::WaitSecond(agent_type),
        )
        .await?;
    }

    start_typing_action(&bot, &msg, ChatAction::Typing).await;

    if message_text.is_empty() {
        info!("Empty message");
        system_message(
            app_state,
            &bot,
            &msg,
            SystemMessage::EmptyMessage(agent_type),
        )
        .await?;
        return Ok(());
    }

    // Moderation checking
    let is_moderation_passed = app_state.nervo_llm.moderate(&message_text).await?;
    let question_msg = create_question_message(
        app_state.clone(),
        is_moderation_passed,
        user_id,
        message_text,
        msg.chat.id.0 as u64,
        agent_type,
    )
    .await?;

    chat_gpt_conversation(
        &bot,
        &msg,
        app_state,
        question_msg,
        parser.is_voice,
        !is_moderation_passed,
        agent_type,
    )
    .await?;

    Ok(())
}

async fn start_typing_action(bot: &Bot, msg: &Message, action_type: ChatAction) {
    tokio::spawn({
        let bot = bot.clone();
        let msg = msg.clone();
        async move {
            set_typing_action(TypingActionType::Acting).await;

            while let Some(TypingActionType::Acting) = get_typing_action().await {
                info!("Show typing action...");
                bot.send_chat_action(msg.chat.id.clone(), action_type)
                    .await
                    .ok();
                sleep(Duration::from_secs(3)).await;
            }

            info!("Stopped typing action");
        }
    });
}

async fn stop_typing_action() {
    info!("Stopping typing action...");
    set_typing_action(TypingActionType::Stopped).await;
}
pub async fn set_typing_action(action: TypingActionType) {
    let mut typing_action = SHOULD_STOP.write().await;
    *typing_action = Some(action);
}

pub async fn get_typing_action() -> Option<TypingActionType> {
    let typing_action = SHOULD_STOP.read().await;
    typing_action.clone()
}

async fn should_answer_as_reply<'a>(
    msg: &Message,
    bot_name: String,
    message_text: String,
) -> anyhow::Result<bool> {
    let is_forwarding = msg.forward_date().is_some();
    info!("is_forwarding {}", is_forwarding);
    
    info!("Do we need to answer this message?");
    // TODO:! We need to discus, do we need to use KEVIN in PUBLIC chats. If not, we need to return false
    // Need to detect it in group chats. To understand whether to answer or not.
    let is_reply = msg
        .clone()
        .reply_to_message()
        .and_then(|message| message.from.as_ref())
        .and_then(|user| user.username.clone())
        .map_or(false, |username| username == bot_name.clone())
        && !is_forwarding;

    let MessageKind::Common(msg_common) = &msg.kind else {
        bail!("Unsupported message content type: {:?}.", msg.kind);
    };

    let should_reply = match &msg.chat.kind {
        ChatKind::Private(_) => true,
        ChatKind::Public(_) => {
            let is_text = matches!(&msg_common.media_kind, MediaKind::Text(_));
            is_text && message_text.contains(&bot_name)
        }
    };

    info!("Should answer as reply: {:?}", should_reply || is_reply);
    Ok(should_reply || is_reply)
}

async fn create_question_message(
    app_state: Arc<JarvisAppState>,
    is_moderation_passed: bool,
    user_id: u64,
    message_text: String,
    chat_id: u64,
    agent_type: AgentType,
) -> anyhow::Result<SendMessageRequest> {
    let string_for_question: LlmMessageContent = if is_moderation_passed {
        let tg_message = TelegramMessage {
            id: user_id,
            message: message_text,
            timestamp: Utc::now().naive_utc(),
        };
        LlmMessageContent::from(tg_message.message.as_str())
    } else {
        let not_moderated_answer =
            create_not_moderated_message(message_text, &app_state.nervo_llm).await?;
        LlmMessageContent::from(not_moderated_answer.as_str())
    };

    // Create question for LLM
    let question_msg = SendMessageRequest {
        chat_id,
        agent_type,
        llm_message: UserLlmMessage {
            sender_id: user_id,
            content: string_for_question,
        },
    };
    info!("Prepared Message Request To LLM");
    Ok(question_msg)
}

// Sending some system messages
pub async fn system_message(
    app_state: Arc<JarvisAppState>,
    bot: &Bot,
    msg: &Message,
    message_type: SystemMessage,
) -> anyhow::Result<()> {
    let introduction_msg = message_type.as_str().await?;
    let reply_parameters = ReplyParameters {
        message_id: msg.id,
        chat_id: None,
        allow_sending_without_reply: None,
        quote: None,
        quote_parse_mode: None,
        quote_entities: None,
        quote_position: None,
    };

    let mut translated_text = String::new();
    {
        let loc_manager = app_state.localisation_manager.read().await;
        translated_text = loc_manager.translate(introduction_msg.as_str()).await?;
    }

    info!("Send system message");
    bot.send_message(msg.chat.id, translated_text)
        .reply_parameters(reply_parameters)
        .await?;

    Ok(())
}

// Work with User Ids
async fn save_user_id(app_state: Arc<JarvisAppState>, user_id: String) -> anyhow::Result<()> {
    let user_ids = load_user_ids(app_state.clone()).await?;

    let contains_id = user_ids.iter().any(|user| user.id == user_id);
    if !contains_id {
        let user = TelegramUser {
            id: user_id.parse()?,
        };
        app_state
            .local_db
            .save_to_local_db(user, "all_users_list", None)
            .await?;
    }
    Ok(())
}

async fn load_user_ids(app_state: Arc<JarvisAppState>) -> anyhow::Result<Vec<TelegramUser>> {
    match app_state
        .local_db
        .read_from_local_db("all_users_list")
        .await
    {
        Ok(ids) => Ok(ids),
        Err(_) => Ok(Vec::new()),
    }
}

pub async fn chat_gpt_conversation<'a>(
    bot: &Bot,
    message: &Message,
    app_state: Arc<JarvisAppState>,
    msg: SendMessageRequest,
    is_voice: bool,
    direct_message: bool,
    agent_type: AgentType,
) -> anyhow::Result<()> {
    info!("Start chat gpt conversation");
    let chat_id = msg.chat_id;

    let final_response = if direct_message {
        info!(
            "Direct message without any LLM handling {}",
            msg.llm_message.content.text()
        );
        msg.llm_message.content.text()
    } else {
        info!("Need to pass few layers of RAG System");
        match agent_type {
            AgentType::Kevin => {
                app_state.clone().user_context.use_memory_in_conversation(
                    &message,
                    app_state.clone(),
                    agent_type,
                ).await?
            }
            _ => {
                llm_conversation(app_state.clone(), msg, agent_type)
                    .await?
                    .content
                    .text()
            }
        }
    };
    
    translate_and_send_response(
        app_state.clone(),
        final_response.as_str(), 
        is_voice,
        &bot,
        chat_id,
        &message
    ).await?;

    Ok(())
}

async fn translate_and_send_response(
    app_state: Arc<JarvisAppState>,
    final_response: &str,
    is_voice: bool,
    bot: &Bot,
    chat_id: u64,
    message: &Message
) -> anyhow::Result<()> {

    let mut translated_text = String::new();
    {
        let loc_manager = app_state.localisation_manager.read().await;
        translated_text = loc_manager.translate(final_response).await?;
    }

    info!("Stop typing!");
    stop_typing_action().await;
    let keyboard = button_creation(is_voice).await?;
    let message_id = if is_voice {
        handle_voice_message(&bot, translated_text, chat_id, app_state, keyboard).await?
    } else {
        handle_text_message(&bot, translated_text, chat_id, message, keyboard).await?
    };

    switch_button_to_message(&bot, chat_id, Some(message_id)).await?;
    Ok(())
}

async fn switch_button_to_message(
    bot: &Bot,
    chat_id: u64,
    message_id: Option<MessageId>,
) -> anyhow::Result<()> {
    remove_last_message_button(&bot, ChatId(chat_id as i64)).await?;
    let mut last_msg_lock = LAST_MESSAGE_ID.lock().await;
    *last_msg_lock = match message_id {
        None => None,
        Some(_) => message_id,
    };
    Ok(())
}

async fn handle_voice_message(
    bot: &Bot,
    user_final_question: String,
    chat_id: u64,
    app_state: Arc<JarvisAppState>,
    keyboard: InlineKeyboardMarkup,
) -> anyhow::Result<MessageId> {
    info!("Handle Voice TG message");
    let voice_input = create_speech(user_final_question.as_str(), app_state).await?;
    let sent_message = bot
        .send_voice(ChatId(chat_id as i64), voice_input)
        .reply_markup(keyboard)
        .await?;

    info!("Successfully sent voice answer to user");
    Ok(sent_message.id)
}

async fn handle_text_message(
    bot: &Bot,
    user_final_question: String,
    chat_id: u64,
    message: &Message,
    keyboard: InlineKeyboardMarkup,
) -> anyhow::Result<MessageId> {
    info!("Handle Text TG message");
    let reply_parameters = ReplyParameters {
        message_id: message.id,
        chat_id: None,
        allow_sending_without_reply: None,
        quote: None,
        quote_parse_mode: None,
        quote_entities: None,
        quote_position: None,
    };
    let escaped_message = escape_markdown(&user_final_question);

    let sent_message = bot
        .send_message(ChatId(chat_id as i64), escaped_message)
        .reply_markup(keyboard)
        .parse_mode(ParseMode::MarkdownV2)
        .reply_parameters(reply_parameters)
        .await?;
    info!("Successfully sent text answer to user");
    Ok(sent_message.id)
}

async fn remove_last_message_button<'a>(bot: &Bot, chat_id: ChatId) -> anyhow::Result<()> {
    info!("Need to remove last message button");
    let last_msg_lock = LAST_MESSAGE_ID.lock().await;
    if let Some(last_msg_id) = *last_msg_lock {
        bot.edit_message_reply_markup(chat_id, last_msg_id)
            .reply_markup(InlineKeyboardMarkup::new(
                Vec::<Vec<InlineKeyboardButton>>::new(),
            ))
            .await?;
    }
    info!("Button has been removed successfully");
    Ok(())
}

async fn create_speech(text: &str, app_state: Arc<JarvisAppState>) -> anyhow::Result<InputFile> {
    let client = Client::new(app_state.nervo_llm.api_key().to_string());
    let parameters = AudioSpeechParameters {
        model: "tts-1".to_string(),
        input: text.to_string(),
        voice: AudioVoice::Onyx,
        response_format: Some(AudioSpeechResponseFormat::Mp3),
        speed: Some(1.0),
    };
    let response = client.audio().create_speech(parameters).await;

    match response {
        Ok(audio) => Ok(InputFile::memory(audio.bytes)),
        Err(err) => bail!("ERROR: {:?}", err),
    }
}

pub async fn transcribe_message(
    app_state: Arc<JarvisAppState>,
    bot: &Bot,
    message: &Message,
    transcription_type: MessageTranscriptionType,
) -> anyhow::Result<()> {
    info!("Transcribe message TTS or STT");
    let mut parser = MessageParser {
        bot: &bot,
        msg: &message,
        app_state: &app_state,
        is_voice: false,
    };

    match transcription_type {
        MessageTranscriptionType::Tts => {
            start_typing_action(&bot, &message, ChatAction::RecordVoice).await
        }
        MessageTranscriptionType::Stt => {
            start_typing_action(&bot, &message, ChatAction::Typing).await
        }
    }

    let chat_id = message.chat.id;
    let parsed_voice_to_text = parser.parse_tg_message_content().await?;

    match transcription_type {
        MessageTranscriptionType::Tts => {
            info!("Transcription type TTS");
            let audio_file = create_speech(parsed_voice_to_text.as_str(), app_state).await?;
            info!("Audio from Text has been created");
            bot.send_voice(chat_id, audio_file).await?;
        }
        MessageTranscriptionType::Stt => {
            info!("Transcription type STT");
            bot.send_message(chat_id, parsed_voice_to_text).await?;
        }
    }

    switch_button_to_message(&bot, chat_id.0 as u64, None).await?;
    stop_typing_action().await;
    info!("Transcription is OK");
    Ok(())
}

fn escape_markdown(text: &str) -> String {
    text.replace('_', "\\_")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace('~', "\\~")
        .replace('>', "\\>")
        .replace('#', "\\#")
        .replace('+', "\\+")
        .replace('-', "\\-")
        .replace('=', "\\=")
        .replace('|', "\\|")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('.', "\\.")
        .replace('!', "\\!")
}

async fn button_creation(is_voice: bool) -> anyhow::Result<InlineKeyboardMarkup> {
    let button_title = if is_voice {
        "Прочитать текстом"
    } else {
        "Озвучить голосом"
    };
    let button_action = if is_voice {
        MessageTranscriptionType::Stt
    } else {
        MessageTranscriptionType::Tts
    };

    Ok(InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback(button_title.to_string(), button_action.as_str()),
    ]]))
}

async fn create_not_moderated_message(text: String, nervo_llm: &NervoLlm) -> anyhow::Result<String> {
    let system_role_instructions = format!("I have a message from the user, I know the message is unacceptable, can you please read the message and reply that the message is not acceptable. Here is the message: {:?}", text);
    let language_detecting_layer = QdrantSearchLayer {
        index: None,
        user_role_params: vec![],
        system_role_text: system_role_instructions.to_string(),
        temperature: 0.2,
        max_tokens: 4096,
        common_token_limit: 30000,
        vectors_limit: 0,
        layer_for_search: false,
    };

    let system_role_msg = formation_system_role_llm_message(language_detecting_layer).await?;

    info!(
        "Full not moderated text role message: {}",
        system_role_msg.content.0
    );
    let chat: LlmChat = LlmChat {
        chat_id: None,
        messages: vec![system_role_msg],
    };
    let llm_response = nervo_llm.send_msg_batch(chat).await?;
    Ok(llm_response)
}

pub async fn get_message_related_points(
    message: &str,
    collection_name: &str,
    system_role_to_clear_request: &str,
    app_state: Arc<JarvisAppState>
) -> anyhow::Result<Vec<String>> {
    
    let clear_user_request = app_state.nervo_llm.raw_llm_processing(
        system_role_to_clear_request,
        &message
    ).await?;
    
    let embeddings_response = app_state.nervo_llm.embedding(clear_user_request.as_str()).await?;
    let embeddings = embeddings_response.data.into_iter().next().unwrap(); // TODO: NO FUC**N UNWRAPS

    let unique_points_response = search_unique_points(
        &collection_name,
        embeddings,
        app_state.clone()
    ).await?;
    
    let payload = get_payload(unique_points_response)?;
    
    Ok(payload)
}

async fn search_unique_points(
    collection_name: &str,
    embedding: Embedding,
    app_state: Arc<JarvisAppState>
) -> anyhow::Result<Vec<ScoredPoint>> {
    let vector_search_response_of_user_request = app_state.nervo_ai_db.qdrant.vector_search(
        &collection_name,
        embedding.embedding,
        3 // TODO: Read from (where??)
    ).await?;
    
    let search_result = vector_search_response_of_user_request.result;
    let filtered_search_result = filter_search_result(
        search_result,
        Ascending,
        TruncatingType::None,
        0.3
    )?;
    
    let mut final_search_result: Vec<ScoredPoint> = filtered_search_result.clone();
    
    for point in filtered_search_result {
        if let Some(vector) = point.vectors {
            if let Some(VectorsOptions::Vector(vec_f32)) = vector.vectors_options {
                let result_vector: Vec<f32> = vec_f32.clone().data;
                let vector_search_response_of_point = app_state.nervo_ai_db.qdrant.vector_search(
                    &collection_name,
                    result_vector,
                    3 // TODO: Read from (where??)
                ).await?;
                let search_result_of_point = vector_search_response_of_point.result;
                let mut existing_ids: HashSet<_> = final_search_result.iter().map(|point| format!("{:?}", point.id)).collect();

                for point in search_result_of_point {
                    if existing_ids.insert(format!("{:?}", point.id)) {
                        final_search_result.push(point);
                    }
                }
            }
        }
    }

    Ok(final_search_result)
}

pub fn get_payload(
    points: Vec<ScoredPoint>
) -> anyhow::Result<Vec<String>> {
    let mut combined_payloads = Vec::new();
    for point in points {
        if let Some(payload) = point.payload.get("text") {
            if let Some(text) = payload.as_str() {
                combined_payloads.push(text.to_string());
            }
        }
    }
    
    Ok(combined_payloads)
}

// pub fn get_payload_string(
//     combined_payloads: Vec<String>,
//     token_limits: Option<i64>,
//     join_symbol: &str
// ) -> anyhow::Result<String> {
//     if combined_payloads.is_empty() {
//             info!("No data found.");
//             Ok("No data found.".to_string())
//         } else {
//             let mut combined_payload = combined_payloads.join(join_symbol);
//             if let Some(token_limits) = token_limits {
//                 combined_payload = update_search_content(token_limits as usize, combined_payload)?;
//             }
//             info!("Combined_payload: {:?}", combined_payload);
//             Ok(combined_payload)
//         }
// }

#[cfg(test)]
mod test {
    use crate::telegram::bot_utils::{button_creation, escape_markdown};

    #[tokio::test]
    async fn test_button_creation_is_voice() -> anyhow::Result<()> {
        let keyboard = button_creation(true).await?;
        let button = keyboard.inline_keyboard.first().unwrap().first();
        assert_eq!(button.unwrap().text, String::from("Прочитать текстом"));
        Ok(())
    }

    #[tokio::test]
    async fn test_button_creation_not_voice() -> anyhow::Result<()> {
        let keyboard = button_creation(false).await?;
        let button = keyboard.inline_keyboard.first().unwrap().first();
        assert_eq!(button.unwrap().text, String::from("Озвучить голосом"));
        Ok(())
    }

    #[test]
    fn test_escape_markdown() {
        let input_string = "_ [ ] ( ) ~ > # + - = | { } . !";
        let output_string =
            String::from("\\_ \\[ \\] \\( \\) \\~ \\> \\# \\+ \\- \\= \\| \\{ \\} \\. \\!");
        let input_escaped_string = escape_markdown(input_string);
        assert_eq!(input_escaped_string, output_string);
    }
}
