use std::cmp;

use anyhow::anyhow;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::{header::AUTHORIZATION, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose, Engine};
use chrono::NaiveDateTime;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, SqlitePool};
use tokio::sync::broadcast;

use crate::{
    users::{authorize_user, UserToken},
    AppError, AppState,
};

#[derive(Serialize, Deserialize, FromRow)]
pub struct Conversation {
    pub id: i64,
    pub title: String,
    pub created_at: NaiveDateTime,
    pub last_message_at: NaiveDateTime,
}

pub async fn get_user_conversations(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let user = match authorize_user(&headers) {
        Ok(k) => k,
        Err(e) => return Ok((StatusCode::UNAUTHORIZED, e.to_string()).into_response()),
    };
    let res = 
        &sqlx::query_as!(
            Conversation,
            r#"SELECT id, title, created_at, conversations.last_message_at FROM conversations
            JOIN user_conversations ON user_conversations.conversation_id = conversations.id
            WHERE user_conversations.user_id = ? 
            ORDER BY conversations.last_message_at DESC"#,
            user.id,
        )
        .fetch_all(&pool)
        .await?;
    Ok((StatusCode::OK, Json(res)).into_response())
}

pub async fn create_conversation(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Json(init_message): Json<ChatMessage>,
) -> Result<Response, AppError> {
    let user = match authorize_user(&headers) {
        Ok(k) => k,
        Err(e) => return Ok((StatusCode::UNAUTHORIZED, e.to_string()).into_response()),
    };
    // Limit the title to 32 characters
    let title = &init_message.message[..cmp::min(init_message.message.len(), 32)];

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
    // Send the initial message
    sqlx::query!(
        "INSERT INTO messages (conversation_id, message) VALUES (?, ?)",
        conversation_id,
        init_message.message
    )
    .execute(&mut *tx)
    .await?;

    // Everything went well, commit the transaction
    tx.commit().await?;

    // Return the new conversation for future messages
    let res = sqlx::query_as!(
            Conversation,
            "SELECT id, title, created_at, last_message_at  FROM conversations where id = ? ORDER BY last_message_at DESC",
            conversation_id,
        )
        .fetch_one(&pool)
        .await?;
    Ok((StatusCode::OK, Json(res)).into_response())
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    /// The id of the message
    /// If this is None, the message has not been saved to the database yet
    pub id: Option<i64>,
    pub conversation_id: i64,
    pub message: String,
    pub user_id: i64,
    pub created_at: Option<NaiveDateTime>,
    pub modified_at: Option<NaiveDateTime>,
}

pub async fn get_conversation(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Path(conversation_id): Path<i64>,
) -> Result<Response, AppError> {
    let user = match authorize_user(&headers) {
        Ok(k) => k,
        Err(e) => return Ok((StatusCode::UNAUTHORIZED, e.to_string()).into_response()),
    };
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
            r#"SELECT messages.id, message, messages.created_at, modified_at, conversation_id, user_id FROM messages
            JOIN conversations ON conversations.id = messages.conversation_id 
            WHERE conversations.id = ? 
            ORDER BY last_message_at DESC"#,
            conversation_id,
        )
        .fetch_all(&pool)
        .await?;
    Ok((StatusCode::OK, Json(res)).into_response())
}

// Initializing a websocket connection should look like the following in js
// let ws = new WebSocket("ws://localhost:3000/ws", [
// "fakeProtocol",
// btoa("Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpZCI6MSwidXNlcm5hbWUiOiJDeWFuIiwiZXhwIjoxNzI3NDA2MDQ1fQ.lxlii16WpcD0gdkIOWcTCzPSmnlS0Dmp5uFVqY-VxoQ")
// .replace(/=/g, '')
// ]);
pub async fn connect_conversation(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    mut headers: HeaderMap,
) -> Result<Response, AppError> {
    // Doing this header shinanigans because websockets are doodoo
    // #5 on https://stackoverflow.com/a/77060459 explains what's going on here
    let Some(protocol) = headers.get("sec-websocket-protocol") else {
        return Ok((StatusCode::BAD_REQUEST, "No protocol provided\nPlease provide your authorization token as the second protocol in the list").into_response());
    };
    let protocols = match protocol.to_str() {
        Ok(k) => k,
        Err(e) => return Ok((StatusCode::BAD_REQUEST, e.to_string()).into_response()),
    }
    .split(',')
    .map(|s| s.trim())
    .collect::<Vec<&str>>();
    let Some(auth_token) = protocols.get(1) else {
        return Ok((StatusCode::BAD_REQUEST, "No authorization token provided").into_response());
    };
    // Authorization token must be base64 encoded, since protocols ase not allowed to contain
    // certain characters which are present in JWTs
    // No padding must be used because "=" is not allowed in the protocol
    let auth_token = match general_purpose::STANDARD_NO_PAD.decode(auth_token) {
        Ok(k) => String::from_utf8(k)?,
        Err(e) => return Ok((StatusCode::BAD_REQUEST, e.to_string()).into_response()),
    };
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_token)?);
    let user = match authorize_user(&headers) {
        Ok(k) => k,
        Err(e) => return Ok((StatusCode::UNAUTHORIZED, e.to_string()).into_response()),
    };

    Ok(ws
        .protocols(["fakeProtocol"])
        .on_upgrade(|socket| conversations_socket(socket, state, user)))
}

/// The types of responses from the socket
#[derive(Serialize, Deserialize, Clone)]
pub enum SocketResponse {
    /// Message to be sent to the client
    Message(ChatMessage),
    /// Invite to a conversation
    Invite(InviteData),
    /// Error to inform the client
    Error(String),
    /// Pong to the client
    Pong(Vec<u8>),
    /// Close the connection
    Close,
}

/// Invite data to a conversation
#[derive(Serialize, Deserialize, Clone)]
pub struct InviteData {
    pub conversation_id: i64,
    pub inviter: i64,
    pub invitee: i64,
    pub invited_at: Option<NaiveDateTime>,
}

/// The types of requests that can be made to the websocket
#[derive(Serialize, Deserialize)]
enum SocketRequest {
    /// Send a message to the conversation
    SendMessage(ChatMessage),
    /// Invite a user to the conversation
    InviteUser(InviteData),
    /// Requst the previous messages in the conversation
    /// The i64 is the id of the last message the client received
    RequestMessages(RequestMessage),
}

#[derive(Serialize, Deserialize)]
struct RequestMessage {
    /// The id of the last message the client received from the conversation
    /// If this is None, the client has not received any messages yet
    message_id: Option<i64>,
    conversation_id: i64,
    /// The maximum number of messages to request
    /// If this is None, the client is requesting 50 messages
    message_num: Option<i64>,
}

#[derive(Serialize, Deserialize)]
struct WebSocketMessage {
    #[serde(flatten)]
    message_type: SocketRequest,
}

// TODO: Implement querying AI model for responses
// TODO: Implement saving messages to the database
// TODO: Change database schema to accommodate multi-user conversations
// TODO: Add read receipts to conversations, this requires the previous TODO
pub async fn conversations_socket(stream: WebSocket, state: AppState, user: UserToken) {
    let (mut sender, mut receiver) = stream.split();
    let mut user_connections = *state
        .user_connections
        .entry(user.id)
        .and_modify(|entry| *entry += 1)
        .or_insert(1)
        .value();

    if user_connections == 1 {
        let (tx, _) = broadcast::channel(10);
        state.user_sockets.insert(user.id, tx);
    }

    let channel = state.user_sockets.get(&user.id).unwrap();

    let mut rx = channel.subscribe();
    let tx = channel.clone();
    let state_clone = state.clone();
    let user_clone = user.clone();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            match msg {
                SocketResponse::Message(chat_msg) => {
                    if let Err(e) = sender
                        .send(Message::Text(serde_json::to_string(&chat_msg).unwrap()))
                        .await
                    {
                        eprintln!("Error sending message: {}", e);
                    }
                }
                SocketResponse::Invite(invite) => {
                    if let Err(e) = sender
                        .send(Message::Text(serde_json::to_string(&invite).unwrap()))
                        .await
                    {
                        eprintln!("Error sending invite data: {}", e);
                    }
                }
                SocketResponse::Error(e) => {
                    sender.send(Message::Text(e)).await.unwrap();
                }
                SocketResponse::Pong(payload) => {
                    if let Err(e) = sender.send(Message::Pong(payload)).await {
                        eprintln!("Error sending pong: {}", e);
                    }
                }
                SocketResponse::Close => {
                    if let Err(e) = sender.close().await {
                        eprintln!("Error sending close frame: {}", e);
                    }
                    break;
                }
            }
        }
    });

    let mut receive_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(msg) => match msg {
                    Message::Text(text) => {
                        let msg: WebSocketMessage = match serde_json::from_str(&text) {
                            Ok(k) => k,
                            Err(e) => {
                                let _ = tx.send(SocketResponse::Error(e.to_string()));
                                continue;
                            }
                        };
                        match msg.message_type {
                            SocketRequest::SendMessage(chat_message) => {
                                if let Err(e) =
                                    save_message(&state_clone.pool, &chat_message, &user_clone)
                                        .await
                                {
                                    eprintln!("Error saving message: {}", e.0);
                                    tx.send(SocketResponse::Error(e.0.to_string()));
                                } else if let Err(e) =
                                    tx.send(SocketResponse::Message(chat_message))
                                {
                                    eprintln!("Error sending message: {}", e);
                                    tx.send(SocketResponse::Error(e.to_string()));
                                }
                            }
                            SocketRequest::InviteUser(invite) => {
                                if let Err(e) =
                                    invite_user(&state_clone.pool, &invite, &user_clone).await
                                {
                                    eprintln!("Error inviting user: {}", e.0);
                                    tx.send(SocketResponse::Error(e.0.to_string()));
                                } else if let Err(e) = tx.send(SocketResponse::Invite(invite)) {
                                    eprintln!("Error sending invite message: {}", e);
                                    tx.send(SocketResponse::Error(e.to_string()));
                                }
                            }
                            SocketRequest::RequestMessages(request_message) => {
                                match request_messages(
                                    &state_clone.pool,
                                    &request_message,
                                    &user_clone,
                                )
                                .await
                                {
                                    Ok(k) => {
                                        for message in k {
                                            if let Err(e) =
                                                tx.send(SocketResponse::Message(message))
                                            {
                                                eprintln!("Error sending message: {}", e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Error requesting messages: {}", e.0);
                                    }
                                }
                            }
                        }
                    }
                    Message::Binary(_) => {
                        //TODO
                    }
                    Message::Ping(payload) => {
                        if let Err(e) = tx.send(SocketResponse::Pong(payload)) {
                            eprintln!("Error sending pong: {}", e);
                        }
                    }
                    Message::Close(_) => {
                        if let Err(e) = tx.send(SocketResponse::Close) {
                            eprintln!("Error sending close frame: {}", e);
                        }
                        break;
                    }
                    _ => (),
                },
                Err(e) => {
                    eprintln!("Error receiving message: {}", e);
                    break;
                }
            }
        }
    });

    // If either of the tasks completes, we want to abort the other one
    tokio::select! {
        _ = &mut receive_task => send_task.abort(),
        _ = &mut send_task => receive_task.abort()
    };

    // Remove the user from the connection once all the tasks are
    // complete and all user devices have disconnected
    state
        .user_connections
        .entry(user.id)
        .and_modify(|entry| *entry -= 1);
    user_connections = *state.user_connections.get(&user.id).unwrap().value();
    if user_connections == 0 {
        state.user_connections.remove(&user.id);
        state.user_sockets.remove(&user.id);
    }
}

async fn request_messages(
    pool: &SqlitePool,
    request: &RequestMessage,
    user: &UserToken,
) -> Result<Vec<ChatMessage>, AppError> {
    if sqlx::query!(
        r#"SELECT conversation_id FROM user_conversations
            WHERE conversation_id = ? and user_id = ?"#,
        request.conversation_id,
        user.id
    )
    .fetch_optional(pool)
    .await?
    .is_none()
    {
        return Err(anyhow!("User is not in the conversation").into());
    }

    let limit = request.message_num.unwrap_or(50);
    let message_id = request.message_num.unwrap_or(i64::MAX);
    Ok(sqlx::query_as!(
        ChatMessage,
        r#"SELECT messages.id, message, messages.created_at, modified_at, conversation_id, user_id FROM messages
        JOIN conversations ON conversations.id = messages.conversation_id 
        WHERE conversations.id = ? AND messages.id < ?
        ORDER BY last_message_at DESC
        LIMIT ?"#,
        request.conversation_id,
        message_id,
        limit
    )
    .fetch_all(pool)
    .await?)
}

async fn save_message(
    pool: &SqlitePool,
    message: &ChatMessage,
    user: &UserToken,
) -> Result<(), AppError> {
    if user.id != message.user_id {
        return Err(anyhow!("User does not have permission to send message").into());
    }
    if sqlx::query!(
        "SELECT conversation_id FROM user_conversations WHERE conversation_id = ? and user_id = ?",
        message.conversation_id,
        user.id
    )
    .fetch_optional(pool)
    .await?
    .is_none()
    {
        return Err(anyhow!("User is not in the conversation").into());
    }
    sqlx::query!(
        "INSERT INTO messages (conversation_id, message) VALUES (?, ?)",
        message.conversation_id,
        message.message
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn invite_user(
    pool: &SqlitePool,
    message: &InviteData,
    user: &UserToken,
) -> Result<(), AppError> {
    if user.id != message.inviter {
        return Err(anyhow!("User does not have permission to invite user").into());
    }

    if sqlx::query!(
        "SELECT conversation_id FROM user_conversations WHERE conversation_id = ? and user_id = ?",
        message.conversation_id,
        user.id
    )
    .fetch_optional(pool)
    .await?
    .is_none()
    {
        return Err(anyhow!("User is not in the conversation").into());
    }

    // Check if the user is already in the conversation
    if sqlx::query!(
        "SELECT conversation_id FROM user_conversations WHERE user_id = ? AND conversation_id = ?",
        message.invitee,
        message.conversation_id
    )
    .fetch_optional(pool)
    .await?
    .is_some()
    {
        return Err(anyhow!("User is already in the conversation").into());
    }
    // Check if the user is in the database
    if sqlx::query!("SELECT id FROM users WHERE id = ?", message.invitee)
        .fetch_optional(pool)
        .await?
        .is_none()
    {
        return Err(anyhow!("User does not exist").into());
    }
    // Add the user to the conversation
    sqlx::query!(
        "INSERT INTO user_conversations (user_id, conversation_id) VALUES (?, ?)",
        message.invitee,
        message.conversation_id
    )
    .execute(pool)
    .await?;
    Ok(())
}
