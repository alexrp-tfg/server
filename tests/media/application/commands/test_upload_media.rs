// Simple test to validate media upload functionality
#[cfg(test)]
mod media_upload_tests {
    use crate::media::{MockMediaRepository, MockStorageService};
    use lib::media::{
        application::commands::upload_media::{UploadMediaCommand, upload_media_command_handler},
        domain::MediaUploadError,
    };
    use uuid::Uuid;
    use std::pin::Pin;
    use bytes::Bytes;
    use futures_core::Stream;
    use futures_util::stream;

    fn create_file_stream(data: Vec<u8>) -> Pin<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send + Sync>> {
        Box::pin(stream::once(async move { 
            Ok(Bytes::from(data)) 
        }))
    }

    #[tokio::test]
    async fn test_upload_media_valid_image() {
        let mock_repo = MockMediaRepository::default();
        let mock_storage = MockStorageService::default();

        let user_id = Uuid::new_v4();
        let file_data = b"fake image data".to_vec();
        let file_size = file_data.len() as u64;
        
        let command = UploadMediaCommand {
            user_id,
            filename: "test.jpg".to_string(),
            original_filename: "original.jpg".to_string(),
            file_data: create_file_stream(file_data),
            file_size: Some(file_size),
            content_type: "image/jpeg".to_string(),
        };

        let result = upload_media_command_handler(&mock_repo, &mock_storage, command).await;

        assert!(result.is_ok());
        let upload_result = result.unwrap();
        assert_eq!(upload_result.filename, "test.jpg");
        assert_eq!(upload_result.original_filename, "original.jpg");
        assert_eq!(upload_result.content_type, "image/jpeg");
        assert_eq!(upload_result.file_size, file_size as i64);
    }

    #[tokio::test]
    async fn test_upload_media_invalid_file_type() {
        let mock_repo = MockMediaRepository::default();
        let mock_storage = MockStorageService::default();

        let user_id = Uuid::new_v4();
        let file_data = b"text content".to_vec();
        let file_size = file_data.len() as u64;
        
        let command = UploadMediaCommand {
            user_id,
            filename: "test.txt".to_string(),
            original_filename: "original.txt".to_string(),
            file_data: create_file_stream(file_data),
            file_size: Some(file_size),
            content_type: "text/plain".to_string(),
        };

        let result = upload_media_command_handler(&mock_repo, &mock_storage, command).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            MediaUploadError::InvalidFileType => {} // Expected
            _ => panic!("Expected InvalidFileType error"),
        }
    }
}
