use crate::config::jarvis::JarvisAppState;
use crate::models::nervo_message_model::TelegramMessage;
use crate::models::system_messages::SystemMessages;
use crate::models::user_model::TelegramUser;
use crate::utils::ai_utils::{llm_conversation, RESOURCES_DIR};
use anyhow::{bail};
use chrono::Utc;
use nervo_sdk::agent_type::{AgentType, NervoAgentType};
use nervo_sdk::api::spec::{LlmMessageContent, SendMessageRequest, UserLlmMessage};
use openai_dive::v1::api::Client;
use openai_dive::v1::resources::audio::{
    AudioSpeechParameters, AudioSpeechResponseFormat,
    AudioVoice,
};
use std::sync::Arc;
use sqlx::encode::IsNull::No;
use teloxide::prelude::ChatId;
use teloxide::prelude::*;
use teloxide::types::{ChatKind, InlineKeyboardButton, InlineKeyboardMarkup, InputFile, MediaKind, MessageId, MessageKind, ParseMode, ReplyParameters};
use teloxide::{Bot, RequestError};
use tokio::fs;
use tracing::{info};
use crate::telegram::message_parser::MessageParser;

static mut LAST_MESSAGE_ID: Option<MessageId> = None;

pub async  fn start_conversation<'a>(
    app_state: &Arc<JarvisAppState>,
    bot: &Bot,
    user_id: u64,
    msg: &Message,
    bot_name: String,
    agent_type: AgentType,
    mut parser: MessageParser<'a>,
) -> anyhow::Result<()> {
    info!("COMMON: Start conversation");
    // We need it for future. Just to send spam etc.
    save_user_id(app_state.clone(), user_id.to_string()).await?;

    let message_text = match parser.parse_text(false).await {
        Ok(result) => {result}
        Err(err) => {
            print!("Errrrrrrrrr: {:?}", err);
            "text".to_string()
        }
    };
    let should_answer_as_reply = should_answer_as_reply(
        &msg, 
        bot_name,
        message_text.clone(),
    ).await?;
    
    // Answer formation
    if should_answer_as_reply {
        // Show typing indicator
        info!("COMMON: Show typing action...");
        bot.send_chat_action(msg.chat.id.clone(), teloxide::types::ChatAction::Typing).await?;
        
        let reply_parameters = ReplyParameters {
            message_id: msg.id,
            chat_id: None,
            allow_sending_without_reply: None,
            quote: None,
            quote_parse_mode: None,
            quote_entities: None,
            quote_position: None,
        };
        if message_text.is_empty() {
            info!("COMMON: Empty message");
            bot.send_message(msg.chat.id, "Please provide a message to send.")
                .reply_parameters(reply_parameters)
                .await?;
            return Ok(());
        }

        // Moderation checking
        info!("$5");
        let is_moderation_passed = app_state.nervo_llm.moderate(&message_text).await?;
        info!("$6");
        let question_msg = create_question_message(
            is_moderation_passed,
            user_id,
            message_text,
            msg.chat.id.0 as u64,
            agent_type
        ).await?;
        
        chat_gpt_conversation(
            &bot,
            &msg,
            app_state.clone(),
            question_msg,
            parser.is_voice,
            !is_moderation_passed,
            agent_type,
        ).await?;
        
        return Ok(());
    }
    Ok(())
}

pub async fn should_answer_as_reply<'a>(
    msg: &Message,
    bot_name: String, 
    message_text: String
) -> anyhow::Result<bool> {
    // Need to detect it in group chats. To understand whether to answer or not.
    let is_reply = msg
        .clone()
        .reply_to_message()
        .and_then(|message| message.from.as_ref())
        .and_then(|user| user.username.clone())
        .map_or(false, |username| username == bot_name.clone());

    let MessageKind::Common(msg_common) = &msg.kind else {
        bail!("COMMON: Unsupported message content type: {:?}.", msg.kind);
    };
    let is_text = matches!(&msg_common.media_kind, MediaKind::Text(_media_text));
    let is_private = matches!(&msg.chat.kind, ChatKind::Private(_));
    let is_public = matches!(&msg.chat.kind, ChatKind::Public(_));
    let is_public_and_text = is_public && is_text;

    // Parse message to raw text
    let contains_bot_name = message_text.contains(&bot_name);
    info!("COMMON: Should answer as reply: {:?}", (is_private || is_reply || (is_public_and_text && contains_bot_name)));
    Ok(is_private || is_reply || (is_public_and_text && contains_bot_name))
}

async fn create_question_message(
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
        let question = format!("I have a message from the user, I know the message is unacceptable, can you please read the message and reply that the message is not acceptable. Reply using the same language the massage uses. Here is the message: {:?}", &message_text);
        LlmMessageContent::from(question.as_str())
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
    info!("COMMON: Question message request was created");
    Ok(question_msg)
}

// Sending some system messages
pub async fn system_message(
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

    info!("COMMON: Send system message");
    bot.send_message(msg.chat.id, introduction_msg)
        .reply_parameters(reply_parameters)
        .await?;

    Ok(())
}

pub enum SystemMessage {
    Start(AgentType),
    Manual(AgentType),
}

impl SystemMessage {
    fn agent_type(&self) -> AgentType {
        match self {
            SystemMessage::Start(agent_type) => agent_type.clone(),
            SystemMessage::Manual(agent_type) => agent_type.clone(),
        }
    }

    pub async fn as_str(&self) -> anyhow::Result<String> {
        let agent = NervoAgentType::get_name(self.agent_type());
        let system_msg_file = format!("{}/agent/{}/system_messages.json", RESOURCES_DIR, agent);

        let json_string = fs::read_to_string(system_msg_file).await?;
        let system_messages_models: SystemMessages = serde_json::from_str(&json_string)?;

        match self {
            SystemMessage::Start(_) => Ok(system_messages_models.start.clone()),
            SystemMessage::Manual(_) => Ok(system_messages_models.manual.clone()),
        }
    }
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

pub async  fn chat_gpt_conversation<'a>(
    bot: &Bot,
    message: &Message,
    app_state: Arc<JarvisAppState>,
    msg: SendMessageRequest,
    is_voice: bool,
    direct_message: bool,
    agent_type: AgentType,
) -> anyhow::Result<()> {
    info!("COMMON: Start chat gpt conversation");
    let chat_id = msg.chat_id;
    let table_name = msg.llm_message.sender_id.to_string();

    let user_final_question = if direct_message {
        info!("Direct message");
        msg.llm_message.content.text()
    } else {
        info!("Conversationt message");
        llm_conversation(app_state.clone(), msg, table_name, agent_type)
            .await?
            .content
            .text()
    };

    let keyboard = button_creation(is_voice).await?;
    if is_voice {
        let voice_input = create_speech(
            bot, user_final_question.as_str(), chat_id, app_state
        ).await?;
        remove_last_message_button(&bot, ChatId(chat_id as i64)).await?;
        let sent_message = bot.send_voice(ChatId(chat_id as i64), voice_input)
            .reply_markup(keyboard)
            .await?;
        unsafe {
            LAST_MESSAGE_ID = Some(sent_message.id);
        }
    } else {
        let reply_parameters = ReplyParameters {
            message_id: message.id,
            chat_id: None,
            allow_sending_without_reply: None,
            quote: None,
            quote_parse_mode: None,
            quote_entities: None,
            quote_position: None,
        };

        info!("COMMON: Send ANSWER");
        info!("COMMON: user_final_question {:?}", user_final_question);
        let escaped_message = escape_markdown(&user_final_question);
        info!("COMMON: escaped_message {:?}", escaped_message);
        
        remove_last_message_button(&bot, ChatId(chat_id as i64)).await?;
        match bot.send_message(ChatId(chat_id as i64), escaped_message)
            .reply_markup(keyboard)
            .parse_mode(ParseMode::MarkdownV2)
            .reply_parameters(reply_parameters)
            .await {
            Ok(sent_message) => {
                unsafe {
                    LAST_MESSAGE_ID = Some(sent_message.id);
                }
                info!("COMMON: Message has been sent successfully");
            }
            Err(e) => {
                eprintln!("COMON: Ошибка при отправке сообщения: {:?}", e);
            }
        }
    }

    Ok(())
}

async fn remove_last_message_button<'a>(bot: &Bot, chat_id: ChatId) -> anyhow::Result<()> {
    info!("COMMON: LAST_MESSAGE_ID {:?}", unsafe {LAST_MESSAGE_ID} );
    if let  Some(last_msg_id) = unsafe {LAST_MESSAGE_ID} {
        match bot.edit_message_reply_markup(chat_id, last_msg_id)
            .reply_markup(InlineKeyboardMarkup::new(Vec::<Vec<InlineKeyboardButton>>::new()))
            .await {
            Ok(_) => {
                info!("COMMON: Reply markup has been removed successfully");
            }
            Err(e) => {
                eprintln!("COMMON: Ошибка при удалении клавиатуры: {:?}", e);
            }
        }
    }
    Ok(())
}

async fn create_speech(bot: &Bot, text: &str, chat_id: u64, app_state: Arc<JarvisAppState>) -> anyhow::Result<InputFile> {
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
        Ok(audio) => {
            Ok(InputFile::memory(audio.bytes))
        }
        Err(err) => {
            info!("COMMON: Send ERROR: {:?}", err);
            let _ = bot
                .send_message(ChatId(chat_id as i64), err.to_string())
                .await;
            bail!("COMMON: ERROR: {:?}", err);
        }
    }
}

pub async fn transcribe_message(app_state: Arc<JarvisAppState>, bot: &Bot, message: &Message, transcription_type: MessageTranscriptionType) -> anyhow::Result<()> {
    let mut parser = MessageParser {
        bot: &bot,
        msg: &message,
        app_state: &app_state,
        is_voice: false,
    };

    let chat_id = message.chat.id;
    let message_id = message.id;
    
    info!("COMMON: Getting text message in EDITING mode");
    let parsed_voice_to_text = parser.parse_text(true).await?;

    match transcription_type {
        MessageTranscriptionType::TTS => {
            info!("COMMON: Transcription type TTS");
            let audio_file = create_speech(
                bot, parsed_voice_to_text.as_str(), chat_id.0 as u64, app_state
            ).await?;
            info!("COMMON: Audio from Text has been created");
            remove_last_message_button(&bot, chat_id).await?;
            bot.send_voice(chat_id, audio_file).await?;
            
        }
        MessageTranscriptionType::STT => {
            info!("COMMON: Transcription type STT");
            remove_last_message_button(&bot, chat_id).await?;
            bot.send_message(chat_id,parsed_voice_to_text ).await?;
            unsafe {
                LAST_MESSAGE_ID = None;
            }
        }
    }

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
    let button_title = if is_voice { "Прочитать текстом" } else { "Озвучить голосом" };
    let button_action = if is_voice { MessageTranscriptionType::STT } else {  MessageTranscriptionType::TTS };

    Ok(InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback(button_title.to_string(), button_action.as_str()),
    ]]))
}

pub enum MessageTranscriptionType {
    TTS, // Text To Speach
    STT, // Speach To Text
}

impl MessageTranscriptionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageTranscriptionType::TTS => {"Text To Speach"}
            MessageTranscriptionType::STT => {"Speach To Text"}
        }
    }
}