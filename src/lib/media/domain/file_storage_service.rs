use std::pin::Pin;

use async_trait::async_trait;
use bytes::Bytes;
use futures_core::Stream;

#[derive(Debug, thiserror::Error)]
pub enum FileStorageError {
    #[error("File does not exist")]
    NotFound,
    #[error("File already exists")]
    AlreadyExists(String),
    #[error("Internal server error: {0}")]
    InternalError(String),
}

pub struct UploadedFileMetadata {
    pub file_path: String,
    pub file_size: u64,
}

pub type FileStream = Pin<Box<dyn Stream<Item = Result<Bytes, FileStorageError>> + Send>>;

#[async_trait]
pub trait FileStorageService: Send + Sync {
    async fn store_file(
        &self,
        file_path: &str,
        content_type: &str,
        file_size: Option<u64>,
        file_data: Pin<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send + Sync + 'static>> ,
    ) -> Result<UploadedFileMetadata, FileStorageError>;
    async fn delete_file(&self, file_path: &str) -> Result<(), FileStorageError>;
    async fn get_file_url(&self, file_path: &str) -> Result<String, FileStorageError>;
    async fn get_file_stream(&self, file_path: &str) -> Result<FileStream, FileStorageError>;
}
