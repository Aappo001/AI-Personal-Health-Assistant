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
/// Contains logic for uploading files to the server.
pub mod upload;
/// Contains the logic for the users side of the application. Including the routes for creating a
/// user, authenticating a user, and getting a user's profile.
pub mod users;
/// Contains utility functions that are used throughout the application.
pub mod utils;
use std::{
    fmt::Debug,
    net::SocketAddr,
    ops::Deref,
    str::FromStr,
    sync::{atomic::AtomicI64, Arc},
    time::Duration
};

use anyhow::Result;
use axum::{
    extract::FromRef,
    http::{HeaderName, HeaderValue},
    routing::{delete, get, post, put},
    Router,
};
use forms::{get_forms, get_health_form, save_health_form, update_health_form};
use reqwest::{header, Client};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer, cors::{self, AllowOrigin, CorsLayer}, services::{ServeDir, ServeFile}, timeout::TimeoutLayer, trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer}, LatencyUnit, ServiceBuilderExt
};

use chat::{
    connect_conversation, create_conversation_rest, get_ai_models, get_conversation,
    get_user_conversations,
};
use cli::Args;
use scc::HashMap;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous},
    SqlitePool,
};
use tokio::{net::TcpListener, sync::broadcast};
use tracing::info;
use upload::upload_file;
use users::{
    authenticate_user, check_email, check_username, create_user, delete_user, get_user_by_id,
    get_user_by_username, get_user_from_token, update_user,
};

/// The name of the package. This is defined in the `Cargo.toml` file.
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

/// The protocol for connecting to a SQLite database.
#[cfg(windows)]
pub const PROTOCOL: &str = "sqlite:///";

/// The protocol for connecting to a SQLite database.
#[cfg(unix)]
pub const PROTOCOL: &str = "sqlite://";

/// The application state that is shared across all routes.
#[derive(Clone, Debug)]
pub struct AppState {
    /// This is a reqwest client that we use to make requests to the AI service.
    client: reqwest::Client,
    /// This is a channel that we can use to send messages to all connected clients on the same
    /// conversation.
    user_sockets: Arc<HashMap<i64, InnerSocket>>,
    /// This is a map that keeps track of how many connections each user has. We use this to
    /// determine when we should remove the user from the `user_sockets` map.
    user_connections: Arc<HashMap<i64, usize>>,
    /// Connection pool to the database. We use a pool to handle multiple requests concurrently
    /// without having to create a new connection for each request.
    pool: SqlitePool,
    /// Stemmer for stemming all messages sent
    stemmer: Arc<Stemmer>,
    // Maybe add a `Arc<HashSet<i64>>` to keep track of the conversation ids
    // that the AI is currently generating messages for.
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

#[derive(Clone, Debug)]
/// The inner state of a user's socket connection.
pub struct InnerSocket {
    /// The sender half of the broadcast channel that we use to send messages to all
    /// connections made by the same user
    channel: broadcast::Sender<chat::SocketResponse>,
    /// A flag that contains the conversation id of the conversation that the AI is currently
    /// generating messages for.
    /// This uses 0 as a sentinel value to represent that the AI is not currently generating
    /// responses since conversation ids are all greater than 0. This would've been better
    /// as an `Arc<Option<AtomicI64>>`, but that doesn't provide mutability. So the other
    /// option is to use `Arc<AtomicPtr<Option<i64>>>` but that requires unsafe code to
    /// manage the pointer.
    ai_responding: Arc<AtomicI64>,
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
            user_connections: Arc::new(HashMap::new()),
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
            HeaderName::from_static("accept"),
        ])
        .expose_headers([HeaderName::from_static("authorization")]);

    let sensitive_headers: Arc<[_]> = [header::AUTHORIZATION, header::COOKIE].into();
       
    let middleware = ServiceBuilder::new()
        // Mark the `Authorization` and `Cookie` headers as sensitive so it doesn't show in logs
        .sensitive_request_headers(sensitive_headers.clone())
        // Add high level tracing/logging to all requests
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_response(DefaultOnResponse::new().include_headers(true).latency_unit(LatencyUnit::Micros)),
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
        .route("/check/username/:username", get(check_username))
        .route("/check/email/:email", get(check_email))
        .route("/account", post(update_user))
        .route("/account", delete(delete_user))
        .route("/chat", get(get_user_conversations))
        .route("/chat/:id/messages", get(get_conversation))
        .route("/chat/create", post(create_conversation_rest))
        .route("/chat/models", get(get_ai_models))
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
        // Used to upload files to the server
        .nest_service("/upload/", ServeDir::new("uploads"))
        // .route("/chat/query_model/*model_name", get(query_model))
        .route("/ws", get(connect_conversation))
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
