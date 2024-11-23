use std::ops::ControlFlow;

use anyhow::{anyhow, Result};
use axum::{
    extract::{Path, State},
    http::{
        header::{self, AUTHORIZATION},
        HeaderMap, StatusCode,
    },
    response::{IntoResponse, Response},
};
use dotenvy_macro::dotenv;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use macros::response;
use password_auth::VerifyError;
use serde::{Deserialize, Serialize};
use sonic_rs::json;
use sqlx::{prelude::Type, SqlitePool};
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
        custom(function = "validate_password")
    )]
    pub password: String,
    #[validate(
        length(
            min = 3,
            max = 20,
            code = "Username must be between 3 and 20 characters"
        ),
        custom(function = "validate_username")
    )]
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_id: Option<i64>,
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
            return Err(AppError::UserError((
                StatusCode::CONFLICT,
                "Username already exists".into(),
            )));
        } else {
            return Err(AppError::UserError((
                StatusCode::CONFLICT,
                "Email already in use".into(),
            )));
        }
    }
    let hashed_password = password_auth::generate_hash(&user_data.password);

    // Insert the user into the database
    let user_id = sqlx::query!(
        "INSERT INTO users (username, email, password_hash, first_name, last_name) VALUES (?, ?, ?, ?, ?) RETURNING id",
        user_data.username,
        user_data.email,
        hashed_password,
        user_data.first_name,
        user_data.last_name
    ).fetch_one(&pool).await?.id;

    // Insert the default user settings
    sqlx::query!("INSERT INTO user_settings (user_id) VALUES (?)", user_id)
        .execute(&pool)
        .await?;

    Ok((
        StatusCode::CREATED,
        AppJson(json!({ "message": "User created" })),
    )
        .into_response())
}

pub async fn check_username(
    State(pool): State<SqlitePool>,
    user: Option<JwtAuth<UserToken>>,
    Path(username): Path<String>,
) -> Result<Response, AppError> {
    if username.len() < 3 || username.len() > 20 || validate_username(&username).is_err() {
        return Err(AppError::UserError((
            StatusCode::BAD_REQUEST,
            "Invalid username".into(),
        )));
    }
    // If the user is authenticated, check if the username is the same
    // as the one already in the database. If it is, then that is allowed
    if user.is_some_and(|JwtAuth(user)| user.username == username) {
        return Ok(StatusCode::OK.into_response());
    }
    match sqlx::query!("SELECT username FROM users WHERE username = ?", username)
        .fetch_optional(&pool)
        .await?
    {
        Some(_) => Ok((
            StatusCode::CONFLICT,
            AppJson(response!("Username is already in use")),
        )
            .into_response()),
        None => Ok(StatusCode::OK.into_response()),
    }
}

pub async fn check_email(
    State(pool): State<SqlitePool>,
    user: Option<JwtAuth<UserToken>>,
    Path(email): Path<String>,
) -> Result<Response, AppError> {
    let email_regex = regex::Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").unwrap();
    if !email_regex.is_match(&email) {
        return Err(AppError::UserError((
            StatusCode::BAD_REQUEST,
            "Invalid email".into(),
        )));
    }
    // If the user is authenticated, check if the email is the same
    // as the one already in the database. If it is, then that is allowed
    if let Some(JwtAuth(user)) = user {
        if sqlx::query!("SELECT email FROM users WHERE id = ?", user.id)
            .fetch_optional(&pool)
            .await?
            .is_some_and(|row| row.email == email)
        {
            return Ok(StatusCode::OK.into_response());
        }
    }
    match sqlx::query!("SELECT email FROM users WHERE email = ?", email)
        .fetch_optional(&pool)
        .await?
    {
        Some(_) => Ok((
            StatusCode::CONFLICT,
            AppJson(response!("Email is already in use")),
        )
            .into_response()),
        None => Ok(StatusCode::OK.into_response()),
    }
}

pub fn validate_username(username: &str) -> Result<(), ValidationError> {
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
fn validate_password(password: &str) -> Result<(), ValidationError> {
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
        custom(function = "validate_username")
    )]
    pub username: String,
    #[validate(
        length(
            min = 8,
            max = 128,
            code = "Password must be between 8 and 128 characters"
        ),
        custom(function = "validate_password")
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
        sqlx::query!("SELECT users.id, username, email, first_name, last_name, password_hash, path as image_path FROM users LEFT JOIN files ON users.image_id = files.id WHERE username = ?", user_data.username)
            .fetch_optional(&pool)
            .await?
    else {
        return Err(AppError::UserError((
            StatusCode::UNAUTHORIZED,
            "Invalid username or password".into(),
        )));
    };

    match password_auth::verify_password(&user_data.password, &existing_user.password_hash) {
        Ok(_) => (),
        Err(VerifyError::PasswordInvalid) => {
            return Err(AppError::UserError((
                StatusCode::UNAUTHORIZED,
                "Invalid username or password".into(),
            )));
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

    let user = SessionUser {
        id: existing_user.id,
        username: existing_user.username,
        email: existing_user.email,
        first_name: existing_user.first_name,
        last_name: existing_user.last_name,
        // Have to check if the image path is empty since it is left join and
        // sqlx can't check if the join has a null column for some reason
        image_path: (!existing_user.image_path.is_empty()).then_some(existing_user.image_path),
    };

    Ok((
        StatusCode::OK,
        [(
            header::AUTHORIZATION,
            format!("Bearer {}", generate_jwt(&token_data)?),
        )],
        // Don't need to set the content-type header since axum does
        // it for us when we wrap the body in a `Json` struct
        AppJson(response!("Successfully authenticated", user)),
    )
        .into_response())
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_path: Option<String>,
}

/// Returns the user data of the currently authenticated user
/// from their JWT
pub async fn get_user_from_token(
    State(pool): State<SqlitePool>,
    JwtAuth(user): JwtAuth<UserToken>,
) -> Result<Response, AppError> {
    let Some(user) = sqlx::query_as!(
        SessionUser,
        "SELECT users.id, username, email, first_name, last_name, path as image_path FROM users LEFT JOIN files ON users.image_id = files.id WHERE users.id = ?",
        user.id
    )
    .fetch_optional(&pool)
    .await?
    else {
        return Err(AppError::UserError((
            StatusCode::NOT_FOUND,
            "User not found".into(),
        )));
    };
    Ok((StatusCode::OK, AppJson(user)).into_response())
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_path: Option<String>,
}

pub async fn get_user_by_id(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let Some(user) = sqlx::query_as!(
        PublicUser,
        "SELECT users.id, username, first_name, last_name, path as image_path FROM users LEFT JOIN files ON files.id = users.image_id WHERE users.id = ?",
        id
    )
    .fetch_optional(&pool)
    .await?
    else {
        return Err(AppError::UserError((
            StatusCode::NOT_FOUND,
            "User not found".into(),
        )));
    };

    Ok((StatusCode::OK, AppJson(user)).into_response())
}

pub async fn get_user_by_username(
    State(pool): State<SqlitePool>,
    Path(username): Path<String>,
) -> Result<Response, AppError> {
    let Some(user) = sqlx::query_as!(
        PublicUser,
        "SELECT users.id, username, first_name, last_name, path as image_path FROM users LEFT JOIN files ON files.id = users.image_id WHERE username = ?",
        username
    )
    .fetch_optional(&pool)
    .await?
    else {
        return Err(AppError::UserError((
            StatusCode::NOT_FOUND,
            "User not found".into(),
        )));
    };

    Ok((StatusCode::OK, AppJson(user)).into_response())
}

pub async fn update_user(
    State(pool): State<SqlitePool>,
    JwtAuth(user): JwtAuth<UserToken>,
    AppJson(user_data): AppJson<CreateUser>,
) -> Result<Response, AppError> {
    // Check the user's password
    let Some(stored_user) = sqlx::query!("SELECT password_hash FROM users WHERE id = ?", user.id)
        .fetch_optional(&pool)
        .await?
    else {
        return Err(AppError::UserError((
            StatusCode::NOT_FOUND,
            "User does not exist".into(),
        )));
    };
    if password_auth::verify_password(user_data.password, &stored_user.password_hash).is_err() {
        return Err(AppError::UserError((
            StatusCode::UNAUTHORIZED,
            "Invalid password".into(),
        )));
    }

    if let Some(image) = user_data.image_id {
        check_image(&pool, image, user.id).await?;
    }

    // Update the user in the database
    sqlx::query!(
        "UPDATE users SET first_name = ?, last_name = ?, email = ?, username = ?, image_id = ? WHERE id = ?",
        user_data.first_name,
        user_data.last_name,
        user_data.email,
        user_data.username,
        user_data.image_id,
        user.id
    ).execute(&pool).await?;

    let user = sqlx::query_as!(
        SessionUser,
        "SELECT users.id as id, username, first_name, last_name, email, path as image_path FROM users LEFT JOIN files ON files.id = users.image_id WHERE users.id = ?",
        user.id
    )
    .fetch_one(&pool)
    .await?;
    // Generate a new token with the updated user data
    let token_data = UserToken {
        id: user.id,
        username: user.username.clone(),
        exp: (chrono::Utc::now() + chrono::Duration::days(1)).timestamp(),
    };

    Ok((
        StatusCode::OK,
        // Give the user a new JWT
        [(
            header::AUTHORIZATION,
            format!("Bearer {}", generate_jwt(&token_data)?),
        )],
        AppJson(response!("User successfully updated", user)),
    )
        .into_response())
}

async fn check_image(pool: &SqlitePool, image_id: i64, user_id: i64) -> Result<(), AppError> {
    let query = sqlx::query!("SELECT id, profile_image FROM files JOIN file_uploads ON files.id = file_uploads.file_id WHERE id = ? AND user_id = ?", image_id, user_id)
        .fetch_optional(pool)
        .await?;
    // Check if the file is uploaded by the user and is an image uploaded as a profile image
    match query.and_then(|row| row.profile_image) {
        Some(val) if val => Ok(()),
        Some(_) => Err(AppError::UserError((
            StatusCode::BAD_REQUEST,
            "File id is not a profile image".into(),
        ))),
        // File was not uploaded by the user
        None => Err(AppError::UserError((
            StatusCode::NOT_FOUND,
            "Image not found".into(),
        ))),
    }
}

pub async fn delete_user(
    State(pool): State<SqlitePool>,
    JwtAuth(user): JwtAuth<UserToken>,
    AppJson(user_data): AppJson<LoginData>,
) -> Result<Response, AppError> {
    if user.username != user_data.username {
        return Err(AppError::AuthError(anyhow!("Token does not match user")));
    }

    let Some(stored_user) = sqlx::query!("SELECT password_hash FROM users WHERE id = ?", user.id)
        .fetch_optional(&pool)
        .await?
    else {
        return Err(AppError::UserError((
            StatusCode::NOT_FOUND,
            "User does not exist".into(),
        )));
    };

    if password_auth::verify_password(&user_data.password, &stored_user.password_hash).is_err() {
        return Err(AppError::UserError((
            StatusCode::UNAUTHORIZED,
            "Invalid password".into(),
        )));
    }

    sqlx::query!("DELETE FROM users WHERE id = ?", user.id)
        .execute(&pool)
        .await?;

    Ok((StatusCode::OK, AppJson(response!("User deleted"))).into_response())
}

fn generate_jwt(token_data: &UserToken) -> Result<String, AppError> {
    Ok(encode(
        &Header::default(),
        token_data,
        &EncodingKey::from_secret(dotenv!("JWT_KEY").as_bytes()),
    )?)
}

pub async fn search_users(
    State(pool): State<SqlitePool>,
    Path(username): Path<String>,
) -> Result<Response, AppError> {
    let username_query = format!("%{}%", username);
    let query = sqlx::query_as!(
        PublicUser,
        "SELECT users.id, username, first_name, last_name, path as image_path FROM users LEFT JOIN files ON files.id = users.image_id WHERE username LIKE ?",
        username_query
    )
    .fetch_all(&pool)
    .await?;

    Ok((StatusCode::OK, AppJson(query)).into_response())
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub ai_enabled: bool,
    pub ai_model_id: Option<i64>,
    /// Could be a bool for now but making it an enum in case we want to add more
    /// themes in the future
    pub theme: Theme,
}

#[derive(Serialize, Deserialize, Type)]
#[sqlx(rename_all = "snake_case")]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    Light,
    Dark,
}

/// Implementing From<String> for Theme so we can convert the theme
/// Need for sqlx to convert the theme from the database to the enum
impl From<String> for Theme {
    fn from(value: String) -> Self {
        match value.as_str() {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            _ => Theme::Dark,
        }
    }
}

/// Update the logged in user's settings
pub async fn update_settings(
    State(pool): State<SqlitePool>,
    JwtAuth(user): JwtAuth<UserToken>,
    AppJson(user_data): AppJson<Settings>,
) -> Result<Response, AppError> {
    sqlx::query!(
        "UPDATE user_settings SET ai_enabled = ?, ai_model_id = ?, theme = ? WHERE user_id = ?",
        user_data.ai_enabled,
        user_data.ai_model_id,
        user_data.theme,
        user.id
    )
    .execute(&pool)
    .await?;
    Ok(StatusCode::OK.into_response())
}

/// Returns the logged in user's settings
pub async fn get_settings(
    State(pool): State<SqlitePool>,
    JwtAuth(user): JwtAuth<UserToken>,
) -> Result<Response, AppError> {
    let settings = sqlx::query_as!(
        Settings,
        "SELECT ai_enabled, ai_model_id, theme FROM user_settings WHERE user_id = ?",
        user.id
    )
    .fetch_one(&pool)
    .await?;
    Ok((StatusCode::OK, AppJson(settings)).into_response())
}
