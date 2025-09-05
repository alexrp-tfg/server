use crate::media::{
    FileStorageError, FileStorageService, FileStream, MediaId, MediaRepository,
    MediaRepositoryError,
};

pub struct GetMediaStreamQuery {
    pub media_id: MediaId,
}

#[derive(thiserror::Error, Debug)]
pub enum GetMediaStreamError {
    #[error("Media not found")]
    NotFound,
    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl From<FileStorageError> for GetMediaStreamError {
    fn from(error: FileStorageError) -> Self {
        match error {
            FileStorageError::NotFound => GetMediaStreamError::NotFound,
            FileStorageError::AlreadyExists(_) => {
                GetMediaStreamError::InternalError("File already exists".to_string())
            }
            FileStorageError::InternalError(e) => {
                GetMediaStreamError::InternalError(format!("Internal server error: {}", e))
            }
        }
    }
}

pub async fn get_media_stream_query_handler(
    query: GetMediaStreamQuery,
    media_storage: &dyn FileStorageService,
    media_repo: &dyn MediaRepository,
) -> Result<FileStream, GetMediaStreamError> {
    let media_path = media_repo
        .get_media_path_by_id(query.media_id)
        .await
        .map_err(|e| match e {
            MediaRepositoryError::MediaFileNotFound => GetMediaStreamError::NotFound,
            _ => GetMediaStreamError::InternalError("Failed to retrieve media path".to_string()),
        })?;

    media_storage
        .get_file_stream(&media_path)
        .await
        .map_err(|e| e.into())
}
