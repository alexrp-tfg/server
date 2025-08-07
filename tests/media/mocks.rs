use async_trait::async_trait;
use lib::{
    media::domain::{
        FileStorageService, MediaFile, MediaRepository, MediaRepositoryError, NewMediaFile,
    },
    users::domain::{Claims, LoginTokenService, Token, user::UserLoginError},
};
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct MockMediaRepository {
    pub fail_save: bool,
    pub fail_get: bool,
    pub saved_media: Option<MediaFile>,
    pub media_files: Vec<MediaFile>,
}

#[async_trait]
impl MediaRepository for MockMediaRepository {
    async fn create_media_file(
        &self,
        media_file: NewMediaFile,
    ) -> Result<MediaFile, MediaRepositoryError> {
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

    async fn get_media_file_by_id(
        &self,
        _id: Uuid,
    ) -> Result<Option<MediaFile>, MediaRepositoryError> {
        if self.fail_get {
            return Err(MediaRepositoryError::InternalServerError);
        }
        Ok(self.saved_media.clone())
    }

    async fn get_media_files_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<MediaFile>, MediaRepositoryError> {
        if self.fail_get {
            return Err(MediaRepositoryError::InternalServerError);
        }
        Ok(self
            .media_files
            .clone()
            .into_iter()
            .filter(|f| f.user_id == user_id)
            .collect())
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
    pub fail_delete: bool,
    pub uploaded_files: Vec<String>,
}

#[async_trait]
impl FileStorageService for MockStorageService {
    async fn store_file(
        &self,
        _file_data: Vec<u8>,
        file_path: &str,
        _content_type: &str,
    ) -> Result<String, String> {
        if self.fail_upload {
            return Err("Mock upload failure".to_string());
        }
        Ok(file_path.to_string())
    }

    async fn delete_file(&self, _file_path: &str) -> Result<(), String> {
        if self.fail_delete {
            return Err("Mock delete failure".to_string());
        }
        Ok(())
    }

    async fn get_file_url(&self, file_path: &str) -> Result<String, String> {
        Ok(format!("https://mock-storage.example.com/{}", file_path))
    }
}

// Fixed user ID for consistent testing
const TEST_USER_ID: &str = "550e8400-e29b-41d4-a716-446655440000";

pub fn get_test_user_id() -> Uuid {
    Uuid::parse_str(TEST_USER_ID).unwrap()
}

// Custom token service that returns our test user ID
#[derive(Clone, Default)]
pub struct TestTokenService;

impl LoginTokenService for TestTokenService {
    fn create_token(&self, _claims: Claims) -> Result<Token, UserLoginError> {
        Ok(Token("test_token".to_string()))
    }

    fn validate_token(&self, _token: &str) -> Result<Claims, UserLoginError> {
        Ok(Claims {
            sub: get_test_user_id(),
            role: lib::users::domain::Role::User,
            username: "testuser".to_string(),
            exp: (chrono::Utc::now().timestamp() + 3600) as u64,
        })
    }
}
