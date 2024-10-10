use std::env;

use anyhow::Result;
use ai_health_assistant_api::{cli::Args, init_db, start_server, PROTOCOL};

use clap::Parser;
use tracing_subscriber::fmt::init;

// TODO: Add better, more integrated and descriptive logging
#[tokio::main]
async fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "debug")
    }

    tracing_subscriber::fmt::init();

    let mut args = Args::parse();
    if !args.db_url.starts_with(PROTOCOL) {
        args.db_url = format!("{}{}", PROTOCOL, args.db_url);
    }
    let pool = init_db(&args.db_url).await?;
    start_server(pool, &args).await
}
