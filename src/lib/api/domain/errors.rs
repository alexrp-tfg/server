use axum::{Json, response::IntoResponse};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, ToSchema, Serialize, Clone, PartialEq, Eq)]
pub struct ApiErrorBody {
    pub message: String,
}

impl ApiErrorBody {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

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
                Json(ApiErrorBody::new(
                    message,
                )),
            )
                .into_response(),
            NotFoundError(message) => (
                axum::http::StatusCode::NOT_FOUND,
                Json(ApiErrorBody::new(message)),
            )
                .into_response(),
            BadRequestError(message) => (
                axum::http::StatusCode::BAD_REQUEST,
                Json(ApiErrorBody::new(message)),
            )
                .into_response(),
            UnauthorizedError(message) => (
                axum::http::StatusCode::UNAUTHORIZED,
                Json(ApiErrorBody::new(message)),
            )
                .into_response(),
            ConflictError(message) => (
                axum::http::StatusCode::CONFLICT,
                Json(ApiErrorBody::new(message)),
            )
                .into_response(),
        }
    }
}
