use std::{
    collections::HashSet,
    net::SocketAddr,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
};

use anyhow::anyhow;
use atomicbox::AtomicOptionBox;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, State,
    },
    http::{header::AUTHORIZATION, HeaderMap, HeaderValue, StatusCode},
    response::Response,
};
use base64::{engine::general_purpose, Engine};
use chrono::{DateTime, NaiveDateTime, Utc};
use futures::{
    future,
    stream::{FuturesUnordered, SplitSink},
    FutureExt, SinkExt, StreamExt, TryStreamExt,
};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, QueryBuilder, Sqlite, SqlitePool};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::{
    chat::{query_model, search::search_message, Conversation, ConversationUser},
    error::{AppError, ErrorResponse},
    state::{AppState, ConnectionState, InnerConnection, Sender},
    users::{authorize_user, UserToken},
    IDLE_TIMEOUT, MAX_MESSAGE_LEN,
};

use super::{
    create_conversation, search::SearchMessage, ChatMessage, DeleteMessage, ReadEvent,
    StreamMessage,
};

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
pub async fn init_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    mut headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<Response, AppError> {
    // Doing this header shinanigans because websockets are doodoo
    // #5 on https://stackoverflow.com/a/77060459 explains what's going on here
    let Some(protocol) = headers.get("sec-websocket-protocol") else {
        return Err(AppError::UserError((StatusCode::BAD_REQUEST, "No protocol provided\nPlease provide your authorization token as the second protocol in the list".into())));
    };
    let encoded_token = match protocol.to_str() {
        Ok(k) => k,
        Err(e) => {
            return Err(AppError::UserError((
                StatusCode::BAD_REQUEST,
                e.to_string().into(),
            )))
        }
    }
    .split(',')
    .map(|s| s.trim())
    .nth(1);
    let Some(auth_token) = encoded_token else {
        return Err(AppError::UserError((
            StatusCode::UNAUTHORIZED,
            "No authorization token provided".into(),
        )));
    };
    // Authorization token must be base64 encoded, since protocols ase not allowed to contain
    // certain characters which are present in JWTs
    // No padding must be used because "=" is not allowed in the protocol
    let auth_token = match general_purpose::STANDARD_NO_PAD.decode(auth_token) {
        Ok(k) => String::from_utf8(k)?,
        Err(e) => {
            return Err(AppError::UserError((
                StatusCode::BAD_REQUEST,
                e.to_string().into(),
            )))
        }
    };

    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_token)?);
    let user = authorize_user(&headers)?;

    info!("Received websocket connection from {}", addr);
    Ok(ws
        .protocols(["fakeProtocol"])
        .on_upgrade(|socket| handle_ws(socket, state, user)))
}

/// The types of responses from the socket
#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum SocketResponse {
    /// Message to be sent to the client
    /// This can either be a newly sent message
    /// or an edited message
    Message(ChatMessage),
    /// Conversation to be sent to the client
    Conversation(Conversation),
    /// The i64 is the id of the message to delete
    DeleteMessage(DeleteMessage),
    /// Stream data from the AI model
    StreamData(StreamMessage),
    /// Invite to a conversation
    #[serde(rename_all = "camelCase")]
    Invite {
        /// The id of the conversation the user was invited to
        conversation_id: i64,
        /// The id of the inviter
        inviter: i64,
        /// When the invite was created
        invited_at: NaiveDateTime,
    },
    /// Event to inform the client that a user has left a conversation
    #[serde(rename_all = "camelCase")]
    LeaveEvent { conversation_id: i64, user_id: i64 },
    /// Event to inform the client that a user renamed a conversation
    #[serde(rename_all = "camelCase")]
    RenameEvent {
        conversation_id: i64,
        user_id: i64,
        name: Option<String>,
    },
    /// Friend request to be sent to the client
    #[serde(rename_all = "camelCase")]
    FriendRequest {
        sender_id: i64,
        receiver_id: i64,
        created_at: chrono::NaiveDateTime,
        status: FriendRequestStatus,
    },
    #[serde(rename_all = "camelCase")]
    FriendData { id: i64, created_at: NaiveDateTime },
    /// Search results from a message query
    SearchMessage(ChatMessage),
    /// Error to inform the client
    Error(ErrorResponse),
    /// Read event to inform the client that messages before a given timestamp
    /// in a conversation were read by a user
    ReadEvent(ReadEvent),
    /// AI generation was canceled in the conversation
    #[serde(rename_all = "camelCase")]
    CanceledGeneration {
        conversation_id: i64,
        querier_id: i64,
    },
    /// A user's online status
    /// Emitted when a user's status has changed inside a focused conversation
    /// or when explicitly requested by the client
    #[serde(rename_all = "camelCase")]
    UserStatus { user_id: i64, status: OnlineStatus },
}

#[derive(Serialize, Clone, Debug)]
pub enum FriendRequestStatus {
    Pending,
    Accepted,
    Rejected,
}

#[derive(Serialize, Clone, Debug)]
pub enum OnlineStatus {
    Online,
    Idle,
    Offline,
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
    /// Deleted a message in the conversation
    #[serde(rename_all = "camelCase")]
    DeleteMessage { message_id: i64 },
    /// Send, accept, reject, or revoke a friend request
    // Put all the friend request stuff in one enum variant
    // so its easier to handle on the frontend
    #[serde(rename_all = "camelCase")]
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
    #[serde(rename_all = "camelCase")]
    InviteUsers {
        /// The id of the conversation to invite the users to
        /// if this is None, a new conversation will be created
        conversation_id: Option<i64>,
        /// The users being invited to the conversation
        invitees: Box<[i64]>,
    },
    /// Leave a conversation
    #[serde(rename_all = "camelCase")]
    LeaveConversation { conversation_id: i64 },
    /// Rename a conversation
    #[serde(rename_all = "camelCase")]
    RenameConversation {
        conversation_id: i64,
        /// The new name of the conversation
        /// If this is None, the frontend should fallback to listing the
        /// usernames of the users in the conversation
        name: Option<String>,
    },
    /// Request to search messages in given conversations
    /// that match the query string
    SearchMessages(SearchMessage),
    /// Messages have been read in given conversation
    /// Does not provide user_id because the user is already authenticated
    /// Does not provide timestamp because the server will set it
    #[serde(rename_all = "camelCase")]
    ReadMessage { conversation_id: i64 },
    /// Request the previous messages in the conversation
    /// Returns messages in order of most recent to least recent
    RequestMessages(RequestMessage),
    /// Request data on a conversation with the given id
    #[serde(rename_all = "camelCase")]
    RequestConversation { conversation_id: i64 },
    /// Request a stream of conversations the user is in
    /// Returns conversations in order of last message sent
    RequestConversations(RequestConversation),
    /// Request a stream of the user's friends
    RequestFriends,
    /// Request a stream of the user's friend requests
    RequestFriendRequests,
    /// Can be used to cancel an ongoing AI generation
    CancelGeneration,
}

/// A chat message sent by the client to the server
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SendMessage {
    /// The id of the conversation the message is being sent to
    /// If this is None, the client is sending the first message in a new conversation
    pub conversation_id: Option<i64>,
    pub message: Option<String>,
    /// The id of the model to query
    /// If this is none, the message will not be sent to the AI model
    pub ai_model_id: Option<i64>,
    /// Any attachments to the message
    pub attachment: Option<SendAttachment>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SendAttachment {
    pub id: i64,
    pub name: String,
}

/// Edit a message in the conversation
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct EditMessage {
    /// The id of the message to edit
    id: i64,
    /// The new content of the message
    message: String,
}

/// A request for the previous messages in a conversation
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
    #[serde(default)]
    pagination: Pagination,
}

#[derive(Deserialize, Debug, Default)]
enum Pagination {
    After,
    Around,
    #[default]
    Before,
}

/// A request for conversations the user is in
/// This api returns a stream of conversation the user is a part of
/// only the most recent conversations with an id are returned
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RequestConversation {
    /// The id of the last conversation the client received from the server
    /// If this is None, the client has not received any messages yet
    last_message_at: Option<NaiveDateTime>,
    /// The maximum number of messages to request
    /// If this is None, the client is requesting 50 conversations
    message_num: Option<i64>,
}

/// Checks if the user is idle and updates their status accordingly
/// Never returns unless the user disconnects
async fn idle_check(state: &AppState, user_id: i64) {
    let mut is_idle = false;
    // Time it takes for a user to be considered idle
    let last_sent_at = state
        .user_sockets
        .read_async(&user_id, |_, v| v.last_sent_at.clone())
        .await
        .expect("Connection should be initialized before this operation completes");
    loop {
        // This is safe to unwrap because the last_sent_at timestamp is always set
        // directly from Utc::now() which is guaranteed to be valid
        let last_sent_timestamp = unsafe {
            DateTime::from_timestamp_millis(last_sent_at.load(Ordering::SeqCst)).unwrap_unchecked()
        };
        // Timestamp at which the user would be considered idle without sending any messages over
        // the websocket
        let idle_timestamp = last_sent_timestamp + IDLE_TIMEOUT;
        let now = Utc::now();
        match (idle_timestamp - now).to_std() {
            Ok(sleep_duration) => {
                is_idle = false;
                // The user is not idle, so wait and then check again
                tokio::time::sleep(sleep_duration).await;
            }
            Err(_) => {
                if !is_idle {
                    // The user went from not idle to idle so update their status
                    let _ = emit_user_status(state, user_id, OnlineStatus::Idle).await;
                }
                is_idle = true;
                // The user is idle, so just wait for the idle duration before checking for any
                // activity again
                tokio::time::sleep(IDLE_TIMEOUT).await;
            }
        }
    }
}

/// Get  the online status of a user
pub async fn get_user_status(state: &AppState, user_id: i64) -> OnlineStatus {
    let Some(conn_state) = state
        .user_sockets
        .read_async(&user_id, |_, v| v.clone())
        .await
    else {
        return OnlineStatus::Offline;
    };

    // If all connections are None, the user is offline
    if conn_state.connections.iter().all(|x| x.is_none()) {
        return OnlineStatus::Offline;
    }

    // If the user has sent a message within the last IDLE_TIMEOUT duration, they are online, but
    // idle
    if conn_state.is_idle() {
        return OnlineStatus::Idle;
    }

    OnlineStatus::Online
}

/// Handles incoming websocket connections
pub async fn handle_ws(stream: WebSocket, state: AppState, user: UserToken) {
    let (mut sender, mut receiver) = stream.split();
    let user = Arc::new(user);

    // Create the connection state for the user
    let (tx, mut rx) = mpsc::channel(30);
    let mut connection = InnerConnection {
        channel: Sender::new(tx, user.id, 0),
        focused_conversation: Arc::new(AtomicI64::new(0)),
        focused_handle: Arc::new(AtomicOptionBox::none()),
    };

    let connection_id = match state.user_sockets.get_async(&user.id).await {
        // The user has other active connections
        Some(mut conn) => {
            let conn_id = match conn.connections.iter().position(|x| x.is_none()) {
                Some(k) => k,
                None => {
                    let _ = sender.close().await;
                    return;
                }
            };
            connection.channel.conn_id = conn_id;
            conn.connections[conn_id] = Some(connection.clone());
            conn_id
        }
        // First time the user has connected to the server
        None => {
            let mut connections = [const { None }; 10];
            connections[0] = Some(connection.clone());
            let _ = state
                .user_sockets
                .insert_async(
                    user.id,
                    ConnectionState {
                        connections: connections.clone(),
                        ai_responding: Arc::new(AtomicI64::new(0)),
                        ai_handle: Arc::new(AtomicOptionBox::none()),
                        last_sent_at: Arc::new(AtomicI64::new(Utc::now().timestamp_millis())),
                        idle_handle: Arc::new(
                            tokio::spawn({
                                let state = state.clone();
                                let user_id = user.id;
                                async move { idle_check(&state, user_id).await }
                            })
                            .abort_handle(),
                        ),
                    },
                )
                .await;

            // Attempt to let other users know that the user is online
            // Do it in a separate task so that the connection isn't blocked
            tokio::spawn({
                let state = state.clone();
                let user_id = user.id;
                async move { emit_user_status(&state, user_id, OnlineStatus::Online).await }
            });
            0
        }
    };

    let socket = state
        .user_sockets
        .read_async(&user.id, |_, v| v.clone())
        .await
        .unwrap();

    // Send messages to the client over the websocket
    // Messages are received from the broadcast channel
    let mut send_task = tokio::spawn({
        let user = user.clone();
        async move {
            // Keep checking for incoming messages and sending messages to the client accordingly
            // until the connection is closed
            while let Some(msg) = rx.recv().await {
                match send_message(&mut sender, msg, &user).await {
                    Ok(true) => (),
                    Ok(false) => {
                        let _ = sender.close().await;
                        break;
                    }
                    Err(e) => {
                        error!("Error sending message: {}", e);
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
        let connection = connection.clone();
        async move {
            // Keep receiving messages until the connection is closed
            while let Some(msg) = receiver.next().await {
                // Spawn a new task for each message received
                tokio::spawn({
                    let connection = connection.clone();
                    let user = user.clone();
                    let socket = socket.clone();
                    let state = state.clone();
                    async move {
                        match msg {
                            Ok(msg) => {
                                // Check if the user was idle and update their status so they are
                                // no longer idle
                                if socket.is_idle() {
                                    let _ = emit_user_status(&state, user.id, OnlineStatus::Online)
                                        .await;
                                }

                                // Update the timestamp of the last sent message for idle checking
                                socket.update_last_sent();
                                // Handle the received message
                                if let Err(e) =
                                    handle_message(msg, &state, &user, &socket, &connection).await
                                {
                                    error!("Error handling message: {}", e);
                                    let _ = connection
                                        .channel
                                        .send(SocketResponse::Error(e.into()))
                                        .await;
                                }
                            }
                            Err(e) => {
                                error!("Error receiving message: {}", e);
                            }
                        }
                    }
                });
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
        .user_sockets
        .entry_async(user.id)
        .await
        .and_modify(|entry| entry.connections[connection_id] = None);

    // Remove the current connection's focus from the conversation
    if let Some(mut set) = state
        .conversation_connections
        .get_async(&connection.focused_conversation.load(Ordering::Relaxed))
        .await
    {
        set.get_mut().remove(&connection.channel);
    }

    // Remove the user from the connection once all the tasks are
    // complete and all user devices have disconnected
    if let Some((_, conn)) = state
        .user_sockets
        .remove_if_async(&user.id, |v| v.connections.iter().all(|x| x.is_none()))
        .await
    {
        // Abort the idle checker since the user has no active connections to check for messages on
        conn.idle_handle.abort();
        // Attempt to let other users know that the user is offline
        let _ = emit_user_status(&state, user.id, OnlineStatus::Offline).await;
    }
}

/// Requests the most recent messages sent in a conversation before the given message id
/// A given id of None will return the most recent messages
async fn request_messages(
    pool: &SqlitePool,
    request: &RequestMessage,
    tx: &mpsc::Sender<SocketResponse>,
    user: &UserToken,
) -> Result<(), AppError> {
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
        return Err(AppError::UserError((
            StatusCode::FORBIDDEN,
            "User is not in the conversation".into(),
        )));
    }

    // Prevent the client from requesting more than 200 messages at a time
    let mut limit = request.message_num.unwrap_or(50).min(200);
    let message_id = request.message_id.unwrap_or(i64::MAX);

    // Messages should be returned in ascending order so that when the frontend
    // receives the messages, they are in the correct order
    let mut query = match request.pagination {
        // No need to order the messages in ascending order since they are already ordered properly
        Pagination::After => sqlx::query_as!(
            ChatMessage,
            r#"SELECT * FROM (
                        SELECT * FROM chat_messages WHERE conversation_id = ? AND id > ?
                        ORDER BY created_at ASC
                        LIMIT ?
                )
                ORDER BY created_at DESC"#,
            request.conversation_id,
            message_id,
            limit
        )
        .fetch(pool),
        // Same with before, but we want to get the messages before the provided message id
        Pagination::Around => {
            limit /= 2;
            sqlx::query_as!(
                ChatMessage,
                r#"SELECT * FROM (
                        SELECT * FROM chat_messages WHERE conversation_id = ? AND id >= ?
                        ORDER BY created_at ASC
                        LIMIT ?
                )
                UNION
                SELECT * FROM (
                        SELECT * FROM chat_messages WHERE conversation_id = ? AND id < ?
                        ORDER BY created_at DESC
                        LIMIT ?
                ) 
                ORDER BY created_at ASC"#,
                request.conversation_id,
                message_id,
                limit,
                request.conversation_id,
                message_id,
                limit,
            )
            .fetch(pool)
        }
        // Using a nested query to order the messages correctly
        // We need to order the messages in descending order first to get the most recently
        // sent messages after the provided message id.
        // Then we order the messages in ascending order to get the messages in the correct order
        Pagination::Before => sqlx::query_as!(
            ChatMessage,
            r#"SELECT * FROM (
                    SELECT * FROM chat_messages WHERE conversation_id = ? AND id < ?
                    ORDER BY created_at DESC
                    LIMIT ?
                ) 
                ORDER BY created_at ASC"#,
            request.conversation_id,
            message_id,
            limit
        )
        .fetch(pool),
    };

    while let Some(message) = query.next().await {
        tx.send(SocketResponse::Message(message?)).await?;
    }
    Ok(())
}

/// Emit a change in the user x's online status to users who are focused
/// on a conversation that user x is in
async fn emit_user_status(
    state: &AppState,
    user_id: i64,
    status: OnlineStatus,
) -> Result<(), AppError> {
    // Collect all of the conversations the user is in
    let conversations: HashSet<i64> = sqlx::query!(
        "SELECT DISTINCT conversation_id FROM user_conversations WHERE user_id = ?",
        user_id
    )
    .fetch(&state.pool)
    .map(|row| row.map(|x| x.conversation_id))
    .try_collect()
    .boxed()
    .await?;

    // Use a FuturesUnordered to collect all the futures
    // and concurrently poll all of them at once to prevent a lagging receiver
    // (slow user connection) from bottlenecking the entire process
    // For each conversation the user is in
    let outer_futures: FuturesUnordered<_> = conversations
        .iter()
        .map(|conversation| {
            let status = status.clone();
            async move {
                // Find the user_id and connection_id of connections
                // that are focused on the conversation
                let Some(connections) = state
                    .conversation_connections
                    .read_async(conversation, |_, v| v.clone())
                    .await
                else {
                    return;
                };
                // Use a second, nested FuturesUnordered to manage sending the messages to all
                // users focused on a given conversation concurrently
                let mut inner_futures: FuturesUnordered<_> = connections
                    .iter()
                    .map(|sender| {
                        sender.send(SocketResponse::UserStatus {
                            user_id,
                            status: status.clone(),
                        })
                    })
                    .collect();
                while let Some(result) = inner_futures.next().await {
                    if let Err(e) = result {
                        warn!("Error sending user status: {}", e);
                    }
                }
            }
        })
        .collect();

    // Wait for all the contained futures to complete
    // Collecting the futures into a Vec of unit structs will not allocate memory
    // so this is just more idiomatic than using `while let Some(...) = ...`
    // See https://doc.rust-lang.org/std/vec/struct.Vec.html#guarantees
    outer_futures.collect::<Vec<()>>().await;
    Ok(())
}

/// Save a message to the database
async fn save_message(
    state: &AppState,
    message: &SendMessage,
    user: &UserToken,
) -> Result<ChatMessage, AppError> {
    // If the conversation_id is None, this is the first message in a conversation
    // so create a new conversation and get the id
    let stemmed_message = match (&message.message, &message.attachment) {
        // The message does not contain any content
        (None, None) => {
            return Err(AppError::UserError((
                StatusCode::BAD_REQUEST,
                "Message cannot be empty".into(),
            )))
        }
        // Check if the message is too long and stem it if it is not
        (Some(message_content), _) => {
            if message_content.chars().count() > MAX_MESSAGE_LEN {
                return Err(AppError::UserError((
                    StatusCode::BAD_REQUEST,
                    "Message too long".into(),
                )));
            }
            Some(state.stemmer.stem_message(message_content))
        }
        _ => None,
    };

    let conversation_id = match message.conversation_id {
        Some(k) => k,
        None => create_conversation(&state.pool, message, user).await?.id,
    };

    if sqlx::query!(
        "SELECT conversation_id FROM user_conversations WHERE conversation_id = ? and user_id = ?",
        conversation_id,
        user.id
    )
    .fetch_optional(&state.pool)
    .await?
    .is_none()
    {
        return Err(AppError::UserError((
            StatusCode::FORBIDDEN,
            "User is not in the conversation".into(),
        )));
    }

    if let Some(attachment) = &message.attachment {
        sqlx::query!(
            "SELECT file_id FROM file_uploads WHERE file_id = ? and user_id = ?",
            attachment.id,
            user.id
        )
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| anyhow!("Image not found"))?;
    }

    let message_id = match &message.attachment {
        Some(attachment) => {
            sqlx::query!(
                "INSERT INTO messages (user_id, conversation_id, message, stemmed_message, file_id, file_name) VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
                user.id,
                conversation_id,
                message.message,
                stemmed_message,
                attachment.id,
                attachment.name,
            )
            .fetch_one(&state.pool)
            .await?.id
        },
        None => {
            sqlx::query!(
                "INSERT INTO messages (user_id, conversation_id, message, stemmed_message) VALUES (?, ?, ?, ?) RETURNING id",
                user.id,
                conversation_id,
                message.message,
                stemmed_message
            )
            .fetch_one(&state.pool)
            .await?.id
        }
    };

    Ok(sqlx::query_as!(
        ChatMessage,
        "SELECT * FROM chat_messages WHERE id = ?",
        message_id
    )
    .fetch_one(&state.pool)
    .await?)
}

/// Edit message in the database
async fn edit_message(
    state: &AppState,
    message: &EditMessage,
    user: &UserToken,
) -> Result<ChatMessage, AppError> {
    // Check if the message exists in the database
    let Some(message_user) = sqlx::query!("SELECT user_id FROM messages WHERE id = ?", message.id)
        .fetch_optional(&state.pool)
        .await?
    else {
        return Err(AppError::UserError((
            StatusCode::FORBIDDEN,
            "Message not found".into(),
        )));
    };

    // Check if the user has permission to edit the message
    if message_user.user_id != Some(user.id) {
        return Err(AppError::UserError((
            StatusCode::FORBIDDEN,
            "User does not have permission to edit message".into(),
        )));
    }

    let stemmed_message = state.stemmer.stem_message(&message.message);

    // Update the message in the database
    // We know the message exists so we can just use `fetch_one`
    sqlx::query!(
        "UPDATE messages SET message = ?, stemmed_message = ? WHERE id = ?",
        message.message,
        stemmed_message,
        message.id
    )
    .execute(&state.pool)
    .await?;

    Ok(sqlx::query_as!(
        ChatMessage,
        "SELECT * FROM chat_messages WHERE id = ?",
        message.id
    )
    .fetch_one(&state.pool)
    .await?)
}

/// Delete a message in the database
async fn delete_message(
    pool: &SqlitePool,
    message_id: i64,
    user: &UserToken,
) -> Result<DeleteMessage, AppError> {
    // Check if the message exists in the database
    let Some(message) = sqlx::query!("SELECT id, user_id FROM messages WHERE id = ?", message_id)
        .fetch_optional(pool)
        .await?
    else {
        return Err(AppError::UserError((
            StatusCode::BAD_REQUEST,
            "Message not found".into(),
        )));
    };
    // Check if the user has permission to delete the message
    if message.user_id != Some(user.id) {
        return Err(AppError::UserError((
            StatusCode::FORBIDDEN,
            "User does not have permission to delete message".into(),
        )));
    }
    // Delete the message from the database
    Ok(sqlx::query_as!(
        DeleteMessage,
        "DELETE FROM messages WHERE id = ? RETURNING id as message_id, conversation_id",
        message.id
    )
    .fetch_one(pool)
    .await?)
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
    // Check that the users are not already friends
    let (user1_id, user2_id) = match user.id.cmp(&other_user_id) {
        std::cmp::Ordering::Less => (user.id, other_user_id),
        std::cmp::Ordering::Greater => (other_user_id, user.id),
        std::cmp::Ordering::Equal => {
            return Err(AppError::UserError((
                StatusCode::FORBIDDEN,
                "User cannot send friend request to themselves".into(),
            )))
        }
    };

    if sqlx::query!(
        "SELECT user1_id FROM friendships WHERE user1_id = ? and user2_id = ?",
        user1_id,
        user2_id
    )
    .fetch_optional(&state.pool)
    .await?
    .is_some()
    {
        return Err(AppError::UserError((
            StatusCode::FORBIDDEN,
            "Users are already friends".into(),
        )));
    }

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
            return Err(AppError::UserError((
                StatusCode::CONFLICT,
                "Friend request already exists".into(),
            )));
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
            return Err(AppError::UserError((StatusCode::NOT_FOUND, "Friend request does not exist".into())));
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
    if let Some(receiver_connections) = state
        .user_sockets
        .read_async(&other_user_id, |_, v| v.connections.clone())
        .await
    {
        for conn in receiver_connections.iter().flatten() {
            conn.channel.send(friend_request.clone()).await?;
        }
    }

    // Send the friend request over the websocket to the sender
    // to let them know that the friend request was sent successfully
    if let Some(sender_connections) = state
        .user_sockets
        .read_async(&user.id, |_, v| v.connections.clone())
        .await
    {
        for conn in sender_connections.iter().flatten() {
            conn.channel.send(friend_request.clone()).await?;
        }
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
                return Err(AppError::UserError((StatusCode::FORBIDDEN, "Inviter is not in the conversation".into())));
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

    // Build a query to check if all the users being invited exist
    // Final query will look like this: SELECT COUNT(id) FROM users WHERE id IN (?, ?, ?)
    let mut query_builder: QueryBuilder<'_, Sqlite> =
        QueryBuilder::new("SELECT COUNT(id) FROM users WHERE id IN (");

    let mut separated = query_builder.separated(", ");
    for invitee in invitees {
        separated.push_bind(invitee);
    }
    // Use query_scalar to extract the value of the first column, COUNT(id) in this case,
    // as a single value
    let query = query_builder.push(")").build_query_scalar::<u64>();
    let num_rows: usize = query.fetch_one(pool).await? as usize;

    // If the number of rows returned is not equal to the number of users being invited
    // then at least one user does not exist
    if num_rows != invitees.len() {
        return Err(AppError::UserError((
            StatusCode::NOT_FOUND,
            "One or more users do not exist".into(),
        )));
    }

    // Use a query builder to invite all the users at once instead of multiple
    // queries in a loop for significantly better performance
    // Can't use the query! macro because it doesn't support bulk inserts
    // Final query will look like this:
    // INSERT INTO user_conversations (user_id, conversation_id)
    // VALUES (?, ?), (?, ?), (?, ?) ON CONFLICT DO NOTHING
    let mut query_builder: QueryBuilder<'_, Sqlite> =
        QueryBuilder::new("INSERT INTO user_conversations (user_id, conversation_id) ");

    // Pushes a VALUES clause with the user_id and conversation_id for each user
    query_builder.push_values(invitees, |mut builder, invitee| {
        builder.push_bind(invitee).push_bind(conversation_id);
    });

    query_builder.push(" ON CONFLICT DO NOTHING");

    let query = query_builder.build();
    query.execute(pool).await?;
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
    state: &AppState,
    user: &UserToken,
    socket: &ConnectionState,
    inner: &InnerConnection,
) -> Result<(), AppError> {
    match msg {
        Message::Text(text) => {
            let msg: SocketRequest = sonic_rs::from_str(&text)?;
            info!("Received {:?}", msg);
            match msg {
                // mmmm spaghetti code branch yummy
                SocketRequest::SendMessage(mut send_message) => {
                    // Check if there is an AI generation in progress started by the user in the
                    // same conversation and prevent them from sending a new message if there is
                    if send_message
                        .conversation_id
                        .is_some_and(|id| id == socket.ai_responding.load(Ordering::SeqCst))
                    {
                        return Err(AppError::UserError((StatusCode::TOO_MANY_REQUESTS, "AI generation is already in progress. Please cancel generation or wait before making another query".into())));
                    }

                    let chat_message = match (&send_message.message, &send_message.attachment) {
                        (None, None) => None,
                        // Only save the message if it is not empty
                        _ => {
                            let chat_message = save_message(state, &send_message, user).await?;
                            send_message.conversation_id = Some(chat_message.conversation_id);
                            Some(chat_message)
                        }
                    };

                    // Broadcast the message in a separate task to prevent the
                    // current thread from being blocked by the broadcast
                    tokio::spawn({
                        let state = state.clone();
                        let chat_message = chat_message.clone();
                        async move {
                            // Only broadcast the message if it is not empty
                            if let Some(chat_message) = chat_message {
                                let _ = broadcast_event(
                                    &state,
                                    SocketResponse::Message(chat_message.clone()),
                                )
                                .await;
                            }
                        }
                    });

                    // Check if the user is attempting to query the model,
                    // if they aren't then we can return early
                    let Some(ai_model_id) = send_message.ai_model_id else {
                        return Ok(());
                    };

                    // The user is explicitly trying to query the model, so check if there is
                    // already an AI generation in progress in any conversation they are a part of
                    // and prevent them from starting a new one
                    if socket.ai_responding.load(Ordering::SeqCst) != 0 {
                        return Err(AppError::UserError((StatusCode::TOO_MANY_REQUESTS, "AI generation is already in progress. Please cancel generation or wait before making another query".into())));
                    }

                    socket.ai_responding.store(
                        send_message.conversation_id.ok_or(AppError::UserError((
                            StatusCode::BAD_REQUEST,
                            "Cannot send ai message in non-existant conversation!".into(),
                        )))?,
                        Ordering::SeqCst,
                    );

                    // Spawn the AI response generation in a separate task to allow cancellation
                    // by another message from the user
                    let handle = tokio::spawn({
                        let state = state.clone();
                        let send_message = send_message.clone();
                        let user = user.clone();
                        async move { query_model(&state, &send_message, &user).await }
                    });

                    // Save an abort handle to the thread in the connection state of the user
                    // to allow another thread to abort the AI generation if requested by the user
                    socket
                        .ai_handle
                        .store(Some(Box::new(handle.abort_handle())), Ordering::SeqCst);

                    // This will be Ok() if the AI response generation was not canceled
                    // If it was canceled then we can just reset the value of the responding
                    // conversation and return early
                    let Ok(ai_message) = handle.await else {
                        socket.ai_responding.store(0, Ordering::SeqCst);
                        return Ok(());
                    };

                    // Reset the AI generation flag to 0 to allow the user to query the model again
                    // Must be done inside this block to prevent the flage from being reset if the user sends another message
                    // before the AI model is finished responding or canceled
                    socket.ai_responding.store(0, Ordering::SeqCst);

                    let ai_message = ai_message?;
                    let stemmed_message = state.stemmer.stem_message(&ai_message);

                    // Save the AI model's response to the database
                    // This is done outside of the `query_model` function to
                    // prevent the message from being lost if the user cancels
                    // the AI generation while writing to the database
                    let message = sqlx::query!(
                            "INSERT INTO messages (conversation_id, message, stemmed_message, ai_model_id) VALUES (?, ?, ?, ?) RETURNING id",
                            send_message.conversation_id,
                            ai_message,
                            stemmed_message,
                            ai_model_id
                        )
                        .fetch_one(&state.pool)
                        .await?.id;

                    let ai_message = sqlx::query_as!(
                        ChatMessage,
                        "SELECT * FROM chat_messages WHERE id = ?",
                        message
                    )
                    .fetch_one(&state.pool)
                    .await?;

                    // Broadcast the AI model's response to the conversation
                    broadcast_event(state, SocketResponse::Message(ai_message)).await?;
                }
                SocketRequest::EditMessage(chat_message) => {
                    let chat_message = edit_message(state, &chat_message, user).await?;
                    // Broadcast the edited message to all the users in the conversation
                    broadcast_event(state, SocketResponse::Message(chat_message.clone())).await?;
                }
                SocketRequest::DeleteMessage { message_id } => {
                    let deleted_message = delete_message(&state.pool, message_id, user).await?;
                    // Broadcast the deleted message to all the users in the conversation
                    broadcast_event(state, SocketResponse::DeleteMessage(deleted_message)).await?;
                }
                SocketRequest::InviteUsers {
                    invitees,
                    mut conversation_id,
                } => {
                    if invitees.is_empty() {
                        return Err(AppError::UserError((
                            StatusCode::BAD_REQUEST,
                            "No users to invite".into(),
                        )));
                    }

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
                SocketRequest::ReadMessage { conversation_id } => {
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
                    request_messages(&state.pool, &request_message, &inner.channel, user).await?;
                }
                SocketRequest::RequestConversation { conversation_id } => {
                    // Get the converation and all of the users inside the conversation in the same
                    // query to minimize the number of database queries
                    let mut query =  sqlx::query!(
                        "SELECT id, title, conversations.created_at, conversations.last_message_at, user_id, user_conversations.last_message_at as user_last_message_at, last_read_at FROM conversations
                        JOIN user_conversations
                        ON conversations.id = user_conversations.conversation_id
                        WHERE conversation_id = ?",
                        conversation_id,
                    ).fetch_all(&state.pool).await?;

                    // Check if the user is in the conversation
                    // Using `iter_mut` instead of iter because we need to take the title
                    // out of the conversation and send it to the client
                    match query.iter_mut().find(|row| row.user_id == user.id) {
                        Some(conversation) => {
                            inner
                                .channel
                                .send(SocketResponse::Conversation(Conversation {
                                    id: conversation.id,
                                    created_at: conversation.created_at,
                                    last_message_at: conversation.last_message_at,
                                    // Have to take the title because we can't move it from the row
                                    // and cloning is more expensive than taking
                                    title: conversation.title.take(),
                                    users: Some(
                                        future::join_all(query.iter().map(|u| async {
                                            ConversationUser {
                                                id: u.user_id,
                                                last_message_at: u.user_last_message_at,
                                                last_read_at: u.last_read_at,
                                                online_status: Some(
                                                    get_user_status(state, u.user_id).await,
                                                ),
                                            }
                                        }))
                                        .await
                                        .into(),
                                    ),
                                }))
                                .await?;
                        }
                        None => {
                            return Err(AppError::UserError((
                                StatusCode::FORBIDDEN,
                                "User is not the conversation".into(),
                            )))
                        }
                    }

                    // Cancel focusing a prevoius conversation if the user switches to another conversation
                    // Sent in case the user switches to another conversation before the update finishes
                    if let Some(handle) = inner.focused_handle.take(Ordering::SeqCst) {
                        handle.abort();
                    }

                    // Update the focused conversation for the current connect
                    // after sending the conversation data to prevent blocking
                    // the connection
                    let last_focused_conversation =
                        inner.focused_conversation.load(Ordering::SeqCst);
                    // No need to update the focused conversation for the current connection
                    if last_focused_conversation == conversation_id {
                        return Ok(());
                    }

                    // Use `tokio::select!` to allow cancelling the focus event
                    // if the user switches to another conversation before the update finishes
                    // which can ocur if the user switches to another conversation before
                    // the attempt to get the map bucket is blocking.
                    // Using canceling the focus event instead of aborting the task
                    let handle = tokio::spawn({
                        let state = state.clone();
                        let inner = inner.clone();
                        async move {
                            // This implementation is not atomic. This means that a conversation
                            // could be removed without another one being added. Ideally
                            // this would be done in a single atomic operation but
                            // all attempts I've tried have caused deadlocks or just
                            // straight up don't work.

                            // Remove the user from the previous conversation if they were in one
                            if last_focused_conversation != 0 {
                                if let Some(mut set) = state
                                    .conversation_connections
                                    .get_async(&last_focused_conversation)
                                    .await
                                {
                                    set.get_mut().remove(&inner.channel);
                                    if set.is_empty() {
                                        // Drop the set to prevent deadlock
                                        // This will deadlock if the set is not dropped
                                        drop(set);
                                        state
                                            .conversation_connections
                                            .remove(&last_focused_conversation);
                                    }
                                }
                            }

                            // Update the focused conversation for the current connection
                            inner
                                .focused_conversation
                                .store(conversation_id, Ordering::SeqCst);

                            // Insert the user into the new conversation
                            match state
                                .conversation_connections
                                .get_async(&conversation_id)
                                .await
                            {
                                Some(mut entry) => {
                                    entry.get_mut().insert(inner.channel.clone());
                                }
                                None => {
                                    let _ = state
                                        .conversation_connections
                                        .insert_async(
                                            conversation_id,
                                            HashSet::from([inner.channel.clone()]),
                                        )
                                        .await;
                                }
                            }
                        }
                    });

                    inner
                        .focused_handle
                        .store(Some(Box::new(handle.abort_handle())), Ordering::SeqCst);
                }
                SocketRequest::RequestConversations(request_message) => {
                    let limit = request_message.message_num.unwrap_or(50);
                    let last_message_at = request_message
                        .last_message_at
                        .unwrap_or(NaiveDateTime::MAX);
                    // Create a helper to map rows to conversation struct easier
                    // Have to use an unchecked query as a workaround because sqlx has a bug where
                    // aggregate functions return the wrong type.
                    // Reference Issue: https://github.com/launchbadge/sqlx/issues/3238
                    // For example in this scenario, GROUP_CONCAT(user_id) should return a string
                    // but sqlx parses it as a i64, preventing us from using it in the struct
                    #[derive(FromRow)]
                    struct ConversationHelper {
                        id: i64,
                        title: Option<String>,
                        created_at: NaiveDateTime,
                        last_message_at: Option<NaiveDateTime>,
                        users: String,
                    }

                    // Query the database for the conversations the user is in
                    // Use fetch instead of fetch all to stream results to the client
                    let mut query = sqlx::query_as::<Sqlite, ConversationHelper>(
                        r#"SELECT conversations.*, GROUP_CONCAT(user_id) as users FROM conversations
                           JOIN user_conversations 
                           ON conversations.id = user_conversations.conversation_id 
                           WHERE id IN 
                           (SELECT id FROM conversations
                           JOIN user_conversations
                           ON conversations.id = user_conversations.conversation_id
                           WHERE user_id = ? AND conversations.last_message_at > ?
                           ORDER BY conversations.last_message_at DESC
                           LIMIT ?) 
                           GROUP BY id"#,
                    )
                    .bind(user.id)
                    .bind(last_message_at)
                    .bind(limit)
                    .fetch(&state.pool);

                    while let Some(conversation) = query.next().await {
                        let conversation = conversation?;
                        inner
                            .channel
                            .send(SocketResponse::Conversation(Conversation {
                                id: conversation.id,
                                title: conversation.title,
                                created_at: conversation.created_at,
                                last_message_at: conversation.last_message_at,
                                users: Some(
                                    conversation
                                        .users
                                        .split(',')
                                        .map(|u| ConversationUser {
                                            id: u.parse::<i64>().unwrap(),
                                            ..Default::default()
                                        })
                                        .collect(),
                                ),
                            }))
                            .await?;
                    }
                }
                SocketRequest::RequestFriends => {
                    let mut query = sqlx::query!(
                        "SELECT * FROM friendships WHERE user1_id = ? OR user2_id = ?",
                        user.id,
                        user.id
                    )
                    .fetch(&state.pool);
                    while let Some(friendship) = query.next().await {
                        let friendship = friendship?;
                        let friend_id = if friendship.user1_id == user.id {
                            friendship.user2_id
                        } else {
                            friendship.user1_id
                        };
                        inner
                            .channel
                            .send(SocketResponse::FriendData {
                                id: friend_id,
                                created_at: friendship.created_at,
                            })
                            .await?;
                    }
                }
                SocketRequest::RequestFriendRequests => {
                    let mut query = sqlx::query!(
                        "SELECT * FROM friend_requests WHERE sender_id = ? OR receiver_id = ?",
                        user.id,
                        user.id
                    )
                    .fetch(&state.pool);

                    while let Some(friend_request) = query.next().await {
                        let friend_request = friend_request?;
                        inner
                            .channel
                            .send(SocketResponse::FriendRequest {
                                sender_id: friend_request.sender_id,
                                receiver_id: friend_request.receiver_id,
                                created_at: friend_request.created_at,
                                status: FriendRequestStatus::Pending,
                            })
                            .await?;
                    }
                }
                SocketRequest::CancelGeneration => {
                    // Use 0 as a sentinel value to indicate that the AI generation
                    // is not running for the current user
                    let conversation_id = socket.ai_responding.load(Ordering::SeqCst);
                    if conversation_id == 0 {
                        inner
                            .channel
                            .send(SocketResponse::Error(
                                AppError::UserError((
                                    StatusCode::BAD_REQUEST,
                                    "No ai response to cancel".into(),
                                ))
                                .into(),
                            ))
                            .await?;
                        return Ok(());
                    }

                    match socket.ai_handle.take(Ordering::SeqCst) {
                        Some(handle) => {
                            // Abort the ongoing AI generation task
                            handle.abort();
                            // Broadcast the cancellation of the AI generation
                            broadcast_event(
                                state,
                                SocketResponse::CanceledGeneration {
                                    conversation_id,
                                    querier_id: user.id,
                                },
                            )
                            .await?;
                        }
                        None => {
                            inner
                                .channel
                                .send(SocketResponse::Error(
                                    AppError::UserError((
                                        StatusCode::BAD_REQUEST,
                                        "No ai response to cancel".into(),
                                    ))
                                    .into(),
                                ))
                                .await?;
                        }
                    }
                }
                SocketRequest::SearchMessages(message) => {
                    search_message(state, &message, &inner.channel).await?;
                }
                SocketRequest::LeaveConversation { conversation_id } => {
                    // Remove the user from the conversation
                    leave_conversation(&state.pool, conversation_id, user.id).await?;

                    let leave_event = SocketResponse::LeaveEvent {
                        conversation_id,
                        user_id: user.id,
                    };

                    // Send the leave event back to the user explicitly
                    // to let them know that they have left the conversation since
                    // `broadcast_event` will not send events to the user that left
                    for connection in socket.connections.iter().flatten() {
                        connection.channel.send(leave_event.clone()).await?;
                    }

                    // Broadcast the user leaving the conversation to all the remaining users in the conversation
                    broadcast_event(state, leave_event).await?;
                }
                SocketRequest::RenameConversation {
                    conversation_id,
                    name,
                } => {
                    rename_conversation(&state.pool, conversation_id, &name, user).await?;
                    broadcast_event(
                        state,
                        SocketResponse::RenameEvent {
                            conversation_id,
                            name,
                            user_id: user.id,
                        },
                    )
                    .await?;
                }
            }
        }
        Message::Binary(_) => {
            //TODO
        }
        // We do not need to handle ping or close messages
        // because tokio_tungstenite will handle them for us
        #[allow(clippy::wildcard_in_or_patterns)]
        Message::Ping(_) | Message::Close(_) | _ => (),
    }
    Ok(())
}

/// Broadcast an event to all the users in a conversation
/// Events include messages, edits, and deletes, ect.
pub async fn broadcast_event(state: &AppState, msg: SocketResponse) -> Result<(), AppError> {
    let id = match &msg {
        SocketResponse::Message(chat_msg) => chat_msg.conversation_id,
        SocketResponse::DeleteMessage(delete_msg) => delete_msg.conversation_id,
        SocketResponse::ReadEvent(event) => event.conversation_id,
        SocketResponse::StreamData(data) => data.conversation_id,
        SocketResponse::LeaveEvent {
            conversation_id, ..
        } => *conversation_id,
        SocketResponse::Invite {
            conversation_id, ..
        } => *conversation_id,
        SocketResponse::CanceledGeneration {
            conversation_id, ..
        } => *conversation_id,
        SocketResponse::RenameEvent {
            conversation_id, ..
        } => *conversation_id,
        _ => unreachable!("uuhhh how"),
    };
    let users = sqlx::query!(
        "SELECT user_id FROM user_conversations WHERE conversation_id = ?",
        id
    )
    .fetch_all(&state.pool)
    .await?;

    // Use `join_all` to broadcast the message to all the users in the conversation
    // concurrently to minimize the time it takes to broadcast the message
    let inner = future::join_all(users.into_iter().map(|user| async move {
        state
            .user_sockets
            .read_async(&user.user_id, |_, v| v.connections.clone())
            .await
    }))
    .await;

    let mut unordered: FuturesUnordered<_> = inner
        .iter()
        .flatten()
        .flatten()
        .flatten()
        .map(|connection| connection.channel.send(msg.clone()))
        .collect();

    while let Some(fut) = unordered.next().await {
        if let Err(e) = fut {
            warn!("Error broadcasting event: {}", e);
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
    // All responses should be serialized to JSON
    // and sent as Text
    sender
        .send(Message::Text(sonic_rs::to_string(&msg).unwrap()))
        .await?;
    Ok(true)
}

/// Removes a user from a conversation
/// If the conversation has no users left, it is also deleted
async fn leave_conversation(
    pool: &SqlitePool,
    conversation_id: i64,
    user_id: i64,
) -> Result<(), AppError> {
    // Remove the user from the conversation
    let query = sqlx::query!(
        "DELETE FROM user_conversations WHERE user_id = ? and conversation_id = ?",
        user_id,
        conversation_id
    )
    .execute(pool)
    .await?;

    if query.rows_affected() == 0 {
        return Err(AppError::UserError((
            StatusCode::FORBIDDEN,
            "User is not in the conversation".into(),
        )));
    }

    // Check how many users are left in the conversation
    let remaining_users = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM user_conversations WHERE conversation_id = ?",
        conversation_id
    )
    .fetch_one(pool)
    .await?;

    // Check if the conversation has no users left and delete it if it does
    if remaining_users == 0 {
        sqlx::query!("DELETE FROM conversations WHERE id = ?", conversation_id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

/// Renames a conversation
async fn rename_conversation(
    pool: &SqlitePool,
    conversation_id: i64,
    name: &Option<String>,
    user: &UserToken,
) -> Result<(), AppError> {
    if sqlx::query!(
        "SELECT user_id FROM user_conversations WHERE conversation_id = ? and user_id = ?",
        conversation_id,
        user.id
    )
    .fetch_optional(pool)
    .await?
    .is_none()
    {
        return Err(AppError::UserError((
            StatusCode::FORBIDDEN,
            "User is not in the conversation".into(),
        )));
    }
    sqlx::query!(
        "UPDATE conversations SET title = ? WHERE id = ?",
        name,
        conversation_id
    )
    .execute(pool)
    .await?;
    Ok(())
}
