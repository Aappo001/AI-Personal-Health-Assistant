use axum::{
    extract::State,
    response::{IntoResponse, Response}, Json,
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
    Ok((StatusCode::CREATED, Json(response!("Form successfully created", data))).into_response())
}
