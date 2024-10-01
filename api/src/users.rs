use std::ops::ControlFlow;

use anyhow::{anyhow, Result};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{
        header::{self, AUTHORIZATION},
        HeaderMap, StatusCode,
    },
    response::{IntoResponse, Response},
    Json,
};
use dotenv_codegen::dotenv;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use validator::{Validate, ValidationError, ValidationErrorsKind};

use crate::AppError;

#[derive(Serialize, Deserialize, Validate)]
pub struct CreateUser {
    #[validate(email(code = "Invalid email address"))]
    pub email: String,
    #[validate(length(
        min = 1,
        max = 30,
        code = "First name must be between 1 and 30 characters"
    ))]
    pub first_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[validate(
        length(
            min = 8,
            max = 128,
            code = "Password must be between 8 and 128 characters"
        ),
        custom(function = "check_password")
    )]
    pub password: String,
    #[validate(
        length(
            min = 3,
            max = 20,
            code = "Username must be between 3 and 20 characters"
        ),
        custom(function = "check_username")
    )]
    pub username: String,
}

pub trait PrettyValidate {
    fn pretty_validate(&self) -> Result<(), String>;
}

impl<T: Validate> PrettyValidate for T {
    fn pretty_validate(&self) -> Result<(), String> {
        if let Err(err) = self.validate() {
            return Err(err
                .0
                .iter()
                .fold(String::from("Validation Error\n"), |acc, x| {
                    acc + &match x.1 {
                        ValidationErrorsKind::Struct(e) => e.to_string(),
                        ValidationErrorsKind::List(e) => e
                            .iter()
                            .fold(String::new(), |acc, y| format!("{} {}\n", acc, y.1)),
                        ValidationErrorsKind::Field(e) => e.iter().fold(String::new(), |acc, y| {
                            format!("{}{}: {}\n", acc, x.0, y.code)
                        }),
                    }
                }));
        }
        Ok(())
    }
}

pub async fn create_user(
    State(pool): State<SqlitePool>,
    Json(user_data): Json<CreateUser>,
) -> Result<Response, AppError> {
    if let Err(err) = user_data.pretty_validate() {
        return Ok((StatusCode::BAD_REQUEST, err).into_response());
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
            return Ok((StatusCode::CONFLICT, "Username already exists").into_response());
        } else {
            return Ok((StatusCode::CONFLICT, "Email already in use").into_response());
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

    Ok((StatusCode::CREATED, "User created").into_response())
}

pub fn check_username(username: &str) -> Result<(), ValidationError> {
    match username
        .chars()
        .try_fold((0, 0), |(alphanumeric, underscore), c| {
            if c.is_alphanumeric() {
                ControlFlow::Continue((alphanumeric + 1, underscore))
            } else if c == '_' {
                ControlFlow::Continue((alphanumeric, underscore + 1))
            } else {
                ControlFlow::Break(ValidationError::new(
                    r#"must only contain alphanumeric characters and _"#,
                ))
            }
        }) {
        ControlFlow::Continue((a, u)) => {
            if a > u {
                Ok(())
            } else {
                // So we don't end up with usernames like "_a_" or "______"
                Err(ValidationError::new(
                    r#"must contain more alphanumeric characters than underscores"#,
                ))
            }
        }
        ControlFlow::Break(e) => Err(e),
    }
}

pub fn check_password(password: &str) -> Result<(), ValidationError> {
    if !password.chars().all(|c| c.is_ascii()) {
        Err(ValidationError::new(
            r#"must only contain alphanumeric characters and ASCII symbols"#,
        ))
    } else {
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Validate)]
pub struct LoginData {
    #[validate(
        length(
            min = 3,
            max = 20,
            code = "Username must be between 3 and 20 characters"
        ),
        custom(function = "check_username")
    )]
    pub username: String,
    #[validate(
        length(
            min = 8,
            max = 128,
            code = "Password must be between 8 and 128 characters"
        ),
        custom(function = "check_password")
    )]
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserToken {
    pub id: i64,
    pub username: String,
    pub exp: i64,
}

pub async fn authenticate_user(
    State(pool): State<SqlitePool>,
    Json(user_data): Json<LoginData>,
) -> Result<Response, AppError> {
    if let Err(err) = user_data.pretty_validate() {
        return Ok((StatusCode::BAD_REQUEST, err).into_response());
    }

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
        exp: (chrono::Utc::now() + chrono::Duration::days(1)).timestamp(),
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

pub fn authorize_user(headers: &HeaderMap) -> Result<UserToken, anyhow::Error> {
    let Some(token) = headers.get(AUTHORIZATION) else {
        return Err(anyhow!("No token provided"));
    };
    let token_data = decode::<UserToken>(
        token
            .to_str()?
            .strip_prefix("Bearer ")
            .ok_or_else(|| anyhow!("Invalid token"))?,
        &DecodingKey::from_secret(dotenv!("JWT_KEY").as_bytes()),
        &Validation::default(),
    )?;

    if token_data.claims.exp < chrono::Utc::now().timestamp() {
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
    let Some(user) = sqlx::query!("SELECT * FROM users where id = ?", id)
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

pub async fn delete_user(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Json(user_data): Json<LoginData>,
) -> Result<Response, AppError> {
    let token_user = match authorize_user(&headers){
        Ok(k) => k,
        Err(e) => return Ok((StatusCode::UNAUTHORIZED, e.to_string()).into_response())
    };

    if token_user.username != user_data.username {
        return Ok((StatusCode::UNAUTHORIZED, "Invalid token").into_response());
    }

    if sqlx::query!(
        "SELECT id FROM users where id = ? and username = ?",
        token_user.id,
        token_user.username
    )
    .fetch_optional(&pool)
    .await?
    .is_none()
    {
        return Ok((StatusCode::NOT_FOUND, "User does not exist").into_response());
    }

    sqlx::query!("DELETE FROM users where id = ?", token_user.id)
        .execute(&pool)
        .await?;

    Ok((StatusCode::OK, "User deleted").into_response())
}
