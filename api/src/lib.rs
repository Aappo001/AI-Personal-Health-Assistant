mod users;
use anyhow::Result;
use axum::{routing::post, Router};

use tokio::net::TcpListener;
use users::create_user;

pub async fn start_server() -> Result<()>{
    let app = Router::new()
        .route("/", post(create_user));
    let tcp_listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(tcp_listener, app).await?;
    Ok(())
}
