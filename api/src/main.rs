use anyhow::Result;
use api::{init_db, start_server};

use api::cli::Args;
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

    start_server().await
}
