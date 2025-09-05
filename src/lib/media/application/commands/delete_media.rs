use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::media::domain::{
    FileStorageService, MediaDeleteError, MediaRepository, MediaRepositoryError,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeleteMediaCommand {
    pub media_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, ToSchema, Clone, PartialEq, Eq)]
pub struct DeleteMediaResult {
    pub success: bool,
    pub media_id: Uuid,
}

pub async fn delete_media_command_handler<
    MR: MediaRepository + ?Sized,
    FS: FileStorageService + ?Sized,
>(
    command: DeleteMediaCommand,
    media_repository: &MR,
    storage_service: &FS,
) -> Result<DeleteMediaResult, MediaDeleteError> {
    // First, get the media file to ensure it exists and get the file path
    let media_file = media_repository
        .get_media_file_by_id(command.media_id)
        .await
        .map_err(|e| match e {
            MediaRepositoryError::MediaFileNotFound => MediaDeleteError::MediaFileNotFound,
            MediaRepositoryError::InternalServerError => {
                MediaDeleteError::InternalServerError("Database error".to_string())
            }
        })?
        .ok_or(MediaDeleteError::MediaFileNotFound)?;

    // Verify the media file belongs to the user
    if media_file.user_id != command.user_id {
        return Err(MediaDeleteError::MediaFileNotFound);
    }

    // Delete file from MinIO storage
    storage_service
        .delete_file(&media_file.file_path)
        .await
        .map_err(|e| match e {
            crate::media::FileStorageError::NotFound => MediaDeleteError::MediaFileNotFound,
            error => {
                tracing::error!("Failed to delete file from storage: {:?}", error);
                MediaDeleteError::StorageError("Storage error".to_string())
            }
        })?;

    // Delete media file record from database
    media_repository
        .delete_media_file(command.media_id)
        .await
        .map_err(|e| match e {
            MediaRepositoryError::MediaFileNotFound => MediaDeleteError::MediaFileNotFound,
            MediaRepositoryError::InternalServerError => {
                MediaDeleteError::InternalServerError("Database error".to_string())
            }
        })?;

    Ok(DeleteMediaResult {
        success: true,
        media_id: command.media_id,
    })
}
