use async_trait::async_trait;
use minio::s3::{creds::StaticProvider, http::BaseUrl, types::S3Api};

use crate::media::domain::FileStorageService;

pub struct MinioStorageService {
    client: minio::s3::Client,
    bucket: String,
}

impl MinioStorageService {
    pub async fn new(
        endpoint: String,
        access_key: String,
        secret_key: String,
        bucket_name: String,
    ) -> Result<Self, String> {
        let base_url: BaseUrl = endpoint
            .parse()
            .map_err(|e| format!("Invalid endpoint URL: {}", e))?;
        let static_provider = StaticProvider::new(&access_key, &secret_key, None);

        let client = minio::s3::Client::new(base_url, Some(Box::new(static_provider)), None, None)
            .map_err(|e| format!("Failed to create MinIO client: {}", e))?;

        // Ensure the bucket exists, create it if it doesn't
        if !client
            .bucket_exists(&bucket_name)
            .send()
            .await
            .map_err(|e| format!("Failed to check bucket existence: {}", e))?
            .exists
        {
            client
                .create_bucket(&bucket_name)
                .send()
                .await
                .map_err(|e| format!("Failed to create bucket: {}", e))?;
        }

        Ok(Self {
            client,
            bucket: bucket_name,
        })
    }
}

#[async_trait]
impl FileStorageService for MinioStorageService {
    async fn store_file(
        &self,
        file_data: Vec<u8>,
        file_path: &str,
        content_type: &str,
    ) -> Result<String, String> {
        Ok(self
            .client
            .put_object_content(&self.bucket, file_path, file_data)
            .content_type(content_type.to_string())
            .send()
            .await
            .map_err(|e| format!("Failed to store file: {}", e))?
            .object)
    }

    async fn delete_file(&self, file_path: &str) -> Result<(), String> {
        self.client
            .delete_object(&self.bucket, file_path)
            .send()
            .await
            .map_err(|e| format!("Failed to delete file: {}", e))?;

        Ok(())
    }

    async fn get_file_url(&self, file_path: &str) -> Result<String, String> {
        // For MinIO, we can construct the URL directly
        Ok(format!(
            "{}/{}/{}",
            "https://your-minio-endpoint", // Store this in the struct for real use
            self.bucket,
            file_path
        ))
    }
}
