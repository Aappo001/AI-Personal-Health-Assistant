use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use chrono::NaiveDateTime;
use macros::response;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    auth::JwtAuth,
    error::{AppError, AppJson},
    users::UserToken,
    AppState,
};

#[derive(Serialize, Deserialize)]
pub struct HealthForm {
    pub height: Option<f64>,
    pub weight: Option<f64>,
    pub exercise_duration: Option<f64>,
    pub sleep_hours: Option<f64>,
    pub notes: Option<String>,
    pub food_intake: Option<String>,
    // Don't provide any of these fields.
    // They're for the database to fill in for the response.
    pub user_id: Option<i64>,
    pub id: Option<i64>,
    pub created_at: Option<NaiveDateTime>,
    pub modified_at: Option<NaiveDateTime>,
}

pub async fn save_health_form(
    State(state): State<AppState>,
    JwtAuth(user): JwtAuth<UserToken>,
    AppJson(form): AppJson<HealthForm>,
) -> Result<Response, AppError> {
    let data = sqlx::query_as!(
        HealthForm,
        "INSERT INTO user_statistics (user_id, height, weight, exercise_duration, sleep_hours, notes, food_intake)
        VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING *",
            user.id,
            form.height,
            form.weight,
            form.exercise_duration,
            form.sleep_hours,
            form.notes,
            form.food_intake
    ).fetch_one(&state.pool).await?;
    Ok((
        StatusCode::CREATED,
        AppJson(response!("Form successfully created", data)),
    )
        .into_response())
}

/// Get the most recent health form for the current user
pub async fn get_health_form(
    State(state): State<AppState>,
    JwtAuth(user): JwtAuth<UserToken>,
) -> Result<Response, AppError> {
    let data = sqlx::query_as!(
        HealthForm,
        "SELECT * FROM user_statistics WHERE user_id = ? ORDER BY created_at DESC LIMIT 1",
        user.id
    )
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::OK, AppJson(data)).into_response())
}

/// Get all the saved forms for the current user
pub async fn get_forms(
    State(state): State<AppState>,
    JwtAuth(user): JwtAuth<UserToken>,
) -> Result<Response, AppError> {
    let data = sqlx::query_as!(
        HealthForm,
        "SELECT * FROM user_statistics WHERE user_id = ? ORDER BY created_at DESC",
        user.id
    )
    .fetch_all(&state.pool)
    .await?;
    Ok((StatusCode::OK, AppJson(data)).into_response())
}

/// Get the most recent health form for the current user
pub async fn update_health_form(
    State(state): State<AppState>,
    JwtAuth(user): JwtAuth<UserToken>,
    Path(id): Path<i64>,
    AppJson(form): AppJson<HealthForm>,
) -> Result<Response, AppError> {
    let Some(row) = sqlx::query!("SELECT user_id FROM user_statistics WHERE id = ?", id)
        .fetch_optional(&state.pool)
        .await?
    else {
        return Err(AppError::UserError((
            StatusCode::NOT_FOUND,
            "Form not found".into(),
        )));
    };

    if row.user_id != user.id {
        return Err(AppError::UserError((
            StatusCode::FORBIDDEN,
            "You do not have permission to update this form".into(),
        )));
    }

    let data = sqlx::query_as!(
        HealthForm,
        "UPDATE user_statistics SET height = ?, weight = ?, exercise_duration = ?, sleep_hours = ?, notes = ?, food_intake = ? WHERE user_id = ? AND id = ? RETURNING *",
            form.height,
            form.weight,
            form.exercise_duration,
            form.sleep_hours,
            form.notes,
            form.food_intake,
            user.id,
            id
    ).fetch_one(&state.pool).await?;

    Ok((
        StatusCode::CREATED,
        AppJson(response!("Form successfully created", data)),
    )
        .into_response())
}
