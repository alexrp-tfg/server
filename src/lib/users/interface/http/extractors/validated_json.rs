use axum::{
    Json,
    body::Body,
    extract::{FromRequest, Request, rejection::JsonRejection},
    http::{Response, StatusCode},
    response::IntoResponse,
};
use serde::de::DeserializeOwned;
use thiserror::Error;
use validator::Validate;

use crate::api::domain::{errors::ApiError, response_body::ApiResponseBody};

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),

    #[error(transparent)]
    JsonRejectionError(#[from] JsonRejection),
}

impl IntoResponse for ValidationError {
    fn into_response(self) -> Response<Body> {
        match self {
            ValidationError::ValidationError(err) => {
                let mut errors_string = String::new();

                for (field, field_errors) in err.field_errors() {
                    for error in field_errors {
                        if let Some(message) = &error.message {
                            if !errors_string.is_empty() {
                                errors_string.push_str(", ");
                            }
                            errors_string.push_str(&format!("{}: {}", field, message));
                        }
                    }
                }
                (ApiError::BadRequestError(errors_string)).into_response()
            }
            ValidationError::JsonRejectionError(err) => match err {
                JsonRejection::JsonDataError(_) => (ApiError::BadRequestError(
                    "Missing body properties, please check the documentation".to_string(),
                ))
                .into_response(),
                JsonRejection::JsonSyntaxError(err) => {
                    (ApiError::BadRequestError(err.to_string())).into_response()
                }
                JsonRejection::MissingJsonContentType(_) => {
                    (ApiError::BadRequestError("Missing JSON content type".to_string()))
                        .into_response()
                        .into_response()
                }
                _ => (ApiError::BadRequestError("Invalid JSON data".to_string())).into_response(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = ValidationError;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let Json(data) = Json::<T>::from_request(req, state).await?;
        data.validate()?;
        Ok(ValidatedJson(data))
    }
}
