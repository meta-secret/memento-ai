use std::fs::File;
use std::io::Read;
use std::sync::Arc;

use async_openai::types::{
    ChatCompletionRequestMessage,
    ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent,
    Role
};
use axum::{
    http::StatusCode,
    Json,
    Router, routing::{get, post},
};
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
use nervo_bot_core::ai::ai_db::NervoAiDb;

use nervo_bot_core::ai::nervo_llm::NervoLlm;
use nervo_bot_core::common::{AppState, NervoConfig};
use nervo_bot_core::db::local_db::LocalDb;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();
    
    let nervo_config = NervoConfig::load()?;

    let nervo_llm = NervoLlm::from(nervo_config.llm.clone());

    let nervo_ai_db = NervoAiDb::try_from(&nervo_config.qdrant)?;

    let local_db = LocalDb::try_init(nervo_config.database.clone()).await?;

    let app_state = Arc::from(AppState {
        nervo_llm,
        nervo_ai_db,
        local_db,
        nervo_config,
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/chat/:user_id", get(chat))
        .route("/users", post(create_user))
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await?;
    
    Ok(())
}


// basic handler that responds with a static string
async fn root() -> &'static str {
    "Nervo!!!"
}

async fn chat(
    Path(user_id): Path<String>, 
    State(state): State<Arc<AppState>>
) -> Result<Json<Chat>, StatusCode> {
    // OpenAI interacting
    let message = ChatCompletionRequestUserMessage {
        content: ChatCompletionRequestUserMessageContent::Text(String::from("Привет, похвали меня за выполненную работу!")),
        role: Role::User,
        name: None,
    };
    
    let message_text = state
        .nervo_llm.chat_batch(vec![ChatCompletionRequestMessage::from(message)])
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .unwrap_or_else(|| String::from("Can't talk, too busy now..."));
    
    // Формируем объект сообщения
    let chat = Chat {
        user: user_id,
        messages: vec![Message { role: "user".to_string(), content: message_text }],
    };

    Ok(Json(chat))
}

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Serialize, Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}

#[derive(Serialize, Deserialize)]
struct ChatParams {
    user_id: String,
}

#[derive(Serialize, Deserialize)]
struct Chat {
    user: String,
    messages: Vec<Message>,
}

// #[derive(Serialize, Deserialize)]
// struct Message {
//     role: String,
//     content: String,
// }

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}