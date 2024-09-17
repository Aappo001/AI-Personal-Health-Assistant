use axum::{response::IntoResponse, Json};

pub async fn create_user(Json(data): Json<serde_json::Value>) -> impl IntoResponse {
    "Hello, World!"
}
