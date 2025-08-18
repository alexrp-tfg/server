use async_trait::async_trait;
use aws_sdk_s3::config::{RequestChecksumCalculation, ResponseChecksumValidation};
use aws_sdk_s3::error::DisplayErrorContext;
use http_body::Frame;
use http_body_util::StreamBody;
use std::pin::Pin;
use bytes::Bytes;
use futures_core::Stream;
use futures_util::{StreamExt, TryStreamExt};
use aws_sdk_s3::{Client, Error as S3Error};
use aws_sdk_s3::primitives::{ByteStream, SdkBody};
use aws_config::{BehaviorVersion, Region};
use tokio_util::io::ReaderStream;

use crate::media::domain::file_storage_service::{
    FileStorageService, FileStorageError, UploadedFileMetadata, FileStream
};

pub struct MinioStorageService {
    client: Client,
    bucket: String,
    endpoint: String,
}

impl MinioStorageService {
    pub async fn new(
        endpoint: &str,
        access_key: &str,
        secret_key: &str,
        bucket: &str,
    ) -> Result<Self, S3Error> {
        // Configure AWS SDK to work with MinIO
        let config = aws_config::defaults(BehaviorVersion::latest())
            .endpoint_url(endpoint)
            .region(Region::new("us-east-1")) // MinIO uses this as default
            .credentials_provider(aws_sdk_s3::config::Credentials::new(
                access_key,
                secret_key,
                None,
                None,
                "static"
            ))
            .load()
            .await;

        let s3_config = aws_sdk_s3::config::Builder::from(&config)
            .force_path_style(true) // Required for MinIO
            .request_checksum_calculation(RequestChecksumCalculation::WhenRequired)
            .response_checksum_validation(ResponseChecksumValidation::WhenRequired)
            .build();


        let client = Client::from_conf(s3_config);

        // Check if bucket exists and create if necessary
        match client.head_bucket().bucket(bucket).send().await {
            Ok(_) => {}, // Bucket exists
            Err(_) => {
                // Try to create bucket
                let _ = client.create_bucket().bucket(bucket).send().await;
            }
        }

        Ok(Self {
            client,
            bucket: bucket.to_string(),
            endpoint: endpoint.to_string(),
        })
    }
}

#[async_trait]
impl FileStorageService for MinioStorageService {
    async fn store_file(
        &self,
        file_path: &str,
        content_type: &str,
        file_size: Option<u64>,
        file_data: Pin<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send + Sync + 'static>>,
    ) -> Result<UploadedFileMetadata, FileStorageError> {
        let frames = file_data.map_ok(Frame::data);

        let body = StreamBody::new(frames);

        let bytestream = ByteStream::new(SdkBody::from_body_1_x(body));

        let mut put_request = self.client
            .put_object()
            .bucket(&self.bucket)
            .key(file_path)
            .body(bytestream)
            .content_type(content_type);

        // Set content length if known
        if let Some(size) = file_size {
            put_request = put_request.content_length(size as i64);
        }

        put_request.send().await.map_err(|e| {
            FileStorageError::InternalError(format!(
                "Failed to upload file: {}",
                DisplayErrorContext(e)
            ))
        })?;

        Ok(UploadedFileMetadata {
            file_path: file_path.to_string(),
            file_size: file_size.unwrap_or(0),
        })
    }

    async fn delete_file(&self, file_path: &str) -> Result<(), FileStorageError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(file_path)
            .send()
            .await
            .map_err(|e| FileStorageError::InternalError(format!("Failed to delete file: {}", e)))?;

        Ok(())
    }

    async fn get_file_url(&self, file_path: &str) -> Result<String, FileStorageError> {
        // For MinIO, construct the URL directly since it's accessible via HTTP
        let url = format!("{}/{}/{}", self.endpoint, self.bucket, file_path);
        Ok(url)
    }

    async fn get_file_stream(&self, file_path: &str) -> Result<FileStream, FileStorageError> {
        let response = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(file_path)
            .send()
            .await;

        match response {
            Ok(output) => {
                // Convert ByteStream to a proper Stream that returns Result<Bytes, FileStorageError>
                let async_read = output.body.into_async_read();
                let reader_stream = ReaderStream::new(async_read);
                
                let stream = reader_stream.map(|result| {
                    match result {
                        Ok(bytes) => Ok(bytes),
                        Err(e) => Err(FileStorageError::InternalError(format!("Stream error: {}", e))),
                    }
                });
                
                Ok(Box::pin(stream))
            }
            Err(err) => {
                if err.to_string().contains("NoSuchKey") || err.to_string().contains("404") {
                    Err(FileStorageError::NotFound)
                } else {
                    Err(FileStorageError::InternalError(format!("Failed to get file stream: {}", err)))
                }
            }
        }
    }
}
