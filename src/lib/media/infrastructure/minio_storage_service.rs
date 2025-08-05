use async_trait::async_trait;
use s3::{Bucket, Region, creds::Credentials};

use crate::media::domain::FileStorageService;

pub struct MinioStorageService {
    bucket: Bucket,
}

impl MinioStorageService {
    pub fn new(endpoint: String, access_key: String, secret_key: String, bucket_name: String) -> Result<Self, String> {
        let region = Region::Custom {
            region: "us-east-1".to_string(),
            endpoint,
        };
        
        let credentials = Credentials::new(
            Some(&access_key),
            Some(&secret_key),
            None,
            None,
            None,
        ).map_err(|e| format!("Failed to create credentials: {}", e))?;

        let bucket = Bucket::new(&bucket_name, region, credentials)
            .map_err(|e| format!("Failed to create bucket: {}", e))?;

        Ok(Self { bucket: *bucket })
    }
}

#[async_trait]
impl FileStorageService for MinioStorageService {
    async fn store_file(&self, file_data: &[u8], file_path: &str, content_type: &str) -> Result<String, String> {
        let response = self.bucket
            .put_object_with_content_type(file_path, file_data, content_type)
            .await
            .map_err(|e| format!("Failed to store file: {}", e))?;

        if response.status_code() == 200 {
            Ok(file_path.to_string())
        } else {
            Err(format!("Failed to store file, status: {}", response.status_code()))
        }
    }

    async fn delete_file(&self, file_path: &str) -> Result<(), String> {
        let response = self.bucket
            .delete_object(file_path)
            .await
            .map_err(|e| format!("Failed to delete file: {}", e))?;

        if response.status_code() == 204 {
            Ok(())
        } else {
            Err(format!("Failed to delete file, status: {}", response.status_code()))
        }
    }

    async fn get_file_url(&self, file_path: &str) -> Result<String, String> {
        // For MinIO, we can construct the URL directly
        Ok(format!("{}/{}/{}", self.bucket.url(), self.bucket.name(), file_path))
    }
}