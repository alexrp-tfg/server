use async_trait::async_trait;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::{RequestChecksumCalculation, ResponseChecksumValidation};
use aws_sdk_s3::error::DisplayErrorContext;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
use aws_sdk_s3::{Client, Error as S3Error};
use bytes::Bytes;
use futures_core::Stream;
use futures_util::StreamExt;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{Semaphore};
use tokio_util::io::ReaderStream;

use crate::media::domain::file_storage_service::{
    FileStorageError, FileStorageService, FileStream, UploadedFileMetadata,
};

pub struct MinioStorageService {
    client: Client,
    bucket: String,
    endpoint: String,
}

const CHUNK_SIZE: u64 = 8 * 1024 * 1024;

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
                access_key, secret_key, None, None, "static",
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
            Ok(_) => {} // Bucket exists
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

    async fn upload_part(
        &self,
        file_path: &str,
        upload_id: &str,
        part_number: i32,
        data: Vec<u8>,
    ) -> Result<CompletedPart, FileStorageError> {
        let part = self
            .client
            .upload_part()
            .bucket(&self.bucket)
            .key(file_path)
            .upload_id(upload_id)
            .part_number(part_number)
            .body(ByteStream::from(data))
            .send()
            .await
            .map_err(|e| {
                FileStorageError::InternalError(format!(
                    "Failed to upload part: {}",
                    DisplayErrorContext(e)
                ))
            })?;

        Ok(CompletedPart::builder()
            .part_number(part_number)
            .e_tag(part.e_tag().unwrap_or_default())
            .build())
    }
}

// TODO: Get this from environment configuration
const MAX_CONCURRENT_UPLOADS: usize = 8;

#[async_trait]
impl FileStorageService for MinioStorageService {
    async fn store_file(
        &self,
        file_path: &str,
        content_type: &str,
        file_size: Option<u64>,
        mut file_data: Pin<
            Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send + Sync + 'static>,
        >,
    ) -> Result<UploadedFileMetadata, FileStorageError> {
        // Create multipart upload
        let multipart_upload_res = self
            .client
            .create_multipart_upload()
            .bucket(&self.bucket)
            .key(file_path)
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| {
                FileStorageError::InternalError(format!(
                    "Failed to create multipart upload: {}",
                    DisplayErrorContext(e)
                ))
            })?;

        let upload_id = multipart_upload_res.upload_id().ok_or_else(|| {
            FileStorageError::InternalError("Failed to get upload ID".to_string())
        })?;

        // Spawn upload workers
        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_UPLOADS));

        // Send chunks for upload
        let mut part_number = 1i32;
        let mut total_size = 0u64;
        let mut current_chunk = Vec::with_capacity(CHUNK_SIZE as usize);

        let (result_tx, mut result_rx) = tokio::sync::mpsc::channel::<
            Result<CompletedPart, FileStorageError>,
        >(MAX_CONCURRENT_UPLOADS);

        let receive_handle = tokio::spawn(async move {
            let mut results = Vec::new();
            while let Some(result) = result_rx.recv().await {
                match result {
                    Ok(part) => results.push(part),
                    Err(e) => {
                        tracing::error!("Error receiving upload part: {}", e);
                        return Err(e);
                    }
                }
            }
            Ok(results)
        });

        while let Some(chunk_result) = file_data.next().await {
            let chunk = chunk_result.map_err(|e| {
                FileStorageError::InternalError(format!("Failed to read file data: {}", e))
            })?;

            current_chunk.extend_from_slice(&chunk);
            total_size += chunk.len() as u64;

            if current_chunk.len() as u64 >= CHUNK_SIZE {
                let permit = semaphore.clone().acquire_owned().await.unwrap();

                let client = self.client.clone();
                let bucket = self.bucket.clone();
                let key = file_path.to_string();
                let upload_id_clone = upload_id.to_string();
                let result_tx_clone = result_tx.clone();

                tokio::spawn(async move {
                    let _permit = permit; // Hold the permit until the task completes
                    let upload_result = client
                        .upload_part()
                        .bucket(&bucket)
                        .key(&key)
                        .upload_id(&upload_id_clone)
                        .part_number(part_number)
                        .body(ByteStream::from(current_chunk))
                        .send()
                        .await;

                    let result = match upload_result {
                        Ok(part_output) => Ok(CompletedPart::builder()
                            .part_number(part_number)
                            .e_tag(part_output.e_tag().unwrap_or_default())
                            .build()),
                        Err(e) => Err(FileStorageError::InternalError(format!(
                            "Failed to upload part {}: {}",
                            part_number,
                            DisplayErrorContext(e)
                        ))),
                    };

                    if result_tx_clone.send(result).await.is_err() {
                        result_tx_clone
                            .send(Err(FileStorageError::InternalError(
                                "Failed to send upload result".to_string(),
                            )))
                            .await
                            .ok();
                    }
                });

                part_number += 1;
                current_chunk = Vec::with_capacity(CHUNK_SIZE as usize);
            }
        }
        if !current_chunk.is_empty() {
            let chunk_size = current_chunk.len() as u64;
            self.upload_part(file_path, upload_id, part_number, current_chunk)
                .await?;
            total_size += chunk_size;
        }

        drop(result_tx);

        // Collect results
        let mut upload_parts = receive_handle.await.map_err(|e| {
            FileStorageError::InternalError(format!(
                "Failed to receive upload results: {}",
                e
            ))
        })??;

        // Sort and complete upload
        upload_parts.sort_by_key(|part| part.part_number());

        let completed_multipart_upload = CompletedMultipartUpload::builder()
            .set_parts(Some(upload_parts))
            .build();

        self.client
            .complete_multipart_upload()
            .bucket(&self.bucket)
            .key(file_path)
            .upload_id(upload_id)
            .multipart_upload(completed_multipart_upload)
            .send()
            .await
            .map_err(|e| {
                FileStorageError::InternalError(format!(
                    "Failed to complete multipart upload: {}",
                    DisplayErrorContext(e)
                ))
            })?;

        Ok(UploadedFileMetadata {
            file_path: file_path.to_string(),
            file_size: total_size,
        })
    }

    async fn delete_file(&self, file_path: &str) -> Result<(), FileStorageError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(file_path)
            .send()
            .await
            .map_err(|e| {
                FileStorageError::InternalError(format!("Failed to delete file: {}", e))
            })?;

        Ok(())
    }

    async fn get_file_url(&self, file_path: &str) -> Result<String, FileStorageError> {
        // For MinIO, construct the URL directly since it's accessible via HTTP
        let url = format!("{}/{}/{}", self.endpoint, self.bucket, file_path);
        Ok(url)
    }

    async fn get_file_stream(&self, file_path: &str) -> Result<FileStream, FileStorageError> {
        let response = self
            .client
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

                let stream = reader_stream.map(|result| match result {
                    Ok(bytes) => Ok(bytes),
                    Err(e) => Err(FileStorageError::InternalError(format!(
                        "Stream error: {}",
                        e
                    ))),
                });

                Ok(Box::pin(stream))
            }
            Err(err) => {
                if err.to_string().contains("NoSuchKey") || err.to_string().contains("404") {
                    Err(FileStorageError::NotFound)
                } else {
                    Err(FileStorageError::InternalError(format!(
                        "Failed to get file stream: {}",
                        err
                    )))
                }
            }
        }
    }
}
