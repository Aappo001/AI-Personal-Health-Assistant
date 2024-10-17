pub mod auth;
/// Contains the logic for the chat side of the application. Including the routes for creating a
/// conversation, getting a conversation, and connecting to a websocket for chatting.
pub mod chat;
/// Contains the logic for the command line interface (CLI) of the application.
pub mod cli;
/// Contains the error type and error handling logic for the application.
pub mod error;
/// Contains the logic for the users side of the application. Including the routes for creating a
/// user, authenticating a user, and getting a user's profile.
pub mod users;
/// Contains utility functions that are used throughout the application.
pub mod utils;
use std::{
    fs::{create_dir_all, File},
    net::SocketAddr,
    path::PathBuf,
    str::FromStr,
    sync::Arc,
};

use anyhow::Result;
use axum::{
    extract::FromRef,
    http::{HeaderValue, HeaderName},
    routing::{delete, get, post},
    Router,
};
use tower_http::{
    cors::{self, AllowOrigin, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};

use chat::{
    connect_conversation, create_conversation_rest, get_conversation, get_user_conversations,
};
use cli::Args;
use dashmap::DashMap;
use sqlx::SqlitePool;
use tokio::{net::TcpListener, sync::broadcast};
use tracing::info;
use users::{authenticate_user, create_user, delete_user, get_user_by_id, get_user_by_username, get_user_from_token};

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
    /// This is a channel that we can use to send messages to all connected clients on the same
    /// conversation.
    user_sockets: Arc<DashMap<i64, broadcast::Sender<chat::SocketResponse>>>,
    /// This is a map that keeps track of how many connections each user has. We use this to
    /// determine when we should remove the user from the `user_sockets` map.
    user_connections: Arc<DashMap<i64, usize>>,
    /// Connection pool to the database. We use a pool to handle multiple requests concurrently
    /// without having to create a new connection for each request.
    pool: SqlitePool,
}

impl AppState {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            user_sockets: Arc::new(DashMap::new()),
            user_connections: Arc::new(DashMap::new()),
            pool,
        }
    }
}

// Support for automatically converting an `AppState` into an `SqlitePool`
impl FromRef<AppState> for SqlitePool {
    fn from_ref(app_state: &AppState) -> SqlitePool {
        app_state.pool.clone()
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
            HeaderName::from_static("accept")
            ]
        )
        .expose_headers([HeaderName::from_static("authorization")]);

    let api = Router::new()
        .route("/register", post(create_user))
        // Logins users in based on the JSON data in the response body
        .route("/login", post(authenticate_user))
        // Logins users in based on the authorization header
        .route("/login", get(get_user_from_token))
        .route("/users/id/:id", get(get_user_by_id))
        .route("/users/username/:username", get(get_user_by_username))
        .route("/users/delete", delete(delete_user))
        .route("/chat", get(get_user_conversations))
        .route("/chat/:id/messages", get(get_conversation))
        .route("/chat/create", post(create_conversation_rest))
        .route("/ws", get(connect_conversation))
        .layer(cors);

    let app = Router::new()
        .nest("/api", api)
        .nest_service("/", ServeDir::new("../client/dist"))
        // Add the trace layer to log all incoming requests
        // This logs the request method, path, response status, and response time
        .layer(TraceLayer::new_for_http())
        .with_state(AppState::new(pool.clone()));

    let tcp_listener = TcpListener::bind(format!("0.0.0.0:{}", args.port)).await?;
    info!("Server listening on port {}", args.port);
    axum::serve(
        tcp_listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}

/// Initialize the database by creating the database file and running the migrations.
/// Returns a connection pool to the database.
pub async fn init_db(db_url: &str) -> Result<SqlitePool> {
    if let Ok(path) = PathBuf::from_str(db_url.strip_prefix(PROTOCOL).unwrap_or(db_url)) {
        if !path.is_file() {
            create_dir_all(path.parent().expect("Expected parent directory to exist"))?;
            File::create(&path)?;
        }
    }
    let pool = SqlitePool::connect_lazy(db_url)?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
