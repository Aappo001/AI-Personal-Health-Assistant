use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use anyhow::{anyhow, Result};

use crate::AppError;

#[derive(Serialize, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub first_name: String,
    pub last_name: Option<String>,
    pub password: String,
    pub username: String,
}

pub async fn create_user(
    State(pool): State<SqlitePool>,
    Json(user_data): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    if !check_email(&user_data.email) {
        return Ok((StatusCode::BAD_REQUEST, "Invalid email".into()));
    }
    if let Err(err) = check_username(&user_data.username) {
        return Ok((StatusCode::BAD_REQUEST, err));
    }
    
    if let Some(existing_user) = sqlx::query!(
        "SELECT username, email FROM users where username = ? or email = ?",
        user_data.username,
        user_data.email
    )
    .fetch_optional(&pool)
    .await
    ?
    {
        if existing_user.username == user_data.username {
            return Ok((StatusCode::CONFLICT, "Username already exists".into()));
        } else {
            return Ok((StatusCode::CONFLICT, "Email already in use".into()));
        }
    }
    let hashed_password = password_auth::generate_hash(&user_data.password);

    sqlx::query!(
        "INSERT INTO users (username, email, password_hash, first_name, last_name) VALUES (?, ?, ?, ?, ?)",
        user_data.username,
        user_data.email,
        hashed_password,
        user_data.first_name,
        user_data.last_name
    ).execute(&pool).await?;

    Ok((StatusCode::CREATED, "User created".into()))
}

pub fn check_email(email: &str) -> bool {
    let re = regex::Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").expect("Should be a valid regex");
    re.is_match(email)
}

pub fn check_username(username: &str) -> Result<(), Box<str>> {
    if username.len() < 3 {
        Err("Username must be at least 3 characters long".into())
    } else if username.len() > 20 {
        Err("Username must be at most 20 characters long".into())
    } else if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        Err(r#"Username must only contain alphanumeric characters and _"#.into())
    } else {
        Ok(())
    }
}
