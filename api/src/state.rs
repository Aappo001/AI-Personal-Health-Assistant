use std::{
    collections::HashSet,
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::Deref,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
};

use atomicbox::AtomicOptionBox;
use axum::extract::FromRef;
use chrono::DateTime;
use reqwest::{header, Client};
use scc::HashMap;
use sqlx::SqlitePool;
use tokio::{sync::mpsc, task::AbortHandle};

use crate::{chat::SocketResponse, IDLE_TIMEOUT};

/// The application state that is shared across all routes.
#[derive(Clone, Debug)]
pub struct AppState {
    /// This is a reqwest client that we use to make requests to the AI service.
    pub(crate) client: reqwest::Client,
    /// This is a channel that we can use to send messages to all connected clients on the same
    /// conversation.
    pub(crate) user_sockets: Arc<HashMap<i64, ConnectionState>>,
    /// Map of conversation ids to the (user_id, connection_id) of users
    /// who are focused on that conversation.
    /// Using a RwLock to allow multiple users to be focused on the same
    /// conversation without having to clone the underlying HashSet.
    pub(crate) conversation_connections: Arc<HashMap<i64, HashSet<Sender<SocketResponse>>>>,
    /// Connection pool to the database. We use a pool to handle multiple requests concurrently
    /// without having to create a new connection for each request.
    pub(crate) pool: SqlitePool,
    /// Stemmer for stemming all messages sent
    pub(crate) stemmer: Arc<Stemmer>,
    // Maybe add a `Arc<HashSet<i64>>` to keep track of the conversation ids
    // that the AI is currently generating messages for.
}

#[derive(Clone, Debug)]
pub struct Sender<T> {
    pub(crate) channel: mpsc::Sender<T>,
    pub(crate) user_id: Arc<i64>,
    pub conn_id: usize,
}

impl<T> Eq for Sender<T> {}

impl<T> PartialEq for Sender<T> {
    fn eq(&self, other: &Self) -> bool {
        self.channel.same_channel(&other.channel)
    }
}

impl<T> Hash for Sender<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.user_id.deref(), self.conn_id).hash(state);
    }
}

impl<T> Deref for Sender<T> {
    type Target = mpsc::Sender<T>;

    fn deref(&self) -> &Self::Target {
        &self.channel
    }
}

impl<T> Sender<T> {
    pub fn new(sender: mpsc::Sender<T>, user_id: i64, conn_id: usize) -> Self {
        Self {
            channel: sender,
            user_id: Arc::new(user_id),
            conn_id,
        }
    }
}

/// Wrapper around the `rust_stemmers::Stemmer` struct to allow it to be used in the `AppState`.
pub struct Stemmer(pub rust_stemmers::Stemmer);

impl Debug for Stemmer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Opaque Stemmer")
    }
}

/// Make `Stemmer` deref to `rust_stemmers::Stemmer` for easier access to the stemmer functions.
impl Deref for Stemmer {
    type Target = rust_stemmers::Stemmer;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Stemmer {
    /// Stems an entire message
    pub fn stem_message(&self, message: &str) -> String {
        message
            .to_lowercase()
            // Remove all punctuation so stems work properly
            .replace(['(', ')', ',', '\"', '.', ';', ':', '\'', '?', '!'], "")
            .split_whitespace()
            .map(|s| self.stem(s))
            .fold(String::new(), |mut acc, s| {
                acc.push_str(&s);
                acc.push(' ');
                acc
            })
    }
}

/// All the websocket connections for a user.
#[derive(Clone, Debug)]
pub struct ConnectionState {
    pub(crate) connections: [Option<InnerConnection>; 10],
    /// A flag that contains the conversation id of the conversation that the AI is currently
    /// generating messages for.
    /// This uses 0 as a sentinel value to represent that the AI is not currently generating
    /// responses since conversation ids are all greater than 0. This would've been better
    /// as an `Arc<Option<AtomicI64>>`, but that doesn't provide mutability. So the other
    /// option is to use `Arc<AtomicPtr<Option<i64>>>` but that requires unsafe code to
    /// manage the pointer.
    pub(crate) ai_responding: Arc<AtomicI64>,
    /// The timestamp of the last message recieved from any connection from the user over the
    /// websocket. Used to determine if the user is idle
    pub(crate) last_sent_at: Arc<AtomicI64>,
    /// The handle to the indle checking task
    /// Held in this struct so that any connection can cancel it, regardless of the connection that
    /// initiated the task
    pub(crate) idle_handle: Arc<AbortHandle>,
    pub(crate) ai_handle: Arc<AtomicOptionBox<AbortHandle>>,
}

impl ConnectionState {
    #[inline]
    pub fn is_idle(&self) -> bool {
        // Safe to unwrap since the timestamp is always set by Utc::now()
        (unsafe {
            DateTime::from_timestamp_millis(self.last_sent_at.load(Ordering::SeqCst))
                .unwrap_unchecked()
        }) + IDLE_TIMEOUT
            < chrono::Utc::now()
    }

    #[inline]
    pub fn update_last_sent(&self) {
        self.last_sent_at
            .store(chrono::Utc::now().timestamp_millis(), Ordering::SeqCst);
    }
}

/// The inner state of a user's connection to the server.
#[derive(Clone, Debug)]
pub struct InnerConnection {
    /// The sender channel for sending messages to the user.
    /// Each individual connection from the user has its own sender channel.
    /// Cap the number of connections to 10 to prevent abuse and simplify the implementation.
    pub(crate) channel: Sender<SocketResponse>,
    /// The id of the last conversation a user Requested using `SocketRequest::RequestConversation`
    /// This is assumed to be the last conversation the user was focused on.
    pub(crate) focused_conversation: Arc<AtomicI64>,
    pub(crate) focused_handle: Arc<AtomicOptionBox<AbortHandle>>,
}

impl AppState {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            client: reqwest::ClientBuilder::new()
                .default_headers({
                    let mut headers = reqwest::header::HeaderMap::new();
                    headers.insert(
                        header::CONTENT_TYPE,
                        "application/json"
                            .parse()
                            .expect("Failed to parse content type"),
                    );
                    headers
                })
                .build()
                .expect("Failed to build reqwest client"),
            user_sockets: Arc::new(HashMap::new()),
            conversation_connections: Arc::new(HashMap::new()),
            pool,
            stemmer: Arc::new(Stemmer(rust_stemmers::Stemmer::create(
                rust_stemmers::Algorithm::English,
            ))),
        }
    }
}

// Support for automatically converting an `AppState` into an `SqlitePool`
impl FromRef<AppState> for SqlitePool {
    fn from_ref(app_state: &AppState) -> SqlitePool {
        app_state.pool.clone()
    }
}

// Support for automatically converting an `AppState` into an `Client`
impl FromRef<AppState> for Client {
    fn from_ref(app_state: &AppState) -> Client {
        app_state.client.clone()
    }
}
