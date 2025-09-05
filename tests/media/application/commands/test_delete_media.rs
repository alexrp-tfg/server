// Simple test to validate media delete functionality
#[cfg(test)]
mod media_delete_tests {
    use crate::media::{MockMediaRepository, MockStorageService};
    use lib::media::{
        application::commands::delete_media::{DeleteMediaCommand, delete_media_command_handler},
        domain::{MediaDeleteError, MediaFile},
    };
    use uuid::Uuid;

    #[tokio::test]
    async fn test_delete_media_success() {
        let user_id = Uuid::new_v4();
        let media_id = Uuid::new_v4();

        let media_file = MediaFile {
            id: media_id,
            user_id,
            filename: "test.jpg".to_string(),
            original_filename: "original.jpg".to_string(),
            file_size: 1024,
            content_type: "image/jpeg".to_string(),
            file_path: format!("media/{}/test.jpg", user_id),
            uploaded_at: Some(chrono::Utc::now().naive_utc()),
            updated_at: Some(chrono::Utc::now().naive_utc()),
            thumbnail_path: None,
        };

        let mock_repo = MockMediaRepository {
            saved_media: Some(media_file),
            ..MockMediaRepository::default()
        };
        let mock_storage = MockStorageService::default();

        let command = DeleteMediaCommand { media_id, user_id };

        let result = delete_media_command_handler(command, &mock_repo, &mock_storage).await;

        assert!(result.is_ok());
        let delete_result = result.unwrap();
        assert!(delete_result.success);
        assert_eq!(delete_result.media_id, media_id);
    }

    #[tokio::test]
    async fn test_delete_media_not_found() {
        let user_id = Uuid::new_v4();
        let media_id = Uuid::new_v4();

        let mock_repo = MockMediaRepository {
            saved_media: None, // No media file found
            ..MockMediaRepository::default()
        };
        let mock_storage = MockStorageService::default();

        let command = DeleteMediaCommand { media_id, user_id };

        let result = delete_media_command_handler(command, &mock_repo, &mock_storage).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            MediaDeleteError::MediaFileNotFound => {} // Expected
            _ => panic!("Expected MediaFileNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_delete_media_wrong_user() {
        let user_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4(); // Different user
        let media_id = Uuid::new_v4();

        let media_file = MediaFile {
            id: media_id,
            user_id: other_user_id, // Belongs to different user
            filename: "test.jpg".to_string(),
            original_filename: "original.jpg".to_string(),
            file_size: 1024,
            content_type: "image/jpeg".to_string(),
            file_path: format!("media/{}/test.jpg", other_user_id),
            uploaded_at: Some(chrono::Utc::now().naive_utc()),
            updated_at: Some(chrono::Utc::now().naive_utc()),
            thumbnail_path: None,
        };

        let mock_repo = MockMediaRepository {
            saved_media: Some(media_file),
            ..MockMediaRepository::default()
        };
        let mock_storage = MockStorageService::default();

        let command = DeleteMediaCommand {
            media_id,
            user_id, // Different from media file's user_id
        };

        let result = delete_media_command_handler(command, &mock_repo, &mock_storage).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            MediaDeleteError::MediaFileNotFound => {} // Expected - should act as not found for security
            _ => panic!("Expected MediaFileNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_delete_media_repository_error() {
        let user_id = Uuid::new_v4();
        let media_id = Uuid::new_v4();

        let media_file = MediaFile {
            id: media_id,
            user_id,
            filename: "test.jpg".to_string(),
            original_filename: "original.jpg".to_string(),
            file_size: 1024,
            content_type: "image/jpeg".to_string(),
            file_path: format!("media/{}/test.jpg", user_id),
            uploaded_at: Some(chrono::Utc::now().naive_utc()),
            updated_at: Some(chrono::Utc::now().naive_utc()),
            thumbnail_path: None,
        };

        let mock_repo = MockMediaRepository {
            saved_media: Some(media_file),
            fail_save: true, // This will cause delete to fail
            ..MockMediaRepository::default()
        };
        let mock_storage = MockStorageService::default();

        let command = DeleteMediaCommand { media_id, user_id };

        let result = delete_media_command_handler(command, &mock_repo, &mock_storage).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            MediaDeleteError::InternalServerError(_) => {} // Expected
            _ => panic!("Expected InternalServerError"),
        }
    }

    #[tokio::test]
    async fn test_delete_media_storage_error() {
        let user_id = Uuid::new_v4();
        let media_id = Uuid::new_v4();

        let media_file = MediaFile {
            id: media_id,
            user_id,
            filename: "test.jpg".to_string(),
            original_filename: "original.jpg".to_string(),
            file_size: 1024,
            content_type: "image/jpeg".to_string(),
            file_path: format!("media/{}/test.jpg", user_id),
            uploaded_at: Some(chrono::Utc::now().naive_utc()),
            updated_at: Some(chrono::Utc::now().naive_utc()),
            thumbnail_path: None,
        };

        let mock_repo = MockMediaRepository {
            saved_media: Some(media_file),
            ..MockMediaRepository::default()
        };
        let mock_storage = MockStorageService {
            fail_delete: true, // This will cause storage delete to fail
            ..MockStorageService::default()
        };

        let command = DeleteMediaCommand { media_id, user_id };

        let result = delete_media_command_handler(command, &mock_repo, &mock_storage).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            MediaDeleteError::StorageError(_) => {} // Expected
            _ => panic!("Expected StorageError"),
        }
    }
}
