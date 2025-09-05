use super::models::{MediaFileModel, NewMediaFileModel};
use crate::media::domain::{MediaFile, NewMediaFile};

impl From<MediaFileModel> for MediaFile {
    fn from(model: MediaFileModel) -> Self {
        MediaFile {
            id: model.id,
            user_id: model.user_id,
            filename: model.filename,
            original_filename: model.original_filename,
            file_size: model.file_size,
            content_type: model.content_type,
            file_path: model.file_path,
            thumbnail_path: model.thumbnail_path,
            uploaded_at: model.uploaded_at,
            updated_at: model.updated_at,
        }
    }
}

impl From<NewMediaFile> for NewMediaFileModel {
    fn from(new_media: NewMediaFile) -> Self {
        NewMediaFileModel {
            user_id: new_media.user_id,
            filename: new_media.filename,
            original_filename: new_media.original_filename,
            file_size: new_media.file_size,
            content_type: new_media.content_type,
            file_path: new_media.file_path,
            thumbnail_path: new_media.thumbnail_path,
        }
    }
}
