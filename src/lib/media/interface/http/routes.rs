use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    routing::{get, post},
    Json, Extension,
};
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{
    api::{
        domain::{
            errors::{ApiError, ApiErrorBody},
            response_body::ApiResponseBody,
        },
        http_server::AppState,
    },
    media::{
        application::{
            commands::upload_media::{upload_media_command_handler, UploadMediaCommand, UploadMediaResult},
            queries::get_media_files::{get_media_files_query_handler, GetMediaFilesQuery, GetMediaFilesResult},
        },
        domain::{MediaUploadError},
    },
    protected,
    users::domain::Claims,
};

#[utoipa::path(
    post,
    path = "/upload",
    description = "Upload media files (images/videos)",
    tag = "media",
    request_body(
        content_type = "multipart/form-data"
    ),
    responses(
        (status = 201, description = "Media uploaded successfully", body = ApiResponseBody<UploadMediaResult>),
        (status = 400, description = "Invalid file or file too large", body = ApiErrorBody),
        (status = 500, description = "Internal server error", body = ApiErrorBody)
    ),
    security(("bearer_auth" = [])),
)]
pub async fn upload_media(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<ApiResponseBody<UploadMediaResult>>), ApiError> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    let mut content_type: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ApiError::BadRequestError(format!("Invalid multipart data: {}", e))
    })? {
        let field_name = field.name().unwrap_or("").to_string();
        
        if field_name == "file" {
            filename = field.file_name().map(|name| name.to_string());
            content_type = field.content_type().map(|ct| ct.to_string());
            file_data = Some(field.bytes().await.map_err(|e| {
                ApiError::BadRequestError(format!("Failed to read file data: {}", e))
            })?.to_vec());
        }
    }

    let file_data = file_data.ok_or_else(|| {
        ApiError::BadRequestError("No file provided".to_string())
    })?;

    let original_filename = filename.ok_or_else(|| {
        ApiError::BadRequestError("No filename provided".to_string())
    })?;

    let content_type = content_type.ok_or_else(|| {
        ApiError::BadRequestError("No content type provided".to_string())
    })?;

    // Generate unique filename
    let file_extension = original_filename.split('.').last().unwrap_or("unknown");
    let unique_filename = format!("{}_{}.{}", Uuid::new_v4(), chrono::Utc::now().timestamp(), file_extension);

    let command = UploadMediaCommand {
        user_id: claims.sub,
        filename: unique_filename,
        original_filename,
        file_data,
        content_type,
    };

    match upload_media_command_handler(
        command,
        state.media_repository.as_ref(),
        state.storage_service.as_ref(),
    )
    .await
    {
        Ok(result) => Ok((StatusCode::CREATED, ApiResponseBody::new(result).into())),
        Err(err) => match err {
            MediaUploadError::InvalidFileType => Err(ApiError::BadRequestError("Invalid file type. Only images and videos are allowed".to_string())),
            MediaUploadError::FileTooLarge => Err(ApiError::BadRequestError("File too large. Maximum size is 100MB".to_string())),
            MediaUploadError::StorageError(msg) => Err(ApiError::InternalServerError(format!("Storage error: {}", msg))),
            MediaUploadError::InternalServerError(msg) => Err(ApiError::InternalServerError(msg)),
        }
    }
}

#[utoipa::path(
    get,
    path = "",
    description = "Get user's media files",
    tag = "media",
    responses(
        (status = 200, description = "Media files retrieved successfully", body = ApiResponseBody<Vec<GetMediaFilesResult>>),
        (status = 500, description = "Internal server error", body = ApiErrorBody)
    ),
    security(("bearer_auth" = [])),
)]
pub async fn get_media_files(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<(StatusCode, Json<ApiResponseBody<Vec<GetMediaFilesResult>>>), ApiError> {
    let query = GetMediaFilesQuery {
        user_id: claims.sub,
    };

    match get_media_files_query_handler(query, state.media_repository.as_ref()).await {
        Ok(media_files) => Ok((StatusCode::OK, ApiResponseBody::new(media_files).into())),
        Err(_) => Err(ApiError::InternalServerError("Failed to retrieve media files".to_string())),
    }
}

pub fn api_routes(
    state: AppState,
) -> axum::Router<AppState> {
    axum::Router::new()
        .route("/upload", post(upload_media))
        .route("/", get(get_media_files))
        .route_layer(protected!(state.clone()))
}

#[derive(OpenApi)]
#[openapi(
    paths(upload_media, get_media_files),
    tags(
        (name = "media", description = "Media upload and management API")
    )
)]
pub struct ApiDoc;

pub fn combine_openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
