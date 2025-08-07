use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::media::domain::{
    FileStorageService, MediaFile, MediaRepository, MediaRepositoryError, MediaUploadError,
    NewMediaFile,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UploadMediaCommand {
    pub user_id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub file_data: Vec<u8>,
    pub content_type: String,
}

#[derive(Debug, Serialize, ToSchema, Clone, PartialEq, Eq)]
pub struct UploadMediaResult {
    pub id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub file_size: i64,
    pub content_type: String,
    pub uploaded_at: Option<chrono::NaiveDateTime>,
}

pub async fn upload_media_command_handler<
    MR: MediaRepository + ?Sized,
    FS: FileStorageService + ?Sized,
>(
    command: UploadMediaCommand,
    media_repository: &MR,
    storage_service: &FS,
) -> Result<UploadMediaResult, MediaUploadError> {
    // Validate file type (only allow images and videos)
    if !is_valid_media_type(&command.content_type) {
        return Err(MediaUploadError::InvalidFileType);
    }

    // Check file size (max 100MB)
    const MAX_FILE_SIZE: usize = 100 * 1024 * 1024;
    if command.file_data.len() > MAX_FILE_SIZE {
        return Err(MediaUploadError::FileTooLarge);
    }

    // Generate unique file path
    let file_path = format!("media/{}/{}", command.user_id, command.filename);
    let file_size = command.file_data.len() as i64;

    // Store file in MinIO
    storage_service
        .store_file(command.file_data, &file_path, &command.content_type)
        .await
        .map_err(MediaUploadError::StorageError)?;

    // Create media file record in database
    let new_media_file = NewMediaFile {
        user_id: command.user_id,
        filename: command.filename,
        original_filename: command.original_filename,
        file_size,
        content_type: command.content_type,
        file_path,
    };

    let created_media = media_repository
        .create_media_file(new_media_file)
        .await
        .map_err(|e| match e {
            MediaRepositoryError::InternalServerError => {
                MediaUploadError::InternalServerError("Database error".to_string())
            }
            MediaRepositoryError::MediaFileNotFound => {
                MediaUploadError::InternalServerError("Unexpected error".to_string())
            }
        })?;

    Ok(created_media.into())
}

fn is_valid_media_type(content_type: &str) -> bool {
    content_type.starts_with("image/") || content_type.starts_with("video/")
}

impl From<MediaFile> for UploadMediaResult {
    fn from(media_file: MediaFile) -> Self {
        UploadMediaResult {
            id: media_file.id,
            filename: media_file.filename,
            original_filename: media_file.original_filename,
            file_size: media_file.file_size,
            content_type: media_file.content_type,
            uploaded_at: media_file.uploaded_at,
        }
    }
}
