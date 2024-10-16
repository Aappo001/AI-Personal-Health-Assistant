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
use password_auth::VerifyError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::SqlitePool;
use validator::{Validate, ValidationError, ValidationErrorsKind};

use crate::{
    auth::JwtAuth,
    error::{AppError, AppJson, AppValidate},
};

/// The data required to create a new user
#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
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
    AppJson(user_data): AppJson<CreateUser>,
) -> Result<Response, AppError> {
    user_data.app_validate()?;

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

    Ok((
        StatusCode::CREATED,
        Json(json!({ "message": "User created" })),
    )
        .into_response())
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

/// Verify that the password only contains ASCII characters
fn check_password(password: &str) -> Result<(), ValidationError> {
    if !password.is_ascii() {
        Err(ValidationError::new(
            r#"must only contain alphanumeric characters and ASCII symbols"#,
        ))
    } else {
        Ok(())
    }
}

/// The data required to authenticate a user
#[derive(Deserialize, Validate)]
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

/// The data stored in the JWT token
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserToken {
    pub id: i64,
    pub username: String,
    pub exp: i64,
}

pub async fn authenticate_user(
    State(pool): State<SqlitePool>,
    AppJson(user_data): AppJson<LoginData>,
) -> Result<Response, AppError> {
    user_data.app_validate()?;

    let Some(existing_user) =
        sqlx::query!("SELECT * FROM users where username = ?", user_data.username)
            .fetch_optional(&pool)
            .await?
    else {
        return Ok((StatusCode::UNAUTHORIZED, "Invalid username or password").into_response());
    };

    match password_auth::verify_password(&user_data.password, &existing_user.password_hash) {
        Ok(_) => (),
        Err(VerifyError::PasswordInvalid) => {
            return Ok((StatusCode::UNAUTHORIZED, "Invalid username or password").into_response());
        }
        Err(e) => {
            return Err(e.into());
        }
    }

    let token_data = UserToken {
        id: existing_user.id,
        username: existing_user.username.clone(),
        exp: (chrono::Utc::now() + chrono::Duration::days(1)).timestamp(),
    };

    let token = encode(
        &Header::default(),
        &token_data,
        &EncodingKey::from_secret(dotenv!("JWT_KEY").as_bytes()),
    )?;

    let user = PublicUser {
        id: existing_user.id,
        username: existing_user.username,
        first_name: existing_user.first_name,
        last_name: existing_user.last_name,
    };

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({"message": "Successfully authenticated", "user": serde_json::to_string(&user).unwrap() }).to_string(),
        ))?;
    Ok(response)
}

/// Data of the currently authenticated user
/// Contains all user data except password
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionUser {
    pub id: i64,
    pub first_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    pub username: String,
    pub email: String,
}

/// Returns the user data of the currently authenticated user
/// from their JWT
pub async fn get_user_from_token(
    State(pool): State<SqlitePool>,
    JwtAuth(user): JwtAuth<UserToken>,
) -> Result<Response, AppError> {
    let Some(user) = sqlx::query_as!(
        SessionUser,
        "SELECT id, username, email, first_name, last_name FROM users WHERE id = ?",
        user.id
    )
    .fetch_optional(&pool)
    .await?
    else {
        return Ok((
            StatusCode::NOT_FOUND,
            Json(json!({ "message": "User not found" })),
        )
            .into_response());
    };
    Ok((StatusCode::OK, Json(user)).into_response())
}

pub fn authorize_user(headers: &HeaderMap) -> Result<UserToken, AppError> {
    let Some(token) = headers.get(AUTHORIZATION) else {
        return Err(AppError::AuthError(anyhow!("No token provided")));
    };
    let token_data = decode::<UserToken>(
        token
            .to_str()?
            .strip_prefix("Bearer ")
            .ok_or_else(|| anyhow!("Invalid token"))?,
        &DecodingKey::from_secret(dotenv!("JWT_KEY").as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AppError::AuthError(e.into()))?;

    if token_data.claims.exp < chrono::Utc::now().timestamp() {
        return Err(AppError::AuthError(anyhow!("Token expired")));
    }

    Ok(token_data.claims)
}

/// Public user data that can be shared with other users
/// Does not include sensitive information such as email or password
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicUser {
    pub id: i64,
    pub username: String,
    pub first_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
}

pub async fn get_user_by_id(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let Some(user) = sqlx::query_as!(
        PublicUser,
        "SELECT id, username, first_name, last_name FROM users WHERE id = ?",
        id
    )
    .fetch_optional(&pool)
    .await?
    else {
        return Ok((
            StatusCode::NOT_FOUND,
            Json(json!({ "message": "User not found" })),
        )
            .into_response());
    };

    Ok((StatusCode::OK, Json(user)).into_response())
}

pub async fn get_user_by_username(
    State(pool): State<SqlitePool>,
    Path(username): Path<String>,
) -> Result<Response, AppError> {
    let Some(user) = sqlx::query_as!(
        PublicUser,
        "SELECT id, username, first_name, last_name FROM users WHERE username = ?",
        username
    )
    .fetch_optional(&pool)
    .await?
    else {
        return Ok((
            StatusCode::NOT_FOUND,
            Json(json!({ "message": "User not found" })),
        )
            .into_response());
    };

    Ok((StatusCode::OK, Json(user)).into_response())
}

pub async fn delete_user(
    State(pool): State<SqlitePool>,
    JwtAuth(user): JwtAuth<UserToken>,
    AppJson(user_data): AppJson<LoginData>,
) -> Result<Response, AppError> {
    if user.username != user_data.username {
        return Err(AppError::AuthError(anyhow!("Token does not match user")));
    }

    if sqlx::query!(
        "SELECT id FROM users where id = ? and username = ?",
        user.id,
        user.username
    )
    .fetch_optional(&pool)
    .await?
    .is_none()
    {
        return Ok((StatusCode::NOT_FOUND, "User does not exist").into_response());
    }

    sqlx::query!("DELETE FROM users where id = ?", user.id)
        .execute(&pool)
        .await?;

    Ok((StatusCode::OK, "User deleted").into_response())
}
