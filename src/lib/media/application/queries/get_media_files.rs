use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::media::domain::{MediaFile, MediaRepository, MediaRepositoryError};

#[derive(Debug, Deserialize, ToSchema)]
pub struct GetMediaFilesQuery {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, ToSchema, Clone, PartialEq, Eq)]
pub struct GetMediaFilesResult {
    pub id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub file_size: i64,
    pub content_type: String,
    pub file_path: String,
    pub uploaded_at: Option<chrono::NaiveDateTime>,
}

pub async fn get_media_files_query_handler<MR: MediaRepository + ?Sized>(
    query: GetMediaFilesQuery,
    media_repository: &MR,
) -> Result<Vec<GetMediaFilesResult>, MediaRepositoryError> {
    let media_files = media_repository
        .get_media_files_by_user_id(query.user_id)
        .await?;

    Ok(media_files.into_iter().map(|media| media.into()).collect())
}

impl From<MediaFile> for GetMediaFilesResult {
    fn from(media_file: MediaFile) -> Self {
        GetMediaFilesResult {
            id: media_file.id,
            filename: media_file.filename,
            original_filename: media_file.original_filename,
            file_size: media_file.file_size,
            content_type: media_file.content_type,
            file_path: media_file.file_path,
            uploaded_at: media_file.uploaded_at,
        }
    }
}
