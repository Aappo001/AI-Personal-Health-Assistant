pub mod cli;
pub mod users;
use std::{
    fs::{create_dir_all, File}, path::PathBuf, str::FromStr
};

use anyhow::Result;
use axum::{routing::post, Router};

use sqlx::SqlitePool;
use tokio::net::TcpListener;
use users::create_user;

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

pub async fn start_server(pool: SqlitePool) -> Result<()> {
    let app = Router::new().route("/", post(create_user)).with_state(pool);
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
