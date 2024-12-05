use std::{
    env,
    fs::{read_to_string, File, OpenOptions},
    io::Write,
    path::PathBuf,
};

use dotenvy::var;
use sqlx::SqlitePool;

#[cfg(windows)]
const PROTOCOL: &str = "sqlite:///";

#[cfg(unix)]
const PROTOCOL: &str = "sqlite://";

// This function creates the database file and runs the migrations before attempting to compile the
// rest of the program
// This is necessary to use the sqlx query! macro because it checks the database at compile time to
// generate the necessary structs
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !env::var("BUILD_ENABLED").map(|v| v == "1").unwrap_or(false) {
        return Ok(());
    }

    println!("cargo:rerun-if-changed=migrations");
    println!("cargo:rerun-if-changed=.env");

    update_db_url()?;
    let db_file = var("DATABASE_URL")?;
    if db_file.starts_with("sqlite") {
        let path = if db_file.contains("file:") {
            db_file.split("file:").collect::<Vec<&str>>()[1]
        } else {
            db_file.split(PROTOCOL).collect::<Vec<&str>>()[1]
        };
        let path = PathBuf::from(path);
        if !path.exists() {
            File::create(&path)?;
        }
    }
    let pool = SqlitePool::connect_lazy(&db_file)?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(())
}

fn default_db_url() -> Result<String, String> {
    let data_dir = dirs::data_dir()
        .ok_or("Could not find data directory!")?
        .join(env!("CARGO_PKG_NAME"));
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    }
    let db_dir = data_dir.join("build.db");
    if !db_dir.exists() {
        File::create(&db_dir).map_err(|e| e.to_string())?;
    }
    Ok(if cfg!(windows) {
        format!(
            "{PROTOCOL}{}",
            db_dir.display().to_string().replace('\\', "/")
        )
    } else {
        format!("{PROTOCOL}{}", db_dir.display())
    })
}

fn update_db_url() -> Result<(), Box<dyn std::error::Error>> {
    // Read the existing .env file
    let contents = read_to_string(".env").ok();
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .read(true)
        .open(".env")?;

    // If the file is empty, write the default DATABASE_URL
    let Some(contents) = contents else {
        writeln!(file, "DATABASE_URL={}\n", default_db_url()?)?;
        return Ok(());
    };
    // Otherwise, update the DATABASE_URL
    let mut final_str: Vec<String> = Vec::new();
    let mut seen = false;
    for line in contents.lines() {
        if let Some(suffix) = line
            .trim_start()
            .strip_prefix("DATABASE_URL=")
            .map(|suffix| suffix.trim())
        {
            seen = true;
            if suffix.is_empty() {
                final_str.push(format!("DATABASE_URL={}", default_db_url()?));
            } else {
                final_str.push(line.to_string());
            }
        } else {
            final_str.push(line.to_string());
        }
    }
    if !seen {
        final_str.push(format!("DATABASE_URL={}", default_db_url()?));
    }
    file.write_all(
        // Join the final string with the appropriate line endings
        final_str
            .join(if cfg!(windows) { "\r\n" } else { "\n" })
            .as_bytes(),
    )?;
    Ok(())
}
