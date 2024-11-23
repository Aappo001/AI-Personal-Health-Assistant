use core::fmt;
use std::fmt::{Display, Formatter};

use axum::{
    async_trait,
    extract::{
        rejection::{JsonRejection, MissingJsonContentType},
        FromRequest, Request,
    },
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use bytes::{BufMut, Bytes, BytesMut};
use reqwest::header;
use serde::{de::DeserializeOwned, Serialize};
use tracing::{error, warn};
use validator::Validate;

use crate::auth::JwtError;

/// Error that wraps `anyhow::Error`.
/// Useful to provide more fine grained error handling in our application.
/// Helps us debug errors in the code easier and gives the client a better idea of what went wrong.
pub enum AppError {
    JsonRejection(JsonRejection),
    SqlxError(sqlx::Error),
    SerdeError(sonic_rs::Error),
    ValidationError(Vec<AppValidationError>),
    AuthError(anyhow::Error),
    UserError((StatusCode, Box<str>)),
    Generic(anyhow::Error),
}

/// A JSON response for errors that includes the error type and message
/// Used in both WebSockets and HTTP responses to notify the client of errors
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    error_type: String,
    message: String,
}

impl From<AppError> for ErrorResponse {
    fn from(value: AppError) -> Self {
        ErrorResponse {
            error_type: value.r#type(),
            message: value.to_string(),
        }
    }
}

impl From<JwtError> for ErrorResponse {
    fn from(value: JwtError) -> Self {
        ErrorResponse {
            error_type: "AuthError".to_owned(),
            message: value.to_string(),
        }
    }
}

// Create our own JSON extractor by wrapping `axum::Json`. This makes it easy to override the
// rejection and provide our own which formats errors to match our application.
//
// `axum::Json` responds with plain text if the input is invalid.
/// A wrapper around `axum::Json` that provides a custom rejection to return JSON errors
/// and allows us to intercept errors and provide a more detailed error message
pub struct AppJson<T>(pub T);

/// A more descriptive error message for validation errors
#[derive(Serialize, Debug)]
pub struct AppValidationError {
    field: String,
    message: String,
}

/// An error type for validation errors
/// This is useful because we can return a JSON response with the error type and message
/// to provide the client with a clearer error message than what the default `validator`
/// crate provides.
pub trait AppValidate {
    fn app_validate(&self) -> Result<(), AppError>;
}

impl<T: Validate> AppValidate for T {
    fn app_validate(&self) -> Result<(), AppError> {
        // If validation fails, return a JSON response with the error type and message
        if let Err(err) = self.validate() {
            // Iterater over the field errors and map them to `AppValidationError`
            let errors: Vec<AppValidationError> = err
                .field_errors()
                .iter()
                .flat_map(|(field, errors)| {
                    errors.iter().map(move |error| AppValidationError {
                        field: field.to_string(),
                        message: error.code.to_string(),
                    })
                })
                .collect();
            return Err(AppError::ValidationError(errors));
        }
        Ok(())
    }
}

/// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Warn about user errors and log them, but error about server errors
        match self {
            AppError::JsonRejection(_)
            | AppError::AuthError(_)
            | AppError::SerdeError(_)
            | AppError::ValidationError(_)
            | AppError::UserError(_) => warn!("{}", self),
            AppError::SqlxError(_) | AppError::Generic(_) => error!("{}", self),
        }
        let (status, message) = match &self {
            AppError::JsonRejection(rejection) => (rejection.status(), rejection.body_text()),
            AppError::ValidationError(e) => {
                (StatusCode::BAD_REQUEST, sonic_rs::to_string(&e).unwrap())
            }
            AppError::SerdeError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::AuthError(e) => (StatusCode::UNAUTHORIZED, e.to_string()),
            AppError::UserError((code, e)) => (*code, e.to_string()),
            AppError::SqlxError(_) | AppError::Generic(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_owned(),
            ),
        };
        // Return a JSON response with the error type and message.
        (
            status,
            AppJson(ErrorResponse {
                error_type: self.r#type(),
                message,
            }),
        )
            .into_response()
    }
}

impl AppError {
    /// Get the error type as a string to notify the client of what went wrong
    pub fn r#type(&self) -> String {
        match self {
            AppError::JsonRejection(_) => "JsonRejection".to_owned(),
            AppError::ValidationError(_) => "ValidationError".to_owned(),
            AppError::SerdeError(_) => "SerdeError".to_owned(),
            AppError::AuthError(_) => "AuthError".to_owned(),
            AppError::SqlxError(_) => "SqlxError".to_owned(),
            AppError::Generic(_) => "Generic".to_owned(),
            AppError::UserError(_) => "User".to_owned(),
        }
    }
}

// Implement `Display` for `AppError` to allow us to format the error as a string.
impl Display for AppError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            AppError::JsonRejection(rejection) => write!(f, "{}", rejection.body_text()),
            AppError::SerdeError(e) => write!(f, "{}", e),
            AppError::ValidationError(e) => write!(f, "{}", sonic_rs::to_string(&e).unwrap()),
            AppError::AuthError(e) => write!(f, "{}", e),
            AppError::SqlxError(e) => write!(f, "{}", e),
            AppError::Generic(err) => write!(f, "{}", err),
            AppError::UserError((_, err)) => write!(f, "{}", err),
        }
    }
}

// Implement `From` for `AppError` to implicitly convert from `anyhow::Error`
// This lets us use `?` without having to wrap every error in `AppError` because the compiler will
// authomatically convert it for us.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        let err: anyhow::Error = err.into();
        // Use downcast_ref to check the underlying error type and return the appropriate variant
        // we can't use downcast to check because it consumes the error and does not implement `Clone`
        // We don't need to add `AuthError` or `ValidationError` because we will handle those
        // explicitly in our application.
        if err.downcast_ref::<JsonRejection>().is_some() {
            return Self::JsonRejection(err.downcast().unwrap());
        } else if err.downcast_ref::<sqlx::Error>().is_some() {
            return Self::SqlxError(err.downcast().unwrap());
        } else if err.downcast_ref::<sonic_rs::Error>().is_some() {
            return Self::SerdeError(err.downcast().unwrap());
        } else {
            return Self::Generic(err);
        }
    }
}

impl<T> AppJson<T>
where
    T: DeserializeOwned,
{
    /// Construct a `Json<T>` from a byte slice. Most users should prefer to use the `FromRequest` impl
    /// but special cases may require first extracting a `Request` into `Bytes` then optionally
    /// constructing a `Json<T>`.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, AppError> {
        let deserializer = &mut sonic_rs::Deserializer::from_slice(bytes);

        let value = match serde::Deserialize::deserialize(deserializer) {
            Ok(value) => value,
            Err(err) => {
                return Err(err.into());
            }
        };

        Ok(AppJson(value))
    }
}

fn json_content_type(headers: &HeaderMap) -> bool {
    let content_type = if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
        content_type
    } else {
        return false;
    };

    let content_type = if let Ok(content_type) = content_type.to_str() {
        content_type
    } else {
        return false;
    };

    let mime = if let Ok(mime) = content_type.parse::<mime::Mime>() {
        mime
    } else {
        return false;
    };

    let is_json_content_type = mime.type_() == "application"
        && (mime.subtype() == "json" || mime.suffix().map_or(false, |name| name == "json"));

    is_json_content_type
}

#[async_trait]
impl<T, S> FromRequest<S> for AppJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        if json_content_type(req.headers()) {
            let bytes = Bytes::from_request(req, state).await?;
            Self::from_bytes(&bytes)
        } else {
            Err(AppError::JsonRejection(
                JsonRejection::MissingJsonContentType(MissingJsonContentType::default()),
            ))
        }
    }
}

// Use `AppJson` instead of `Json` to utilize `sonic_rs` for serialization
// instead of `serde_json` which is slower.
impl<T> IntoResponse for AppJson<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        // Use a small initial capacity of 128 bytes like sonic_rs::to_vec
        // https://docs.rs/sonic_rs/1.0.82/src/sonic_rs/ser.rs.html#2189
        let mut buf = BytesMut::with_capacity(128).writer();
        match sonic_rs::to_writer(&mut buf, &self.0) {
            Ok(()) => (
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
                )],
                buf.into_inner().freeze(),
            )
                .into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                )],
                err.to_string(),
            )
                .into_response(),
        }
    }
}
