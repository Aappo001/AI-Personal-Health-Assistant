use std::fmt::Display;

use crate::error::ErrorResponse;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use dotenv_codegen::dotenv;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::de::DeserializeOwned;

/// Custom extractor for JWT authoriation
pub struct JwtAuth<T>(pub T);

/// Error that occurs when JWT authorization fails
pub enum JwtError {
    InvalidToken,
    MissingToken,
}

/// Error message for `JwtError`
impl Display for JwtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidToken => write!(f, "Invalid token"),
            Self::MissingToken => write!(f, "No token provided"),
        }
    }
}

impl IntoResponse for JwtError {
    fn into_response(self) -> Response {
        (StatusCode::UNAUTHORIZED, Json(ErrorResponse::from(self))).into_response()
    }
}

// Trait that allows us to use the struct as an extractor in the function
// signature of a request handler
#[async_trait]
impl<T, S> FromRequestParts<S> for JwtAuth<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = JwtError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the request headers
        let Some(token) = parts.headers.get(AUTHORIZATION) else {
            return Err(JwtError::MissingToken);
        };
        // Attempt to decode the token
        let user: T = decode(
            token
                .to_str()
                .map_err(|_| JwtError::InvalidToken)?
                .strip_prefix("Bearer ")
                .ok_or(JwtError::InvalidToken)?,
            &DecodingKey::from_secret(dotenv!("JWT_KEY").as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| JwtError::InvalidToken)?
        .claims;
        Ok(Self(user))
    }
}
