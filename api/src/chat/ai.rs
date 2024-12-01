use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use dotenvy::var;
use futures::StreamExt;
use reqwest::{header, StatusCode};
use reqwest_streams::*;
use serde::Serialize;
// use sonic_rs::{json, JsonValueTrait, JsonValueMutTrait};
use serde_json::json;
use sqlx::SqlitePool;
use tracing::debug;

use crate::{
    error::{AppError, AppJson},
    state::{AppState, Sender},
    users::UserToken,
};

use super::{SendMessage, SocketResponse};

/// Stream data from the AI model
// Might add a field for whether the message should trigger the AI
// Does not have an id because it is not yet saved in the database
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StreamMessage {
    pub conversation_id: i64,
    /// The content of the current message
    // You must combine consecutive messages from the same role into a single message
    pub message: Option<String>,
    /// The id of the user who initiated the ai the message
    pub querier_id: i64,
}

/// An AI model that can be used to generate responses
#[derive(Serialize)]
pub struct AiModel {
    pub id: i64,
    pub name: String,
}

/// Query the AI model with the messages in the conversation
/// Return's the ai's response
pub async fn query_model(
    state: &AppState,
    message: &SendMessage,
    user: &UserToken,
) -> Result<String, AppError> {
    let model_id = message.ai_model_id.expect("Model ID should be provided");
    let conversation_id = message
        .conversation_id
        .expect("Conversation ID should be provided");
    let model = sqlx::query!("SELECT name FROM ai_models WHERE id = ?", model_id)
        .fetch_one(&state.pool)
        .await?;
    // Build the default request body for the AI model
    let mut body = json!({
        "model": model.name,
        "messages": [
        { "role": "system", "content": r#"You are a medical professional who knows about medicine.  When the user tells you about a health problem that they are facing, continue probing through the problem to extract more information and attempt to gain a better understanding of a root cause and potential remedies. Do not simply give a list of potential causes without asking further questions. If you are unsure about something refer user to a doctor or medical professional. The name of the user who sent the message will be enclosed in braces like "{username}:". You should refer to the user who you are responding to by name"# },
    ],
        "temperature": 0.5,
        "max_tokens": 1024,
        "top_p": 0.7,
    // Enable streaming so we can get the response as it comes in
        "stream": true
    });

    // Populate the messages array with the messages in the conversation
    if let Some(req_messages) = body["messages"].as_array_mut() {
        // Query the messages as a stream to save memory
        // This saves a ton on longer conversations
        // Only select the most recent messages that add up to less than 5000 characters
        // This is to prevent the AI from getting stuck on very long conversations
        // and token limits from the api
        let mut db_messages = sqlx::query!(
        "WITH ranked_messages AS (
            SELECT
                messages.message,
                messages.user_id,
                users.username,
                SUM(LENGTH(messages.message)) OVER (PARTITION BY messages.conversation_id ORDER BY messages.created_at DESC) AS cumulative_length,
                messages.created_at
            FROM
                messages
            LEFT JOIN
                users ON messages.user_id = users.id
            WHERE
                messages.conversation_id = ?
        )
        SELECT
            message,
            user_id,
            username
        FROM
            ranked_messages
        WHERE
            cumulative_length <= 5000
        ORDER BY
            created_at ASC",
            conversation_id
        )
        .fetch(&state.pool);

        // If we don't alternate between user and assistant messages, the AI will give us an error and
        // get stuck so we need to concatenate consecutive user and system messages together
        let mut last_user = None;
        let mut cur_content = String::new();
        let mut first = true;
        while let Some(message) = db_messages.next().await {
            let message = message?;
            match (&last_user, &message.username) {
                // If the last message was from a user and the current message is from the assistant
                // or vice versa
                (None, Some(_)) | (Some(_), None) if !first => {
                    req_messages.push(json!({
                        "role": if last_user.is_some() { "user" } else { "assistant" },
                        "content": cur_content
                    }));
                    cur_content.clear();
                }
                _ => (),
            }
            match (&last_user, &message.username) {
                (Some(last), Some(cur)) => {
                    if last != cur {
                        // Prepend the user's username to the message only if they are not the
                        // sender of the previous message.
                        // Uses `{{{}}}` insteadd of `{{}}` because `{{}}` is used to escape curly braces
                        cur_content.push_str(&format!("{{{}}}:", cur));
                    }
                }
                (None, Some(cur)) => {
                    cur_content.push_str(&format!("{{{}}}:", cur));
                }
                (None, None) | (Some(_), None) => (),
            }
            cur_content.push_str(&message.message);
            last_user = message.username;
            first = false;
        }
        req_messages.push(json!({
        "role": if last_user.is_some() { "user" } else { "assistant" },
        "content": cur_content
        }));

        let form = sqlx::query!(
            "SELECT height, weight, sleep_hours, exercise_duration, food_intake, notes, modified_at FROM user_statistics WHERE user_id = ? ORDER BY created_at DESC LIMIT 1",
            user.id
        )
        .fetch_optional(&state.pool)
        .await?;

        if let Some(form) = form {
            let time_diff = chrono::Utc::now().naive_utc() - form.modified_at;
            let content = format!("{} filled out a health form {} that contains the following details: {}{}{}{}{}{}{}",
                user.username,
                match time_diff {
                    _ if time_diff.num_weeks() > 0 => format!("{} weeks ago", time_diff.num_weeks()),
                    _ if time_diff.num_days() > 0 => format!("{} days ago", time_diff.num_days()),
                    _ if time_diff.num_hours() > 0 => format!("{} hours ago", time_diff.num_hours()),
                    _ if time_diff.num_minutes() > 0 => format!("{} minutes ago", time_diff.num_minutes()),
                    _ if time_diff.num_seconds() > 0 => format!("{} seconds ago", time_diff.num_minutes()),
                    _ => "just now".to_string()
                },
                (chrono::Utc::now().naive_utc() - form.modified_at),
                match form.height {
                    Some(height) => format!("Height: {} cm\n", height),
                    None => "".to_string()
                },
                match form.weight {
                    Some(weight) => format!("Weight: {} kg\n", weight),
                    None => "".to_string()
                },
                match form.sleep_hours {
                    Some(sleep_hours) => format!("Sleep Hours: {} hours\n", sleep_hours),
                    None => "".to_string()
                },
                match form.exercise_duration {
                    Some(exercise_duration) => format!("Exercise Duration: {} minutes\n", exercise_duration),
                    None => "".to_string()
                },
                match form.food_intake {
                    Some(food_intake) => format!("Food Intake: {}\n", food_intake),
                    None => "".to_string()
                },
                match form.notes {
                    Some(notes) => format!("Notes: {}\n", notes),
                    None => "".to_string()
                }
            );
            req_messages.push(json!({
                "role": "system",
                "content": content
            }));
        }

        debug!("Querying AI model with: {:?}", req_messages);
    }

    let mut response = state
        .client
        .post(format!(
            "https://api-inference.huggingface.co/models/{}/v1/chat/completions",
            model.name
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
        .await?
        // Handle the response as a stream
        // Using serde_json::Value instead of sonic_rs::Value because it breaks for some reason
        // and gives a CodecError. I tried looking it up every where and even read through the
        // source of both reqwest_streams and sonic_rs but I couldn't figure it out.
        .json_array_stream::<serde_json::Value>(2048);

    // The accumulated response from the AI model
    let mut res_content = String::new();

    // Get a sender handle to all of the connected clients in the conversation
    // This is done, instead of calling `broadcast_event` in a loop, before streaming the response for two main reasons
    // #1 it prevents newly connected clients from receiving a half-baked response
    // #2 it avoids having to query the database for the conversation senders for each message in
    // the stream, which can be very expensive for large messages and conversations
    let senders = get_conversation_senders(state, conversation_id).await?;

    while let Some(mut bytes) = response.next().await {
        match bytes {
            Ok(ref mut bytes) => {
                // Stream the individual messages to the clients
                for sender in &senders {
                    sender
                        .send(SocketResponse::StreamData(StreamMessage {
                            conversation_id,
                            message: Some(
                                bytes["choices"][0]["delta"]["content"]
                                    .as_str()
                                    .unwrap_or("")
                                    .to_string(),
                            ),
                            querier_id: user.id,
                        }))
                        .await?;
                }
                // Accumulate the response content
                res_content += bytes["choices"][0]["delta"]["content"]
                    .as_str()
                    .unwrap_or("");
            }
            Err(e) => return Err(AppError::from(e)),
        }
    }

    // Broadcast the that the AI model has finished processing
    for sender in &senders {
        sender
            .send(SocketResponse::StreamData(StreamMessage {
                conversation_id,
                message: None,
                querier_id: user.id,
            }))
            .await?;
    }

    Ok(res_content)
}

/// Get sender handles for all the connected clients in the conversation
async fn get_conversation_senders(
    state: &AppState,
    conversation_id: i64,
) -> Result<Vec<Sender<SocketResponse>>, AppError> {
    let user_records = sqlx::query!(
        "SELECT user_id FROM user_conversations WHERE conversation_id = ?",
        conversation_id
    )
    .fetch_all(&state.pool)
    .await?;

    let mut senders = Vec::new();

    for record in user_records {
        if let Some(user_connections) = state
            .user_sockets
            .read_async(&record.user_id, |_, v| v.connections.clone())
            .await
        {
            for sender in user_connections.into_iter().flatten() {
                senders.push(sender.channel);
            }
        }
    }
    Ok(senders)
}

/// Returns all the AI models in the database
pub async fn get_ai_models(State(pool): State<SqlitePool>) -> Result<Response, AppError> {
    Ok((
        StatusCode::OK,
        AppJson(
            sqlx::query_as!(AiModel, "SELECT * FROM ai_models")
                .fetch_all(&pool)
                .await
                .map_err(AppError::from)?,
        ),
    )
        .into_response())
}
