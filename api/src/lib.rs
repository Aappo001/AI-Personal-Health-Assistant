pub mod auth;
/// Contains the logic for the chat side of the application. Including the routes for creating a
/// conversation, getting a conversation, and connecting to a websocket for chatting.
pub mod chat;
/// Contains the logic for the command line interface (CLI) of the application.
pub mod cli;
/// Contains the error type and error handling logic for the application.
pub mod error;
/// Contains logic for processing user forms saving them to the database as statistics.
pub mod forms;
pub mod report;
/// Contains logic for uploading files to the server.
pub mod upload;
/// Contains the logic for the users side of the application. Including the routes for creating a
/// user, authenticating a user, and getting a user's profile.
pub mod users;
/// Contains utility functions that are used throughout the application.
pub mod utils;

use anyhow::Result;
use axum::{
    extract::{DefaultBodyLimit, FromRef},
    http::{HeaderName, HeaderValue},
    routing::{delete, get, post, put},
    Router,
};
use forms::{get_forms, get_health_form, save_health_form, update_health_form};
use report::generate_pdf_report;
use reqwest::{
    header::{self, CONTENT_ENCODING, CONTENT_LENGTH},
    Client,
};
use std::{
    collections::HashSet,
    fmt::Debug,
    hash::{Hash, Hasher},
    net::SocketAddr,
    ops::Deref,
    str::FromStr,
    sync::{atomic::AtomicI64, Arc},
    time::Duration,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::{self, AllowOrigin, CorsLayer},
    services::{ServeDir, ServeFile},
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit, ServiceBuilderExt,
};

use chat::{create_conversation_rest, get_ai_models, get_conversation, init_ws, SocketResponse};
use cli::Args;
use scc::HashMap;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous},
    SqlitePool,
};
use tokio::{net::TcpListener, sync::broadcast, task::JoinHandle};
use tracing::info;
use upload::{upload_file, upload_profile_image};
use users::{
    authenticate_user, check_email, check_username, create_user, delete_user, get_settings,
    get_user_by_id, get_user_by_username, get_user_from_token, search_users, update_settings,
    update_user,
};

/// The name of the package. This is defined in the `Cargo.toml` file.
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

/// The protocol for connecting to a SQLite database.
#[cfg(windows)]
pub const PROTOCOL: &str = "sqlite:///";

/// The protocol for connecting to a SQLite database.
#[cfg(unix)]
pub const PROTOCOL: &str = "sqlite://";

pub const IDLE_TIMEOUT: Duration = Duration::from_secs(5 * 60);

/// The application state that is shared across all routes.
#[derive(Clone, Debug)]
pub struct AppState {
    /// This is a reqwest client that we use to make requests to the AI service.
    client: reqwest::Client,
    /// This is a channel that we can use to send messages to all connected clients on the same
    /// conversation.
    user_sockets: Arc<HashMap<i64, ConnectionState>>,
    /// Map of conversation ids to the (user_id, connection_id) of users
    /// who are focused on that conversation.
    /// Using a RwLock to allow multiple users to be focused on the same
    /// conversation without having to clone the underlying HashSet.
    conversation_connections: Arc<HashMap<i64, HashSet<Sender<SocketResponse>>>>,
    /// Connection pool to the database. We use a pool to handle multiple requests concurrently
    /// without having to create a new connection for each request.
    pool: SqlitePool,
    /// Stemmer for stemming all messages sent
    stemmer: Arc<Stemmer>,
    // Maybe add a `Arc<HashSet<i64>>` to keep track of the conversation ids
    // that the AI is currently generating messages for.
}

#[derive(Clone, Debug)]
pub struct Sender<T> {
    channel: broadcast::Sender<T>,
    user_id: Arc<i64>,
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
    type Target = broadcast::Sender<T>;

    fn deref(&self) -> &Self::Target {
        &self.channel
    }
}

impl<T> Sender<T> {
    pub fn new(sender: broadcast::Sender<T>, user_id: i64, conn_id: usize) -> Self {
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
    connections: [Option<InnerConnection>; 10],
    /// A flag that contains the conversation id of the conversation that the AI is currently
    /// generating messages for.
    /// This uses 0 as a sentinel value to represent that the AI is not currently generating
    /// responses since conversation ids are all greater than 0. This would've been better
    /// as an `Arc<Option<AtomicI64>>`, but that doesn't provide mutability. So the other
    /// option is to use `Arc<AtomicPtr<Option<i64>>>` but that requires unsafe code to
    /// manage the pointer.
    ai_responding: Arc<AtomicI64>,
    /// The timestamp of the last message recieved from any connection from the user over the
    /// websocket. Used to determine if the user is idle
    last_sent_at: Arc<AtomicI64>,
    /// The handle to the indle checking task
    /// Held in this struct so that any connection can cancel it, regardless of the connection that
    /// initiated the task
    idle_handle: Arc<JoinHandle<()>>,
}

/// The inner state of a user's connection to the server.
#[derive(Clone, Debug)]
pub struct InnerConnection {
    /// The sender channel for sending messages to the user.
    /// Each individual connection from the user has its own sender channel.
    /// Cap the number of connections to 10 to prevent abuse and simplify the implementation.
    channel: Sender<SocketResponse>,
    /// The id of the last conversation a user Requested using `SocketRequest::RequestConversation`
    /// This is assumed to be the last conversation the user was focused on.
    focused_conversation: Arc<AtomicI64>,
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

/// Start the server and listen for incoming connections.
pub async fn start_server(pool: SqlitePool, args: &Args) -> Result<()> {
    let origin_regex = regex::Regex::new(r"^https?://localhost:\d+/?$").unwrap();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(move |origin: &HeaderValue, _: _| {
            origin_regex.is_match(origin.to_str().unwrap_or_default())
        }))
        .allow_methods(cors::Any)
        .allow_headers([
            HeaderName::from_static("authorization"),
            HeaderName::from_static("content-type"),
            HeaderName::from_static("content-length"),
            HeaderName::from_static("accept"),
        ])
        .expose_headers([
            HeaderName::from_static("authorization"),
            HeaderName::from_static("content-type"),
            CONTENT_ENCODING,
            CONTENT_LENGTH,
            HeaderName::from_static("accept"),
        ]);

    let sensitive_headers: Arc<[_]> = [header::AUTHORIZATION, header::COOKIE].into();

    let middleware = ServiceBuilder::new()
        // Mark the `Authorization` and `Cookie` headers as sensitive so it doesn't show in logs
        .sensitive_request_headers(sensitive_headers.clone())
        // Add high level tracing/logging to all requests
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_response(
                    DefaultOnResponse::new()
                        .include_headers(true)
                        .latency_unit(LatencyUnit::Micros),
                ),
        )
        .sensitive_response_headers(sensitive_headers)
        // Set a timeout
        .layer(TimeoutLayer::new(Duration::from_secs(15)))
        // Compress responses
        .compression()
        // Set a `Content-Type` if there isn't one already.
        .insert_response_header_if_not_present(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        );

    let api = Router::new()
        .route("/register", post(create_user))
        // Logins users in based on the JSON data in the response body
        .route("/login", post(authenticate_user))
        // Logins users in based on the authorization header
        .route("/login", get(get_user_from_token))
        .route("/users/id/:id", get(get_user_by_id))
        .route("/users/username/:username", get(get_user_by_username))
        .route("/users/search/:username", get(search_users))
        .route("/check/username/:username", get(check_username))
        .route("/check/email/:email", get(check_email))
        // Update user account data (email, username, etc.)
        .route("/account", post(update_user))
        // Delete user account
        .route("/account", delete(delete_user))
        // Get user settings
        .route("/account/settings", get(get_settings))
        // Update user settings
        .route("/account/settings", post(update_settings))
        // Upload a profile image
        .route("/account/upload", post(upload_profile_image))
        .layer(DefaultBodyLimit::max(10_100_000))
        .route("/chat/:id/messages", get(get_conversation))
        .route("/chat/create", post(create_conversation_rest))
        .route("/chat/models", get(get_ai_models))
        .route("/report/pdf", get(generate_pdf_report))
        // Used to submit a new health form
        .route("/forms/health", post(save_health_form))
        // Used to quickly check if a user should submit another health form
        // can also be used to edit the most recent health form
        .route("/forms/health", get(get_health_form))
        // Userd to update a health form with the given id
        .route("/forms/health/:id", put(update_health_form))
        // Used to show a user all the health forms they have submitted
        .route("/forms", get(get_forms))
        // Used to upload files to the server
        .route("/upload", post(upload_file))
        .layer(DefaultBodyLimit::max(10_100_000))
        // Used to upload files to the server
        .nest_service("/upload/", ServeDir::new("uploads"))
        // .route("/chat/query_model/*model_name", get(query_model))
        .route("/ws", get(init_ws))
        // Add CORS headers to all responses
        .layer(cors);

    let app = Router::new()
        .nest("/api", api)
        .fallback_service(
            ServeDir::new("../client/dist").fallback(ServeFile::new("../client/dist/index.html")),
        )
        // Add the trace layer to log all incoming requests
        // This logs the request method, path, response status, and response time
        .layer(middleware)
        .with_state(AppState::new(pool.clone()));

    let tcp_listener = TcpListener::bind(format!("0.0.0.0:{}", args.port)).await?;
    info!("Server listening on port {}", args.port);
    axum::serve(
        tcp_listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        // Wait for the CTRL+C signal
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
    })
    .await?;
    pool.close().await;
    Ok(())
}

/// Initialize the database by creating the database file and running the migrations.
/// Returns a connection pool to the database.
pub async fn init_db(db_url: &str) -> Result<SqlitePool> {
    let pool: SqlitePool = SqlitePool::connect_lazy_with(
        SqliteConnectOptions::from_str(db_url)?
            .foreign_keys(true)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            // Only user NORMAL is WAL mode is enabled
            // as it provides extra performance benefits
            // at the cost of durability
            .synchronous(SqliteSynchronous::Normal),
    );
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
