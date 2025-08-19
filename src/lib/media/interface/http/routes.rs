use std::str::FromStr;

use axum::{
    Extension, Json,
    body::Body,
    extract::{Path, Request, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use bytes::Bytes;
use futures_util::{StreamExt, TryStreamExt};
use multer::Multipart;
use serde;
use utoipa::OpenApi;
use uuid::Uuid;
use validator::Validate;

use crate::{
    api::{
        domain::{
            errors::{ApiError, ApiErrorBody},
            response_body::ApiResponseBody,
        },
        http_server::AppState,
    },
    media::{
        GetMediaStreamError, GetMediaStreamQuery,
        application::{
            commands::{
                delete_media::{
                    DeleteMediaCommand, DeleteMediaResult, delete_media_command_handler,
                },
                upload_media::{
                    UploadMediaCommand, UploadMediaResult, upload_media_command_handler,
                },
            },
            queries::get_media_files::{
                GetMediaFilesQuery, GetMediaFilesResult, get_media_files_query_handler,
            },
        },
        domain::{MediaDeleteError, MediaUploadError},
        get_media_stream_query_handler,
    },
    protected,
    users::domain::Claims,
};

#[derive(Validate, serde::Deserialize, utoipa::ToSchema)]
pub struct RequestUploadRequestBody {
    #[validate(length(min = 1, message = "Filename cannot be empty"))]
    filename: String,
    #[validate(range(min = 1, message = "File size must be greater than 0"))]
    file_size: u64,
    #[validate(length(min = 1, message = "Content type cannot be empty"))]
    content_type: String,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
#[allow(unused)]
struct UploadMediaRequestBody {
    #[schema(value_type = u8, format = "binary")]
    file: u8,
}

#[utoipa::path(
    post,
    path = "/upload",
    description = "Upload media files (images/videos)",
    tag = "media",
    request_body(
        content_type = "multipart/form-data", content = UploadMediaRequestBody,
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
    req: Request<Body>,
) -> Result<(StatusCode, Json<ApiResponseBody<UploadMediaResult>>), ApiError> {
    // Get content type for boundary detection
    let content_type = req
        .headers()
        .get("content-type")
        .and_then(|ct| ct.to_str().ok())
        .ok_or_else(|| ApiError::BadRequestError("Missing content-type header".to_string()))?;

    let file_size = req
        .headers()
        .get("x-file-size")
        .and_then(|cl| cl.to_str().ok())
        .and_then(|cl| cl.parse::<u64>().ok())
        .ok_or_else(|| {
            ApiError::BadRequestError("Missing or invalid x-file-size header".to_string())
        })?;

    // Extract boundary from content-type
    let boundary = multer::parse_boundary(content_type)
        .map_err(|_| ApiError::BadRequestError("Invalid multipart boundary".to_string()))?;

    // Convert axum body to stream with larger buffer sizes for better performance
    let body_stream = req
        .into_body()
        .into_data_stream()
        .map(|result| result.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err)));

    // Create multer multipart
    let mut multipart = Multipart::new(body_stream, boundary);

    // Store field data outside the loop
    let mut file_field: Option<multer::Field> = None;
    let mut filename: Option<String> = None;
    let mut field_content_type: Option<String> = None;

    // Process multipart fields
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::BadRequestError("Invalid multipart data".to_string()))?
    {
        if let Some(field_name) = field.name() {
            if field_name == "file" {
                filename = field.file_name().map(|name| name.to_string());
                field_content_type = field.content_type().map(|ct| ct.to_string());
                file_field = Some(field);
                break; // Exit loop once we find the file field
            }
        }
    }

    // Extract the stored field data
    let file_field =
        file_field.ok_or_else(|| ApiError::BadRequestError("No file provided".to_string()))?;

    let filename =
        filename.ok_or_else(|| ApiError::BadRequestError("No filename provided".to_string()))?;

    let content_type = field_content_type
        .ok_or_else(|| ApiError::BadRequestError("No content type provided".to_string()))?;

    // Generate unique filename
    let file_extension = filename.split('.').next_back().unwrap_or("unknown");
    let unique_filename = format!(
        "{}_{}.{}",
        Uuid::new_v4(),
        chrono::Utc::now().timestamp(),
        file_extension
    );

    // Create stream from field
    let file_stream = file_field.map(|chunk_result| {
        chunk_result.map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to read file data: {}", e),
            )
        })
    });

    let boxed_file_data = Box::pin(file_stream);

    let command = UploadMediaCommand {
        user_id: claims.sub,
        filename: unique_filename.clone(),
        file_size: Some(file_size),
        original_filename: filename,
        file_data: boxed_file_data,
        content_type: content_type.clone(),
    };

    match upload_media_command_handler(
        state.media_repository.as_ref(),
        state.storage_service.as_ref(),
        command,
    )
    .await
    {
        Ok(result) => {
            let media_id = result.id;
            let file_path = format!("media/{}/{}", claims.sub, unique_filename);
            let thumbnail_service = state.thumbnail_service.clone();
            let storage_service = state.storage_service.clone();

            tokio::spawn(async move {
                tracing::info!("Generating thumbnail for media {}", media_id);
                match storage_service.get_file_stream(&file_path).await {
                    Ok(file_stream) => match file_stream.try_collect::<Vec<Bytes>>().await {
                        Ok(chunks) => {
                            let total_len = chunks.iter().map(|b| b.len()).sum::<usize>();
                            let mut image_data = Vec::with_capacity(total_len);
                            for chunk in chunks {
                                image_data.extend_from_slice(&chunk);
                            }
                            if let Err(e) = thumbnail_service
                                .generate_thumbnail(media_id, &file_path, image_data, &content_type)
                                .await
                            {
                                tracing::warn!(
                                    "Failed to generate thumbnail for media {}: {}",
                                    media_id,
                                    e
                                );
                            }
                        }
                        Err(e) => {
                            tracing::error!(
                                "Failed to read file stream for media {} when generating thumbnail: {}",
                                media_id,
                                e
                            );
                        }
                    },
                    Err(e) => {
                        tracing::error!(
                            "Failed to get file stream for media {} when generating thumbnail: {}",
                            media_id,
                            e
                        );
                    }
                };
            });

            Ok((StatusCode::CREATED, ApiResponseBody::new(result).into()))
        }
        Err(err) => match err {
            MediaUploadError::InvalidFileType => Err(ApiError::BadRequestError(
                "Invalid file type. Only images and videos are allowed".to_string(),
            )),
            MediaUploadError::FileTooLarge => Err(ApiError::BadRequestError(
                "File too large. Maximum size is 100MB".to_string(),
            )),
            MediaUploadError::StorageError(msg) => {
                tracing::event!(target: "server_error", 
                    tracing::Level::ERROR,
                    "Failed to store file in storage service, {}", msg);
                Err(ApiError::InternalServerError(
                    "Internal server error, Failed to store file".to_string(),
                ))
            }
            MediaUploadError::InternalServerError(msg) => {
                tracing::error!("Internal server error, {}", msg);
                Err(ApiError::InternalServerError(
                    "Internal server error".to_string(),
                ))
            }
        },
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
        Err(_) => Err(ApiError::InternalServerError(
            "Failed to retrieve media files".to_string(),
        )),
    }
}

#[utoipa::path(
    delete,
    path = "/{media_id}",
    description = "Delete a media file",
    tag = "media",
    params(
        ("media_id" = String, Path, description = "ID of the media file to delete")
    ),
    responses(
        (status = 200, description = "Media deleted successfully", body = ApiResponseBody<DeleteMediaResult>),
        (status = 400, description = "Invalid media ID format", body = ApiErrorBody),
        (status = 404, description = "Media file not found", body = ApiErrorBody),
        (status = 500, description = "Internal server error", body = ApiErrorBody)
    ),
    security(("bearer_auth" = [])),
)]
pub async fn delete_media(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(media_id_str): Path<String>,
) -> Result<(StatusCode, Json<ApiResponseBody<DeleteMediaResult>>), ApiError> {
    // Parse UUID manually to provide better error messages
    let media_id = Uuid::parse_str(&media_id_str)
        .map_err(|_| ApiError::BadRequestError("Invalid media ID format".to_string()))?;

    let command = DeleteMediaCommand {
        media_id,
        user_id: claims.sub,
    };

    match delete_media_command_handler(
        command,
        state.media_repository.as_ref(),
        state.storage_service.as_ref(),
    )
    .await
    {
        Ok(result) => Ok((StatusCode::OK, ApiResponseBody::new(result).into())),
        Err(err) => match err {
            MediaDeleteError::MediaFileNotFound => {
                Err(ApiError::NotFoundError("Media file not found".to_string()))
            }
            MediaDeleteError::StorageError(msg) => {
                tracing::event!(target: "server_error", 
                    tracing::Level::ERROR,
                    "Failed to delete file from storage service, {}", msg);
                Err(ApiError::InternalServerError(
                    "Internal server error, Failed to delete file".to_string(),
                ))
            }
            MediaDeleteError::InternalServerError(msg) => {
                tracing::error!("Internal server error, {}", msg);
                Err(ApiError::InternalServerError(
                    "Internal server error".to_string(),
                ))
            }
        },
    }
}

#[utoipa::path(
    get,
    path = "/stream/{media_id}",
    description = "Stream media file",
    tag = "media",
    responses(
        (status = 200, description = "Media file streamed correctly", body = [u8]),
        (status = 400, description = "Invalid media ID format", body = ApiErrorBody),
        (status = 404, description = "Media file not found", body = ApiErrorBody),
        (status = 500, description = "Internal server error", body = ApiErrorBody)
    ),
)]
pub async fn get_media_stream(
    State(state): State<AppState>,
    Path(media_id): Path<String>,
) -> Result<Body, ApiError> {
    let query = GetMediaStreamQuery {
        media_id: Uuid::from_str(&media_id)
            .map_err(|_| ApiError::BadRequestError("Invalid media ID format".to_string()))?,
    };

    let file_stream = get_media_stream_query_handler(
        query,
        state.storage_service.as_ref(),
        state.media_repository.as_ref(),
    )
    .await
    .map_err(|e| match e {
        GetMediaStreamError::NotFound => {
            ApiError::NotFoundError("Media file not found".to_string())
        }
        GetMediaStreamError::InternalError(error) => {
            tracing::error!(
                "Internal server error while streaming media file: {}",
                error
            );
            ApiError::InternalServerError("Internal server error".to_string())
        }
    })?;

    Ok(Body::from_stream(file_stream))
}

pub fn api_routes(state: AppState) -> axum::Router<AppState> {
    axum::Router::new()
        .route("/upload", post(upload_media))
        .route("/", get(get_media_files))
        .route("/{media_id}", delete(delete_media))
        .route_layer(protected!(state.clone()))
        .route("/stream/{media_path}", get(get_media_stream))
}

#[derive(OpenApi)]
#[openapi(
    paths(upload_media, get_media_files, delete_media, get_media_stream),
    tags(
        (name = "media", description = "Media upload and management API")
    )
)]
pub struct ApiDoc;

pub fn combine_openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
