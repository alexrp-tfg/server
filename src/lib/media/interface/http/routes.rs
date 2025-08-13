use std::str::FromStr;

use axum::{
    Extension, Json,
    body::Body,
    extract::{FromRequest, Multipart, Path, Request, State},
    http::StatusCode,
    routing::{delete, get, post},
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
    let mut multipart = Multipart::from_request(req, &state)
        .await
        .map_err(|_| ApiError::BadRequestError("Invalid multipart data".to_string()))?;

    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    let mut content_type: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::BadRequestError("Invalid multipart data".to_string()))?
    {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "file" {
            filename = field.file_name().map(|name| name.to_string());
            content_type = field.content_type().map(|ct| ct.to_string());
            file_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| {
                        ApiError::BadRequestError(format!("Failed to read file data: {}", e))
                    })?
                    .to_vec(),
            );
        }
    }

    let file_data =
        file_data.ok_or_else(|| ApiError::BadRequestError("No file provided".to_string()))?;

    let original_filename =
        filename.ok_or_else(|| ApiError::BadRequestError("No filename provided".to_string()))?;

    let content_type = content_type
        .ok_or_else(|| ApiError::BadRequestError("No content type provided".to_string()))?;

    // Generate unique filename
    let file_extension = original_filename
        .split('.')
        .next_back()
        .unwrap_or("unknown");
    let unique_filename = format!(
        "{}_{}.{}",
        Uuid::new_v4(),
        chrono::Utc::now().timestamp(),
        file_extension
    );

    let command = UploadMediaCommand {
        user_id: claims.sub,
        filename: unique_filename.clone(),
        original_filename,
        file_data: file_data.clone(),
        content_type: content_type.clone(),
    };

    match upload_media_command_handler(
        command,
        state.media_repository.as_ref(),
        state.storage_service.as_ref(),
    )
    .await
    {
        Ok(result) => {
            let media_id = result.id;
            let file_path = format!("media/{}/{}", claims.sub, unique_filename);
            let thumbnail_service = state.thumbnail_service.clone();

            tokio::spawn(async move {
                tracing::info!("Generating thumbnail for media {}", media_id);
                if let Err(e) = thumbnail_service
                    .generate_thumbnail(media_id, &file_path, file_data, &content_type)
                    .await
                {
                    tracing::warn!("Failed to generate thumbnail for media {}: {}", media_id, e);
                }
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
