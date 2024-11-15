use clap::Parser;

use crate::utils::data_dir;
use dotenvy::var;

/// The backend API for the chat application
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The URL of the database to connect to
    /// Will default to DATABASE_URL variable inside .env file if a .env file is found in the current project directory, otherwise `dirs::data_dir` if not provided
    #[arg(short='u', long, default_value_t = var("DATABASE_URL").unwrap_or(default_db_url()))]
    pub db_url: String,
    /// The port to listen on for connections
    #[arg(short, long, default_value_t = 3000)]
    pub port: u16,
    /// Enable trace debugging for tokio-console
    #[arg(short, long)]
    pub debug: bool,
}

/// We know that windows paths use `\` instead of `/` as file separators and file names cannot contain `\` inside them.
/// Therefore, every `\` we encounter is a file separator and can safely be replaced with `/`.
/// This function returns the default database URL based on the operating system
#[cfg(windows)]
fn default_db_url() -> String {
    format!(
        "sqlite:///{}",
        data_dir()
            .join("api.db")
            .display()
            .to_string()
            .replace("\\", "/")
    )
}

/// This function returns the default database URL based on the operating system
#[cfg(unix)]
fn default_db_url() -> String {
    format!("sqlite://{}", data_dir().join("api.db").display())
}
