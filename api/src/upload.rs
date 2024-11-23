use std::{fs::create_dir, io::ErrorKind, path::PathBuf};

use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use base64::{engine::general_purpose, Engine};
use macros::response;
use mime::Mime;
use mime_guess::get_mime_extensions;
use reqwest::StatusCode;
use serde::Deserialize;
use sha2::{Digest, Sha256};
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
    /// Base64 encoded file data.
    file_data: String,
}

/// A file processed by the server.
pub struct AppFile {
    data: Vec<u8>,
    mime: Option<Mime>,
}

impl AppFile {
    /// Parse the base64 encoded data into a file.
    /// Data is expected to be in the format `data:[mime type];base64,[base64 encoded data]`
    /// Or just the base64 encoded data.
    pub fn from_base64(data: &str) -> Result<Self, AppError> {
        // Split the data into the mime type and the base64 encoded data
        // head should contain `data:[mime type];base64` and encoded_data should contain the base64 encoded data
        let (head, encoded_data) = match data.split_once(',') {
            Some((head, encoded_data)) => (Some(head), encoded_data),
            // Assume that the data is just the base64 encoded data
            None => (None, data),
        };

        let data = general_purpose::STANDARD.decode(encoded_data.as_bytes())?;

        // Attempt to infer the mime type from the file data
        let mut mime = infer::get(&data).and_then(|x| x.mime_type().parse::<Mime>().ok());

        // If the mime type is provided, check if it matches the actual mime type
        if let Some(mut head) = head {
            head = head
                .split_once(';')
                .ok_or_else(|| {
                    AppError::UserError((StatusCode::BAD_REQUEST, "Invalid data".into()))
                })?
                .0;
            head = head.strip_prefix("data:").unwrap_or(head);
            // Head should contain the mime type
            if mime.is_none() {
                // We could not determine the file type from the file data so
                // attempt to parse the mime type from the head
                mime = head.parse().ok();
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
    // Check if the base64 encoded file data is too large
    if upload_data.file_data.len() > 10_000_000 {
        return Err(AppError::UserError((
            StatusCode::PAYLOAD_TOO_LARGE,
            "File size too large".into(),
        )));
    }

    // Decode the base64 encoded data
    let upload_file = AppFile::from_base64(&upload_data.file_data)?;

    // Check if the file size is too large
    if upload_file.data.len() > 10_000_000 {
        return Err(AppError::UserError((
            StatusCode::PAYLOAD_TOO_LARGE,
            "File size too large".into(),
        )));
    }

    // Calculate the hash of the file to use as the filename
    let hash = Sha256::digest(&upload_file.data);

    let file_name = format!(
        "{:x}{}",
        hash,
        match upload_file
            .mime
            .as_ref()
            .and_then(|mime| get_mime_extensions(mime))
            .and_then(|exts| exts.first())
        {
            Some(ext) => format!(".{}", ext),
            None => String::new(),
        },
    );

    // Create the uploads directory if it does not
    // already exist and ignore the error if it does
    match create_dir("./uploads") {
        Err(e) if e.kind() != ErrorKind::AlreadyExists => return Err(e.into()),
        _ => (),
    }

    let mime = upload_file.mime.map(|mime| mime.to_string());
    let path = PathBuf::from(format!("uploads/{}", file_name));

    if !path.exists() {
        let mut file = File::create(&path).await?;
        file.write_all(&upload_file.data).await?;
    }

    let file_id = sqlx::query!(
            "INSERT INTO files (path, mime) VALUES (?, ?) ON CONFLICT DO UPDATE SET path = path RETURNING id",
            file_name,
            mime
        )
        .fetch_one(&state)
        .await?
        .id;

    let id = sqlx::query!(
            "INSERT INTO file_uploads (file_id, user_id) VALUES (?, ?) ON CONFLICT DO UPDATE SET file_id = file_id RETURNING file_id as id",
            file_id,
            user.id
        )
        .fetch_one(&state)
        .await?.id;

    Ok((
        StatusCode::CREATED,
        AppJson(response!("File uploaded successfully", id)),
    )
        .into_response())
}
