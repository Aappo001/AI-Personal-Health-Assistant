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
use futures::{stream::SplitSink, SinkExt, StreamExt};
use log::error;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, SqlitePool};
use tokio::sync::broadcast::{self};

use crate::error::AppError;
use crate::{
    error::AppJson,
    users::{authorize_user, UserToken},
    AppState,
};

#[derive(Serialize, Deserialize, FromRow)]
pub struct Conversation {
    pub id: i64,
    pub title: Option<String>,
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
    Ok((StatusCode::OK, Json(res)).into_response())
}

pub async fn create_conversation(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    AppJson(init_message): AppJson<ChatMessage>,
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
            "SELECT id, title, created_at, last_message_at FROM conversations where id = ? ORDER BY last_message_at DESC",
            conversation_id,
        )
        .fetch_one(&pool)
        .await?;
    Ok((StatusCode::OK, Json(res)).into_response())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SocketResponse {
    /// Message to be sent to the client
    Message(ChatMessage),
    /// Invite to a conversation
    Invite(InviteData),
    /// Error to inform the client
    Error(String),
    /// Pong to the client
    Pong(Vec<u8>),
    /// Read receipt
    ReadEvent(ReadEvent),
    /// Close the connection
    Close,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReadEvent {
    pub conversation_id: i64,
    pub user_id: i64,
    pub timestamp: NaiveDateTime,
}

/// Invite data to a conversation
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InviteData {
    pub conversation_id: Option<i64>,
    pub inviter: i64,
    pub invitees: Vec<i64>,
    pub invited_at: Option<NaiveDateTime>,
}

/// The types of requests that can be made to the websocket
#[derive(Serialize, Deserialize)]
enum SocketRequest {
    /// Send a message to the conversation
    SendMessage(ChatMessage),
    /// Invite a user to the conversation
    InviteUser(InviteData),
    /// Message has been read in given conversation
    ReadMessage(i64),
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
// TODO: Refactor this function so the receive and send tasks are separate functions
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

    // Send messages to the client over the websocket
    // Messages are received from the broadcast channel
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            match send_message(&mut sender, msg).await {
                // The message was sent successfully to the client, continue
                Ok(Some(())) => (),
                // The connection was closed, break the loop
                Ok(None) => break,
                // There was an error sending the message, but the connection is still open
                Err(e) => {
                    error!("Error sending message: {}", e);
                    sender.send(Message::Text(e.to_string())).await.unwrap();
                }
            }
        }
    });

    // Handle incoming messages from the client over the websocket
    let mut receive_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(msg) => {
                    if let Err(e) = handle_message(msg, &tx, &state_clone, &user_clone).await {
                        error!("Error handling message: {}", e);
                        let _ = tx.send(SocketResponse::Error(e.to_string()));
                    }
                }
                Err(e) => {
                    error!("Error receiving message: {}", e);
                    break;
                }
            }
        }
    });

    // If a task completes, that means that the websocket connection has been closed
    // If either of the tasks completes, we want to abort the other one
    tokio::select! {
        _ = &mut receive_task => send_task.abort(),
        _ = &mut send_task => receive_task.abort()
    };

    // Decrease the number of connections the user has
    state
        .user_connections
        .entry(user.id)
        .and_modify(|entry| *entry -= 1);
    user_connections = *state.user_connections.get(&user.id).unwrap().value();
    // Remove the user from the connection once all the tasks are
    // complete and all user devices have disconnected
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

/// Save a message to the database
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

/// Invite multiple users to a conversation
async fn invite_user(
    pool: &SqlitePool,
    invite: &InviteData,
    user: &UserToken,
) -> Result<(), AppError> {
    if user.id != invite.inviter {
        return Err(anyhow!("User does not have permission to invite users").into());
    }
    let conversation_id = match invite.conversation_id {
        // Conversation already exists so check if inviter is in it
        Some(conversation_id) => {
            if sqlx::query!(
                "SELECT conversation_id FROM user_conversations WHERE conversation_id = ? and user_id = ?",
                conversation_id,
                user.id
            )
                .fetch_optional(pool)
                .await?
                .is_none()
            {
                return Err(anyhow!("Inviter is not in the conversation").into());
            }
            conversation_id
        }
        // Conversation does not exist so create a new one and invite the inviter
        None => {
            let mut tx = pool.begin().await?;
            let conversation_id = sqlx::query!("INSERT INTO conversations DEFAULT VALUES")
                .execute(&mut *tx)
                .await?
                .last_insert_rowid();
            sqlx::query!(
                "INSERT INTO user_conversations (user_id, conversation_id) VALUES (?, ?)",
                user.id,
                conversation_id
            )
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            conversation_id
        }
    };

    // Begin a transaction to ensure that all the users are added to the converation at the same
    // time
    let mut tx = pool.begin().await?;
    for invitee in &invite.invitees {
        // Check if the user is already in the conversation
        if sqlx::query!(
            "SELECT conversation_id FROM user_conversations WHERE user_id = ? AND conversation_id = ?",
            invitee,
            conversation_id
        )
            .fetch_optional(&mut *tx)
            .await?
            .is_some()
        {
            return Err(anyhow!("User {} is already in the conversation", invitee).into());
        }
        // Check if the user is in the database
        if sqlx::query!("SELECT id FROM users WHERE id = ?", invitee)
            .fetch_optional(&mut *tx)
            .await?
            .is_none()
        {
            return Err(anyhow!("User {} does not exist", invitee).into());
        }
        // Add the user to the conversation
        sqlx::query!(
            "INSERT INTO user_conversations (user_id, conversation_id) VALUES (?, ?)",
            invitee,
            conversation_id
        )
        .execute(pool)
        .await?;
    }
    tx.commit().await?;

    Ok(())
}

/// Mark the conversation as read by the logged in user
async fn read_event(
    pool: &SqlitePool,
    conversation_id: i64,
    user: &UserToken,
) -> Result<(), AppError> {
    let now = chrono::Utc::now();
    sqlx::query!(
        "UPDATE user_conversations SET last_read_at = ? WHERE user_id = ? and conversation_id = ?",
        now,
        user.id,
        conversation_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

// Handle incoming websocket messages from the client
async fn handle_message(
    msg: Message,
    tx: &broadcast::Sender<SocketResponse>,
    state: &AppState,
    user: &UserToken,
) -> Result<(), AppError> {
    match msg {
        Message::Text(text) => {
            let msg: WebSocketMessage = serde_json::from_str(&text)?;
            match msg.message_type {
                SocketRequest::SendMessage(chat_message) => {
                    save_message(&state.pool, &chat_message, user).await?;
                    // Find all the users in the conversation
                    let users = sqlx::query!(
                        "SELECT user_id FROM user_conversations WHERE conversation_id = ?",
                        chat_message.conversation_id
                    )
                    .fetch_all(&state.pool)
                    .await?;
                    // Send the message to all the users in the conversation
                    for user in users {
                        // Only send the message to users who are connected
                        if let Some(user) = state.user_sockets.get(&user.user_id) {
                            if let Err(e) = user.send(SocketResponse::Message(chat_message.clone()))
                            {
                                eprintln!("Error sending message: {}", e);
                            }
                        }
                    }
                }
                SocketRequest::InviteUser(invite) => {
                    invite_user(&state.pool, &invite, user).await?;
                    tx.send(SocketResponse::Invite(invite))?;
                }
                SocketRequest::ReadMessage(conversation_id) => {
                    read_event(&state.pool, conversation_id, user).await?;
                    tx.send(SocketResponse::ReadEvent(ReadEvent {
                        conversation_id,
                        user_id: user.id,
                        timestamp: chrono::Utc::now().naive_utc(),
                    }))?;
                }
                SocketRequest::RequestMessages(request_message) => {
                    for message in request_messages(&state.pool, &request_message, user).await? {
                        if let Err(e) = tx.send(SocketResponse::Message(message)) {
                            eprintln!("Error sending message: {}", e);
                        }
                    }
                }
            }
        }
        Message::Binary(_) => {
            //TODO
        }
        Message::Ping(payload) => {
            tx.send(SocketResponse::Pong(payload))?;
        }
        Message::Close(_) => {
            tx.send(SocketResponse::Close)?;
        }
        _ => (),
    }
    Ok(())
}

// Send a message to the client over the websocket
// Option<()> is returned because the connection may have been closed
// Some(()) is returned if the message was sent successfully
// None is returned if the connection was closed
async fn send_message(
    sender: &mut SplitSink<WebSocket, Message>,
    msg: SocketResponse,
) -> Result<Option<()>, AppError> {
    match msg {
        SocketResponse::Message(chat_msg) => {
            sender
                .send(Message::Text(serde_json::to_string(&chat_msg).unwrap()))
                .await?;
        }
        SocketResponse::Invite(invite) => {
            sender
                .send(Message::Text(serde_json::to_string(&invite).unwrap()))
                .await?;
        }
        SocketResponse::ReadEvent(event) => {
            sender
                .send(Message::Text(serde_json::to_string(&event).unwrap()))
                .await?;
        }
        SocketResponse::Error(e) => {
            sender.send(Message::Text(e)).await?;
        }
        SocketResponse::Pong(payload) => {
            sender.send(Message::Pong(payload)).await?;
        }
        SocketResponse::Close => {
            sender.close().await?;
            return Ok(None);
        }
    }
    Ok(Some(()))
}
