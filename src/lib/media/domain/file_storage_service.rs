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
    #[error("Internal server error")]
    InternalError(String),
}

pub type FileStream = Pin<Box<dyn Stream<Item = Result<Bytes, FileStorageError>> + Send>>;

#[async_trait]
pub trait FileStorageService: Send + Sync {
    async fn store_file(
        &self,
        file_data: Vec<u8>,
        file_path: &str,
        content_type: &str,
    ) -> Result<String, FileStorageError>;
    async fn delete_file(&self, file_path: &str) -> Result<(), FileStorageError>;
    async fn get_file_url(&self, file_path: &str) -> Result<String, FileStorageError>;
    async fn get_file_stream(&self, file_path: &str) -> Result<FileStream, FileStorageError>;
}
