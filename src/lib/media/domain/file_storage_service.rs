use async_trait::async_trait;

#[async_trait]
pub trait FileStorageService: Send + Sync {
    async fn store_file(
        &self,
        file_data: Vec<u8>,
        file_path: &str,
        content_type: &str,
    ) -> Result<String, String>;
    async fn delete_file(&self, file_path: &str) -> Result<(), String>;
    async fn get_file_url(&self, file_path: &str) -> Result<String, String>;
}
