use anyhow::{anyhow, Result};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use dotenv_codegen::dotenv;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

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
    if let Err(err) = check_password(&user_data.password) {
        return Ok((StatusCode::BAD_REQUEST, err));
    }

    if let Some(existing_user) = sqlx::query!(
        "SELECT username, email FROM users where username = ? or email = ?",
        user_data.username,
        user_data.email
    )
    .fetch_optional(&pool)
    .await?
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
    let re = regex::Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$")
        .expect("Should be a valid regex");
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

pub fn check_password(password: &str) -> Result<(), Box<str>> {
    if password.len() < 8 {
        Err("Password must be at least 8 characters long".into())
    } else if password.len() > 128 {
        Err("Password must be at most 128 characters long".into())
    } else if !password.chars().all(|c| c.is_ascii()) {
        Err(r#"Password must only contain alphanumeric characters and ASCII symbols"#.into())
    } else {
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct LoginData {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserToken {
    pub id: i64,
    pub username: String,
    pub iat: i64,
}

pub async fn authenticate_user(
    State(pool): State<SqlitePool>,
    Json(user_data): Json<LoginData>,
) -> Result<Response, AppError> {
    let Some(existing_user) =
        sqlx::query!("SELECT * FROM users where username = ?", user_data.username)
            .fetch_optional(&pool)
            .await?
    else {
        return Ok((StatusCode::UNAUTHORIZED, "Invalid username or password").into_response());
    };
    if let Err(_) =
        password_auth::verify_password(&user_data.password, &existing_user.password_hash)
    {
        return Ok((StatusCode::UNAUTHORIZED, "Invalid username or password").into_response());
    }

    let token_data = UserToken {
        id: existing_user.id.unwrap(),
        username: existing_user.username,
        iat: (chrono::Utc::now() + chrono::Duration::days(1)).timestamp(),
    };

    let token = encode(
        &Header::default(),
        &token_data,
        &EncodingKey::from_secret(dotenv!("JWT_KEY").as_bytes()),
    )?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from("Successfully authenticated"))?;
    Ok(response)
}

pub async fn authorize_user(token: &str) -> Result<UserToken, anyhow::Error> {
    let token_data = decode::<UserToken>(
        token
            .strip_prefix("Bearer ")
            .ok_or_else(|| anyhow!("Invalid token"))?,
        &DecodingKey::from_secret(dotenv!("JWT_KEY").as_bytes()),
        &Validation::default(),
    )?;
    if token_data.claims.iat < chrono::Utc::now().timestamp() {
        return Err(anyhow!("Token expired"));
    }

    Ok(token_data.claims)
}

#[derive(Serialize, Deserialize)]
pub struct PublicUser {
    pub id: i64,
    pub username: String,
    pub first_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
}

pub async fn get_user_profile(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let Some(user) =
        sqlx::query!("SELECT * FROM users where id = ?", id)
            .fetch_optional(&pool)
            .await?
    else {
        return Ok((StatusCode::NOT_FOUND, "User not found").into_response());
    };

    let public_user = PublicUser {
        id: user.id,
        username: user.username,
        first_name: user.first_name,
        last_name: user.last_name,
    };
    
    Ok((StatusCode::OK, serde_json::to_string_pretty(&public_user)?).into_response())
}
