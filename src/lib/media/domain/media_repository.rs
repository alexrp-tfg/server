use async_trait::async_trait;
use uuid::Uuid;

use super::media_file::{MediaFile, NewMediaFile};

#[derive(Debug, thiserror::Error)]
pub enum MediaRepositoryError {
    #[error("Internal server error")]
    InternalServerError,
    #[error("Media file not found")]
    MediaFileNotFound,
}

#[async_trait]
pub trait MediaRepository: Send + Sync {
    async fn create_media_file(
        &self,
        media_file: NewMediaFile,
    ) -> Result<MediaFile, MediaRepositoryError>;
    async fn get_media_file_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<MediaFile>, MediaRepositoryError>;
    async fn get_media_files_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<MediaFile>, MediaRepositoryError>;
    async fn delete_media_file(&self, id: Uuid) -> Result<(), MediaRepositoryError>;
}
