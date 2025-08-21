use std::io::Cursor;

use async_trait::async_trait;
use image::{DynamicImage, GenericImageView, ImageFormat, imageops::FilterType};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::media::MediaId;

use super::{FileStorageService, MediaRepository, MediaRepositoryError};

const THUMBNAIL_WIDTH: u32 = 300;
const THUMBNAIL_HEIGHT: u32 = 300;

#[derive(Debug, thiserror::Error)]
pub enum ThumbnailError {
    #[error("Image processing error: {0}")]
    ImageProcessingError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Repository error: {0}")]
    RepositoryError(#[from] MediaRepositoryError),
}

#[async_trait]
pub trait ThumbnailService: Send + Sync {
    async fn generate_thumbnail(
        &self,
        media_id: Uuid,
        original_path: &str,
        image_data: Vec<u8>,
        content_type: &str,
    ) -> Result<(), ThumbnailError>;
}

pub struct ImageThumbnailService<MR, FS> {
    media_repository: MR,
    storage_service: FS,
}

impl<MR, FS> ImageThumbnailService<MR, FS>
where
    MR: MediaRepository,
    FS: FileStorageService,
{
    pub fn new(media_repository: MR, storage_service: FS) -> Self {
        Self {
            media_repository,
            storage_service,
        }
    }

    fn should_create_thumbnail(&self, image: &DynamicImage) -> bool {
        let (width, height) = image.dimensions();
        width > THUMBNAIL_WIDTH || height > THUMBNAIL_HEIGHT
    }

    fn create_thumbnail_data(&self, image_data: &[u8]) -> Result<Vec<u8>, ThumbnailError> {
        let image = image::load_from_memory(image_data)
            .map_err(|e| ThumbnailError::ImageProcessingError(e.to_string()))?;

        if !self.should_create_thumbnail(&image) {
            return Err(ThumbnailError::ImageProcessingError(
                "Image is already smaller than thumbnail size".to_string(),
            ));
        }

        let thumbnail = image.resize(THUMBNAIL_WIDTH, THUMBNAIL_HEIGHT, FilterType::Lanczos3);

        // Convert to RGB if needed
        let rgb_thumbnail = match thumbnail.color() {
            image::ColorType::Rgba8 | image::ColorType::La8 => {
                DynamicImage::ImageRgb8(thumbnail.to_rgb8())
            }
            _ => thumbnail,
        };

        let mut output = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut output);

        rgb_thumbnail
            .write_to(&mut cursor, ImageFormat::Jpeg)
            .map_err(|e| ThumbnailError::ImageProcessingError(e.to_string()))?;

        Ok(output)
    }

    fn generate_thumbnail_path(&self, original_path: &str, media_id: MediaId) -> String {
        let path_parts: Vec<&str> = original_path.split('/').collect();
        let dir = &path_parts[..path_parts.len() - 1].join("/");
        format!("{}/thumb_{}.jpg", dir, media_id)
    }
}

#[async_trait]
impl<MR, FS> ThumbnailService for ImageThumbnailService<MR, FS>
where
    MR: MediaRepository + 'static,
    FS: FileStorageService + 'static,
{
    async fn generate_thumbnail(
        &self,
        media_id: MediaId,
        original_path: &str,
        image_data: Vec<u8>,
        content_type: &str,
    ) -> Result<(), ThumbnailError> {
        if !content_type.starts_with("image/") {
            return Ok(());
        }

        let thumbnail_data = match self.create_thumbnail_data(&image_data) {
            Ok(data) => data,
            Err(ThumbnailError::ImageProcessingError(msg)) if msg.contains("already smaller") => {
                return Ok(());
            }
            Err(e) => return Err(e),
        };

        let thumbnail_path = self.generate_thumbnail_path(original_path, media_id);

        let thumbnail_stream = Box::pin(ReaderStream::new(Cursor::new(thumbnail_data)));

        self.storage_service
            .store_file(
                &thumbnail_path,
                "image/jpeg",
                Some(image_data.len() as u64),
                thumbnail_stream,
            )
            .await
            .map_err(|_| {
                ThumbnailError::StorageError("An error saving the image occurred".to_string())
            })?;

        self.media_repository
            .update_thumbnail_path(media_id, Some(thumbnail_path))
            .await?;

        Ok(())
    }
}
