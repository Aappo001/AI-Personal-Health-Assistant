use std::{env::current_dir, fs::File, io::Write, path::PathBuf};

use dotenv::dotenv;
use sqlx::SqlitePool;

const PROTOCOL: &str = "sqlite://";

// This function creates the database file and runs the migrations before attempting to compile the
// rest of the program
// This is necessary to use the sqlx query! macro because it checks the database at compile time to
// generate the necessary structs
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=migrations");
    let db_file = match dotenv() {
        Ok(_) => PathBuf::from(match dotenv::var("DATABASE_URL") {
            Ok(url) => {
                if url.starts_with(PROTOCOL) {
                    url.strip_prefix(PROTOCOL).unwrap().to_string()
                } else {
                    url
                }
            }
            Err(_) => "./api.db".to_string(),
        }),
        Err(_) => {
            let db_path = current_dir()?.join("api.db");
            let mut env_file = File::create(".env")?;
            writeln!(env_file, "DATABASE_URL={}", db_path.display().to_string().replace('\\', "/"))?;
            db_path
        }
    };
    File::create(&db_file)?;
    let pool = SqlitePool::connect_lazy(&format!("{}{}", PROTOCOL, db_file.display()))?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(())
}
