use lib::media::domain::{
    MediaFile, NewMediaFile, MediaRepository, MediaRepositoryError, FileStorageService
};
use uuid::Uuid;
use async_trait::async_trait;

#[derive(Debug, Clone, Default)]
pub struct MockMediaRepository {
    pub fail_save: bool,
    pub fail_get: bool,
    pub saved_media: Option<MediaFile>,
    pub media_files: Vec<MediaFile>,
}

#[async_trait]
impl MediaRepository for MockMediaRepository {
    async fn create_media_file(&self, media_file: NewMediaFile) -> Result<MediaFile, MediaRepositoryError> {
        if self.fail_save {
            return Err(MediaRepositoryError::InternalServerError);
        }
        Ok(MediaFile {
            id: Uuid::new_v4(),
            user_id: media_file.user_id,
            filename: media_file.filename,
            original_filename: media_file.original_filename,
            content_type: media_file.content_type,
            file_size: media_file.file_size,
            file_path: media_file.file_path,
            uploaded_at: Some(chrono::Utc::now().naive_utc()),
            updated_at: Some(chrono::Utc::now().naive_utc()),
        })
    }

    async fn get_media_file_by_id(&self, _id: Uuid) -> Result<Option<MediaFile>, MediaRepositoryError> {
        if self.fail_get {
            return Err(MediaRepositoryError::InternalServerError);
        }
        Ok(self.saved_media.clone())
    }

    async fn get_media_files_by_user_id(&self, user_id: Uuid) -> Result<Vec<MediaFile>, MediaRepositoryError> {
        if self.fail_get {
            return Err(MediaRepositoryError::InternalServerError);
        }
        Ok(self.media_files.clone().into_iter().filter(|f| f.user_id == user_id).collect())
    }

    async fn delete_media_file(&self, _id: Uuid) -> Result<(), MediaRepositoryError> {
        if self.fail_save {
            return Err(MediaRepositoryError::InternalServerError);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct MockStorageService {
    pub fail_upload: bool,
    pub uploaded_files: Vec<String>,
}

#[async_trait]
impl FileStorageService for MockStorageService {
    async fn store_file(&self, _file_data: &[u8], file_path: &str, _content_type: &str) -> Result<String, String> {
        if self.fail_upload {
            return Err("Mock upload failure".to_string());
        }
        Ok(file_path.to_string())
    }

    async fn delete_file(&self, _file_path: &str) -> Result<(), String> {
        Ok(())
    }

    async fn get_file_url(&self, file_path: &str) -> Result<String, String> {
        Ok(format!("https://mock-storage.example.com/{}", file_path))
    }
}
