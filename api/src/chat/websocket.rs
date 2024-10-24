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
use chrono::NaiveDateTime;
use futures::{stream::SplitSink, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tokio::sync::broadcast;
use tracing::{error, info, info_span, instrument, warn};
use validator::ValidateRequired;

use crate::{
    chat::{query_model, Conversation},
    error::{AppError, ErrorResponse},
    users::{authorize_user, UserToken},
    AppState,
};

use super::{conversation, create_conversation, ChatMessage, ReadEvent, StreamMessage};

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
    /// Conversation to be sent to the client
    Conversation(Conversation),
    /// The i64 is the id of the message to delete
    DeleteMessage(i64),
    /// Stream data from the AI model
    StreamData(StreamMessage),
    /// Invite to a conversation
    Invite {
        /// The id of the conversation the user was invited to
        conversation_id: i64,
        /// The id of the inviter
        inviter: i64,
        /// When the invite was created
        invited_at: NaiveDateTime,
    },
    /// Friend request to be sent to the client
    FriendRequest {
        sender_id: i64,
        receiver_id: i64,
        created_at: chrono::NaiveDateTime,
        status: FriendRequestStatus,
    },
    /// Error to inform the client
    Error(ErrorResponse),
    /// Read event to inform the client that messages before a given timestamp
    /// in a conversation were read by a user
    ReadEvent(ReadEvent),
}

#[derive(Serialize, Clone, Debug)]
pub enum FriendRequestStatus {
    Pending,
    Accepted,
    Rejected,
}

// The WebSocket API is a bit different than the REST API
// it works by sending JSON serialized `SocketRequest` enums
// to the server and receiving `SocketResponse` enums back
//
// The client will send a message like this to the server
// ws.send(JSON.stringify({
//   type: "SendMessage",
//   message: "Hello, world!",
//   conversationId: 1
// }))
/// The types of requests that can be made to the websocket
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum SocketRequest {
    /// Send a message to the conversation
    SendMessage(SendMessage),
    /// Edit a message in the conversation
    EditMessage(EditMessage),
    /// The i64 is the id of the message to delete
    DeleteMessage(i64),
    /// Send, accept, reject, or revoke a friend request
    // Put all the friend request stuff in one enum variant
    // so its easier to handle on the frontend
    SendFriendRequest {
        /// The id of the user involved in the friend request
        /// This might be the sender or receiver depending on the action
        other_user_id: i64,
        /// The action to take on the friend request
        /// Send or accept a friend request if true
        /// Reject or revoke a friend request if false
        accept: bool,
    },
    /// Invite users to a conversation
    InviteUsers {
        /// The id of the conversation to invite the users to
        /// if this is None, a new conversation will be created
        conversation_id: Option<i64>,
        /// The users being invited to the conversation
        invitees: Box<[i64]>,
    },
    /// Message has been read in given conversation
    /// Does not provide user_id because the user is already authenticated
    /// Does not provide timestamp because the server will set it
    ReadMessage(i64),
    /// Requst the previous messages in the conversation
    /// Returns messages in order of most recent to least recent
    RequestMessages(RequestMessage),
    /// Request the list of conversations the user is in
    /// Returns conversations in order of last message sent
    RequestConversations(RequestConversation),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SendMessage {
    pub conversation_id: Option<i64>,
    pub message: String,
    pub ai_model_id: Option<i64>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct EditMessage {
    id: i64,
    message: String,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RequestConversation {
    /// The id of the last conversation the client received from the server
    /// If this is None, the client has not received any messages yet
    conversation_id: Option<i64>,
    /// The maximum number of messages to request
    /// If this is None, the client is requesting 50 conversations
    message_num: Option<i64>,
}

/// Handles incoming websocket connections
// TODO: Implement querying AI model for responses
#[instrument]
pub async fn conversations_socket(stream: WebSocket, state: AppState, user: UserToken) {
    let (mut sender, mut receiver) = stream.split();

    // Increase the number of connections the user has
    let mut user_connections = *state
        .user_connections
        .entry_async(user.id)
        .await
        .and_modify(|entry| *entry += 1)
        .or_insert(1);

    // This is the first connection the user has, so create a broadcast channel to start sending
    if user_connections == 1 {
        let (tx, _) = broadcast::channel(10);
        let _ = state.user_sockets.insert_async(user.id, tx).await;
    }

    let channel = state
        .user_sockets
        .read_async(&user.id, |_, v| v.clone())
        .await
        .unwrap();

    let mut rx = channel.subscribe();
    let tx = channel.clone();

    // Send messages to the client over the websocket
    // Messages are received from the broadcast channel
    let mut send_task = tokio::spawn({
        let user = user.clone();
        async move {
            // Keep checking for incoming messages and sending messages to the client accordingly
            // until the connection is closed
            while let Ok(msg) = rx.recv().await {
                match send_message(&mut sender, msg, &user).await {
                    Ok(true) => (),
                    Ok(false) => {
                        let _ = sender.close().await;
                        break;
                    }
                    Err(e) => {
                        error!("Error sending messae: {}", e);
                        sender.send(Message::Text(e.to_string())).await.unwrap();
                    }
                }
            }
        }
    });

    // Handle incoming messages from the client over the websocket
    let mut receive_task = tokio::spawn({
        let state = state.clone();
        let user = user.clone();
        async move {
            // Keep receiving messages until the connection is closed
            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(msg) => {
                        if let Err(e) = handle_message(msg, &tx, &state, &user).await {
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
        }
    });

    // If a task completes, that means that the websocket connection has been closed
    // If either of the tasks completes, we want to abort the other one
    tokio::select! {
        _ = &mut receive_task => send_task.abort(),
        _ = &mut send_task => receive_task.abort()
    };

    // Decrease the number of connections the user has
    let _ = state
        .user_connections
        .entry_async(user.id)
        .await
        .and_modify(|entry| *entry -= 1);
    user_connections = *state.user_connections.get_async(&user.id).await.unwrap();
    // Remove the user from the connection once all the tasks are
    // complete and all user devices have disconnected
    if user_connections == 0 {
        state.user_connections.remove_async(&user.id).await;
        state.user_sockets.remove_async(&user.id).await;
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
    let message_id = request.message_id.unwrap_or(i64::MAX);
    Ok(sqlx::query_as!(
        ChatMessage,
        r#"SELECT messages.id, message, messages.created_at, modified_at, conversation_id, user_id, ai_model_id FROM messages
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
    message: &SendMessage,
    user: &UserToken,
) -> Result<ChatMessage, AppError> {
    // If the conversation_id is None, this is the first message in a conversation
    // so create a new conversation and get the id
    let conversation_id = match message.conversation_id {
        Some(k) => k,
        None => create_conversation(pool, message, user).await?.id,
    };

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
    message: &EditMessage,
    user: &UserToken,
) -> Result<ChatMessage, AppError> {
    // Check if the message exists in the database
    let Some(message_user) = sqlx::query!("SELECT user_id FROM messages WHERE id = ?", message.id)
        .fetch_optional(pool)
        .await?
    else {
        return Err(anyhow!("Message not found").into());
    };

    // Check if the user has permission to edit the message
    if message_user.user_id != Some(user.id) {
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
        message.id
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
    if message.user_id != Some(user.id) {
        return Err(anyhow!("User does not have permission to delete message").into());
    }
    // Delete the message from the database
    sqlx::query!("DELETE FROM messages WHERE id = ?", message.id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Handle friend requests
/// If accept is true, the friend request will be accepted if it exists
/// or sent if it does not
///
/// If accept is false, the friend request will be rejected or revoked
async fn handle_friend_request(
    state: &AppState,
    other_user_id: i64,
    accept: bool,
    user: &UserToken,
) -> Result<(), AppError> {
    if user.id == other_user_id {
        return Err(anyhow!("User cannot send friend request to themselves").into());
    }

    // Check that the users are not already friends
    let user1_id = user.id.min(other_user_id);
    let user2_id = user.id.max(other_user_id);
    sqlx::query!(
        "SELECT user1_id FROM friendships WHERE user1_id = ? and user2_id = ?",
        user1_id,
        user2_id
    )
    .fetch_optional(&state.pool)
    .await?;

    let friend_request = if accept {
        // Check that the sender does not already have an outgoing friend request to
        // the recipient
        if sqlx::query!(
            "SELECT sender_id FROM friend_requests WHERE sender_id = ? AND receiver_id = ?",
            user.id,
            other_user_id
        )
        .fetch_optional(&state.pool)
        .await?
        .is_some()
        {
            return Err(anyhow!("Friend request already exists").into());
        }
        // Everything is good so check if we are accepting an existing incoming
        // friend request or sending a new outgoing friend request
        if sqlx::query!(
            "SELECT sender_id FROM friend_requests WHERE sender_id = ? AND receiver_id = ?",
            other_user_id,
            user.id
        )
        .fetch_optional(&state.pool)
        .await?
        .is_some()
        {
            // An incoming friend request already exists so accept it
            // Create a transaction to ensure that the friend request is accepted
            // and the friend request is deleted from the database at the same time
            let mut tx = state.pool.begin().await?;

            let friendship = sqlx::query!(
                "INSERT INTO friendships (user1_id, user2_id) VALUES (?, ?) RETURNING created_at",
                user1_id,
                user2_id,
            )
            .fetch_one(&mut *tx)
            .await?;
            sqlx::query!(
                "DELETE FROM friend_requests WHERE sender_id = ? AND receiver_id = ?",
                other_user_id,
                user.id
            )
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;
            // Have to make the friend request enum manually
            // because the table doesn't have a status column
            // and it doesn't let me add one with select queries
            SocketResponse::FriendRequest {
                sender_id: user.id,
                receiver_id: other_user_id,
                created_at: friendship.created_at,
                status: FriendRequestStatus::Accepted,
            }
        } else {
            // A friend request does not exist so send it
            let friendship = sqlx::query!(
                "INSERT INTO friend_requests (sender_id, receiver_id) VALUES (?, ?) RETURNING created_at",
                user.id,
                other_user_id
            )
            .fetch_one(&state.pool)
            .await?;
            SocketResponse::FriendRequest {
                sender_id: user.id,
                receiver_id: other_user_id,
                created_at: friendship.created_at,
                status: FriendRequestStatus::Pending,
            }
        }
    } else {
        // Friend request was rejected or revoked
        // so attempt to delete the friend request from the database
        let Some(friend_request) = sqlx::query!(
            "DELETE FROM friend_requests WHERE (sender_id = ? or sender_id = ?) AND (receiver_id = ? or receiver_id = ?) RETURNING *",
            user.id,
            other_user_id,
            user.id,
            other_user_id,
        )
            .fetch_optional(&state.pool)
            .await? else {
            return Err(anyhow!("Friend request does not exist").into());
        };
        SocketResponse::FriendRequest {
            sender_id: friend_request.sender_id,
            receiver_id: friend_request.receiver_id,
            created_at: friend_request.created_at,
            status: FriendRequestStatus::Rejected,
        }
    };

    // Only send the friend request over the websocket to the receiver
    // if the receiver is online
    if let Some(receiver) = state.user_sockets.get(&other_user_id) {
        receiver.send(friend_request.clone())?;
    }

    // Send the friend request over the websocket to the sender
    // to let them know that the friend request was sent successfully
    if let Some(sender) = state.user_sockets.get(&user.id) {
        sender.send(friend_request)?;
    }
    Ok(())
}

/// Invite multiple users to a conversation
/// Returns the conversation id that the users were invited to
async fn invite_user(
    pool: &SqlitePool,
    conversation_id: Option<i64>,
    invitees: &[i64],
    user: &UserToken,
) -> Result<i64, AppError> {
    let conversation_id = match conversation_id {
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
    for invitee in invitees {
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
        .execute(&mut *tx)
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
            dbg!("Received", &msg);
            match msg {
                SocketRequest::SendMessage(mut send_message) => {
                    let chat_message = save_message(&state.pool, &send_message, user).await?;
                    send_message.conversation_id = Some(chat_message.conversation_id);
                    // Broadcast the newly saved message to all the users in the conversation
                    if send_message.ai_model_id.is_some() {
                        // Query the AI model for a response and broadcast the user's message
                        // to the conversation at the same time
                        let (_, ai_message) = tokio::join!(
                            broadcast_event(state, SocketResponse::Message(chat_message.clone())),
                            query_model(state, &send_message)
                        );
                        // Broadcast the AI model's response to the conversation
                        broadcast_event(state, SocketResponse::Message(ai_message?)).await?;
                    } else {
                        broadcast_event(state, SocketResponse::Message(chat_message.clone()))
                            .await?;
                    }
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
                SocketRequest::InviteUsers {
                    invitees,
                    mut conversation_id,
                } => {
                    conversation_id =
                        Some(invite_user(&state.pool, conversation_id, &invitees, user).await?);
                    broadcast_event(
                        state,
                        SocketResponse::Invite {
                            conversation_id: conversation_id.unwrap(),
                            inviter: user.id,
                            invited_at: chrono::Utc::now().naive_utc(),
                        },
                    )
                    .await?;
                }
                SocketRequest::SendFriendRequest {
                    other_user_id,
                    accept,
                } => {
                    handle_friend_request(state, other_user_id, accept, user).await?;
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
                SocketRequest::RequestConversations(request_message) => {
                    let limit = request_message.message_num.unwrap_or(50);
                    let conversation_id = request_message.conversation_id.unwrap_or(i64::MAX);
                    // Query the database for the conversations the user is in
                    // Use fetch instead of fetch all to stream results to the client
                    let mut query = sqlx::query_as!(
                        Conversation,
                        r#"SELECT conversations.id, conversations.title, conversations.created_at, conversations.last_message_at FROM conversations
                           JOIN user_conversations
                           ON conversations.id = user_conversations.conversation_id
                           WHERE user_id = ? AND conversations.id < ?
                           ORDER BY conversations.last_message_at DESC
                           LIMIT ?"#,
                        user.id,
                        conversation_id,
                        limit
                    )
                    .fetch(&state.pool);
                    while let Some(conversation) = query.next().await {
                        tx.send(SocketResponse::Conversation(conversation?))?;
                    }
                }
            }
        }
        Message::Binary(_) => {
            //TODO
        }
        // We do not need to handle ping or close messages
        // because tokio_tungstenite will handle them for us
        Message::Ping(_) | Message::Close(_) | _ => (),
    }
    Ok(())
}

/// Broadcast an event to all the users in a conversation
/// Events include messages, edits, and deletes, ect.
pub async fn broadcast_event(state: &AppState, msg: SocketResponse) -> Result<(), AppError> {
    let id = match &msg {
        SocketResponse::Message(chat_msg) => chat_msg.conversation_id,
        SocketResponse::DeleteMessage(id) => *id,
        SocketResponse::ReadEvent(event) => event.conversation_id,
        SocketResponse::StreamData(data) => data.conversation_id,
        SocketResponse::Invite {
            inviter: _,
            conversation_id,
            invited_at: _,
        } => *conversation_id,
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
/// bool is returned because the connection may have been closed
/// true is returned if the message was sent successfully
/// false is returned if the connection was closed
async fn send_message(
    sender: &mut SplitSink<WebSocket, Message>,
    msg: SocketResponse,
    user: &UserToken,
) -> Result<bool, AppError> {
    // Check if the user is still authorized
    // and close the connection if they are not
    if user.exp < chrono::Utc::now().timestamp() {
        return Ok(false);
    }
    // *SAFETY* The `serde_json::to_string` function can safely be unwrapped because the `SocketResponse` enum
    // is serializable and will not panic
    // All responses should be serialized to JSON
    // and sent as Text
    sender
        .send(Message::Text(serde_json::to_string(&msg).unwrap()))
        .await?;
    Ok(true)
}
