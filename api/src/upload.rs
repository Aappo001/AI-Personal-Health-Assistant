use std::path::PathBuf;

use anyhow::anyhow;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use base64::{engine::general_purpose, Engine};
use chrono::NaiveDateTime;
use mime::Mime;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use sqlx::SqlitePool;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::error::{AppError, AppJson};

/// A file to be uploaded to the server.
#[derive(Deserialize)]
pub struct FileUpload {
    /// The name of the file to be stored.
    file_name: Option<String>,
    /// Base64 encoded file data.
    file_data: String,
}

/// A file processed by the server.
pub struct AppFile {
    data: Vec<u8>,
    mime: infer::Type,
}

impl AppFile {
    pub fn from_base64(data: &str) -> Result<Self, AppError> {
        let (head, encoded_data) = match data.split_once(',') {
            Some((head, encoded_data)) => (Some(head), encoded_data),
            // Assume that the data is just the base64 encoded data
            None => (None, data),
        };
        let data = general_purpose::STANDARD.decode(encoded_data.as_bytes())?;
        let mime = infer::get(&data).ok_or_else(|| {
            AppError::UserError((
                StatusCode::BAD_REQUEST,
                "Unable to determine mime type".into(),
            ))
        })?;
        if let Some(mut head) = head {
            head = head
                .split_once(';')
                .ok_or_else(|| {
                    AppError::UserError((StatusCode::BAD_REQUEST, "Invalid data".into()))
                })?
                .0;
            head = head.strip_prefix("data:").unwrap_or(head);
            if head != mime.mime_type() {
                return Err(AppError::UserError((
                    StatusCode::BAD_REQUEST,
                    "Mime type mismatch".into(),
                )));
            }
        }
        Ok(Self { data, mime })
    }
}

/// A file that has been uploaded to the server.
#[derive(Serialize)]
pub struct UploadedFile {
    id: i64,
    name: String,
    path: String,
    created_at: NaiveDateTime,
    modified_at: NaiveDateTime,
}

pub async fn upload_file(
    State(state): State<SqlitePool>,
    AppJson(upload_data): AppJson<FileUpload>,
) -> Result<Response, AppError> {
    // Decode the base64 encoded data
    let upload_file = AppFile::from_base64(&upload_data.file_data)?;

    // Check if the file size is too large
    if upload_file.data.len() > 10_000_000 {
        return Err(AppError::UserError((
            StatusCode::BAD_REQUEST,
            "File size too large".into(),
        )));
    }

    // Calculate the hash of the file to use as the filename
    let hash = Sha1::digest(&upload_file.data).to_vec();

    let file_name = format!(
        "{}.{}",
        hash.iter().map(|&x| x as char).collect::<String>(),
        upload_file.mime.extension()
    );
    let path = PathBuf::from(format!(
        "./uploads/{}",
        file_name
    ));

    if !path.exists() {
        let mut file = File::create(&path).await?;
        file.write_all(&upload_file.data).await?;
        let response_data = sqlx::query_as!(UploadedFile, "INSERT INTO files (name, path) VALUES (?, ?) RETURNING *", upload_data.file_name, file_name)
            .fetch_one(&state)
            .await?;
        Ok((StatusCode::CREATED, AppJson(response_data)).into_response())
    } else {
        let response_data = sqlx::query_as!(UploadedFile, "SELECT * FROM files WHERE path = ?", file_name)
            .fetch_one(&state)
            .await?;
        Ok((StatusCode::OK, AppJson(response_data)).into_response())
    }
}
