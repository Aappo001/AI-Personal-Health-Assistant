use std::net::SocketAddr;

use anyhow::anyhow;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, State,
    },
    http::{header::AUTHORIZATION, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use base64::{engine::general_purpose, Engine};
use futures::{stream::SplitSink, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tokio::sync::broadcast;
use tracing::{error, info, instrument, warn};

use crate::{
    error::{AppError, ErrorResponse},
    users::{authorize_user, UserToken},
    AppState,
};

use super::{create_conversation, ChatMessage, InviteData, ReadEvent};

// Initializing a websocket connection should look like the following in js
// let ws = new WebSocket("ws://localhost:3000/api/ws", [
// "fakeProtocol",
// btoa("Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpZCI6MSwidXNlcm5hbWUiOiJDeWFuIiwiZXhwIjoxNzI3NDA2MDQ1fQ.lxlii16WpcD0gdkIOWcTCzPSmnlS0Dmp5uFVqY-VxoQ")
// .replace(/=/g, '')
// ]);
//
/// Initializer for a websocket connection
/// Doesn't actually do anything with the connection other than authorization
/// Passes on the connection to the `conversations_socket` function where the actual
/// logic for the websocket is implemented
pub async fn connect_conversation(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    mut headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
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
    let user = authorize_user(&headers)?;

    info!("Received websocket connection from {}", addr);
    Ok(ws
        .protocols(["fakeProtocol"])
        .on_upgrade(|socket| conversations_socket(socket, state, user)))
}

/// The types of responses from the socket
#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum SocketResponse {
    /// Message to be sent to the client
    Message(ChatMessage),
    /// The i64 is the id of the message to delete
    DeleteMessage(i64),
    /// Invite to a conversation
    Invite(InviteData),
    /// Error to inform the client
    Error(ErrorResponse),
    /// Pong to the client
    Pong(Vec<u8>),
    /// Read event to inform the client that messages before a given timestamp
    /// in a conversation were read by a user
    ReadEvent(ReadEvent),
    /// Close the connection
    Close,
}

/// The types of requests that can be made to the websocket
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum SocketRequest {
    /// Send a message to the conversation
    SendMessage(ChatMessage),
    /// Edit a message in the conversation
    EditMessage(ChatMessage),
    /// Edit a message in the conversation
    /// The i64 is the id of the message to delete
    DeleteMessage(i64),
    /// Invite a user to the conversation
    InviteUser(InviteData),
    /// Message has been read in given conversation
    /// Does not provide user_id because the user is already authenticated
    /// Does not provide timestamp because the server will set it
    ReadMessage(i64),
    /// Requst the previous messages in the conversation
    /// The i64 is the id of the last message the client received
    RequestMessages(RequestMessage),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RequestMessage {
    /// The id of the last message the client received from the conversation
    /// If this is None, the client has not received any messages yet
    message_id: Option<i64>,
    conversation_id: i64,
    /// The maximum number of messages to request
    /// If this is None, the client is requesting 50 messages
    message_num: Option<i64>,
}

/// Handles incoming websocket connections
// TODO: Implement querying AI model for responses
// TODO: Refactor this function so the receive and send tasks are separate functions
#[instrument]
pub async fn conversations_socket(stream: WebSocket, state: AppState, user: UserToken) {
    let (mut sender, mut receiver) = stream.split();

    // Increase the number of connections the user has
    let mut user_connections = *state
        .user_connections
        .entry(user.id)
        .and_modify(|entry| *entry += 1)
        .or_insert(1)
        .value();

    // This is the first connection the user has, so create a broadcast channel to start sending
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
        // Keep checking for incoming messages and sending messages to the client accordingly
        // until the connection is closed
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
        // Keep receiving messages until the connection is closed
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(msg) => {
                    if let Err(e) = handle_message(msg, &tx, &state_clone, &user_clone).await {
                        error!("Error handling message: {}", e);
                        let _ = tx.send(SocketResponse::Error(e.into()));
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

/// Requests the most recent messages sent in a conversation before the given message id
/// A given id of None will return the most recent messages
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
) -> Result<ChatMessage, AppError> {
    // If the conversation_id is None, this is the first message in a conversation
    // so create a new conversation and get the id
    let conversation_id = message
        .conversation_id
        .unwrap_or(create_conversation(pool, message, user).await?.id);

    if sqlx::query!(
        "SELECT conversation_id FROM user_conversations WHERE conversation_id = ? and user_id = ?",
        conversation_id,
        user.id
    )
    .fetch_optional(pool)
    .await?
    .is_none()
    {
        return Err(anyhow!("User is not in the conversation").into());
    }

    Ok(sqlx::query_as!(
        ChatMessage,
        "INSERT INTO messages (user_id, conversation_id, message) VALUES (?, ?, ?) RETURNING *",
        user.id,
        conversation_id,
        message.message
    )
    .fetch_one(pool)
    .await?)
}

/// Edit message in the database
async fn edit_message(
    pool: &SqlitePool,
    message: &ChatMessage,
    user: &UserToken,
) -> Result<ChatMessage, AppError> {
    // Check if the message id is present
    let Some(message_id) = message.id else {
        return Err(anyhow!("Message id is required to edit message").into());
    };

    // Check if the message exists in the database
    let Some(message_user) = sqlx::query!("SELECT user_id FROM messages WHERE id = ?", message_id)
        .fetch_optional(pool)
        .await?
    else {
        return Err(anyhow!("Message not found").into());
    };

    // Check if the user has permission to edit the message
    if message_user.user_id != user.id {
        return Err(anyhow!("User does not have permission to edit message").into());
    }

    let now = chrono::Utc::now();

    // Update the message in the database
    // We know the message exists so we can just use `fetch_one`
    Ok(sqlx::query_as!(
        ChatMessage,
        "UPDATE messages SET message = ?, modified_at = ? WHERE id = ? RETURNING *",
        message.message,
        now,
        message_id
    )
    .fetch_one(pool)
    .await?)
}

/// Delete a message in the database
async fn delete_message(
    pool: &SqlitePool,
    message_id: i64,
    user: &UserToken,
) -> Result<(), AppError> {
    // Check if the message exists in the database
    let Some(message) = sqlx::query_as!(
        ChatMessage,
        "SELECT * FROM messages WHERE id = ?",
        message_id
    )
    .fetch_optional(pool)
    .await?
    else {
        return Err(anyhow!("Message not found").into());
    };
    // Check if the user has permission to delete the message
    if message.user_id.expect("user id should be set in database") != user.id {
        return Err(anyhow!("User does not have permission to delete message").into());
    }
    // Delete the message from the database
    sqlx::query!("DELETE FROM messages WHERE id = ?", message.id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Invite multiple users to a conversation
/// Returns the conversation id that the users were invited to
async fn invite_user(
    pool: &SqlitePool,
    invite: &InviteData,
    user: &UserToken,
) -> Result<i64, AppError> {
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

    Ok(conversation_id)
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

/// Handle incoming websocket messages from the client
/// This function will parse the message and send the appropriate response based on the enum
/// variant
async fn handle_message(
    msg: Message,
    tx: &broadcast::Sender<SocketResponse>,
    state: &AppState,
    user: &UserToken,
) -> Result<(), AppError> {
    match msg {
        Message::Text(text) => {
            let msg: SocketRequest = serde_json::from_str(&text)?;
            match msg {
                SocketRequest::SendMessage(chat_message) => {
                    let chat_message = save_message(&state.pool, &chat_message, user).await?;
                    // Broadcast the newly saved message to all the users in the conversation
                    broadcast_event(state, SocketResponse::Message(chat_message.clone())).await?;
                }
                SocketRequest::EditMessage(chat_message) => {
                    let chat_message = edit_message(&state.pool, &chat_message, user).await?;
                    // Broadcast the edited message to all the users in the conversation
                    broadcast_event(state, SocketResponse::Message(chat_message.clone())).await?;
                }
                SocketRequest::DeleteMessage(message_id) => {
                    delete_message(&state.pool, message_id, user).await?;
                    // Broadcast the deleted message to all the users in the conversation
                    broadcast_event(state, SocketResponse::DeleteMessage(message_id)).await?;
                }
                SocketRequest::InviteUser(mut invite) => {
                    invite.conversation_id = Some(invite_user(&state.pool, &invite, user).await?);
                    broadcast_event(state, SocketResponse::Invite(invite)).await?;
                }
                SocketRequest::ReadMessage(conversation_id) => {
                    read_event(&state.pool, conversation_id, user).await?;
                    broadcast_event(
                        state,
                        SocketResponse::ReadEvent(ReadEvent {
                            conversation_id,
                            user_id: user.id,
                            timestamp: chrono::Utc::now().naive_utc(),
                        }),
                    )
                    .await?;
                }
                SocketRequest::RequestMessages(request_message) => {
                    for message in request_messages(&state.pool, &request_message, user).await? {
                        tx.send(SocketResponse::Message(message))?;
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

/// Broadcast an event to all the users in a conversation
/// Events include messages, edits, and deletes, ect.
async fn broadcast_event(state: &AppState, msg: SocketResponse) -> Result<(), AppError> {
    let id = match &msg {
        SocketResponse::Message(chat_msg) => chat_msg
            .conversation_id
            .expect("Conversation id should be set"),
        SocketResponse::DeleteMessage(id) => *id,
        SocketResponse::ReadEvent(event) => event.conversation_id,
        SocketResponse::Invite(invite) => invite
            .conversation_id
            .expect("Conversation id should be set"),
        //
        _ => unreachable!("uuhhh how"),
    };
    let users = sqlx::query!(
        "SELECT user_id FROM user_conversations WHERE conversation_id = ?",
        id
    )
    .fetch_all(&state.pool)
    .await?;
    for user in users {
        if let Some(user) = state.user_sockets.get(&user.user_id) {
            if let Err(e) = user.send(msg.clone()) {
                warn!("Error broadcasting event: {}", e);
            }
        }
    }
    Ok(())
}

/// Send a message to the client over the websocket
/// Option<()> is returned because the connection may have been closed
/// Some(()) is returned if the message was sent successfully
/// None is returned if the connection was closed
async fn send_message(
    sender: &mut SplitSink<WebSocket, Message>,
    msg: SocketResponse,
) -> Result<Option<()>, AppError> {
    // *SAFETY* The `serde_json::to_string` function can safely be unwrapped because the `SocketResponse` enum
    // is serializable and will not panic
    match msg {
        SocketResponse::Pong(payload) => {
            sender.send(Message::Pong(payload)).await?;
        }
        SocketResponse::Close => {
            sender.close().await?;
            return Ok(None);
        }
        // All other responses should be serialized to JSON
        // and sent as Text
        response => {
            sender
                .send(Message::Text(serde_json::to_string(&response).unwrap()))
                .await?;
        }
    }
    Ok(Some(()))
}
