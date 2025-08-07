use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MediaFile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub file_size: i64,
    pub content_type: String,
    pub file_path: String,
    pub uploaded_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct NewMediaFile {
    pub user_id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub file_size: i64,
    pub content_type: String,
    pub file_path: String,
}

#[derive(Debug, thiserror::Error)]
pub enum MediaUploadError {
    #[error("Invalid file type")]
    InvalidFileType,
    #[error("File too large")]
    FileTooLarge,
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Internal server error")]
    InternalServerError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum MediaDeleteError {
    #[error("Media file not found")]
    MediaFileNotFound,
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Internal server error")]
    InternalServerError(String),
}
