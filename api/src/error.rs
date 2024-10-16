use core::fmt;
use std::fmt::{Display, Formatter};

use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_macros::FromRequest;
use serde::Serialize;
use tracing::{error, warn};
use validator::Validate;

use crate::auth::JwtError;

/// Error that wraps `anyhow::Error`.
/// Useful to provide more fine grained error handling in our application.
/// Helps us debug errors in the code easier and gives the client a better idea of what went wrong.
pub enum AppError {
    JsonRejection(JsonRejection),
    SqlxError(sqlx::Error),
    SerdeError(serde_json::Error),
    ValidationError(Vec<AppValidationError>),
    AuthError(anyhow::Error),
    Generic(anyhow::Error),
}

/// A JSON response for errors that includes the error type and message
/// Used in both WebSockets and HTTP responses to notify the client of errors
#[derive(Serialize, Debug, Clone)]
pub struct ErrorResponse {
    r#type: String,
    message: String,
}

impl From<AppError> for ErrorResponse {
    fn from(value: AppError) -> Self {
        ErrorResponse {
            r#type: value.r#type(),
            message: value.to_string(),
        }
    }
}

impl From<JwtError> for ErrorResponse{
    fn from(value: JwtError) -> Self {
        ErrorResponse {
            r#type: "AuthError".to_owned(),
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
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
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
            | AppError::ValidationError(_) => warn!("{}", self),
            AppError::SqlxError(_) | AppError::Generic(_) => error!("{}", self),
        }
        let (status, message) = match &self {
            AppError::JsonRejection(rejection) => (rejection.status(), rejection.body_text()),
            AppError::ValidationError(e) => {
                (StatusCode::BAD_REQUEST, serde_json::to_string(&e).unwrap())
            }
            AppError::SerdeError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::AuthError(e) => (StatusCode::UNAUTHORIZED, e.to_string()),
            AppError::SqlxError(_) | AppError::Generic(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_owned(),
            ),
        };
        // Return a JSON response with the error type and message.
        (
            status,
            Json(ErrorResponse {
                r#type: self.r#type(),
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
        }
    }
}

// Implement `Display` for `AppError` to allow us to format the error as a string.
impl Display for AppError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            AppError::JsonRejection(rejection) => write!(f, "{}", rejection.body_text()),
            AppError::SerdeError(e) => write!(f, "{}", e),
            AppError::ValidationError(e) => write!(f, "{}", serde_json::to_string(&e).unwrap()),
            AppError::AuthError(e) => write!(f, "{}", e),
            AppError::SqlxError(e) => write!(f, "{}", e),
            AppError::Generic(err) => write!(f, "{}", err),
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
        } else if err.downcast_ref::<serde_json::Error>().is_some() {
            return Self::SerdeError(err.downcast().unwrap());
        } else {
            return Self::Generic(err);
        }
    }
}
