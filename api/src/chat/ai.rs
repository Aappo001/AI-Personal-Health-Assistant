use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use dotenv::var;
use futures::StreamExt;
use reqwest::{header, StatusCode};
use reqwest_streams::*;
use serde_json::json;

use crate::{error::AppError, AppState};

pub async fn query_model(
    State(state): State<AppState>,
    Path(model_name): Path<String>,
) -> Result<Response, AppError> {
    let mut body = json!({
        "model": model_name,
        "messages": [
        { "role": "system", "content": "You are a medical professional who knows about medicine.  When the user tells you about a health problem that they are facing, continue probing through the problem to extract more information and attempt to gain a better understanding of a root cause and potential remedies. Do not simply give a list of potential causes without asking further questions. If you are unsure about something refer user to a doctor." },
    ],
        "temperature": 0.5,
        "max_tokens": 1024,
        "top_p": 0.7,
        "stream": true
    });
    if let Some(req_messages) = body["messages"].as_array_mut() {
        let db_messages = sqlx::query!(
            "SELECT message, user_id, ai_model_id FROM messages WHERE conversation_id = 1"
        )
        .fetch_all(&state.pool)
        .await?;

        // If we don't alternate between user and system messages, the AI will give us an error and
        // get stuck so we need to concatenate consecutive user and system messages together
        let mut last_role;
        let mut cur_role = "user";
        let mut cur_content = String::new();
        for message in db_messages {
            last_role = cur_role;
            cur_role = if message.user_id.is_some() {
                "user"
            } else {
                "system"
            };
            if last_role != cur_role {
                req_messages.push(json!({
                    "role": last_role,
                    "content": cur_content
                }));
                cur_content.clear();
            }
            cur_content.push_str(&message.message);
        }
        if !cur_content.is_empty() {
            req_messages.push(json!({
            "role": cur_role,
            "content": cur_content
            }));
        }
    }

    let mut response = state
        .client
        .post(format!(
            "https://api-inference.huggingface.co/models/{}/v1/chat/completions",
            model_name
        ))
        .header(
            header::AUTHORIZATION,
            format!(
                "Bearer {}",
                var("HF_API_KEY").expect("Huggingface API key should be provided .env file as HF_API_KEY. Get one at https://huggingface.co/settings/tokens")
            ),
        )
        .json(&body)
        .send()
        .await
        .map_err(AppError::from)?
        .json_array_stream::<serde_json::Value>(2048);

    let mut res_content = String::new();
    while let Some(mut bytes) = response.next().await {
        match bytes {
            Ok(ref mut bytes) => {
                res_content += bytes["choices"][0]["delta"]["content"]
                    .as_str()
                    .unwrap_or("");
            }
            Err(e) => return Err(AppError::from(e)),
        }
    }
    Ok((StatusCode::OK, Json(json!({"content": res_content}))).into_response())
}
