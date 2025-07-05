use axum::{Json, response::IntoResponse};
use serde::Serialize;
use utoipa::ToSchema;

use crate::api::domain::response_body::ApiResponseBody;

#[derive(Debug, ToSchema, Serialize)]
pub enum ApiError {
    InternalServerError(String),
    NotFoundError(String),
    BadRequestError(String),
    UnauthorizedError(String),
    ConflictError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        use ApiError::*;

        match self {
            InternalServerError(message) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponseBody::new(
                    message,
                )),
            )
                .into_response(),
            NotFoundError(message) => (
                axum::http::StatusCode::NOT_FOUND,
                Json(ApiResponseBody::new(message)),
            )
                .into_response(),
            BadRequestError(message) => (
                axum::http::StatusCode::BAD_REQUEST,
                Json(ApiResponseBody::new(message)),
            )
                .into_response(),
            UnauthorizedError(message) => (
                axum::http::StatusCode::UNAUTHORIZED,
                Json(ApiResponseBody::new(message)),
            )
                .into_response(),
            ConflictError(message) => (
                axum::http::StatusCode::CONFLICT,
                Json(ApiResponseBody::new(message)),
            )
                .into_response(),
        }
    }
}
