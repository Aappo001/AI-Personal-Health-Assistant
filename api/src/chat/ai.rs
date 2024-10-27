use axum::{
    extract::State,
    response::{IntoResponse, Json, Response},
};
use dotenvy::var;
use futures::StreamExt;
use reqwest::{header, StatusCode};
use reqwest_streams::*;
use serde::Serialize;
use serde_json::json;
use sqlx::SqlitePool;

use crate::{chat::ChatMessage, error::AppError, AppState};

use super::{broadcast_event, SendMessage, SocketResponse};

/// Stream data from the AI model
// Might add a field for whether the message should trigger the AI
// Does not have an id because it is not yet saved in the database
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StreamMessage {
    pub conversation_id: i64,
    /// The content of the current message
    // You must combine consecutive messages from the same role into a single message
    pub message: String,
}

/// An AI model that can be used to generate responses
#[derive(Serialize)]
pub struct AiModel {
    pub id: i64,
    pub name: String,
}

/// Query the AI model with the messages in the conversation
pub async fn query_model(state: &AppState, message: &SendMessage) -> Result<ChatMessage, AppError> {
    let model_id = message.ai_model_id.expect("Model ID should be provided");
    let conversation_id = message
        .conversation_id
        .expect("Conversation ID should be provided");
    let model = sqlx::query!("SELECT name FROM ai_models WHERE id = ?", model_id)
        .fetch_one(&state.pool)
        .await?;
    // Build the default request body for the AI model
    let mut body = json!({
        "model": model.name,
        "messages": [
        { "role": "system", "content": "You are a medical professional who knows about medicine.  When the user tells you about a health problem that they are facing, continue probing through the problem to extract more information and attempt to gain a better understanding of a root cause and potential remedies. Do not simply give a list of potential causes without asking further questions. If you are unsure about something refer user to a doctor or medical professional." },
    ],
        "temperature": 0.5,
        "max_tokens": 1024,
        "top_p": 0.7,
    // Enable streaming so we can get the response as it comes in
        "stream": true
    });
    // Populate the messages array with the messages in the conversation
    if let Some(req_messages) = body["messages"].as_array_mut() {
        // Query the messages as a stream to save memory
        // This saves a tong on longer conversations
        let mut db_messages = sqlx::query!(
            "SELECT message, user_id, ai_model_id FROM messages WHERE conversation_id = ?",
            conversation_id
        )
        .fetch(&state.pool);

        // If we don't alternate between user and assistant messages, the AI will give us an error and
        // get stuck so we need to concatenate consecutive user and system messages together
        let mut last_role;
        let mut cur_role = "user";
        let mut cur_content = String::new();
        while let Some(message) = db_messages.next().await {
            let message = message?;
            last_role = cur_role;
            cur_role = if message.user_id.is_some() {
                "user"
            } else {
                "assistant"
            };
            if last_role != cur_role {
                req_messages.push(json!({
                    "role": last_role,
                    "content": cur_content
                }));
                cur_content.clear();
            }
            cur_content.push_str(&message.message);
        }
        if !cur_content.is_empty() {
            req_messages.push(json!({
            "role": cur_role,
            "content": cur_content
            }));
        }
    }

    let mut response = state
        .client
        .post(format!(
            "https://api-inference.huggingface.co/models/{}/v1/chat/completions",
            model.name
        ))
        .header(
            header::AUTHORIZATION,
            format!(
                "Bearer {}",
                var("HF_API_KEY").expect("Huggingface API key should be provided .env file as HF_API_KEY. Get one at https://huggingface.co/settings/tokens")
            ),
        )
        .json(&body)
        .send()
        .await
        .map_err(AppError::from)?
        .json_array_stream::<serde_json::Value>(2048);

    let mut res_content = String::new();
    while let Some(mut bytes) = response.next().await {
        match bytes {
            Ok(ref mut bytes) => {
                // Stream the individual messages to the client
                broadcast_event(
                    state,
                    SocketResponse::StreamData(StreamMessage {
                        conversation_id,
                        message: bytes["choices"][0]["delta"]["content"]
                            .as_str()
                            .unwrap_or("")
                            .to_string(),
                    }),
                )
                .await?;
                res_content += bytes["choices"][0]["delta"]["content"]
                    .as_str()
                    .unwrap_or("");
            }
            Err(e) => return Err(AppError::from(e)),
        }
    }
    // Save the final response to the database
    Ok(sqlx::query_as!(
        ChatMessage,
        "INSERT INTO messages (conversation_id, message, ai_model_id) VALUES (?, ?, ?) RETURNING *",
        conversation_id,
        res_content,
        model_id
    )
    .fetch_one(&state.pool)
    .await?)
}

/// Returns all the AI models in the database
pub async fn get_ai_models(State(pool): State<SqlitePool>) -> Result<Response, AppError> {
    Ok((
        StatusCode::OK,
        Json(
            sqlx::query_as!(AiModel, "SELECT * FROM ai_models")
                .fetch_all(&pool)
                .await
                .map_err(AppError::from)?,
        ),
    )
        .into_response())
}
