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

pub struct JwtAuth<T>(pub T);

pub enum JwtError {
    InvalidToken,
    MissingToken,
}

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

#[async_trait]
impl<T, S> FromRequestParts<S> for JwtAuth<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = JwtError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Some(token) = parts.headers.get(AUTHORIZATION) else {
            return Err(JwtError::MissingToken);
        };
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
