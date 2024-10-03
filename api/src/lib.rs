pub mod chat;
pub mod cli;
pub mod users;
pub mod utils;
pub mod error;
use std::{
    fs::{create_dir_all, File},
    path::PathBuf,
    str::FromStr,
    sync::Arc,
};

use anyhow::Result;
use axum::{
    extract::FromRef,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Router,
};
use tower_http::trace::TraceLayer;

use chat::{connect_conversation, create_conversation, get_conversation, get_user_conversations};
use cli::Args;
use dashmap::DashMap;
use log::info;
use sqlx::SqlitePool;
use tokio::{net::TcpListener, sync::broadcast};
use users::{authenticate_user, create_user, delete_user, get_user_profile};

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(windows)]
pub const PROTOCOL: &str = "sqlite:///";

#[cfg(unix)]
pub const PROTOCOL: &str = "sqlite://";

#[derive(Clone)]
pub struct AppState {
    // This is a channel that we can use to send messages to all connected clients on the same
    // conversation.
    user_sockets: Arc<DashMap<i64, broadcast::Sender<chat::SocketResponse>>>,
    user_connections: Arc<DashMap<i64, usize>>,
    // Connection pool to the database.
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

pub async fn start_server(pool: SqlitePool, args: &Args) -> Result<()> {
    let app = Router::new()
        .route("/users/create", post(create_user))
        .route("/users/auth", post(authenticate_user))
        .route("/users/profile/:id", get(get_user_profile))
        .route("/users/delete", delete(delete_user))
        .route("/chat", get(get_user_conversations))
        .route("/chat/:id/messages", get(get_conversation))
        .route("/chat/create", post(create_conversation))
        .route("/ws", get(connect_conversation))
        .layer(TraceLayer::new_for_http())
        .with_state(AppState::new(pool.clone()));
    let tcp_listener = TcpListener::bind(format!("0.0.0.0:{}", args.port)).await?;
    info!("Server listening on port {}", args.port);
    axum::serve(tcp_listener, app).await?;
    Ok(())
}

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

