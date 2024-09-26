use axum::{
    extract::State,
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, SqlitePool};

use crate::{users::authorize_user, AppError};

#[derive(Serialize, Deserialize, FromRow)]
pub struct Conversation {
    pub id: i64,
    pub title: String,
    pub created_at: NaiveDateTime,
    pub last_message_at: NaiveDateTime,
}

pub async fn get_conversations(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let user = match authorize_user(&headers){
        Ok(k) => k,
        Err(e) => return Ok((StatusCode::UNAUTHORIZED, e.to_string()).into_response())
    };
    let res = serde_json::to_string_pretty(
        &sqlx::query_as!(
            Conversation,
            "SELECT id, title, created_at, last_message_at  FROM conversations where user_id = ? ORDER BY last_message_at DESC",
            user.id,
        )
        .fetch_all(&pool)
        .await?,
    )?;
    Ok((StatusCode::OK, res).into_response())
}
