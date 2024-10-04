use core::fmt;
use std::fmt::{Display, Formatter};

use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_macros::FromRequest;
use log::error;
use serde::Serialize;
use validator::Validate;

// Make our own error that wraps `anyhow::Error`.
pub enum AppError {
    JsonRejection(JsonRejection),
    SqlxError(sqlx::Error),
    SerdeError(serde_json::Error),
    ValidationError(Vec<AppValidationError>),
    AuthError(anyhow::Error),
    Generic(anyhow::Error),
}

#[derive(Serialize, Debug, Clone)]
pub struct ErrorResponse {
    r#type: String,
    message: String,
}

impl From<AppError> for ErrorResponse{
    fn from(value: AppError) -> Self {
        ErrorResponse {
            r#type: value.r#type(),
            message: value.to_string(),
        }
    }
} 

// Create our own JSON extractor by wrapping `axum::Json`. This makes it easy to override the
// rejection and provide our own which formats errors to match our application.
//
// `axum::Json` responds with plain text if the input is invalid.
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
pub struct AppJson<T>(pub T);

#[derive(Serialize, Debug)]
pub struct AppValidationError {
    field: String,
    message: String,
}

pub trait AppValidate {
    fn app_validate(&self) -> Result<(), AppError>;
}

impl<T: Validate> AppValidate for T {
    fn app_validate(&self) -> Result<(), AppError> {
        if let Err(err) = self.validate() {
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

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!("{}", self);
        let (status, message) = match &self {
            AppError::JsonRejection(rejection) => (rejection.status(), rejection.body_text()),
            AppError::ValidationError(e) => (StatusCode::BAD_REQUEST, serde_json::to_string(&e).unwrap()),
            AppError::SerdeError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::AuthError(e) => (StatusCode::UNAUTHORIZED, e.to_string()),
            AppError::SqlxError(_) | AppError::Generic(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_owned(),
            ),
        };
        (status, Json(ErrorResponse { r#type: self.r#type(), message })).into_response()
    }
}

impl AppError{
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

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        let err: anyhow::Error = err.into();
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
