pub mod cli;
pub mod users;
pub mod chat;
pub mod utils;
use std::{
    fs::{create_dir_all, File},
    path::PathBuf,
    str::FromStr,
};

use anyhow::Result;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Router,
};

use chat::get_conversations;
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use users::{authenticate_user, create_user, delete_user, get_user_profile};

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

pub async fn start_server(pool: SqlitePool) -> Result<()> {
    let app = Router::new()
        .route("/users/create", post(create_user))
        .route("/users/auth", post(authenticate_user))
        .route("/users/profile/:id", get(get_user_profile))
        .route("/users/delete", delete(delete_user))
        .route("/chat/conversations", get(get_conversations))
        .with_state(pool);
    let tcp_listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(tcp_listener, app).await?;
    Ok(())
}

pub async fn init_db(db_url: &str) -> Result<SqlitePool> {
    if let Ok(path) = PathBuf::from_str(db_url.strip_prefix("sqlite://").unwrap_or(db_url)) {
        if !path.is_file() {
            create_dir_all(path.parent().expect("Expected parent directory to exist"))?;
            File::create(&path)?;
        }
    }
    let pool = SqlitePool::connect_lazy(db_url)?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

// Make our own error that wraps `anyhow::Error`.
pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
