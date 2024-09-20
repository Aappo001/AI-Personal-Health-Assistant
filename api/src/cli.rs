use clap::Parser;

use crate::{utils::data_dir, PKG_NAME};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The URL of the database to connect to
    /// Will default to `dirs::data_dir` if not provided
    #[arg(default_value_t = format!("sqlite://{}", data_dir().join("api.db").display()))]
    pub db_url: String,
}
