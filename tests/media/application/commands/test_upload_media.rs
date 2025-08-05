// Simple test to validate media upload functionality
#[cfg(test)]
mod media_upload_tests {
    use lib::media::{upload_media_command_handler, UploadMediaCommand};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_upload_media_valid_image() {
        let mock_repo = MockMediaRepository;
        let mock_storage = MockStorageService;
        
        let user_id = Uuid::new_v4();
        let command = UploadMediaCommand {
            user_id,
            filename: "test.jpg".to_string(),
            original_filename: "original.jpg".to_string(),
            file_data: b"fake image data".to_vec(),
            content_type: "image/jpeg".to_string(),
        };

        let result = upload_media_command_handler(command, &mock_repo, &mock_storage).await;
        
        assert!(result.is_ok());
        let upload_result = result.unwrap();
        assert_eq!(upload_result.filename, "test.jpg");
        assert_eq!(upload_result.original_filename, "original.jpg");
        assert_eq!(upload_result.content_type, "image/jpeg");
        assert_eq!(upload_result.file_size, 15); // "fake image data".len()
    }

    #[tokio::test]
    async fn test_upload_media_invalid_file_type() {
        let mock_repo = MockMediaRepository;
        let mock_storage = MockStorageService;
        
        let user_id = Uuid::new_v4();
        let command = UploadMediaCommand {
            user_id,
            filename: "test.txt".to_string(),
            original_filename: "original.txt".to_string(),
            file_data: b"text content".to_vec(),
            content_type: "text/plain".to_string(),
        };

        let result = upload_media_command_handler(command, &mock_repo, &mock_storage).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            lib::media::domain::MediaUploadError::InvalidFileType => {}, // Expected
            _ => panic!("Expected InvalidFileType error"),
        }
    }
}
