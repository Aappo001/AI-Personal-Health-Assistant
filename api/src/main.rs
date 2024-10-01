use std::env;

use anyhow::Result;
use ai_health_assistant_api::{cli::Args, init_db, start_server};

use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }
    pretty_env_logger::init();

    let mut args = Args::parse();
    if !args.db_url.starts_with("sqlite://") {
        args.db_url = format!("sqlite://{}", args.db_url);
    }
    let pool = init_db(&args.db_url).await?;
    start_server(pool, &args).await
}
