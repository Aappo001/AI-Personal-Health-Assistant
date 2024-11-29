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
/// Contains the state of the application that is shared across all routes.
pub mod state;
/// Contains logic for uploading files to the server.
pub mod upload;
/// Contains the logic for the users side of the application. Including the routes for creating a
/// user, authenticating a user, and getting a user's profile.
pub mod users;
/// Contains utility functions that are used throughout the application.
pub mod utils;

use anyhow::Result;
use axum::{
    extract::DefaultBodyLimit,
    http::{HeaderName, HeaderValue},
    routing::{delete, get, post, put},
    Router,
};
use forms::{get_forms, get_health_form, save_health_form, update_health_form};
use report::generate_pdf_report;
use reqwest::header::{self, CONTENT_ENCODING, CONTENT_LENGTH};
use state::AppState;
use std::{net::SocketAddr, str::FromStr, sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
    cors::{self, AllowOrigin, CorsLayer},
    services::{ServeDir, ServeFile},
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit, ServiceBuilderExt,
};

use chat::{create_conversation_rest, get_ai_models, get_conversation, init_ws};
use cli::Args;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous},
    SqlitePool,
};
use tokio::net::TcpListener;
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
