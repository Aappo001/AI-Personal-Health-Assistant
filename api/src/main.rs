use std::env;

use ai_health_assistant_api::{cli::Args, init_db, start_server, PROTOCOL};
use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use clap::Parser;

// TODO: Add better, more integrated and descriptive logging
#[tokio::main]
async fn main() -> Result<()> {
    let mut args = Args::parse();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug{}",
                    env!("CARGO_CRATE_NAME"),
                    if args.debug {
                        ",tokio=trace,runtime=trace"
                    } else {
                        ""
                    }
                )
                .into()
            }),
        )
        .with(console_subscriber::spawn())
        .with(tracing_subscriber::fmt::layer())
        .init();

    if !args.db_url.starts_with(PROTOCOL) {
        args.db_url = format!("{}{}", PROTOCOL, args.db_url);
    }
    let pool = init_db(&args.db_url).await?;
    start_server(pool, &args).await
}
