use std::{env::current_dir, path::PathBuf};

use clap::Parser;

use crate::PKG_NAME;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The URL of the database to connect to
    /// Will default to `dirs::data_dir` if not provided
    #[arg(default_value_t = format!("sqlite://{}", dirs::data_dir().unwrap_or(current_dir().expect("Expected current directory to exist")).join(PKG_NAME).join("api.db").display()))]
    pub db_url: String,
}
