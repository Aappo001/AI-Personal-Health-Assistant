use axum::{
    extract::{
        Path, State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::{FromRow, Type}, Decode, SqlitePool};

use crate::{
    auth::JwtAuth,
    error::AppError,
};
use crate::{
    error::AppJson,
    users::UserToken,
};

use super::SendMessage;

/// A conversation between at least one user and an AI
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    /// The id of the conversation
    pub id: i64,
    /// The title of the conversation
    /// If this is None, the frontend should fallback to listing the
    /// usernames of the users in the conversation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub created_at: NaiveDateTime,
    pub last_message_at: NaiveDateTime,
}

/// Get all the conversations the user is in
pub async fn get_user_conversations(
    State(pool): State<SqlitePool>,
    JwtAuth(user): JwtAuth<UserToken>,
) -> Result<Response, AppError> {
    let res = &sqlx::query_as!(
        Conversation,
        r#"SELECT id, title, created_at, conversations.last_message_at FROM conversations
            JOIN user_conversations ON user_conversations.conversation_id = conversations.id
            WHERE user_conversations.user_id = ? 
            ORDER BY conversations.last_message_at DESC"#,
        user.id,
    )
    .fetch_all(&pool)
    .await?;
    Ok((StatusCode::OK, AppJson(res)).into_response())
}

/// Create a conversation between the user and the AI from an initial message
/// Initiated from a POST request
pub async fn create_conversation_rest(
    State(pool): State<SqlitePool>,
    JwtAuth(user): JwtAuth<UserToken>,
    AppJson(init_message): AppJson<SendMessage>,
) -> Result<Response, AppError> {
    // Limit the title to 32 characters
    // let title = &init_message.message[..cmp::min(init_message.message.len(), 32)];
    Ok((
        StatusCode::OK,
        AppJson(create_conversation(&pool, &init_message, &user).await?),
    )
        .into_response())
}

// This is used in both the REST api and the websocket api so it is extracted into a function
/// Create a conversation between the user and the AI from an initial message
pub async fn create_conversation(
    pool: &SqlitePool,
    init_message: &SendMessage,
    user: &UserToken,
) -> Result<Conversation, AppError> {
    let title = init_message.message.chars().take(32).collect::<String>();

    // Begin a transaction to ensure that both the conversation and the initial message are saved
    let mut tx = pool.begin().await?;
    // Create the conversation
    let conversation_id = sqlx::query!(
        "INSERT INTO conversations (title) VALUES (?) RETURNING id",
        title
    )
    .fetch_one(&mut *tx)
    .await?
    .id;
    // Add the user to the conversation
    sqlx::query!(
        "INSERT INTO user_conversations (user_id, conversation_id) VALUES (?, ?)",
        user.id,
        conversation_id
    )
    .execute(&mut *tx)
    .await?;

    // Everything went well, commit the transaction
    tx.commit().await?;

    // Return the new conversation for future messages
    Ok(sqlx::query_as!(
        Conversation,
        "SELECT id, title, created_at, last_message_at FROM conversations where id = ? ORDER BY last_message_at DESC",
        conversation_id,
    )
        .fetch_one(pool)
    .await?)
}

/// A message in a conversation
// Might add a field for whether the message should trigger the AI
#[derive(Serialize, Deserialize, Clone, Debug, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    /// The id of the message
    /// If this is None, the message has not been saved to the database yet
    pub id: i64,
    /// The id of the message
    /// If this is None, this is the first message in the conversation
    /// and a new conversation should be created
    pub conversation_id: i64,
    pub message: String,
    /// The id of the user who sent the message
    /// This will be none if the message was sent by the AI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    /// The id of the AI model that sent the message
    /// This will be none if the message was sent by a user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_model_id: Option<i64>,
    pub created_at: NaiveDateTime,
    pub modified_at: NaiveDateTime,
}

#[derive(Serialize, Debug, Clone)]
pub struct DeleteMessage {
    pub message_id: i64,
    pub conversation_id: i64,
}

/// Get all the messages in a conversation
pub async fn get_conversation(
    State(pool): State<SqlitePool>,
    JwtAuth(user): JwtAuth<UserToken>,
    Path(conversation_id): Path<i64>,
) -> Result<Response, AppError> {
    if sqlx::query!(
        r#"SELECT id FROM conversations
            JOIN user_conversations ON user_conversations.conversation_id = conversations.id
            WHERE id = ? and user_id = ?"#,
        conversation_id,
        user.id
    )
    .fetch_optional(&pool)
    .await?
    .is_none()
    {
        return Ok((StatusCode::NOT_FOUND, "Conversation not found").into_response());
    }
    let res = &sqlx::query_as!(
            ChatMessage,
            r#"SELECT messages.id, message, messages.created_at, modified_at, conversation_id, user_id, ai_model_id, file_name, files.path as file_path FROM messages
            JOIN conversations ON conversations.id = messages.conversation_id 
            LEFT JOIN files ON files.id = messages.file_id
            WHERE conversations.id = ? 
            ORDER BY last_message_at DESC"#,
            conversation_id,
        )
        .fetch_all(&pool)
        .await?;
    Ok((StatusCode::OK, AppJson(res)).into_response())
}

/// A read receipt for a conversation
/// Every message sent before this message is assumed to have been read by the user
/// Sent to the client, but not received from the client so they can't lie about timestamps and
/// do weird shinanigans like reading messages in the future
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReadEvent {
    /// The id of the conversation that was read
    pub conversation_id: i64,
    /// The id of the user who read the conversation
    pub user_id: i64,
    /// The timestamp when the conversation was last read
    pub timestamp: NaiveDateTime,
}
