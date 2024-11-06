use std::{fs::create_dir, io::ErrorKind, path::PathBuf};

use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use base64::{engine::general_purpose, Engine};
use macros::response;
use reqwest::StatusCode;
use serde::Deserialize;
use sha1::{Digest, Sha1};
use sqlx::SqlitePool;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::{
    auth::JwtAuth,
    error::{AppError, AppJson},
    users::UserToken,
};

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

pub async fn upload_file(
    State(state): State<SqlitePool>,
    JwtAuth(user): JwtAuth<UserToken>,
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
    let hash = Sha1::digest(&upload_file.data);

    let file_name = format!("{:x}.{}", hash, upload_file.mime.extension());
    match create_dir("./uploads") {
        Err(e) if e.kind() == ErrorKind::AlreadyExists => (),
        Err(e) => return Err(e.into()),
        _ => (),
    }
    let path = PathBuf::from(format!("./uploads/{}", file_name));

    let file_id = if !path.exists() {
        let mut file = File::create(&path).await?;
        file.write_all(&upload_file.data).await?;
        sqlx::query!(
            "INSERT INTO files (path) VALUES (?) ON CONFLICT DO NOTHING RETURNING id",
            file_name
        )
        .fetch_one(&state)
        .await?
        .id
    } else {
        sqlx::query!("SELECT id FROM files WHERE path = ?", file_name)
            .fetch_one(&state)
            .await?
            .id
    };

    let upload_name = upload_data.file_name.unwrap_or(file_name.clone());
    let id = sqlx::query!(
            "INSERT INTO file_uploads (file_id, user_id, name) VALUES (?, ?, ?) ON CONFLICT DO NOTHING RETURNING file_id as id",
            file_id,
            user.id,
            upload_name
        )
        .fetch_one(&state)
        .await?.id;

    Ok((
        StatusCode::CREATED,
        AppJson(response!("File uploaded successfully", id)),
    )
        .into_response())
}
